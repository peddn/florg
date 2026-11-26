//! Tables for the lists of subjects and class names: `subject`,
//! `school_class`.
//!
//! Names are unique regardless of case. The endpoints check that, because
//! a case-insensitive index is not available in the same form in SQLite
//! and PostgreSQL (data model, D2).

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subject::Table)
                    .col(pk_auto(Subject::Id))
                    .col(string_uniq(Subject::Name))
                    .col(string_uniq(Subject::ShortName))
                    .col(timestamp_with_time_zone(Subject::CreatedAt))
                    .col(timestamp_with_time_zone(Subject::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SchoolClass::Table)
                    .col(pk_auto(SchoolClass::Id))
                    .col(string_uniq(SchoolClass::Name))
                    .col(timestamp_with_time_zone(SchoolClass::CreatedAt))
                    .col(timestamp_with_time_zone(SchoolClass::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SchoolClass::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Subject::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Subject {
    Table,
    Id,
    Name,
    ShortName,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SchoolClass {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}
