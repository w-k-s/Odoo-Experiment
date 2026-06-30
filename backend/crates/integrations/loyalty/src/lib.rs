pub mod db;
pub mod engine;
pub mod error;
mod ids;
pub mod members;
pub mod models;
pub mod programs;
pub mod schema;
pub mod sessions;
pub mod transactions;

use error::EngineResult;
use models::{Member, OwnedSession, Program, Session};

use crate::models::{MemberId, ProgramId, SessionId};

pub struct NewMember<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub external_contact_id: Option<&'a str>,

    pub program_id: &'a ProgramId,
}

#[axum::async_trait]
pub trait LoyaltyEngine: Send + Sync {
    async fn find_member_by_email(&self, email: &str) -> EngineResult<Option<Member>>;
    async fn create_member(&self, member: NewMember<'_>) -> EngineResult<Member>;
    async fn create_program(&self, name: String) -> EngineResult<Program>;
    async fn ensure_default_program(&self, name: &str) -> EngineResult<String>;
    async fn create_session(&self, member_id: MemberId) -> EngineResult<Session>;
    async fn get_owned_session(&self, id: SessionId) -> EngineResult<OwnedSession>;
}
