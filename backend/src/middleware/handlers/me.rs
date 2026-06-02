use axum::extract::State;
use axum::Json;
use jwt_authorizer::JwtClaims;

use crate::middleware::auth::Claims;
use crate::middleware::dto::MemberProfile;
use crate::middleware::error::AppResult;
use crate::middleware::handlers::provisioning::ensure_member;
use crate::middleware::state::AppState;

/// `GET /loyalty/me`
///
/// Returns the authenticated member's profile + point balance, provisioning the
/// member (CRM contact + loyalty row) on first call. On first sight the profile
/// is resolved from the identity provider by `sub`; thereafter it comes from the
/// datastore.
pub async fn me(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
) -> AppResult<Json<MemberProfile>> {
    let member = ensure_member(&state, &claims.sub).await?;
    Ok(Json(member.into()))
}
