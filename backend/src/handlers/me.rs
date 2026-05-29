use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use jwt_authorizer::JwtClaims;

use crate::auth::{bearer_token, fetch_userinfo, Claims};
use crate::error::AppResult;
use crate::handlers::members::ensure_member;
use crate::models::MemberProfile;
use crate::AppState;

/// `GET /loyalty/me`
///
/// Returns the authenticated member's profile + point balance, provisioning the
/// member (Odoo partner + loyalty row) on first call.
pub async fn me(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
    headers: HeaderMap,
) -> AppResult<Json<MemberProfile>> {
    let token = bearer_token(&headers)?;
    let info = fetch_userinfo(&state.auth0_domain, token).await?;
    let name = info.name.unwrap_or_else(|| "Member".to_string());

    let member = ensure_member(&state, &claims.sub, &name, info.email.as_deref()).await?;
    Ok(Json(member.into()))
}
