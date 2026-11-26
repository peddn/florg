//! A student in a learning group, optionally for part of the school year.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "group_membership")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub learning_group_id: i32,
    pub student_id: i32,
    /// `None` means: since the start of the school year.
    pub joined_on: Option<Date>,
    /// `None` means: until the end of the school year.
    pub left_on: Option<Date>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
