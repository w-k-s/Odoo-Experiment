use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use jwt_authorizer::JwtClaims;

use crate::loyalty_engine::models::Session;
use crate::middleware::auth::Claims;
use crate::middleware::dto::{SessionDetail, SessionMember};
use crate::middleware::error::AppResult;
use crate::middleware::handlers::provisioning::ensure_member;
use crate::middleware::state::AppState;

/// `POST /loyalty/sessions`
///
/// Mints a short, scannable session code for the authenticated member (the code
/// they present at the till). The member is derived from the verified token and
/// provisioned on first sight.
pub async fn create_session(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
) -> AppResult<(StatusCode, Json<Session>)> {
    let member = ensure_member(&state, &claims.sub).await?;
    let session = state.sessions.create(member.id).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

/// `GET /loyalty/sessions/{id}`
///
/// Resolves a session code to the member details the POS consumes. Enforces that
/// the caller owns the session — on mismatch we 404 rather than reveal it exists.
pub async fn get_session(
    State(state): State<AppState>,
    JwtClaims(_claims): JwtClaims<Claims>,
    Path(id): Path<String>,
) -> AppResult<Json<SessionDetail>> {
    // The caller must already be a provisioned member to own any session.
    let owned = state
        .sessions
        .get_owned(id)
        .await
        .inspect_err(|e| tracing::error!(error = %e, "failed to get owned session"))?;
    Ok(Json(SessionDetail {
        session_id: owned.session_id,
        status: owned.status,
        member: SessionMember {
            id: owned.member.id,
            name: owned.member.name,
            email: owned.member.email,
        },
    }))
}
