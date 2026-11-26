//! Login by e-mail link and sessions (concept, rules L1 to L13).
//!
//! Secrets are random values from the operating system. The database only
//! ever sees their SHA-256 hashes. A fast hash is appropriate here because
//! the secrets are 256 bit random values, not passwords.

pub mod http;

use chrono::Duration;
use rand::{TryRng, rngs::SysRng};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, sea_query::Expr,
};
use sha2::{Digest, Sha256};

use crate::{
    db::now,
    entity::{account, login_token, session},
};

/// Lifetime of a login link (rule L3).
pub const LOGIN_TOKEN_TTL: Duration = Duration::minutes(15);
/// Login links that may be requested within [`LOGIN_TOKEN_TTL`] (rule L7).
pub const LOGIN_REQUEST_LIMIT: u64 = 3;
/// Lifetime of a session, counted from its last use (rule L8).
pub const SESSION_TTL: Duration = Duration::days(30);
/// A session in use is extended at most this often, to avoid a write on
/// every request.
const SESSION_REFRESH_AFTER: Duration = Duration::hours(1);

#[derive(Debug)]
pub enum AuthError {
    /// The link is unknown, expired, used or opened in another browser.
    /// Deliberately one case, so a response reveals nothing about a token.
    InvalidLink,
    Database(DbErr),
    Random,
}

impl From<DbErr> for AuthError {
    fn from(error: DbErr) -> Self {
        Self::Database(error)
    }
}

/// What a request for a login link led to. The requesting browser never
/// learns which case occurred (rule L2).
pub enum LoginRequestOutcome {
    /// A token was created. Its link still has to be mailed.
    Created(LoginRequest),
    /// No account has been created yet.
    NoAccount,
    /// The entered address is not the address of the account.
    AddressMismatch,
    /// Too many login links were requested (rule L7).
    RateLimited,
}

/// A freshly created login token. The token exists only here and in the
/// mail; the database holds its hash.
pub struct LoginRequest {
    pub token_id: i32,
    pub token: String,
    /// Address of the account, as stored.
    pub email: String,
}

/// A session found for a request.
pub struct ActiveSession {
    pub session_id: i32,
    pub account_id: i32,
    /// The session was extended, so its cookie needs a new lifetime.
    pub refreshed: bool,
}

/// Returns 256 random bits from the operating system, hex encoded.
fn new_secret() -> Result<String, AuthError> {
    let mut bytes = [0u8; 32];
    SysRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| AuthError::Random)?;
    Ok(hex::encode(bytes))
}

fn hash_secret(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

/// Returns a new value for the cookie that ties a login link to the
/// requesting browser (rule L5).
pub fn new_browser_binding() -> Result<String, AuthError> {
    new_secret()
}

/// Addresses are compared without regard to case and surrounding spaces.
fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Returns the account of the instance, if it has been created (D8).
pub async fn find_account(db: &DatabaseConnection) -> Result<Option<account::Model>, DbErr> {
    account::Entity::find()
        .order_by_asc(account::Column::Id)
        .one(db)
        .await
}

/// Creates a login token if `email` is the address of the account and the
/// request limit is not exhausted (rules L2, L3, L4, L5, L7).
///
/// Only requests with the right address count towards the limit, so someone
/// who does not know the address cannot use it up.
pub async fn request_login(
    db: &DatabaseConnection,
    email: &str,
    browser_binding: &str,
) -> Result<LoginRequestOutcome, AuthError> {
    let Some(account) = find_account(db).await? else {
        return Ok(LoginRequestOutcome::NoAccount);
    };
    if normalize_email(email) != normalize_email(&account.email) {
        return Ok(LoginRequestOutcome::AddressMismatch);
    }

    let now = now();
    cleanup(db).await?;

    let recent = login_token::Entity::find()
        .filter(login_token::Column::CreatedAt.gt(now - LOGIN_TOKEN_TTL))
        .count(db)
        .await?;
    if recent >= LOGIN_REQUEST_LIMIT {
        return Ok(LoginRequestOutcome::RateLimited);
    }

    let token = new_secret()?;
    let row = login_token::ActiveModel {
        account_id: Set(account.id),
        token_hash: Set(hash_secret(&token)),
        browser_binding_hash: Set(hash_secret(browser_binding)),
        expires_at: Set(now + LOGIN_TOKEN_TTL),
        used_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(LoginRequestOutcome::Created(LoginRequest {
        token_id: row.id,
        token,
        email: account.email,
    }))
}

/// Withdraws a login token whose mail could not be sent, so a failing mail
/// server does not use up the request limit.
pub async fn discard_login_request(db: &DatabaseConnection, token_id: i32) -> Result<(), DbErr> {
    login_token::Entity::delete_by_id(token_id).exec(db).await?;
    Ok(())
}

/// Redeems a login token and returns the identifier of the new session.
///
/// The token must be unused, unexpired and presented by the browser that
/// requested it (rules L3, L5).
pub async fn confirm_login(
    db: &DatabaseConnection,
    token: &str,
    browser_binding: Option<&str>,
) -> Result<String, AuthError> {
    let now = now();
    let row = login_token::Entity::find()
        .filter(login_token::Column::TokenHash.eq(hash_secret(token)))
        .one(db)
        .await?
        .ok_or(AuthError::InvalidLink)?;

    let binding_matches =
        browser_binding.is_some_and(|binding| hash_secret(binding) == row.browser_binding_hash);
    if row.used_at.is_some() || row.expires_at <= now || !binding_matches {
        return Err(AuthError::InvalidLink);
    }

    // Marking the token as used is conditional, so of two simultaneous
    // attempts only one can succeed.
    let marked = login_token::Entity::update_many()
        .col_expr(login_token::Column::UsedAt, Expr::value(Some(now)))
        .col_expr(login_token::Column::UpdatedAt, Expr::value(now))
        .filter(login_token::Column::Id.eq(row.id))
        .filter(login_token::Column::UsedAt.is_null())
        .exec(db)
        .await?;
    if marked.rows_affected != 1 {
        return Err(AuthError::InvalidLink);
    }

    let session_secret = new_secret()?;
    session::ActiveModel {
        account_id: Set(row.account_id),
        session_hash: Set(hash_secret(&session_secret)),
        last_used_at: Set(now),
        expires_at: Set(now + SESSION_TTL),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(session_secret)
}

/// Looks up the session for an identifier from the cookie and extends it
/// if it has not been extended for a while (rule L8).
pub async fn find_session(
    db: &DatabaseConnection,
    session_secret: &str,
) -> Result<Option<ActiveSession>, DbErr> {
    let now = now();
    let Some(row) = session::Entity::find()
        .filter(session::Column::SessionHash.eq(hash_secret(session_secret)))
        .filter(session::Column::ExpiresAt.gt(now))
        .one(db)
        .await?
    else {
        return Ok(None);
    };

    let refreshed = now - row.last_used_at >= SESSION_REFRESH_AFTER;
    if refreshed {
        session::Entity::update_many()
            .col_expr(session::Column::LastUsedAt, Expr::value(now))
            .col_expr(session::Column::ExpiresAt, Expr::value(now + SESSION_TTL))
            .col_expr(session::Column::UpdatedAt, Expr::value(now))
            .filter(session::Column::Id.eq(row.id))
            .exec(db)
            .await?;
    }

    Ok(Some(ActiveSession {
        session_id: row.id,
        account_id: row.account_id,
        refreshed,
    }))
}

/// Ends one session (rule L9, "Abmelden").
pub async fn logout(db: &DatabaseConnection, session_id: i32) -> Result<(), DbErr> {
    session::Entity::delete_by_id(session_id).exec(db).await?;
    Ok(())
}

/// Ends all sessions of the account (rule L9, "Überall abmelden").
pub async fn logout_all(db: &DatabaseConnection, account_id: i32) -> Result<(), DbErr> {
    session::Entity::delete_many()
        .filter(session::Column::AccountId.eq(account_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Ends all sessions and withdraws all open login links of the account.
/// Used when the address of the account changes (rule L12).
pub async fn revoke_all(db: &DatabaseConnection, account_id: i32) -> Result<(), DbErr> {
    logout_all(db, account_id).await?;
    login_token::Entity::delete_many()
        .filter(login_token::Column::AccountId.eq(account_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Deletes expired login tokens and sessions.
pub async fn cleanup(db: &DatabaseConnection) -> Result<(), DbErr> {
    let now = now();
    login_token::Entity::delete_many()
        .filter(login_token::Column::ExpiresAt.lte(now))
        .exec(db)
        .await?;
    session::Entity::delete_many()
        .filter(session::Column::ExpiresAt.lte(now))
        .exec(db)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_256_bit_hex_and_differ() {
        let first = new_secret().unwrap();
        let second = new_secret().unwrap();
        assert_eq!(first.len(), 64);
        assert_ne!(first, second);
    }

    #[test]
    fn addresses_are_compared_without_case_and_spaces() {
        assert_eq!(
            normalize_email("  Lehrer@Example.ORG "),
            normalize_email("lehrer@example.org")
        );
        assert_ne!(
            normalize_email("lehrer@example.org"),
            normalize_email("lehrerin@example.org")
        );
    }

    #[test]
    fn hash_is_sha256_hex() {
        assert_eq!(
            hash_secret("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
