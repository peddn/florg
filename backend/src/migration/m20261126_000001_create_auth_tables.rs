//! Tables for the login: `account`, `login_token`, `session`.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .col(pk_auto(Account::Id))
                    .col(string(Account::Email))
                    .col(string_null(Account::DisplayName))
                    .col(timestamp_with_time_zone(Account::CreatedAt))
                    .col(timestamp_with_time_zone(Account::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(LoginToken::Table)
                    .col(pk_auto(LoginToken::Id))
                    .col(integer(LoginToken::AccountId))
                    .col(string_uniq(LoginToken::TokenHash))
                    .col(string(LoginToken::BrowserBindingHash))
                    .col(timestamp_with_time_zone(LoginToken::ExpiresAt))
                    .col(timestamp_with_time_zone_null(LoginToken::UsedAt))
                    .col(timestamp_with_time_zone(LoginToken::CreatedAt))
                    .col(timestamp_with_time_zone(LoginToken::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(LoginToken::Table, LoginToken::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .col(pk_auto(Session::Id))
                    .col(integer(Session::AccountId))
                    .col(string_uniq(Session::SessionHash))
                    .col(timestamp_with_time_zone(Session::LastUsedAt))
                    .col(timestamp_with_time_zone(Session::ExpiresAt))
                    .col(timestamp_with_time_zone(Session::CreatedAt))
                    .col(timestamp_with_time_zone(Session::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Session::Table, Session::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LoginToken::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
    Email,
    DisplayName,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum LoginToken {
    Table,
    Id,
    AccountId,
    TokenHash,
    BrowserBindingHash,
    ExpiresAt,
    UsedAt,
    CreatedAt,
    UpdatedAt,
}

// `SessionHash` is the column name from the data model.
#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Session {
    Table,
    Id,
    AccountId,
    SessionHash,
    LastUsedAt,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}
