use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::loyalty_engine::ids::new_id;
use crate::loyalty_engine::models::Member;
use crate::middleware::dto::CreateMember;
use crate::middleware::error::AppResult;
use crate::middleware::state::AppState;

/// `POST /loyalty/members`
///
/// Admin/testing endpoint: registers a member directly (no Auth0 identity / no
/// Odoo contact). When `program_id` is omitted the default program is used.
pub async fn create_member(
    State(state): State<AppState>,
    Json(body): Json<CreateMember>,
) -> AppResult<(StatusCode, Json<Member>)> {
    let program_id = body
        .program_id
        .unwrap_or_else(|| state.default_program_id.clone());

    let member = state
        .members
        .create(&new_id("mem"), None, &body.name, body.email.as_deref(), None, &program_id)
        .await?;
    Ok((StatusCode::CREATED, Json(member)))
}
