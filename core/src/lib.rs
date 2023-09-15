pub mod client;
pub mod stats;
pub mod text;

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
