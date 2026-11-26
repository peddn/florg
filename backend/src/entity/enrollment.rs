//! A student in a school year, optionally with the class (concept Z8a).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "enrollment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub student_id: i32,
    pub school_year_id: i32,
    /// Class in this school year. `None` means: without a class.
    pub school_class_id: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
