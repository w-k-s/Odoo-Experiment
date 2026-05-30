//! Minimal Odoo integration over the external JSON-RPC API (`POST /jsonrpc`).
//!
//! We talk to `/jsonrpc` directly (rather than the `odoo-api` crate, whose
//! `authenticate` uses the web-session endpoint and rejects API keys): the
//! external API's `common.authenticate` accepts an API key as the password,
//! which is the recommended way to authenticate an integration.
//!
//! We authenticate lazily (Odoo may not be up at boot), cache the uid, and
//! expose just the one call Phase 4 needs: creating a contact (`res.partner`).

use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Map, Value};
use tokio::sync::Mutex;

use crate::config::OdooConfig;
use crate::middleware::error::{AppError, AppResult};

/// Lazily-authenticated Odoo client, cheap to `clone` (shared uid cache).
#[derive(Clone)]
pub struct Odoo {
    config: OdooConfig,
    http: Client,
    uid: Arc<Mutex<Option<i64>>>,
}

impl Odoo {
    pub fn new(config: OdooConfig) -> Self {
        Self {
            config,
            http: Client::new(),
            uid: Arc::new(Mutex::new(None)),
        }
    }

    /// Issue a JSON-RPC `call` and return its `result`, surfacing Odoo errors.
    async fn call(&self, service: &str, method: &str, args: Vec<Value>) -> AppResult<Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "call",
            "params": { "service": service, "method": method, "args": args },
        });
        let resp = self
            .http
            .post(format!("{}/jsonrpc", self.config.url))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("odoo request failed: {e}")))?;

        let mut payload: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("odoo decode failed: {e}")))?;

        if let Some(err) = payload.get("error") {
            return Err(AppError::Internal(format!("odoo json-rpc error: {err}")));
        }
        Ok(payload["result"].take())
    }

    /// Authenticate (once) and return the cached uid.
    async fn uid(&self) -> AppResult<i64> {
        let mut slot = self.uid.lock().await;
        if let Some(uid) = *slot {
            return Ok(uid);
        }
        let result = self
            .call(
                "common",
                "authenticate",
                vec![
                    json!(self.config.db),
                    json!(self.config.login),
                    json!(self.config.api_key),
                    json!({}),
                ],
            )
            .await?;
        // Odoo returns the uid on success, or `false` when credentials are bad.
        let uid = result.as_i64().filter(|v| *v > 0).ok_or_else(|| {
            AppError::Internal(format!(
                "odoo authentication failed (login/api key rejected): {result}"
            ))
        })?;
        *slot = Some(uid);
        Ok(uid)
    }

    /// Create a contact (`res.partner`) and return its Odoo id.
    pub async fn create_contact(&self, name: &str, email: Option<&str>) -> AppResult<i32> {
        let uid = self.uid().await?;

        let mut fields = Map::new();
        fields.insert("name".into(), json!(name));
        if let Some(email) = email {
            fields.insert("email".into(), json!(email));
        }

        let result = self
            .call(
                "object",
                "execute_kw",
                vec![
                    json!(self.config.db),
                    json!(uid),
                    json!(self.config.api_key),
                    json!("res.partner"),
                    json!("create"),
                    json!([Value::Object(fields)]),
                ],
            )
            .await?;

        result.as_i64().map(|id| id as i32).ok_or_else(|| {
            AppError::Internal(format!("odoo create_contact unexpected response: {result}"))
        })
    }
}
