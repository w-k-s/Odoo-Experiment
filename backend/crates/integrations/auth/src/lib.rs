//! Identity port: resolving an authenticated caller's profile, abstracted over
//! the concrete identity provider (Auth0 today — see `auth0.rs`).

pub mod auth0;

use serde::Deserialize;

use utils::error::AppResult;

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[axum::async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn fetch_profile(&self, subject: &str) -> AppResult<Profile>;
}
