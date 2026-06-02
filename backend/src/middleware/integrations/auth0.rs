//! Auth0 identity provider.
//!
//! Resolves a caller's profile from their stable `sub` via the Auth0
//! Management API (`GET /api/v2/users/{id}`). The client owns its own
//! machine-to-machine credentials: it exchanges a client-id/secret for a
//! Management API token at `POST /oauth/token` (the `client_credentials` grant),
//! caches it until it nears expiry, and reuses it across lookups.
//!
//! We resolve by `sub` (not by forwarding the caller's access token) so the
//! identity port stays vendor-neutral — callers pass a subject id, not an OAuth
//! bearer.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::config::Auth0MgmtConfig;
use crate::middleware::error::{AppError, AppResult};
use crate::middleware::integrations::identity::{IdentityProvider, Profile};

/// A Management API token plus the instant it stops being usable.
#[derive(Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

/// Auth0 client backed by the Management API. Cheap to `clone` (shared token cache).
#[derive(Clone)]
pub struct Auth0 {
    http: Client,
    /// Management API audience, e.g. `https://{domain}/api/v2/`.
    audience: String,
    /// Token endpoint, e.g. `https://{domain}/oauth/token`.
    token_endpoint: String,
    /// Users endpoint base, e.g. `https://{domain}/api/v2/users`.
    users_endpoint: String,
    mgmt: Auth0MgmtConfig,
    token: Arc<Mutex<Option<CachedToken>>>,
}

/// Shape of the `POST /oauth/token` response we consume.
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

    /// Return a cached, non-expired Management API token, fetching a fresh one
    /// (client-credentials grant) when absent or near expiry.
    async fn mgmt_token(&self) -> AppResult<String> {
        let mut slot = self.token.lock().await;
        if let Some(cached) = slot.as_ref() {
            if cached.expires_at > Instant::now() {
                return Ok(cached.access_token.clone());
            }
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
            return Err(AppError::Internal(format!(
                "auth0 token endpoint returned {}",
                resp.status()
            )));
        }

        let token: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("auth0 token decode failed: {e}")))?;

        // Refresh a minute early to avoid races against the hard expiry.
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

        let resp = self
            .http
            .get(format!("{}/{}", self.users_endpoint, subject))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("auth0 users request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(format!(
                "auth0 users endpoint returned {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::Internal(format!("auth0 users decode failed: {e}")))
    }
}
