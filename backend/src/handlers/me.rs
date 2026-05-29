use axum::extract::State;
use axum::Json;
use jwt_authorizer::JwtClaims;

use crate::auth::Claims;
use crate::error::AppResult;
use crate::handlers::members::ensure_member;
use crate::models::MemberProfile;
use crate::AppState;

/// `GET /loyalty/me`
///
/// Returns the authenticated member's profile + point balance, provisioning the
/// member (Odoo partner + loyalty row) on first call. Profile comes from the
/// verified ID-token claims.
pub async fn me(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
) -> AppResult<Json<MemberProfile>> {
    let name = claims.name.clone().unwrap_or_else(|| "Member".to_string());
    let member = ensure_member(&state, &claims.sub, &name, claims.email.as_deref()).await?;
    Ok(Json(member.into()))
}
