use std::net::SocketAddr;

use axum::routing::{get, post};
use axum::Router;
use deadpool_diesel::postgres::Pool;
use jwt_authorizer::IntoLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod auth;
mod bootstrap;
mod config;
mod db;
mod error;
mod handlers;
mod ids;
mod models;
mod odoo;
mod schema;

use config::Config;
use odoo::Odoo;

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    /// Program members enrol in when a request omits an explicit `program_id`.
    pub default_program_id: String,
    /// Auth0 tenant domain (used for the `/userinfo` lookup).
    pub auth0_domain: String,
    /// Lazily-authenticated Odoo client.
    pub odoo: Odoo,
}

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

    let pool = db::build_pool(&config.database_url, config.pool_max_size);

    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let default_program_id =
        bootstrap::ensure_default_program(&pool, &config.bootstrap_program_name)
            .await
            .expect("failed to bootstrap default program");
    tracing::info!(
        program_id = %default_program_id,
        name = %config.bootstrap_program_name,
        "default loyalty program ready"
    );

    let authorizer = auth::build_authorizer(&config.auth0_domain, &config.auth0_audience).await;

    let state = AppState {
        pool,
        default_program_id,
        auth0_domain: config.auth0_domain.clone(),
        odoo: Odoo::new(config.odoo.clone()),
    };

    // Routes requiring a verified Auth0 access token.
    let protected = Router::new()
        .route("/loyalty/me", get(handlers::me::me))
        .route("/loyalty/sessions", post(handlers::sessions::create_session))
        .route("/loyalty/sessions/:id", get(handlers::sessions::get_session))
        .layer(authorizer.into_layer());

    // Public / admin routes.
    let public = Router::new()
        .route("/health", get(handlers::health))
        .route("/loyalty/programs", post(handlers::programs::create_program))
        .route("/loyalty/members", post(handlers::members::create_member));

    let app = public
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

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
