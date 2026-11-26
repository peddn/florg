//! The students who are taught together in one subject. Not the same as a
//! class: groups may be split or span classes (concept, glossary).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "learning_group")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub school_year_id: i32,
    pub subject_id: i32,
    /// E.g. "7a/b Technik Gruppe 1", unique within the school year.
    pub name: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
