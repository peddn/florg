//! A student. Independent of school years, so that observations can run
//! across years (concept Z9).

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// How report texts refer to a student (concept S7, Z8b). This is not the
/// gender: `NameOnly` simply never uses a pronoun.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum FormOfAddress {
    #[sea_orm(string_value = "he")]
    He,
    #[sea_orm(string_value = "she")]
    She,
    #[sea_orm(string_value = "name_only")]
    NameOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "student")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// First name, or an abbreviation when working without real names.
    pub first_name: String,
    pub last_name: Option<String>,
    pub form_of_address: FormOfAddress,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
