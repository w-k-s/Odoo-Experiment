use std::sync::Arc;

use auth::IdentityProvider;
use crm::Crm;
use loyalty::LoyaltyEngine;

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub loyalty: Arc<dyn LoyaltyEngine>,
    /// CRM the middleware provisions contacts into (Odoo today).
    pub crm: Arc<dyn Crm>,
    /// Identity provider resolving a member's profile on first sight (Auth0 today).
    pub identity: Arc<dyn IdentityProvider>,
    /// Program members enrol in when a request omits an explicit `program_id`.
    pub default_program_id: String,
}
