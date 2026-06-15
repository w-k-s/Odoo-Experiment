//! Auth0 token verification.
//!
//! JWT verification is delegated to `jwt-authorizer`: we build an `Authorizer`
//! from Auth0's JWKS endpoint and mount it as a tower layer on the protected
//! routes (see `middleware::router`). Handlers then extract `JwtClaims<Claims>`.
//!
//! The PWA authenticates with an **access token** for the API audience. Access
//! tokens carry `sub` but not `name`/`email`, so handlers resolve the profile
//! from the identity provider by `sub` (see `integrations::identity`).

use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use serde::Deserialize;

/// The verified claims we care about. `sub` is the stable member key.
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
}

/// Build the JWKS-backed authorizer for the given Auth0 tenant + audience.
///
/// `audience` is the API audience the access token is issued for (the token's
/// `aud`), set via `AUTH0_AUDIENCE`.
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
