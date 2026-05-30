use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;

use crate::loyalty_engine::schema::{loyalty_members, loyalty_programs, loyalty_sessions};

// ---------- Programs ----------

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = loyalty_programs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Program {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = loyalty_programs)]
pub struct NewProgram {
    pub id: String,
    pub name: String,
}

// ---------- Members ----------

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = loyalty_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
    pub id: String,
    pub program_id: String,
    pub name: String,
    pub email: Option<String>,
    pub auth0_sub: Option<String>,
    pub external_contact_id: Option<i32>,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = loyalty_members)]
pub struct NewMember {
    pub id: String,
    pub program_id: String,
    pub name: String,
    pub email: Option<String>,
    pub auth0_sub: Option<String>,
    pub external_contact_id: Option<i32>,
}

// ---------- Sessions ----------

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = loyalty_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub member_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = loyalty_sessions)]
pub struct NewSession {
    pub id: String,
    pub member_id: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// A session resolved together with its owning member and effective status.
/// Returned by `SessionService::get_owned`; the middleware maps it to its DTO.
#[derive(Debug)]
pub struct OwnedSession {
    pub session_id: String,
    pub status: String,
    pub member: Member,
}
