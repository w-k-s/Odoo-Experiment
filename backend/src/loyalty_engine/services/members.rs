use diesel::prelude::*;

use crate::loyalty_engine::db::Pool;
use crate::loyalty_engine::error::EngineResult;
use crate::loyalty_engine::models::{Member, NewMember};

/// Loyalty members enrolled in a program.
#[derive(Clone)]
pub struct MemberService {
    pool: Pool,
}

impl MemberService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Look up a member by their Auth0 `email`, if one exists.
    pub async fn find_by_email(&self, email: &str) -> EngineResult<Option<Member>> {
        let conn = self.pool.get().await?;
        let email = email.to_string();
        let member = conn
            .interact(move |conn| {
                use crate::loyalty_engine::schema::loyalty_members::dsl as m;
                m::loyalty_members
                    .filter(m::email.eq(&email))
                    .select(Member::as_select())
                    .first::<Member>(conn)
                    .optional()
            })
            .await??;
        Ok(member)
    }

    /// Insert a member. `sub`/`external_contact_id` are optional so this serves
    /// both token-provisioned members and the admin/direct-create path.
    pub async fn create(
        &self,
        id: &str,
        name: &str,
        email: &str,
        external_contact_id: Option<&str>,
        program_id: &str,
    ) -> EngineResult<Member> {
        let new = NewMember {
            id: id.to_string(),
            program_id: program_id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            external_contact_id: external_contact_id.map(str::to_string),
        };

        let conn = self.pool.get().await?;
        let member = conn
            .interact(move |conn| {
                use crate::loyalty_engine::schema::loyalty_members::dsl::loyalty_members;
                diesel::insert_into(loyalty_members)
                    .values(&new)
                    .returning(Member::as_returning())
                    .get_result::<Member>(conn)
            })
            .await??;
        Ok(member)
    }
}
