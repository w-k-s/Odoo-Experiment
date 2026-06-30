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
    pub member_ref: Option<&'a str>,
}

#[axum::async_trait]
pub trait Crm: Send + Sync {
    async fn create_contact(&self, contact: NewContact<'_>) -> AppResult<ContactId>;
    async fn update_contact_member_ref(
        &self,
        contact: &ContactId,
        member_ref: &str,
    ) -> AppResult<()>;
}
