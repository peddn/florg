//! Schema migrations (data model, D7).
//!
//! Every schema change is a new module in this directory, registered in
//! `Migrator::migrations` in chronological order. Existing migrations are
//! never edited after they have been merged.

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // No domain tables yet. The first ones arrive with the login.
        vec![]
    }
}
