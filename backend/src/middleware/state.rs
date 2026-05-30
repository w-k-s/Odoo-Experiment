use crate::loyalty_engine::services::members::MemberService;
use crate::loyalty_engine::services::programs::ProgramService;
use crate::loyalty_engine::services::sessions::SessionService;
use crate::middleware::integrations::odoo::Odoo;

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub programs: ProgramService,
    pub members: MemberService,
    pub sessions: SessionService,
    /// Odoo integration (middleware-owned; the engine doesn't know about it).
    pub odoo: Odoo,
    /// Program members enrol in when a request omits an explicit `program_id`.
    pub default_program_id: String,
}
