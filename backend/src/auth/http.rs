//! HTTP side of the login: endpoints, cookies and the session check.

use axum::{
    Extension, Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use super::{ActiveSession, AuthError, LOGIN_TOKEN_TTL, SESSION_TTL};
use crate::{
    AppState,
    mail::{self, Delivery},
};

const SESSION_COOKIE: &str = "florg_session";
/// Ties a login link to the browser that requested it (rule L5).
const BINDING_COOKIE: &str = "florg_login";
/// The binding cookie is only needed by the login endpoints.
const BINDING_COOKIE_PATH: &str = "/api/auth";
/// Path of the confirmation page in the frontend (rule L6).
const CONFIRM_PATH: &str = "/anmelden/bestaetigen";

/// The session of the current request, set by [`require_session`].
#[derive(Debug, Clone, Copy)]
pub struct CurrentSession {
    pub session_id: i32,
    pub account_id: i32,
}

/// Login endpoints that work without a session.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/status", get(status))
        .route("/auth/request-link", post(request_link))
        .route("/auth/confirm", post(confirm))
}

/// Login endpoints that need a session.
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/logout", post(logout))
        .route("/auth/logout-all", post(logout_all))
}

/// Error response of the API: a stable code and a German message for the UI.
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl ApiError {
    const fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
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

    fn internal(error: impl std::fmt::Debug) -> Self {
        error!("internal error: {error:?}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "Ein interner Fehler ist aufgetreten.",
        )
    }
}

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

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::NoAccount => Self::new(
                StatusCode::CONFLICT,
                "no_account",
                "Es ist noch kein Konto angelegt. Lege es auf dem Server mit \"florg account set\" an.",
            ),
            AuthError::RateLimited => Self::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Es wurden zu viele Anmeldelinks angefordert. Versuche es in 15 Minuten erneut.",
            ),
            AuthError::InvalidLink => Self::new(
                StatusCode::BAD_REQUEST,
                "invalid_link",
                "Der Anmeldelink ist ungültig oder abgelaufen. Öffne ihn in dem Browser, in dem du ihn angefordert hast, oder fordere einen neuen an.",
            ),
            AuthError::Database(error) => Self::internal(error),
            AuthError::Random => Self::internal("no randomness available"),
        }
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::internal(error)
    }
}

/// Builds a cookie with the attributes every florg cookie carries (rule L8).
fn cookie(
    name: &'static str,
    value: String,
    path: &'static str,
    ttl: chrono::Duration,
) -> Cookie<'static> {
    Cookie::build((name, value))
        .path(path)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(ttl.num_seconds()))
        .build()
}

fn session_cookie(session_secret: String) -> Cookie<'static> {
    cookie(SESSION_COOKIE, session_secret, "/", SESSION_TTL)
}

/// A cookie that makes the browser delete the cookie of that name and path.
fn removal(name: &'static str, path: &'static str) -> Cookie<'static> {
    Cookie::build((name, "")).path(path).build()
}

async fn session_from_jar(
    state: &AppState,
    jar: &CookieJar,
) -> Result<Option<ActiveSession>, ApiError> {
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return Ok(None);
    };
    Ok(super::find_session(&state.db, cookie.value()).await?)
}

/// Rejects requests without a valid session with 401 and hands the session
/// to the handler as [`CurrentSession`].
pub async fn require_session(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let session = session_from_jar(&state, &jar)
        .await?
        .ok_or(ApiError::UNAUTHENTICATED)?;

    request.extensions_mut().insert(CurrentSession {
        session_id: session.session_id,
        account_id: session.account_id,
    });
    let response = next.run(request).await;

    // An extended session also gets a cookie with a new lifetime, unless
    // the handler has set or removed the cookie itself.
    let handler_set_cookie = response
        .headers()
        .contains_key(axum::http::header::SET_COOKIE);
    if session.refreshed
        && !handler_set_cookie
        && let Some(current) = jar.get(SESSION_COOKIE)
    {
        let renewed = CookieJar::new().add(session_cookie(current.value().to_owned()));
        return Ok((renewed, response).into_response());
    }
    Ok(response)
}

/// Wraps routes so that they require a session.
pub fn with_session_check(routes: Router<AppState>, state: AppState) -> Router<AppState> {
    routes.layer(middleware::from_fn_with_state(state, require_session))
}

#[derive(Serialize)]
struct Status {
    authenticated: bool,
}

/// Tells the frontend whether the browser is signed in.
async fn status(State(state): State<AppState>, jar: CookieJar) -> Result<Json<Status>, ApiError> {
    let authenticated = session_from_jar(&state, &jar).await?.is_some();
    Ok(Json(Status { authenticated }))
}

#[derive(Serialize)]
struct LinkRequested {
    /// "mail" or "log" (rule L11a).
    delivery: &'static str,
}

/// Sends a login link to the address of the account (rules L2, L5, L7).
async fn request_link(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<LinkRequested>), ApiError> {
    let request = super::request_login(&state.db).await?;

    // The token sits in the fragment, which browsers never send to a
    // server, so it stays out of access logs.
    let link = format!("{}{CONFIRM_PATH}#token={}", state.public_url, request.token);
    let (subject, body) = mail::login_mail(&link);
    let delivery = match state.mailer.send(&request.email, subject, &body).await {
        Ok(delivery) => delivery,
        Err(error) => {
            error!("could not send login mail: {error:#}");
            super::discard_login_request(&state.db, request.token_id).await?;
            return Err(ApiError::new(
                StatusCode::BAD_GATEWAY,
                "mail_failed",
                "Die Mail konnte nicht verschickt werden. Versuche es später erneut.",
            ));
        }
    };

    let jar = jar.add(cookie(
        BINDING_COOKIE,
        request.browser_binding,
        BINDING_COOKIE_PATH,
        LOGIN_TOKEN_TTL,
    ));
    let delivery = match delivery {
        Delivery::Mail => "mail",
        Delivery::Log => "log",
    };
    Ok((jar, Json(LinkRequested { delivery })))
}

#[derive(Deserialize)]
struct Confirm {
    token: String,
}

/// Redeems a login link after the user pressed the button (rule L6).
async fn confirm(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<Confirm>,
) -> Result<(CookieJar, StatusCode), ApiError> {
    let binding = jar.get(BINDING_COOKIE).map(|cookie| cookie.value());
    let session_secret = super::confirm_login(&state.db, &body.token, binding).await?;

    let jar = jar
        .remove(removal(BINDING_COOKIE, BINDING_COOKIE_PATH))
        .add(session_cookie(session_secret));
    Ok((jar, StatusCode::NO_CONTENT))
}

/// Ends the session of this device (rule L9).
async fn logout(
    State(state): State<AppState>,
    Extension(session): Extension<CurrentSession>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    super::logout(&state.db, session.session_id).await?;
    Ok((
        jar.remove(removal(SESSION_COOKIE, "/")),
        StatusCode::NO_CONTENT,
    ))
}

/// Ends all sessions of the account (rule L9).
async fn logout_all(
    State(state): State<AppState>,
    Extension(session): Extension<CurrentSession>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    super::logout_all(&state.db, session.account_id).await?;
    Ok((
        jar.remove(removal(SESSION_COOKIE, "/")),
        StatusCode::NO_CONTENT,
    ))
}
