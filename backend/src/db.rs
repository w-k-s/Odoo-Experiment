use deadpool_diesel::postgres::{Manager, Pool, Runtime};
use diesel::pg::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::error::{AppError, AppResult};

/// Migrations embedded at compile time from the `migrations/` directory.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Build a deadpool-backed Diesel connection pool.
pub fn build_pool(database_url: &str, max_size: usize) -> Pool {
    let manager = Manager::new(database_url, Runtime::Tokio1);
    Pool::builder(manager)
        .max_size(max_size)
        .build()
        .expect("failed to build database pool")
}

/// Run any pending migrations on startup.
pub async fn run_migrations(pool: &Pool) -> AppResult<()> {
    let conn = pool.get().await?;
    conn.interact(|conn| {
        conn.run_pending_migrations(MIGRATIONS)
            .map(|_| ())
            .map_err(|e| AppError::Internal(format!("migration failed: {e}")))
    })
    .await??;
    Ok(())
}

/// Helper alias for the kind of connection `interact` hands the closure.
pub type DbConn = PgConnection;
