//! Tables for students and their school years: `student`, `enrollment`.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Student::Table)
                    .col(pk_auto(Student::Id))
                    .col(string(Student::FirstName))
                    .col(string_null(Student::LastName))
                    .col(string(Student::FormOfAddress))
                    .col(timestamp_with_time_zone(Student::CreatedAt))
                    .col(timestamp_with_time_zone(Student::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // The endpoints remove dependent rows themselves and refuse to
        // delete a school year or class that is still in use. The foreign
        // keys only guard against mistakes.
        manager
            .create_table(
                Table::create()
                    .table(Enrollment::Table)
                    .col(pk_auto(Enrollment::Id))
                    .col(integer(Enrollment::StudentId))
                    .col(integer(Enrollment::SchoolYearId))
                    .col(integer_null(Enrollment::SchoolClassId))
                    .col(timestamp_with_time_zone(Enrollment::CreatedAt))
                    .col(timestamp_with_time_zone(Enrollment::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Enrollment::Table, Enrollment::StudentId)
                            .to(Student::Table, Student::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Enrollment::Table, Enrollment::SchoolYearId)
                            .to(SchoolYear::Table, SchoolYear::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Enrollment::Table, Enrollment::SchoolClassId)
                            .to(SchoolClass::Table, SchoolClass::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // A student has at most one enrollment per school year.
        manager
            .create_index(
                Index::create()
                    .name("idx_enrollment_student_school_year")
                    .table(Enrollment::Table)
                    .col(Enrollment::StudentId)
                    .col(Enrollment::SchoolYearId)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_enrollment_school_year_id")
                    .table(Enrollment::Table)
                    .col(Enrollment::SchoolYearId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Enrollment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Student::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Student {
    Table,
    Id,
    FirstName,
    LastName,
    FormOfAddress,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Enrollment {
    Table,
    Id,
    StudentId,
    SchoolYearId,
    SchoolClassId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SchoolYear {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum SchoolClass {
    Table,
    Id,
}
