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

pub enum LoadRulesFrom {
    DefaultInCurrentDir,
    DefaultInDir(PathBuf),
    File(PathBuf),
}

impl LoadRulesFrom {
    const DEFAULT_RULES_FILE: &'static str = "rules.datalog";

    pub fn path(self) -> Result<PathBuf> {
        match self {
            LoadRulesFrom::DefaultInCurrentDir => {
                let mut path = std::env::current_dir()?;
                path.push(Self::DEFAULT_RULES_FILE);
                Ok(path)
            }
            LoadRulesFrom::DefaultInDir(path) => {
                let mut path = path;
                path.push(Self::DEFAULT_RULES_FILE);
                Ok(path)
            }
            LoadRulesFrom::File(path) => Ok(path),
        }
    }
}

pub fn load_rules(lrf: LoadRulesFrom) -> Result<String> {
    let file_path = lrf.path()?;

    if !file_path.exists() {
        log::warn!("rules.datalog not found in: {:?}", file_path);
        return Err(SVLError::RulesFileNotFound(file_path));
    }
    log::info!("Loading rules from: {:?}", file_path);
    let rules = std::fs::read_to_string(file_path).map_err(SVLError::LoadRulesFailed)?;
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rules() {
        let mut root_path = std::env::current_dir().unwrap();
        root_path.pop();
        let rules = load_rules(LoadRulesFrom::DefaultInDir(root_path)).unwrap();
        assert!(rules.len() > 0);
    }
}
