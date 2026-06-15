use deadpool_diesel::{InteractError, PoolError};
use diesel::result::Error as DieselError;

#[derive(Debug)]
pub enum EngineError {
    NotFound(String),
    Db(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::NotFound(m) => write!(f, "not found: {m}"),
            EngineError::Db(m) => write!(f, "database error: {m}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<PoolError> for EngineError {
    fn from(e: PoolError) -> Self {
        EngineError::Db(format!("pool error: {e}"))
    }
}

impl From<InteractError> for EngineError {
    fn from(e: InteractError) -> Self {
        EngineError::Db(format!("interact error: {e}"))
    }
}

impl From<DieselError> for EngineError {
    fn from(e: DieselError) -> Self {
        match e {
            DieselError::NotFound => EngineError::NotFound("record not found".to_string()),
            _ => EngineError::Db(format!("diesel error: {e}")),
        }
    }
}

pub type EngineResult<T> = Result<T, EngineError>;
