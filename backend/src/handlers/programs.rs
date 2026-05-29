use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;

use crate::error::AppResult;
use crate::ids::new_id;
use crate::models::{CreateProgram, NewProgram, Program};
use crate::AppState;

/// `POST /loyalty/programs`
pub async fn create_program(
    State(state): State<AppState>,
    Json(body): Json<CreateProgram>,
) -> AppResult<(StatusCode, Json<Program>)> {
    let conn = state.pool.get().await?;
    let program = conn
        .interact(move |conn| {
            use crate::schema::loyalty_programs::dsl::loyalty_programs;
            let new = NewProgram {
                id: new_id("prog"),
                name: body.name,
            };
            diesel::insert_into(loyalty_programs)
                .values(&new)
                .returning(Program::as_returning())
                .get_result::<Program>(conn)
        })
        .await??;

    Ok((StatusCode::CREATED, Json(program)))
}
