//! Tables for the school year: `school_year`, `school_free_period`.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SchoolYear::Table)
                    .col(pk_auto(SchoolYear::Id))
                    .col(string_uniq(SchoolYear::Name))
                    .col(date(SchoolYear::StartDate))
                    .col(date(SchoolYear::EndDate))
                    .col(timestamp_with_time_zone(SchoolYear::CreatedAt))
                    .col(timestamp_with_time_zone(SchoolYear::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SchoolFreePeriod::Table)
                    .col(pk_auto(SchoolFreePeriod::Id))
                    .col(integer(SchoolFreePeriod::SchoolYearId))
                    .col(string(SchoolFreePeriod::Name))
                    .col(string(SchoolFreePeriod::Kind))
                    .col(date(SchoolFreePeriod::StartDate))
                    .col(date(SchoolFreePeriod::EndDate))
                    .col(timestamp_with_time_zone(SchoolFreePeriod::CreatedAt))
                    .col(timestamp_with_time_zone(SchoolFreePeriod::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SchoolFreePeriod::Table, SchoolFreePeriod::SchoolYearId)
                            .to(SchoolYear::Table, SchoolYear::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_school_free_period_school_year_id")
                    .table(SchoolFreePeriod::Table)
                    .col(SchoolFreePeriod::SchoolYearId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SchoolFreePeriod::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SchoolYear::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SchoolYear {
    Table,
    Id,
    Name,
    StartDate,
    EndDate,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SchoolFreePeriod {
    Table,
    Id,
    SchoolYearId,
    Name,
    Kind,
    StartDate,
    EndDate,
    CreatedAt,
    UpdatedAt,
}
