use std::path::PathBuf;

use thiserror::Error;
use tokio::task::JoinError;

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

    #[error("Unknown IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Rules file not found: {0}")]
    RulesFileNotFound(PathBuf),

    #[error("Load rules failed: {0:?}")]
    LoadRulesFailed(std::io::Error),

    #[error("Invalid state")]
    InvalidState,

    #[error("Unknown error: {0:?}")]
    Unknown(Option<String>),
}
