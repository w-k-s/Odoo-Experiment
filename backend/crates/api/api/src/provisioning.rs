//! Member provisioning orchestration — the one place the web layer stitches the
//! CRM integration to the engine, keeping the engine free of any contacts port.

use api_utils::state::AppState;
use crm::NewContact;
use loyalty::NewMember;
use loyalty::ids::new_id;
use loyalty::models::Member;
use utils::error::{AppError, AppResult};

pub async fn ensure_member(state: &AppState, sub: &str) -> AppResult<Member> {
    let profile = state.identity.fetch_profile(sub).await?;
    let name = profile.name.unwrap_or_else(|| "Member".to_string());
    let email = profile
        .email
        .ok_or_else(|| AppError::Internal(format!("identity profile for {sub} has no email")))?;

    if let Some(existing) = state.loyalty.find_member_by_email(&email).await? {
        return Ok(existing);
    }

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
        .loyalty
        .create_member(NewMember {
            id: &member_id,
            name: &name,
            email: &email,
            external_contact_id: Some(&contact.0),
            program_id: &state.default_program_id,
        })
        .await?;
    Ok(member)
}
