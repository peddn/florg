//! Tables for learning groups: `learning_group`, `group_membership`.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The endpoints remove dependent rows themselves and refuse to
        // delete a school year or subject that is still in use. The foreign
        // keys only guard against mistakes.
        manager
            .create_table(
                Table::create()
                    .table(LearningGroup::Table)
                    .col(pk_auto(LearningGroup::Id))
                    .col(integer(LearningGroup::SchoolYearId))
                    .col(integer(LearningGroup::SubjectId))
                    .col(string(LearningGroup::Name))
                    .col(timestamp_with_time_zone(LearningGroup::CreatedAt))
                    .col(timestamp_with_time_zone(LearningGroup::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(LearningGroup::Table, LearningGroup::SchoolYearId)
                            .to(SchoolYear::Table, SchoolYear::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(LearningGroup::Table, LearningGroup::SubjectId)
                            .to(Subject::Table, Subject::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_learning_group_school_year_id")
                    .table(LearningGroup::Table)
                    .col(LearningGroup::SchoolYearId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GroupMembership::Table)
                    .col(pk_auto(GroupMembership::Id))
                    .col(integer(GroupMembership::LearningGroupId))
                    .col(integer(GroupMembership::StudentId))
                    .col(date_null(GroupMembership::JoinedOn))
                    .col(date_null(GroupMembership::LeftOn))
                    .col(timestamp_with_time_zone(GroupMembership::CreatedAt))
                    .col(timestamp_with_time_zone(GroupMembership::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupMembership::Table, GroupMembership::LearningGroupId)
                            .to(LearningGroup::Table, LearningGroup::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupMembership::Table, GroupMembership::StudentId)
                            .to(Student::Table, Student::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        // At most one membership per student and learning group.
        manager
            .create_index(
                Index::create()
                    .name("idx_group_membership_group_student")
                    .table(GroupMembership::Table)
                    .col(GroupMembership::LearningGroupId)
                    .col(GroupMembership::StudentId)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_group_membership_student_id")
                    .table(GroupMembership::Table)
                    .col(GroupMembership::StudentId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupMembership::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LearningGroup::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LearningGroup {
    Table,
    Id,
    SchoolYearId,
    SubjectId,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GroupMembership {
    Table,
    Id,
    LearningGroupId,
    StudentId,
    JoinedOn,
    LeftOn,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SchoolYear {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Subject {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Student {
    Table,
    Id,
}
