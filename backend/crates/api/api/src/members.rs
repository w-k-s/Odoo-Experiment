use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use api_utils::state::AppState;
use loyalty::NewMember;
use loyalty::models::{Member, ProgramId};
use utils::error::AppResult;

use crate::dto::CreateMember;

/// `POST /loyalty/members`
///
/// Admin/testing endpoint: registers a member directly (no Auth0 / no CRM contact).
/// When `program_id` is omitted the default program is used.
pub async fn create_member(
    State(state): State<AppState>,
    Json(body): Json<CreateMember>,
) -> AppResult<(StatusCode, Json<Member>)> {
    let program_id = body
        .program_id
        .unwrap_or_else(|| state.default_program_id.clone());

    let member = state
        .loyalty
        .create_member(NewMember {
            name: &body.name,
            email: &body.email,
            external_contact_id: None,
            program_id: &ProgramId::from(program_id),
        })
        .await?;
    Ok((StatusCode::CREATED, Json(member)))
}
