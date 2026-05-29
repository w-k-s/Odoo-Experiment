use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{loyalty_members, loyalty_programs, loyalty_sessions};

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

#[derive(Debug, Deserialize)]
pub struct CreateProgram {
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

#[derive(Debug, Deserialize)]
pub struct CreateMember {
    /// Optional: defaults to the bootstrapped program when omitted.
    pub program_id: Option<String>,
    pub name: String,
    pub email: Option<String>,
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

// ---------- API response shapes ----------

/// Member subset embedded in a session lookup (what the POS consumes).
#[derive(Debug, Serialize)]
pub struct SessionMember {
    pub name: String,
    pub email: Option<String>,
}

/// Response for `GET /loyalty/sessions/{id}`.
#[derive(Debug, Serialize)]
pub struct SessionDetail {
    pub session_id: String,
    pub status: String,
    pub member: SessionMember,
}

/// Response for `GET /loyalty/me` — the authenticated member's profile + balance.
#[derive(Debug, Serialize)]
pub struct MemberProfile {
    pub member_id: String,
    pub name: String,
    pub email: Option<String>,
    pub points: i32,
}

impl From<Member> for MemberProfile {
    fn from(m: Member) -> Self {
        Self {
            member_id: m.id,
            name: m.name,
            email: m.email,
            points: m.points,
        }
    }
}
