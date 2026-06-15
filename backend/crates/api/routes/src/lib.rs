use axum::routing::{get, post};
use axum::Router;
use jwt_authorizer::{Authorizer, IntoLayer};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use api::health;
use api::me::me;
use api::members::create_member;
use api::programs::create_program;
use api::sessions::{create_session, get_session};
use api_utils::auth::Claims;
use api_utils::state::AppState;

pub fn router(state: AppState, authorizer: Authorizer<Claims>) -> Router {
    let protected = Router::new()
        .route("/loyalty/me", get(me))
        .route("/loyalty/sessions", post(create_session))
        .route("/loyalty/sessions/:id", get(get_session))
        .layer(authorizer.into_layer());

    let public = Router::new()
        .route("/health", get(health))
        .route("/loyalty/programs", post(create_program))
        .route("/loyalty/members", post(create_member));

    public
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
