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

    /// Look up a member by their Auth0 `sub`, if one exists.
    pub async fn find_by_sub(&self, sub: &str) -> EngineResult<Option<Member>> {
        let conn = self.pool.get().await?;
        let sub = sub.to_string();
        let member = conn
            .interact(move |conn| {
                use crate::loyalty_engine::schema::loyalty_members::dsl as m;
                m::loyalty_members
                    .filter(m::auth0_sub.eq(&sub))
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
        sub: Option<&str>,
        name: &str,
        email: Option<&str>,
        external_contact_id: Option<i32>,
        program_id: &str,
    ) -> EngineResult<Member> {
        let new = NewMember {
            id: id.to_string(),
            program_id: program_id.to_string(),
            name: name.to_string(),
            email: email.map(str::to_string),
            auth0_sub: sub.map(str::to_string),
            external_contact_id,
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
