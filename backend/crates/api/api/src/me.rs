use axum::Json;
use axum::extract::State;
use jwt_authorizer::JwtClaims;

use api_utils::auth::Claims;
use api_utils::state::AppState;
use utils::error::AppResult;

use crate::dto::MemberProfile;
use crate::provisioning::ensure_member;

/// `GET /loyalty/me`
pub async fn me(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
) -> AppResult<Json<MemberProfile>> {
    let member = ensure_member(&state, &claims.sub).await?;
    Ok(Json(member.into()))
}
