//! A school year: the period from the first to the last school day.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "school_year")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// E.g. "2026/27".
    #[sea_orm(unique)]
    pub name: String,
    /// First school day.
    pub start_date: Date,
    /// Last school day.
    pub end_date: Date,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
