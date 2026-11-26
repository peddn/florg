//! Schema migrations (data model, D7).
//!
//! Every schema change is a new module in this directory, registered in
//! `Migrator::migrations` in chronological order. Existing migrations are
//! never edited after they have been merged.

use sea_orm_migration::prelude::*;

mod m20261126_000001_create_auth_tables;
mod m20261126_000002_create_school_year_tables;
mod m20261126_000003_create_subject_and_school_class;
mod m20261126_000004_create_student_and_enrollment;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261126_000001_create_auth_tables::Migration),
            Box::new(m20261126_000002_create_school_year_tables::Migration),
            Box::new(m20261126_000003_create_subject_and_school_class::Migration),
            Box::new(m20261126_000004_create_student_and_enrollment::Migration),
        ]
    }
}
