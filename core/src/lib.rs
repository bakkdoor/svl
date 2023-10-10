//! The `core` crate provides essential functionality for the application's core logic.
//! It encompasses modules for handling database operations, interacting with external APIs,
//! and performing various statistical computations and data manipulation.
//!
//! # Modules
//!
//! - `client`: Contains functionality for making HTTP requests and interacting with external APIs.
//! - `db`: Provides abstractions and utilities for managing database connections and executing queries.
//! - `queries`: Defines pre-defined queries and functions for executing database queries.
//! - `stats`: Handles statistical computations and manages data related to application statistics.
//! - `text`: Contains data structures and operations for handling text and word processing tasks.

pub mod client;
pub mod db;
pub mod errors;
pub mod queries;
pub mod stats;
pub mod text;

use errors::SVLError;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, SVLError>;

pub fn load_rules(root_path: Option<PathBuf>) -> Result<String> {
    let mut file_path = if let Some(path) = root_path {
        path
    } else {
        std::env::current_dir()?
    };

    file_path.push(std::path::Path::new("rules.datalog"));

    if !file_path.exists() {
        log::warn!("rules.datalog not found in: {:?}", file_path);
        return Err(SVLError::LoadRulesFailed(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "rules.datalog not found",
        )));
    }
    log::info!("Loading rules from: {:?}", file_path);
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
