//! Member provisioning orchestration — the one place the web layer stitches the
//! CRM integration to the engine, keeping the engine free of any contacts port.

use api_utils::state::AppState;
use crm::NewContact;
use loyalty::NewMember;
use loyalty::models::{Member, ProgramId};
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

    let program_id: ProgramId = ProgramId::from(state.default_program_id.clone());
    let contact = state
        .crm
        .create_contact(NewContact {
            name: &name,
            email: &email,
            member_ref: None,
        })
        .await?;
    let member = state
        .loyalty
        .create_member(NewMember {
            name: &name,
            email: &email,
            external_contact_id: Some(&contact.0),
            program_id: &program_id,
        })
        .await?;
    state
        .crm
        .update_contact_member_ref(&contact, &member.id.to_string())
        .await?;
    Ok(member)
}
