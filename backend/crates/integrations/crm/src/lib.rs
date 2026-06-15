//! CRM port: creating contacts in an external CRM, abstracted over the
//! concrete implementation (Odoo today — see `odoo.rs`).

pub mod odoo;

use utils::error::AppResult;

/// Opaque CRM contact identifier (the provider-side id as a string).
pub struct ContactId(pub String);

/// Input for contact creation.
pub struct NewContact<'a> {
    pub name: &'a str,
    pub email: &'a str,
    /// Loyalty member reference to store on the CRM contact.
    pub member_ref: &'a str,
}

#[axum::async_trait]
pub trait Crm: Send + Sync {
    async fn create_contact(&self, contact: NewContact<'_>) -> AppResult<ContactId>;
}
