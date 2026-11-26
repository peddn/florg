//! Database connection and schema setup.

use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;

use crate::migration::Migrator;

/// Opens the database, enables foreign keys and applies pending migrations.
pub async fn connect(database_url: &str) -> Result<DatabaseConnection> {
    let db = Database::connect(database_url)
        .await
        .with_context(|| format!("could not open database {database_url}"))?;

    // SQLite does not enforce foreign keys unless asked to (data model, D2).
    if db.get_database_backend() == DbBackend::Sqlite {
        db.execute_raw(Statement::from_string(
            DbBackend::Sqlite,
            "PRAGMA foreign_keys = ON",
        ))
        .await
        .context("could not enable foreign keys")?;
    }

    Migrator::up(&db, None)
        .await
        .context("could not apply migrations")?;

    Ok(db)
}
