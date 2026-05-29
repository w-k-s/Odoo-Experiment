use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;

use crate::error::{AppError, AppResult};
use crate::ids::new_id;
use crate::models::{NewProgram, Program};

/// Ensure a default loyalty program exists, returning its id.
///
/// Idempotent: matches on program name, creating it only when absent. This is
/// the "bootstrap data" hook — extend it to seed more rows as needed.
pub async fn ensure_default_program(pool: &Pool, name: &str) -> AppResult<String> {
    let conn = pool.get().await?;
    let name = name.to_string();

    let id = conn
        .interact(move |conn| {
            use crate::schema::loyalty_programs::dsl as p;

            let existing: Option<Program> = p::loyalty_programs
                .filter(p::name.eq(&name))
                .select(Program::as_select())
                .first(conn)
                .optional()?;

            if let Some(program) = existing {
                return Ok::<String, diesel::result::Error>(program.id);
            }

            let new = NewProgram {
                id: new_id("prog"),
                name,
            };
            let created: Program = diesel::insert_into(p::loyalty_programs)
                .values(&new)
                .returning(Program::as_returning())
                .get_result(conn)?;
            Ok(created.id)
        })
        .await
        .map_err(AppError::from)??;

    Ok(id)
}
