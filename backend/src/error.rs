//! Error responses of the JSON API.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

/// Error response of the API: a stable code and a German message for the UI.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl ApiError {
    pub const fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            code,
            message,
        }
    }

    pub const UNAUTHENTICATED: Self = Self::new(
        StatusCode::UNAUTHORIZED,
        "unauthenticated",
        "Du bist nicht angemeldet.",
    );

    pub const NOT_FOUND: Self = Self::new(
        StatusCode::NOT_FOUND,
        "not_found",
        "Der Eintrag wurde nicht gefunden.",
    );

    /// The request is well-formed but breaks a rule of the data model.
    pub const fn invalid(message: &'static str) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, "invalid", message)
    }

    /// The request collides with existing data.
    pub const fn conflict(message: &'static str) -> Self {
        Self::new(StatusCode::CONFLICT, "conflict", message)
    }

    /// Logs the cause and answers without details.
    pub fn internal(error: impl std::fmt::Debug) -> Self {
        error!("internal error: {error:?}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "Ein interner Fehler ist aufgetreten.",
        )
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: self.code,
            message: self.message,
        };
        (self.status, Json(body)).into_response()
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::internal(error)
    }
}
