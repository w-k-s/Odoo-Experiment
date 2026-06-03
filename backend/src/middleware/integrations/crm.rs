//! CRM port: the capability the middleware needs from a customer-relationship
//! system, abstracted over the concrete vendor (Odoo today — see `odoo.rs`).

use crate::middleware::error::AppResult;

/// CRM-side contact id, opaque to the app. Odoo stringifies its `res.partner`
/// id; another CRM might use a UUID. Wrapping it keeps the vendor's id type out
/// of the abstraction.
#[derive(Debug, Clone)]
pub struct ContactId(pub String);

/// A contact to create in the CRM. `member_ref` is the loyalty member id; the
/// implementation stores it so the CRM record links back to the member (the
/// Odoo impl puts it in the partner's `ref`).
pub struct NewContact<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub member_ref: &'a str,
}

/// The CRM the middleware provisions contacts into.
#[axum::async_trait]
pub trait Crm: Send + Sync {
    /// Create a contact and return its CRM-side id.
    async fn create_contact(&self, contact: NewContact<'_>) -> AppResult<ContactId>;
}
