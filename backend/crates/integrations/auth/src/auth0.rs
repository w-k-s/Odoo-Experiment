//! Auth0 identity provider: resolves a caller's profile via the Management API.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::Deserialize;
use tokio::sync::Mutex;

use utils::config::Auth0MgmtConfig;
use utils::error::{AppError, AppResult};

use crate::{IdentityProvider, Profile};

#[derive(Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct Auth0 {
    http: Client,
    audience: String,
    token_endpoint: String,
    users_endpoint: String,
    mgmt: Auth0MgmtConfig,
    token: Arc<Mutex<Option<CachedToken>>>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

impl Auth0 {
    pub fn new(domain: &str, mgmt: Auth0MgmtConfig) -> Self {
        Self {
            http: Client::new(),
            audience: format!("https://{domain}/api/v2/"),
            token_endpoint: format!("https://{domain}/oauth/token"),
            users_endpoint: format!("https://{domain}/api/v2/users"),
            mgmt,
            token: Arc::new(Mutex::new(None)),
        }
    }

    async fn mgmt_token(&self) -> AppResult<String> {
        let mut slot = self.token.lock().await;
        if let Some(cached) = slot.as_ref() && cached.expires_at > Instant::now() {
            return Ok(cached.access_token.clone());
        }

        let resp = self
            .http
            .post(&self.token_endpoint)
            .json(&serde_json::json!({
                "client_id": self.mgmt.client_id,
                "client_secret": self.mgmt.client_secret,
                "audience": self.audience,
                "grant_type": "client_credentials",
            }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("auth0 token request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let endpoint = &self.token_endpoint;
            tracing::error!(
                "auth0 token endpoint returned {status}; replay: \
                 curl -X POST '{endpoint}' -H 'Content-Type: application/json' \
                 -d '{{\"client_id\":\"{client_id}\",\"client_secret\":\"{client_secret}\",\
                 \"audience\":\"{audience}\",\"grant_type\":\"client_credentials\"}}'",
                client_id = self.mgmt.client_id,
                client_secret = self.mgmt.client_secret,
                audience = self.audience,
            );
            return Err(AppError::Internal(format!(
                "auth0 token endpoint returned {status}"
            )));
        }

        let token: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("auth0 token decode failed: {e}")))?;

        let lifetime = Duration::from_secs(token.expires_in.saturating_sub(60));
        *slot = Some(CachedToken {
            access_token: token.access_token.clone(),
            expires_at: Instant::now() + lifetime,
        });
        Ok(token.access_token)
    }
}

#[axum::async_trait]
impl IdentityProvider for Auth0 {
    async fn fetch_profile(&self, subject: &str) -> AppResult<Profile> {
        let token = self.mgmt_token().await?;
        let url = format!("{}/{}", self.users_endpoint, subject);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("auth0 users request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            tracing::error!(
                "auth0 users endpoint returned {status}; replay: \
                 curl -H 'Authorization: Bearer {token}' '{url}'"
            );
            return Err(AppError::Internal(format!(
                "auth0 users endpoint returned {status}"
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::Internal(format!("auth0 users decode failed: {e}")))
    }
}
