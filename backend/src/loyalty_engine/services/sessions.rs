use chrono::{Duration, Utc};
use diesel::prelude::*;

use crate::loyalty_engine::db::Pool;
use crate::loyalty_engine::error::{EngineError, EngineResult};
use crate::loyalty_engine::ids::new_session_code;
use crate::loyalty_engine::models::{Member, NewSession, OwnedSession, Session};

/// Sessions live for 24h before a fresh one must be minted.
const SESSION_TTL_HOURS: i64 = 24;

/// The short-lived codes a member presents at the till.
#[derive(Clone)]
pub struct SessionService {
    pool: Pool,
}

impl SessionService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Mint a new session code for a member.
    pub async fn create(&self, member_id: String) -> EngineResult<Session> {
        let new = NewSession {
            id: new_session_code(),
            member_id,
            expires_at: Some(Utc::now() + Duration::hours(SESSION_TTL_HOURS)),
        };

        let conn = self.pool.get().await?;
        let session = conn
            .interact(move |conn| {
                use crate::loyalty_engine::schema::loyalty_sessions::dsl::loyalty_sessions;
                diesel::insert_into(loyalty_sessions)
                    .values(&new)
                    .returning(Session::as_returning())
                    .get_result::<Session>(conn)
            })
            .await??;
        Ok(session)
    }

    /// Resolve a session the given member owns, with its effective status.
    /// Returns `NotFound` when the session is missing OR owned by someone else
    /// (so we never reveal which codes exist).
    pub async fn get_owned(&self, id: String, member_id: String) -> EngineResult<OwnedSession> {
        let conn = self.pool.get().await?;
        let owned = conn
            .interact(move |conn| -> EngineResult<OwnedSession> {
                use crate::loyalty_engine::schema::loyalty_members::dsl as m;
                use crate::loyalty_engine::schema::loyalty_sessions::dsl as s;

                let session = s::loyalty_sessions
                    .filter(s::id.eq(&id))
                    .select(Session::as_select())
                    .first::<Session>(conn)
                    .optional()?
                    .filter(|session| session.member_id == member_id)
                    .ok_or_else(|| EngineError::NotFound(format!("session {id} not found")))?;

                let member = m::loyalty_members
                    .filter(m::id.eq(&session.member_id))
                    .select(Member::as_select())
                    .first::<Member>(conn)?;

                let status = match session.expires_at {
                    Some(expires_at) if expires_at < Utc::now() => "expired".to_string(),
                    _ => session.status,
                };

                Ok(OwnedSession {
                    session_id: session.id,
                    status,
                    member,
                })
            })
            .await??;
        Ok(owned)
    }
}
