//! External-system integrations, each consumed through a role-based trait (the
//! *port*) so the rest of the app depends on the capability, not the vendor.
//! See `backend/CLAUDE.md` for the rule.

pub mod auth0;
pub mod crm;
pub mod identity;
pub mod odoo;

pub use crm::{Crm, NewContact};
pub use identity::IdentityProvider;
