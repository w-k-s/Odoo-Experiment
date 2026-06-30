use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use derive_more::Display;
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use serde::Serialize;

use crate::ids::new_id;
use crate::schema::{loyalty_members, loyalty_programs, loyalty_sessions, points_transactions};

#[derive(Serialize, Debug, Clone, DieselNewType, Display)]
pub struct ProgramId(String);

impl Default for ProgramId {
    fn default() -> Self {
        ProgramId(new_id("prog"))
    }
}

impl From<String> for ProgramId {
    fn from(s: String) -> Self {
        ProgramId(s)
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = loyalty_programs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Program {
    pub id: ProgramId,
    pub name: String,
    pub points_per_currency_minor_unit: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, Default)]
#[diesel(table_name = loyalty_programs)]
pub struct NewProgram {
    pub id: ProgramId,
    pub name: String,
}

#[derive(Serialize, Debug, Clone, DieselNewType, Display)]
pub struct TransactionId(String);

impl Default for TransactionId {
    fn default() -> Self {
        TransactionId(new_id("txn"))
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name= points_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PointsTransaction {
    pub id: TransactionId,
    pub member_id: MemberId,
    pub program_id: ProgramId,
    pub source_system: String,
    pub source_order: String,
    pub delta: i32,
    pub amount_total: BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, Default)]
#[diesel(table_name = points_transactions)]
pub struct NewTransaction {
    pub id: TransactionId,
    pub member_id: MemberId,
    pub program_id: ProgramId,
    pub source_system: String,
    pub source_order: String,
    pub delta: i32,
    pub amount_total: BigDecimal,
}

#[derive(Serialize, Debug, Clone, DieselNewType, Display)]
pub struct MemberId(String);

impl Default for MemberId {
    fn default() -> Self {
        MemberId(new_id("m"))
    }
}

impl From<MemberId> for String {
    fn from(val: MemberId) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = loyalty_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
    pub id: MemberId,
    pub program_id: ProgramId,
    pub name: String,
    pub email: Option<String>,
    pub external_contact_id: Option<String>,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, Default)]
#[diesel(table_name = loyalty_members)]
pub struct NewMember {
    pub id: MemberId,
    pub program_id: ProgramId,
    pub name: String,
    pub email: String,
    pub external_contact_id: Option<String>,
}

#[derive(Serialize, Clone, Debug, DieselNewType, Display)]
pub struct SessionId(String);

impl Default for SessionId {
    fn default() -> Self {
        SessionId(new_id("sess"))
    }
}

impl From<SessionId> for String {
    fn from(val: SessionId) -> Self {
        val.0
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        SessionId(s)
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = loyalty_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: SessionId,
    pub member_id: MemberId,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable, Default)]
#[diesel(table_name = loyalty_sessions)]
pub struct NewSession {
    pub id: SessionId,
    pub member_id: MemberId,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct OwnedSession {
    pub session_id: SessionId,
    pub status: String,
    pub member: Member,
}
