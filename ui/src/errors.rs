use std::fmt::Display;

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

    #[error("Invalid type for {0} - Expected {1}")]
    InvalidType(String, ExpectedType),
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedType {
    Boolean,
    Float,
    Integer,
    String,
    Usize,
}

impl Display for ExpectedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedType::Boolean => write!(f, "Boolean"),
            ExpectedType::Float => write!(f, "Float"),
            ExpectedType::Integer => write!(f, "Integer"),
            ExpectedType::String => write!(f, "String"),
            ExpectedType::Usize => write!(f, "Usize"),
        }
    }
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

    pub fn invalid_type<S: ToString>(property: S, expected_type: ExpectedType) -> Self {
        Self::InvalidType(property.to_string(), expected_type)
    }
}
