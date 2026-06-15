use deadpool_diesel::postgres::{Manager, Runtime};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::error::{EngineError, EngineResult};

pub use deadpool_diesel::postgres::Pool;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");

pub fn build_pool(database_url: &str, max_size: usize) -> Pool {
    let manager = Manager::new(database_url, Runtime::Tokio1);
    Pool::builder(manager)
        .max_size(max_size)
        .build()
        .expect("failed to build database pool")
}

pub async fn run_migrations(pool: &Pool) -> EngineResult<()> {
    let conn = pool.get().await?;
    conn.interact(|conn| {
        conn.run_pending_migrations(MIGRATIONS)
            .map(|_| ())
            .map_err(|e| EngineError::Db(format!("migration failed: {e}")))
    })
    .await??;
    Ok(())
}
