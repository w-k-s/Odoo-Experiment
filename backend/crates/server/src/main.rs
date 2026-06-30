use std::net::SocketAddr;
use std::sync::Arc;

use api_utils::auth::build_authorizer;
use api_utils::state::AppState;
use auth::IdentityProvider;
use auth::auth0::Auth0;
use consumer::run;
use crm::Crm;
use crm::odoo::Odoo;
use loyalty::LoyaltyEngine;
use loyalty::db;
use loyalty::engine::DbLoyaltyEngine;
use utils::config::Config;

#[tokio::main]
async fn main() {
    tracing::error!("spawn");
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
    let loyalty = Arc::new(loyalty);

    let state = AppState {
        loyalty: loyalty.clone(),
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

    tokio::spawn(async move {
        tracing::error!("spawn");
        if let Err(e) = run(loyalty.clone(), config.consumer.clone()).await {
            tracing::error!("consumer exited with error: {e}");
        }
    });
    axum::serve(listener, app).await.expect("server error");
}
