use std::sync::Arc;

use crate::loyalty_engine::services::members::MemberService;
use crate::loyalty_engine::services::programs::ProgramService;
use crate::loyalty_engine::services::sessions::SessionService;
use crate::middleware::integrations::{Crm, IdentityProvider};

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub programs: ProgramService,
    pub members: MemberService,
    pub sessions: SessionService,
    /// CRM the middleware provisions contacts into (Odoo today; the engine
    /// doesn't know about it).
    pub crm: Arc<dyn Crm>,
    /// Identity provider resolving a member's profile on first sight (Auth0 today).
    pub identity: Arc<dyn IdentityProvider>,
    /// Program members enrol in when a request omits an explicit `program_id`.
    pub default_program_id: String,
}
