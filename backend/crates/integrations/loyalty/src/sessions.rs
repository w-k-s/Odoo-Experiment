use chrono::{Duration, Utc};
use diesel::prelude::*;

use crate::db::Pool;
use crate::error::{EngineError, EngineResult};
use crate::models::{Member, MemberId, NewSession, OwnedSession, Session, SessionId};

const SESSION_TTL_HOURS: i64 = 24;

#[derive(Clone)]
pub struct SessionService {
    pool: Pool,
}

impl SessionService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, member_id: MemberId) -> EngineResult<Session> {
        let new = NewSession {
            member_id,
            expires_at: Some(Utc::now() + Duration::hours(SESSION_TTL_HOURS)),
            ..Default::default()
        };

        let conn = self.pool.get().await?;
        let session = conn
            .interact(move |conn| {
                use crate::schema::loyalty_sessions::dsl::loyalty_sessions;
                diesel::insert_into(loyalty_sessions)
                    .values(&new)
                    .returning(Session::as_returning())
                    .get_result::<Session>(conn)
            })
            .await??;
        Ok(session)
    }

    pub async fn get_owned(&self, id: SessionId) -> EngineResult<OwnedSession> {
        let conn = self.pool.get().await?;
        let owned = conn
            .interact(move |conn| -> EngineResult<OwnedSession> {
                use crate::schema::loyalty_members::dsl as m;
                use crate::schema::loyalty_sessions::dsl as s;

                let session = s::loyalty_sessions
                    .filter(s::id.eq(&id))
                    .select(Session::as_select())
                    .first::<Session>(conn)
                    .optional()?
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
