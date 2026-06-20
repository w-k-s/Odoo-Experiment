//! Odoo JSON-RPC CRM implementation.

use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use utils::config::OdooConfig;
use utils::error::{AppError, AppResult};

use crate::{ContactId, Crm, NewContact};

#[derive(Clone)]
pub struct Odoo {
    http: Client,
    config: OdooConfig,
    session: Arc<Mutex<Option<OdooSession>>>,
}

#[derive(Clone)]
struct OdooSession {
    session_id: String,
}

#[derive(Serialize)]
struct JsonRpcRequest<P: Serialize> {
    jsonrpc: &'static str,
    method: &'static str,
    id: u32,
    params: P,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<Value>,
    error: Option<Value>,
}

#[derive(Serialize)]
struct AuthParams {
    db: String,
    login: String,
    password: String,
}

#[derive(Serialize)]
struct CallKwParams {
    model: &'static str,
    method: &'static str,
    args: Value,
    kwargs: Value,
}

impl Odoo {
    pub fn new(config: OdooConfig) -> Self {
        Self {
            http: Client::new(),
            config,
            session: Arc::new(Mutex::new(None)),
        }
    }

    async fn session(&self) -> AppResult<OdooSession> {
        let mut slot = self.session.lock().await;
        if let Some(s) = slot.clone() {
            return Ok(s);
        }

        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            method: "call",
            id: 1,
            params: AuthParams {
                db: self.config.db.clone(),
                login: self.config.login.clone(),
                password: self.config.api_key.clone(),
            },
        };

        let resp = self
            .http
            .post(format!("{}/web/session/authenticate", self.config.url))
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("odoo auth request failed: {e}")))?;

        let session_id = resp
            .cookies()
            .find(|c| c.name() == "session_id")
            .map(|c| c.value().to_string())
            .unwrap_or_default();

        let body: JsonRpcResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("odoo auth decode failed: {e}")))?;

        if let Some(err) = body.error {
            return Err(AppError::Internal(format!("odoo auth error: {err}")));
        }

        // Verify auth succeeded by checking uid is present in the response.
        let uid_present = body
            .result
            .as_ref()
            .and_then(|r| r.get("uid"))
            .and_then(|v| v.as_i64())
            .is_some();
        if !uid_present {
            return Err(AppError::Internal("odoo auth: missing uid in response".into()));
        }

        let s = OdooSession { session_id };
        *slot = Some(s.clone());
        Ok(s)
    }

    async fn call_kw(&self, model: &'static str, method: &'static str, args: Value) -> AppResult<Value> {
        let s = self.session().await?;

        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            method: "call",
            id: 2,
            params: CallKwParams {
                model,
                method,
                args,
                kwargs: serde_json::json!({}),
            },
        };

        let resp = self
            .http
            .post(format!("{}/web/dataset/call_kw", self.config.url))
            .header("Cookie", format!("session_id={}", s.session_id))
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("odoo call_kw request failed: {e}")))?;

        let body: JsonRpcResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("odoo call_kw decode failed: {e}")))?;

        if let Some(err) = body.error {
            return Err(AppError::Internal(format!("odoo call_kw error: {err}")));
        }

        body.result
            .ok_or_else(|| AppError::Internal("odoo call_kw: empty result".into()))
    }
}

#[axum::async_trait]
impl Crm for Odoo {
    async fn create_contact(&self, contact: NewContact<'_>) -> AppResult<ContactId> {
        let args = serde_json::json!([[{
            "name": contact.name,
            "email": contact.email,
            "ref": contact.member_ref,
        }]]);

        let result = self.call_kw("res.partner", "create", args).await?;

        let id = result
            .as_i64()
            .ok_or_else(|| AppError::Internal("odoo create_contact: unexpected response shape".into()))?;

        tracing::info!(odoo_partner_id = id, member_ref = contact.member_ref, "odoo contact created");
        Ok(ContactId(id.to_string()))
    }
}
