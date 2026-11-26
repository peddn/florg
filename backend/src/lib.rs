//! florg backend library.
//!
//! The binary in `main.rs` only wires configuration, database and router
//! together. Everything else lives here so integration tests can use it.

pub mod auth;
pub mod calendar;
pub mod cli;
pub mod config;
pub mod db;
pub mod demo_data;
pub mod entity;
pub mod error;
pub mod mail;
pub mod migration;
pub mod routes;
pub mod school_class;
pub mod school_year;
pub mod student;
pub mod student_import;
pub mod subject;
pub mod validate;

use sea_orm::DatabaseConnection;
use tokio_util::task::TaskTracker;

use crate::mail::Mailer;

/// Shared state handed to every request handler.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub mailer: Mailer,
    /// Public address of the application, without a trailing slash.
    pub public_url: String,
    /// Work that continues after a response has been sent, such as sending
    /// a login mail. The server waits for it when shutting down.
    pub background: TaskTracker,
}

impl AppState {
    pub fn new(db: DatabaseConnection, mailer: Mailer, public_url: impl Into<String>) -> Self {
        Self {
            db,
            mailer,
            public_url: public_url.into(),
            background: TaskTracker::new(),
        }
    }
}
