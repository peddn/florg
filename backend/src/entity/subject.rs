//! A school subject taught to learning groups.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "subject")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// E.g. "Berufs- und Lebensorientierung – Praxis".
    pub name: String,
    /// E.g. "BLO-P", for narrow displays such as the timetable.
    pub short_name: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
