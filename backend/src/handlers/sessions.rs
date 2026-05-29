use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jwt_authorizer::JwtClaims;

use crate::auth::Claims;
use crate::error::{AppError, AppResult};
use crate::handlers::members::{ensure_member, find_member_by_sub};
use crate::ids::new_session_code;
use crate::models::{Member, NewSession, Session, SessionDetail, SessionMember};
use crate::AppState;

/// Sessions live for 24h before a fresh one must be minted.
const SESSION_TTL_HOURS: i64 = 24;

/// `POST /loyalty/sessions`
///
/// Mints a short, scannable session code for the authenticated member (the code
/// they present at the till). The member is derived from the verified token and
/// provisioned on first sight.
pub async fn create_session(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
) -> AppResult<(StatusCode, Json<Session>)> {
    let name = claims.name.clone().unwrap_or_else(|| "Member".to_string());
    let member = ensure_member(&state, &claims.sub, &name, claims.email.as_deref()).await?;

    let new = NewSession {
        id: new_session_code(),
        member_id: member.id,
        expires_at: Some(Utc::now() + Duration::hours(SESSION_TTL_HOURS)),
    };

    let conn = state.pool.get().await?;
    let session = conn
        .interact(move |conn| {
            use crate::schema::loyalty_sessions::dsl::loyalty_sessions;
            diesel::insert_into(loyalty_sessions)
                .values(&new)
                .returning(Session::as_returning())
                .get_result::<Session>(conn)
        })
        .await??;

    Ok((StatusCode::CREATED, Json(session)))
}

/// `GET /loyalty/sessions/{id}`
///
/// Resolves a session code to the member details the POS consumes. Enforces that
/// the caller owns the session — on mismatch we 404 rather than reveal it exists.
pub async fn get_session(
    State(state): State<AppState>,
    JwtClaims(claims): JwtClaims<Claims>,
    Path(id): Path<String>,
) -> AppResult<Json<SessionDetail>> {
    let not_found = || AppError::NotFound(format!("session {id} not found"));

    // The caller must already be a provisioned member to own any session.
    let caller = find_member_by_sub(&state, &claims.sub)
        .await?
        .ok_or_else(not_found)?;
    let caller_id = caller.id;

    let conn = state.pool.get().await?;
    let detail = conn
        .interact(move |conn| -> AppResult<SessionDetail> {
            use crate::schema::loyalty_members::dsl as m;
            use crate::schema::loyalty_sessions::dsl as s;

            let session = s::loyalty_sessions
                .filter(s::id.eq(&id))
                .select(Session::as_select())
                .first::<Session>(conn)
                .optional()?
                .ok_or_else(|| AppError::NotFound(format!("session {id} not found")))?;

            if session.member_id != caller_id {
                return Err(AppError::NotFound(format!("session {id} not found")));
            }

            let member = m::loyalty_members
                .filter(m::id.eq(&session.member_id))
                .select(Member::as_select())
                .first::<Member>(conn)?;

            let status = match session.expires_at {
                Some(expires_at) if expires_at < Utc::now() => "expired".to_string(),
                _ => session.status,
            };

            Ok(SessionDetail {
                session_id: session.id,
                status,
                member: SessionMember {
                    name: member.name,
                    email: member.email,
                },
            })
        })
        .await??;

    Ok(Json(detail))
}
