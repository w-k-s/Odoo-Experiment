//! The loyalty domain: models, services, and its own datastore.
//!
//! This module must stay free of web/integration concerns — no axum, no
//! jwt-authorizer/Auth0, no reqwest, no Odoo. The middleware depends on the
//! engine, never the reverse.

pub mod db;
pub mod error;
pub mod ids;
pub mod models;
pub mod schema;
pub mod services;
