use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Clone, Error)]
pub enum SearchError {
    #[error("DBError: {0}")]
    Db(String),

    #[error("Missing column: {0}")]
    MissingColumn(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl SearchError {
    pub fn db<S: ToString>(err: S) -> Self {
        Self::Db(err.to_string())
    }

    pub fn missing_column<S: ToString>(err: S) -> Self {
        Self::MissingColumn(err.to_string())
    }

    pub fn other<S: ToString>(err: S) -> Self {
        Self::Other(err.to_string())
    }
}
