pub mod client;
pub mod db;
pub mod queries;
pub mod stats;
pub mod text;

use std::path::PathBuf;

use thiserror::Error;
use tokio::task::JoinError;

pub type Result<T> = std::result::Result<T, SVLError>;

#[derive(Error, Debug)]
pub enum SVLError {
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("task join error: {0}")]
    TaskJoin(#[from] JoinError),

    #[error("failed to acquire semaphore permit: {0}")]
    SemaphoreAcquire(#[from] tokio::sync::AcquireError),

    #[error("IO error: {0:?}")]
    IOError(#[from] std::io::Error),

    #[error("Invalid state")]
    InvalidState,

    #[error("Unknown error: {0:?}")]
    Unknown(Option<String>),
}

pub fn load_rules(root_path: Option<PathBuf>) -> Result<String> {
    let mut file_path = if let Some(path) = root_path {
        path
    } else {
        std::env::current_dir()?
    };

    file_path.push(std::path::Path::new("rules.datalog"));

    if !file_path.exists() {
        println!("rules.datalog not found in: {:?}", file_path);
        return Err(SVLError::IOError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "rules.datalog not found",
        )));
    }
    println!("Loading rules from: {:?}", file_path);
    let rules = std::fs::read_to_string(file_path)?;
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rules() {
        let mut root_path = std::env::current_dir().unwrap();
        root_path.pop();
        let rules = load_rules(Some(root_path)).unwrap();
        assert!(rules.len() > 0);
    }
}
