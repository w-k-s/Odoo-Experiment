//! Loyalty port: the capability the middleware needs from the loyalty backend,
//! abstracted over the concrete implementation (Diesel/Postgres today — see
//! `engine.rs`).

pub mod db;
pub mod engine;
pub mod error;
pub mod ids;
pub mod members;
pub mod models;
pub mod programs;
pub mod schema;
pub mod sessions;
pub mod transactions;

use error::EngineResult;
use models::{Member, OwnedSession, Program, Session};

/// Input for member creation. `external_contact_id` is the CRM-side id minted
/// before calling this, so the member row links back to the CRM contact.
pub struct NewMember<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub email: &'a str,
    pub external_contact_id: Option<&'a str>,
    pub program_id: &'a str,
}

/// The loyalty backend the middleware drives.
#[axum::async_trait]
pub trait LoyaltyEngine: Send + Sync {
    // --- members ---

    /// Look up an existing member by email, if one exists.
    async fn find_member_by_email(&self, email: &str) -> EngineResult<Option<Member>>;

    /// Enrol a new member into a program.
    async fn create_member(&self, member: NewMember<'_>) -> EngineResult<Member>;

    // --- programs ---

    /// Create a new loyalty program.
    async fn create_program(&self, name: String) -> EngineResult<Program>;

    /// Ensure a default program exists (matched by name) and return its id.
    /// Idempotent — safe to call at startup.
    async fn ensure_default_program(&self, name: &str) -> EngineResult<String>;

    // --- sessions ---

    /// Mint a new short-lived session code for a member.
    async fn create_session(&self, member_id: String) -> EngineResult<Session>;

    /// Resolve a session to its owning member. Returns `NotFound` for missing
    /// or unowned sessions (we never reveal which codes exist).
    async fn get_owned_session(&self, id: String) -> EngineResult<OwnedSession>;
}
