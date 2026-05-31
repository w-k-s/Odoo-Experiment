//! Auth0 token verification.
//!
//! JWT verification is delegated to `jwt-authorizer`: we build an `Authorizer`
//! from Auth0's JWKS endpoint and mount it as a tower layer on the protected
//! routes (see `middleware::router`). Handlers then extract `JwtClaims<Claims>`.
//!
//! POC note: the PWA authenticates with the **ID token** (audience = the SPA
//! client id), which carries the member's `name`/`email`/`sub` — so we read the
//! profile straight from the verified claims, no `/userinfo` round-trip.

use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use serde::Deserialize;

/// The verified claims we care about. `sub` is the stable member key; the OIDC
/// `name`/`email` claims are present because the app requests `profile email`.
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Build the JWKS-backed authorizer for the given Auth0 tenant + audience.
///
/// For ID-token auth `audience` is the SPA client id (the token's `aud`).
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
        .inspect_err(|e| tracing::error!(error = %e, "failed to build Auth0 JWT authorizer"))
        .expect("failed to build Auth0 JWT authorizer")
}
