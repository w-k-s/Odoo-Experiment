use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use jwt_authorizer::JwtClaims;

use api_utils::auth::Claims;
use api_utils::state::AppState;
use loyalty::models::{Session, SessionId};
use utils::error::AppResult;

use crate::dto::{SessionDetail, SessionMember};
use crate::provisioning::ensure_member;

/// `POST /loyalty/sessions`
pub async fn create_session(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
) -> AppResult<(StatusCode, Json<Session>)> {
    let member = ensure_member(&state, &claims.sub).await?;
    let session = state.loyalty.create_session(member.id).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

/// `GET /loyalty/sessions/{id}`
pub async fn get_session(
    State(state): State<AppState>,
    JwtClaims(_claims): JwtClaims<Claims>,
    Path(id): Path<String>,
) -> AppResult<Json<SessionDetail>> {
    let owned = state
        .loyalty
        .get_owned_session(SessionId::from(id))
        .await
        .inspect_err(|e| tracing::error!(error = %e, "failed to get owned session"))?;
    Ok(Json(SessionDetail {
        session_id: owned.session_id.into(),
        status: owned.status,
        member: SessionMember {
            id: owned.member.id.into(),
            name: owned.member.name,
            email: owned.member.email,
        },
    }))
}
