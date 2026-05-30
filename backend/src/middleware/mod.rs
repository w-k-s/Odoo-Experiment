//! The outward-facing layer: HTTP handlers, Auth0 auth, and integrations.
//! Depends on `loyalty_engine`; the engine never depends back on this.

pub mod auth;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod integrations;
pub mod state;

use axum::routing::{get, post};
use axum::Router;
use jwt_authorizer::{Authorizer, IntoLayer};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::middleware::auth::Claims;
use crate::middleware::state::AppState;

/// Build the application router: public routes + Auth0-protected routes.
pub fn router(state: AppState, authorizer: Authorizer<Claims>) -> Router {
    // Routes requiring a verified Auth0 token.
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

    public
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
