//! Minimal Odoo integration over the `odoo-api` JSON-RPC client.
//!
//! We authenticate lazily (Odoo may not be up when the backend boots) using a
//! dedicated bot user + API key, cache the authenticated session, and expose
//! just the one call Phase 4 needs: creating a `res.partner`.

use std::sync::Arc;

use odoo_api::client::{Authed, ReqwestAsync};
use odoo_api::OdooClient;
use serde_json::{json, Map, Value};
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};

/// An authenticated async Odoo session.
type OdooSession = OdooClient<Authed, ReqwestAsync>;

/// Connection settings for the Odoo JSON-RPC endpoint.
#[derive(Debug, Clone)]
pub struct OdooConfig {
    pub url: String,
    pub db: String,
    pub login: String,
    /// API key minted for the bot user (used in place of a password).
    pub api_key: String,
}

/// Lazily-authenticated Odoo client, cheap to `clone` (shared session).
#[derive(Clone)]
pub struct Odoo {
    config: OdooConfig,
    session: Arc<Mutex<Option<OdooSession>>>,
}

impl Odoo {
    pub fn new(config: OdooConfig) -> Self {
        Self {
            config,
            session: Arc::new(Mutex::new(None)),
        }
    }

    /// Ensure the cached session is authenticated, (re)authenticating if absent.
    async fn ensure_session(&self, slot: &mut Option<OdooSession>) -> AppResult<()> {
        if slot.is_some() {
            return Ok(());
        }
        let client = OdooClient::new_reqwest_async(&self.config.url)
            .map_err(|e| AppError::Internal(format!("odoo connect failed: {e}")))?
            .authenticate(&self.config.db, &self.config.login, &self.config.api_key)
            .await
            .map_err(|e| AppError::Internal(format!("odoo authentication failed: {e}")))?;
        *slot = Some(client);
        Ok(())
    }

    /// Create a `res.partner` and return its Odoo id.
    pub async fn create_partner(&self, name: &str, email: Option<&str>) -> AppResult<i32> {
        let mut slot = self.session.lock().await;
        self.ensure_session(&mut slot).await?;
        let client = slot.as_mut().expect("session ensured above");

        let mut fields = Map::new();
        fields.insert("name".into(), json!(name));
        if let Some(email) = email {
            fields.insert("email".into(), json!(email));
        }
        let args: Vec<Value> = vec![Value::Object(fields)];

        let resp = client
            .execute_kw("res.partner", "create", args, Map::new())
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("odoo create_partner failed: {e}")))?;

        resp.data.as_i64().map(|id| id as i32).ok_or_else(|| {
            AppError::Internal(format!(
                "odoo create_partner: unexpected response {:?}",
                resp.data
            ))
        })
    }
}
