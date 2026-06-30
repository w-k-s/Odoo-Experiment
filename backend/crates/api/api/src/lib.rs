use axum::Json;
use serde_json::{Value, json};

pub mod dto;
pub mod me;
pub mod members;
pub mod programs;
pub mod provisioning;
pub mod sessions;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
