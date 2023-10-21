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
