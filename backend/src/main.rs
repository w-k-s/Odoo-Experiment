use std::net::SocketAddr;

mod config;
mod loyalty_engine;
mod middleware;

use config::Config;
use loyalty_engine::db;
use loyalty_engine::services::members::MemberService;
use loyalty_engine::services::programs::ProgramService;
use loyalty_engine::services::sessions::SessionService;
use middleware::integrations::odoo::Odoo;
use middleware::state::AppState;

#[tokio::main]
async fn main() {
    // Load `.env` if present (no-op in production where env is set directly).
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,loyalty_backend=debug".into()),
        )
        .init();

    let config = Config::from_env();

    // ---- loyalty_engine: datastore + services ----
    let pool = db::build_pool(&config.database_url, config.pool_max_size);
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let programs = ProgramService::new(pool.clone());
    let members = MemberService::new(pool.clone());
    let sessions = SessionService::new(pool.clone());

    let default_program_id = programs
        .ensure_default(&config.bootstrap_program_name)
        .await
        .expect("failed to bootstrap default program");
    tracing::info!(
        program_id = %default_program_id,
        name = %config.bootstrap_program_name,
        "default loyalty program ready"
    );

    // ---- middleware: integrations + auth + router ----
    let odoo = Odoo::new(config.odoo.clone());
    let authorizer =
        middleware::auth::build_authorizer(&config.auth0_domain, &config.auth0_audience).await;

    let state = AppState {
        programs,
        members,
        sessions,
        odoo,
        default_program_id,
    };

    let app = middleware::router(state, authorizer);

    let addr: SocketAddr = config
        .bind_addr
        .parse()
        .unwrap_or_else(|_| panic!("invalid BIND_ADDR: {}", config.bind_addr));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {addr}: {e}"));
    tracing::info!("loyalty backend listening on {addr}");

    axum::serve(listener, app).await.expect("server error");
}
