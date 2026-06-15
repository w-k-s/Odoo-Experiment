use diesel::prelude::*;

use crate::db::Pool;
use crate::error::EngineResult;
use crate::ids::new_id;
use crate::models::{NewProgram, Program};

#[derive(Clone)]
pub struct ProgramService {
    pool: Pool,
}

impl ProgramService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, name: String) -> EngineResult<Program> {
        let conn = self.pool.get().await?;
        let program = conn
            .interact(move |conn| {
                use crate::schema::loyalty_programs::dsl::loyalty_programs;
                let new = NewProgram {
                    id: new_id("prog"),
                    name,
                };
                diesel::insert_into(loyalty_programs)
                    .values(&new)
                    .returning(Program::as_returning())
                    .get_result::<Program>(conn)
            })
            .await??;
        Ok(program)
    }

    /// Ensure a default program exists (matched by name), returning its id.
    /// Idempotent — safe to call at startup.
    pub async fn ensure_default(&self, name: &str) -> EngineResult<String> {
        let conn = self.pool.get().await?;
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
            .await??;
        Ok(id)
    }
}
