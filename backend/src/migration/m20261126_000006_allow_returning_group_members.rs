//! A student may have several periods in the same learning group, e.g.
//! after leaving and returning. The unique index of migration 5 is replaced
//! by a plain one; overlapping periods are refused by the endpoints.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_group_membership_group_student")
                    .table(GroupMembership::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_group_membership_group_student")
                    .table(GroupMembership::Table)
                    .col(GroupMembership::LearningGroupId)
                    .col(GroupMembership::StudentId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_group_membership_group_student")
                    .table(GroupMembership::Table)
                    .to_owned(),
            )
            .await?;
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
            .await
    }
}

#[derive(DeriveIden)]
enum GroupMembership {
    Table,
    LearningGroupId,
    StudentId,
}
