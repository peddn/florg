//! florg backend library.
//!
//! The binary in `main.rs` only wires configuration, database and router
//! together. Everything else lives here so integration tests can use it.

pub mod auth;
pub mod cli;
pub mod config;
pub mod db;
pub mod entity;
pub mod mail;
pub mod migration;
pub mod routes;

use sea_orm::DatabaseConnection;

use crate::mail::Mailer;

/// Shared state handed to every request handler.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub mailer: Mailer,
    /// Public address of the application, without a trailing slash.
    pub public_url: String,
}
