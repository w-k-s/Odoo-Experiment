//! Member provisioning orchestration — the one place the web layer stitches the
//! CRM integration to the engine, keeping the engine free of any contacts port.

use crate::loyalty_engine::ids::new_id;
use crate::loyalty_engine::models::Member;
use crate::middleware::error::{AppError, AppResult};
use crate::middleware::integrations::NewContact;
use crate::middleware::state::AppState;

/// Resolve the member for an authenticated caller, provisioning on first sight:
/// look up by `sub`; if absent, resolve the profile from the identity provider
/// (access tokens don't carry `name`/`email`), create the CRM contact, then the
/// member row.
pub async fn ensure_member(state: &AppState, sub: &str) -> AppResult<Member> {
    let profile = state.identity.fetch_profile(sub).await?;
    let name = profile.name.unwrap_or_else(|| "Member".to_string());
    let email = profile
        .email
        .ok_or_else(|| AppError::Internal(format!("identity profile for {sub} has no email")))?;

    if let Some(existing) = state.members.find_by_email(&email).await? {
        return Ok(existing);
    }

    // Generate the member id up front so we can stamp it onto the CRM contact's
    // `member_ref`, linking the contact back to the loyalty member.
    let member_id = new_id("mem");
    let contact = state
        .crm
        .create_contact(NewContact {
            name: &name,
            email: &email,
            member_ref: &member_id,
        })
        .await?;
    let member = state
        .members
        .create(
            &member_id,
            &name,
            &email,
            Some(&contact.0),
            &state.default_program_id,
        )
        .await?;
    Ok(member)
}
