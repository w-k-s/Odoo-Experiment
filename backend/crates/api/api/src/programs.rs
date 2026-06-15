use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use api_utils::state::AppState;
use loyalty_engine::models::Program;
use utils::error::AppResult;

use crate::dto::CreateProgram;

/// `POST /loyalty/programs`
pub async fn create_program(
    State(state): State<AppState>,
    Json(body): Json<CreateProgram>,
) -> AppResult<(StatusCode, Json<Program>)> {
    let program = state.loyalty.create_program(body.name).await?;
    Ok((StatusCode::CREATED, Json(program)))
}
