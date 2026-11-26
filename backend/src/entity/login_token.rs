//! A requested login link (rules L3 to L5).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "login_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub account_id: i32,
    /// Hash of the token. The token itself is never stored.
    #[sea_orm(unique)]
    pub token_hash: String,
    /// Hash of a value that only the requesting browser holds.
    pub browser_binding_hash: String,
    pub expires_at: DateTimeUtc,
    /// Set once the token has been redeemed.
    pub used_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
