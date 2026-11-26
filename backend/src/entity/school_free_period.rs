//! Holidays, public holidays and other days without lessons.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Kind of a school-free period (data model, D6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// School holidays ("Ferien").
    #[sea_orm(string_value = "holiday")]
    Holiday,
    /// A public holiday ("Feiertag").
    #[sea_orm(string_value = "public_holiday")]
    PublicHoliday,
    /// Any other day without lessons, e.g. one set by the school.
    #[sea_orm(string_value = "other")]
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "school_free_period")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub school_year_id: i32,
    pub name: String,
    pub kind: Kind,
    /// First free day.
    pub start_date: Date,
    /// Last free day. Equal to `start_date` for a single day.
    pub end_date: Date,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
