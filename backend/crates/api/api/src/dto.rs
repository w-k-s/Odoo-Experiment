//! API request/response shapes (the web boundary). Engine entities are mapped
//! to/from these so domain types don't leak HTTP concerns.

use serde::{Deserialize, Serialize};

use loyalty::models::Member;

// ---------- Requests ----------

#[derive(Debug, Deserialize)]
pub struct CreateProgram {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMember {
    pub program_id: Option<String>,
    pub name: String,
    pub email: String,
}

// ---------- Responses ----------

#[derive(Debug, Serialize)]
pub struct SessionMember {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionDetail {
    pub session_id: String,
    pub status: String,
    pub member: SessionMember,
}

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
