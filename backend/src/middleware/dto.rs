//! API request/response shapes (the web boundary). Engine entities are mapped
//! to/from these so domain types don't leak HTTP concerns.

use serde::{Deserialize, Serialize};

use crate::loyalty_engine::models::Member;

// ---------- Requests ----------

#[derive(Debug, Deserialize)]
pub struct CreateProgram {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMember {
    /// Optional: defaults to the bootstrapped program when omitted.
    pub program_id: Option<String>,
    pub name: String,
    pub email: Option<String>,
}

// ---------- Responses ----------

/// Member subset embedded in a session lookup (what the POS consumes).
#[derive(Debug, Serialize)]
pub struct SessionMember {
    pub id: String,
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
