use diesel::prelude::*;

use crate::db::Pool;
use crate::error::EngineResult;
use crate::models::{Member, NewMember, ProgramId};

#[derive(Clone)]
pub struct MemberService {
    pool: Pool,
}

impl MemberService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> EngineResult<Option<Member>> {
        let conn = self.pool.get().await?;
        let email = email.to_string();
        let member = conn
            .interact(move |conn| {
                use crate::schema::loyalty_members::dsl as m;
                m::loyalty_members
                    .filter(m::email.eq(&email))
                    .select(Member::as_select())
                    .first::<Member>(conn)
                    .optional()
            })
            .await??;
        Ok(member)
    }

    pub async fn create(
        &self,
        name: &str,
        email: &str,
        external_contact_id: Option<&str>,
        program_id: &ProgramId,
    ) -> EngineResult<Member> {
        let new = NewMember {
            program_id: program_id.clone(),
            name: name.to_string(),
            email: email.to_string(),
            external_contact_id: external_contact_id.map(str::to_string),
            ..Default::default()
        };

        let conn = self.pool.get().await?;
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
}
