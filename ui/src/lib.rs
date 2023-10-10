//! The `ui` crate provides the user interface components and logic for the application.
//! It is responsible for rendering the user interface, handling user interactions, and
//! communicating with the core functionality provided by other crates.
//!
//! # Modules
//!
//! - `app`: Contains the main application logic and state management.
//! - `errors`: Defines error types and utilities for handling UI-specific errors.
//! - `message`: Defines the message passing mechanism between UI components.
//! - `search`: Implements search-related functionality, including search states and views.

mod app;
mod errors;
mod message;
mod search;

use app::App;
use iced::{Application, Settings};
use search::{SearchKind, SearchResult, SearchRows};
use svl_core::db::{DBConnection, DBParams};

pub fn run_ui() -> iced::Result {
    App::run(Settings::default())
}

#[allow(dead_code)]
async fn search_authors(db: &DBConnection, term: &str) -> SearchResult {
    let script = r#"
    ?[author_id,name,url] :=
        *Author { name, url },
        str_include(name, $name)
    "#;
    let params = DBParams::from_iter(vec![("name".into(), term.into())]);
    let rows = db.run_immutable(script, params).await?;
    Ok(SearchRows::new(SearchKind::Author, rows))
}
