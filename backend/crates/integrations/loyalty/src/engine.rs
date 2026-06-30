//! Concrete `LoyaltyEngine` implementation backed by the Diesel/Postgres
//! loyalty engine services.

use crate::db::Pool;
use crate::error::EngineResult;
use crate::members::MemberService;
use crate::models::{Member, MemberId, OwnedSession, Program, Session, SessionId};
use crate::programs::ProgramService;
use crate::sessions::SessionService;

use crate::{LoyaltyEngine, NewMember};

/// Wraps the three engine services behind the `LoyaltyEngine` trait.
#[derive(Clone)]
pub struct DbLoyaltyEngine {
    members: MemberService,
    programs: ProgramService,
    sessions: SessionService,
}

impl DbLoyaltyEngine {
    pub fn new(pool: Pool) -> Self {
        Self {
            members: MemberService::new(pool.clone()),
            programs: ProgramService::new(pool.clone()),
            sessions: SessionService::new(pool.clone()),
        }
    }
}

#[axum::async_trait]
impl LoyaltyEngine for DbLoyaltyEngine {
    async fn find_member_by_email(&self, email: &str) -> EngineResult<Option<Member>> {
        self.members.find_by_email(email).await
    }

    async fn create_member(&self, m: NewMember<'_>) -> EngineResult<Member> {
        self.members
            .create(m.name, m.email, m.external_contact_id, m.program_id)
            .await
    }

    async fn create_program(&self, name: String) -> EngineResult<Program> {
        self.programs.create(name).await
    }

    async fn ensure_default_program(&self, name: &str) -> EngineResult<String> {
        self.programs.ensure_default(name).await
    }

    async fn create_session(&self, member_id: MemberId) -> EngineResult<Session> {
        self.sessions.create(member_id).await
    }

    async fn get_owned_session(&self, id: SessionId) -> EngineResult<OwnedSession> {
        self.sessions.get_owned(id).await
    }
}
