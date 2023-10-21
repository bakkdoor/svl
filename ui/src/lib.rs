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
mod query;
mod search;

use app::App;
use iced::{Application, Settings};
use svl_core::db::DBConnection;

pub fn run_ui(db: DBConnection) -> iced::Result {
    App::run(Settings::with_flags(app::Args { db }))
}
