//! Identity port: resolving an authenticated caller's profile, abstracted over
//! the concrete identity provider (Auth0 today — see `auth0.rs`).

use serde::Deserialize;

use crate::middleware::error::AppResult;

/// The profile fields we consume. Both are optional: a member may have no
/// `name`/`email` depending on the identity provider's record.
#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Resolves an authenticated caller's profile from their stable subject id.
/// The implementation owns whatever credentials it needs to do so.
#[axum::async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn fetch_profile(&self, subject: &str) -> AppResult<Profile>;
}
