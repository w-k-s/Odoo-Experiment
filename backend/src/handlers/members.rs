use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;

use crate::error::AppResult;
use crate::ids::new_id;
use crate::models::{CreateMember, Member, NewMember};
use crate::AppState;

/// `POST /loyalty/members`
///
/// Admin/testing endpoint: registers a member directly (no Auth0 identity).
/// When `program_id` is omitted the member is enrolled in the default program.
pub async fn create_member(
    State(state): State<AppState>,
    Json(body): Json<CreateMember>,
) -> AppResult<(StatusCode, Json<Member>)> {
    let program_id = body
        .program_id
        .unwrap_or_else(|| state.default_program_id.clone());

    let conn = state.pool.get().await?;
    let member = conn
        .interact(move |conn| {
            use crate::schema::loyalty_members::dsl::loyalty_members;
            let new = NewMember {
                id: new_id("mem"),
                program_id,
                name: body.name,
                email: body.email,
                auth0_sub: None,
                external_contact_id: None,
            };
            diesel::insert_into(loyalty_members)
                .values(&new)
                .returning(Member::as_returning())
                .get_result::<Member>(conn)
        })
        .await??;

    Ok((StatusCode::CREATED, Json(member)))
}

/// Look up a member by their Auth0 `sub`, if one exists.
pub async fn find_member_by_sub(state: &AppState, sub: &str) -> AppResult<Option<Member>> {
    let conn = state.pool.get().await?;
    let sub = sub.to_string();
    let member = conn
        .interact(move |conn| {
            use crate::schema::loyalty_members::dsl as m;
            m::loyalty_members
                .filter(m::auth0_sub.eq(&sub))
                .select(Member::as_select())
                .first::<Member>(conn)
                .optional()
        })
        .await??;
    Ok(member)
}

/// Resolve the member for an authenticated caller, provisioning on first sight.
///
/// On first sight we create the Odoo `res.partner`, then insert a member row in
/// the default program linked to both the Auth0 `sub` and the Odoo partner id.
pub async fn ensure_member(
    state: &AppState,
    sub: &str,
    name: &str,
    email: Option<&str>,
) -> AppResult<Member> {
    if let Some(existing) = find_member_by_sub(state, sub).await? {
        return Ok(existing);
    }

    let external_contact_id = state.odoo.create_partner(name, email).await?;

    let new = NewMember {
        id: new_id("mem"),
        program_id: state.default_program_id.clone(),
        name: name.to_string(),
        email: email.map(str::to_string),
        auth0_sub: Some(sub.to_string()),
        external_contact_id: Some(external_contact_id),
    };

    let conn = state.pool.get().await?;
    let member = conn
        .interact(move |conn| {
            use crate::schema::loyalty_members::dsl::loyalty_members;
            diesel::insert_into(loyalty_members)
                .values(&new)
                .returning(Member::as_returning())
                .get_result::<Member>(conn)
        })
        .await??;

    Ok(member)
}
