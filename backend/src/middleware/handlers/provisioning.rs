//! Member provisioning orchestration — the one place the web layer stitches the
//! Odoo integration to the engine, keeping the engine free of any contacts port.

use crate::loyalty_engine::ids::new_id;
use crate::loyalty_engine::models::Member;
use crate::middleware::error::AppResult;
use crate::middleware::state::AppState;

/// Resolve the member for an authenticated caller, provisioning on first sight:
/// look up by `sub`; if absent, create the Odoo contact, then the member row.
pub async fn ensure_member(
    state: &AppState,
    sub: &str,
    name: &str,
    email: Option<&str>,
) -> AppResult<Member> {
    if let Some(existing) = state.members.find_by_sub(sub).await? {
        return Ok(existing);
    }

    // Generate the member id up front so we can stamp it onto the Odoo
    // contact's `ref`, linking the partner back to the loyalty member.
    let member_id = new_id("mem");
    let contact_id = state.odoo.create_contact(name, email, &member_id).await?;
    let member = state
        .members
        .create(
            &member_id,
            Some(sub),
            name,
            email,
            Some(contact_id),
            &state.default_program_id,
        )
        .await?;
    Ok(member)
}
