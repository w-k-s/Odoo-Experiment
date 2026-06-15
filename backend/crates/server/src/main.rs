use std::net::SocketAddr;
use std::sync::Arc;

use api_utils::auth::build_authorizer;
use api_utils::state::AppState;
use auth::auth0::Auth0;
use auth::IdentityProvider;
use crm::odoo::Odoo;
use crm::Crm;
use loyalty::engine::DbLoyaltyEngine;
use loyalty::LoyaltyEngine;
use loyalty_engine::db;
use utils::config::Config;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,server=debug".into()),
        )
        .init();

    let config = Config::from_env();

    // ---- loyalty engine: pool + services ----
    let pool = db::build_pool(&config.database_url, config.pool_max_size);
    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let loyalty = DbLoyaltyEngine::new(pool);

    let default_program_id = loyalty
        .ensure_default_program(&config.bootstrap_program_name)
        .await
        .expect("failed to bootstrap default program");
    tracing::info!(
        program_id = %default_program_id,
        name = %config.bootstrap_program_name,
        "default loyalty program ready"
    );

    // ---- integrations + auth + router ----
    let crm: Arc<dyn Crm> = Arc::new(Odoo::new(config.odoo.clone()));
    let identity: Arc<dyn IdentityProvider> =
        Arc::new(Auth0::new(&config.auth0_domain, config.auth0_mgmt.clone()));
    let authorizer = build_authorizer(&config.auth0_domain, &config.auth0_audience).await;

    let state = AppState {
        loyalty: Arc::new(loyalty),
        crm,
        identity,
        default_program_id,
    };

    let app = routes::router(state, authorizer);

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
