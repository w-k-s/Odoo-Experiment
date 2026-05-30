//! Domain error type for the loyalty engine. Deliberately free of any HTTP /
//! web framework concepts — the middleware maps these into responses.

#[derive(Debug)]
pub enum EngineError {
    NotFound(String),
    /// Anything from the datastore (pool, interact join, diesel).
    Db(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::NotFound(m) => write!(f, "{m}"),
            EngineError::Db(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<diesel::result::Error> for EngineError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => EngineError::NotFound("not found".into()),
            other => EngineError::Db(format!("db error: {other}")),
        }
    }
}

impl From<deadpool_diesel::PoolError> for EngineError {
    fn from(e: deadpool_diesel::PoolError) -> Self {
        EngineError::Db(format!("pool error: {e}"))
    }
}

impl From<deadpool_diesel::InteractError> for EngineError {
    fn from(e: deadpool_diesel::InteractError) -> Self {
        EngineError::Db(format!("interact error: {e}"))
    }
}

pub type EngineResult<T> = Result<T, EngineError>;
