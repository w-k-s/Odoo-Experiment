//! Auth0 token verification.
//!
//! JWT verification is delegated to `jwt-authorizer`: we build an `Authorizer`
//! from Auth0's JWKS endpoint and mount it as a tower layer on the protected
//! routes (see `main.rs`). Handlers then extract `JwtClaims<Claims>`.
//!
//! Display name / email are not in the access token, so we resolve them from
//! Auth0's `/userinfo` endpoint using the caller's bearer token.

use axum::http::{header::AUTHORIZATION, HeaderMap};
use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use serde::Deserialize;

use crate::error::{AppError, AppResult};

/// The verified claims we care about. `sub` is the stable member key.
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
}

/// Build the JWKS-backed authorizer for the given Auth0 tenant + API audience.
pub async fn build_authorizer(domain: &str, audience: &str) -> Authorizer<Claims> {
    let issuer = format!("https://{domain}/");
    let jwks_url = format!("https://{domain}/.well-known/jwks.json");
    let validation = Validation::new()
        .iss(&[issuer])
        .aud(&[audience.to_string()]);

    JwtAuthorizer::<Claims>::from_jwks_url(&jwks_url)
        .validation(validation)
        .build()
        .await
        .expect("failed to build Auth0 JWT authorizer")
}

/// Pull the raw bearer token out of the request headers (needed for `/userinfo`).
pub fn bearer_token(headers: &HeaderMap) -> AppResult<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))
}

/// Auth0 `/userinfo` profile (only the fields we use).
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Resolve the caller's profile from Auth0's `/userinfo` endpoint.
pub async fn fetch_userinfo(domain: &str, token: &str) -> AppResult<UserInfo> {
    let url = format!("https://{domain}/userinfo");
    let resp = reqwest::Client::new()
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| AppError::Unauthorized(format!("userinfo request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Unauthorized(format!(
            "userinfo returned {}",
            resp.status()
        )));
    }

    resp.json::<UserInfo>()
        .await
        .map_err(|e| AppError::Internal(format!("userinfo decode failed: {e}")))
}
