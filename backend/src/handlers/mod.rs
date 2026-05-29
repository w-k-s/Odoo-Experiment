pub mod me;
pub mod members;
pub mod programs;
pub mod sessions;

use axum::Json;
use serde_json::{json, Value};

/// Liveness probe.
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
