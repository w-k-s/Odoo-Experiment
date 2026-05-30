use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::loyalty_engine::models::Program;
use crate::middleware::dto::CreateProgram;
use crate::middleware::error::AppResult;
use crate::middleware::state::AppState;

/// `POST /loyalty/programs`
pub async fn create_program(
    State(state): State<AppState>,
    Json(body): Json<CreateProgram>,
) -> AppResult<(StatusCode, Json<Program>)> {
    let program = state.programs.create(body.name).await?;
    Ok((StatusCode::CREATED, Json(program)))
}
