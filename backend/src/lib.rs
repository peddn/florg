//! florg backend library.
//!
//! The binary in `main.rs` only wires configuration, database and router
//! together. Everything else lives here so integration tests can use it.

pub mod config;
pub mod db;
pub mod migration;
pub mod routes;

use sea_orm::DatabaseConnection;

/// Shared state handed to every request handler.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}
