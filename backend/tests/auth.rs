//! Login by e-mail link, end to end against the HTTP API.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, Ordering},
};

use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode, header},
};
use chrono::{Duration, Utc};
use florg::{
    AppState,
    cli::{self, AccountSet},
    db,
    entity::{login_token, session},
    mail::{CapturedMail, Mailer},
    routes,
};
use http_body_util::BodyExt;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use tower::ServiceExt;

const PUBLIC_URL: &str = "https://florg.example.org";
const EMAIL: &str = "lehrer@example.org";

struct TestApp {
    app: Router,
    db: DatabaseConnection,
    mails: Arc<Mutex<Vec<CapturedMail>>>,
    db_file: std::path::PathBuf,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        std::fs::remove_file(&self.db_file).ok();
    }
}

/// The cookies one browser holds.
#[derive(Default, Clone)]
struct Browser {
    cookies: Vec<(String, String)>,
}

impl Browser {
    fn header(&self) -> String {
        self.cookies
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Applies the `Set-Cookie` headers of a response.
    fn store(&mut self, response: &Response<Body>) {
        for value in response.headers().get_all(header::SET_COOKIE) {
            let pair = value.to_str().unwrap().split(';').next().unwrap();
            let (name, value) = pair.split_once('=').unwrap();
            self.cookies.retain(|(existing, _)| existing != name);
            if !value.is_empty() {
                self.cookies.push((name.to_owned(), value.to_owned()));
            }
        }
    }

    fn has(&self, name: &str) -> bool {
        self.cookies.iter().any(|(existing, _)| existing == name)
    }
}

impl TestApp {
    async fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let db_file = std::env::temp_dir().join(format!(
            "florg-test-auth-{}-{}.db",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::remove_file(&db_file).ok();
        let db = db::connect(&format!("sqlite://{}?mode=rwc", db_file.display()))
            .await
            .unwrap();
        let mails = Arc::new(Mutex::new(Vec::new()));
        let state = AppState {
            db: db.clone(),
            mailer: Mailer::Capture(mails.clone()),
            public_url: PUBLIC_URL.to_owned(),
        };
        Self {
            app: routes::router(state, None),
            db,
            mails,
            db_file,
        }
    }

    async fn with_account() -> Self {
        let app = Self::new().await;
        app.set_email(EMAIL).await;
        app
    }

    async fn set_email(&self, email: &str) {
        let args = AccountSet {
            email: email.to_owned(),
            name: None,
        };
        cli::account_set(&self.db, args).await.unwrap();
    }

    async fn send(
        &self,
        browser: &mut Browser,
        method: &str,
        uri: &str,
        json: Option<serde_json::Value>,
    ) -> Response<Body> {
        let mut request = Request::builder().method(method).uri(uri);
        if !browser.cookies.is_empty() {
            request = request.header(header::COOKIE, browser.header());
        }
        let body = match json {
            Some(json) => {
                request = request.header(header::CONTENT_TYPE, "application/json");
                Body::from(json.to_string())
            }
            None => Body::empty(),
        };
        let response = self
            .app
            .clone()
            .oneshot(request.body(body).unwrap())
            .await
            .unwrap();
        browser.store(&response);
        response
    }

    async fn post(&self, browser: &mut Browser, uri: &str) -> Response<Body> {
        self.send(browser, "POST", uri, None).await
    }

    /// Requests a link and returns the token from the captured mail.
    async fn request_link(&self, browser: &mut Browser) -> String {
        let response = self.post(browser, "/api/auth/request-link").await;
        assert_eq!(response.status(), StatusCode::OK);
        self.last_token()
    }

    fn last_token(&self) -> String {
        let mails = self.mails.lock().unwrap();
        let body = &mails.last().expect("a mail was sent").body;
        let start = body.find("#token=").expect("mail contains a link") + "#token=".len();
        body[start..].split_whitespace().next().unwrap().to_owned()
    }

    async fn confirm(&self, browser: &mut Browser, token: &str) -> Response<Body> {
        let json = serde_json::json!({ "token": token });
        self.send(browser, "POST", "/api/auth/confirm", Some(json))
            .await
    }

    /// Runs the whole login and returns the signed-in browser.
    async fn sign_in(&self) -> Browser {
        let mut browser = Browser::default();
        let token = self.request_link(&mut browser).await;
        let response = self.confirm(&mut browser, &token).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        browser
    }

    async fn is_authenticated(&self, browser: &mut Browser) -> bool {
        let response = self.send(browser, "GET", "/api/auth/status", None).await;
        assert_eq!(response.status(), StatusCode::OK);
        json(response).await["authenticated"] == true
    }
}

async fn json(response: Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn login_by_link_signs_in_and_logout_signs_out() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    assert!(!app.is_authenticated(&mut browser).await);

    let response = app.post(&mut browser, "/api/auth/request-link").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await["delivery"], "mail");

    // The mail goes to the account and the link is built from the
    // configured public address (rules L2, L10).
    let mail = app.mails.lock().unwrap().last().unwrap().clone();
    assert_eq!(mail.to, EMAIL);
    let token = app.last_token();
    assert!(
        mail.body
            .contains(&format!("{PUBLIC_URL}/anmelden/bestaetigen#token={token}"))
    );

    // Following the link alone does not sign in (rule L6).
    assert!(!app.is_authenticated(&mut browser).await);

    let response = app.confirm(&mut browser, &token).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let session_cookie = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().to_owned())
        .find(|value| value.starts_with("florg_session="))
        .expect("session cookie is set");
    for attribute in ["HttpOnly", "Secure", "SameSite=Strict", "Path=/"] {
        assert!(session_cookie.contains(attribute), "missing {attribute}");
    }
    assert!(!browser.has("florg_login"), "binding cookie is removed");
    assert!(app.is_authenticated(&mut browser).await);

    let response = app.post(&mut browser, "/api/auth/logout").await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(!browser.has("florg_session"));
    assert!(!app.is_authenticated(&mut browser).await);
}

#[tokio::test]
async fn database_stores_only_hashes() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;
    app.confirm(&mut browser, &token).await;

    let tokens = login_token::Entity::find().all(&app.db).await.unwrap();
    assert_eq!(tokens.len(), 1);
    assert_ne!(tokens[0].token_hash, token);
    assert!(tokens[0].used_at.is_some());

    let (_, session_secret) = browser
        .cookies
        .iter()
        .find(|(name, _)| name == "florg_session")
        .unwrap();
    let sessions = session::Entity::find().all(&app.db).await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_ne!(&sessions[0].session_hash, session_secret);
}

#[tokio::test]
async fn link_works_only_once() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;
    // Keep the binding cookie, which a successful login removes.
    let mut second_attempt = browser.clone();

    assert_eq!(
        app.confirm(&mut browser, &token).await.status(),
        StatusCode::NO_CONTENT
    );
    let response = app.confirm(&mut second_attempt, &token).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(response).await["error"], "invalid_link");
}

#[tokio::test]
async fn link_works_only_in_the_requesting_browser() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;

    // Another browser without the binding cookie.
    let mut stranger = Browser::default();
    assert_eq!(
        app.confirm(&mut stranger, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
    // Another browser that requested its own link.
    let mut other = Browser::default();
    app.request_link(&mut other).await;
    assert_eq!(
        app.confirm(&mut other, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
    assert!(!app.is_authenticated(&mut stranger).await);
    assert!(!app.is_authenticated(&mut other).await);

    // The failed attempts have not used up the link.
    assert_eq!(
        app.confirm(&mut browser, &token).await.status(),
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn expired_link_is_rejected() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;

    let row = login_token::Entity::find()
        .one(&app.db)
        .await
        .unwrap()
        .unwrap();
    let mut row = row.into_active_model();
    row.expires_at = Set(Utc::now() - Duration::seconds(1));
    row.update(&app.db).await.unwrap();

    assert_eq!(
        app.confirm(&mut browser, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn unknown_token_is_rejected() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    app.request_link(&mut browser).await;

    assert_eq!(
        app.confirm(&mut browser, "not-a-token").await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn link_requests_are_limited() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    for _ in 0..3 {
        app.request_link(&mut browser).await;
    }

    let response = app.post(&mut browser, "/api/auth/request-link").await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(json(response).await["error"], "rate_limited");
    assert_eq!(app.mails.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn link_request_without_account_is_refused() {
    let app = TestApp::new().await;
    let mut browser = Browser::default();

    let response = app.post(&mut browser, "/api/auth/request-link").await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(json(response).await["error"], "no_account");
    assert!(app.mails.lock().unwrap().is_empty());
}

#[tokio::test]
async fn protected_endpoints_need_a_session() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();

    for uri in ["/api/auth/logout", "/api/auth/logout-all"] {
        let response = app.post(&mut browser, uri).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(json(response).await["error"], "unauthenticated");
    }

    // A made-up session identifier does not help.
    browser
        .cookies
        .push(("florg_session".to_owned(), "0".repeat(64)));
    let response = app.post(&mut browser, "/api/auth/logout").await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_ends_one_session_and_logout_all_ends_every_session() {
    let app = TestApp::with_account().await;
    let mut phone = app.sign_in().await;
    let mut laptop = app.sign_in().await;
    let mut tablet = app.sign_in().await;

    app.post(&mut phone, "/api/auth/logout").await;
    assert!(!app.is_authenticated(&mut phone).await);
    assert!(app.is_authenticated(&mut laptop).await);
    assert!(app.is_authenticated(&mut tablet).await);

    let response = app.post(&mut laptop, "/api/auth/logout-all").await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(!app.is_authenticated(&mut laptop).await);
    assert!(!app.is_authenticated(&mut tablet).await);
}

#[tokio::test]
async fn expired_session_is_signed_out() {
    let app = TestApp::with_account().await;
    let mut browser = app.sign_in().await;

    let row = session::Entity::find().one(&app.db).await.unwrap().unwrap();
    let mut row = row.into_active_model();
    row.expires_at = Set(Utc::now() - Duration::seconds(1));
    row.update(&app.db).await.unwrap();

    assert!(!app.is_authenticated(&mut browser).await);
}

#[tokio::test]
async fn session_in_use_is_extended() {
    let app = TestApp::with_account().await;
    let mut browser = app.sign_in().await;

    let before = session::Entity::find().one(&app.db).await.unwrap().unwrap();
    let mut row = before.clone().into_active_model();
    row.last_used_at = Set(before.last_used_at - Duration::hours(2));
    row.expires_at = Set(before.expires_at - Duration::hours(2));
    row.update(&app.db).await.unwrap();

    assert!(app.is_authenticated(&mut browser).await);
    let after = session::Entity::find().one(&app.db).await.unwrap().unwrap();
    assert!(after.expires_at >= before.expires_at);
    assert!(after.last_used_at >= before.last_used_at);
}

#[tokio::test]
async fn changing_the_address_ends_sessions_and_open_links() {
    let app = TestApp::with_account().await;
    let mut signed_in = app.sign_in().await;
    let mut waiting = Browser::default();
    let open_token = app.request_link(&mut waiting).await;

    // Setting the same address again changes nothing.
    app.set_email(EMAIL).await;
    assert!(app.is_authenticated(&mut signed_in).await);

    app.set_email("neu@example.org").await;
    assert!(!app.is_authenticated(&mut signed_in).await);
    assert_eq!(
        app.confirm(&mut waiting, &open_token).await.status(),
        StatusCode::BAD_REQUEST
    );

    let mut browser = Browser::default();
    app.request_link(&mut browser).await;
    assert_eq!(
        app.mails.lock().unwrap().last().unwrap().to,
        "neu@example.org"
    );
}

#[tokio::test]
async fn invalid_address_is_refused() {
    let app = TestApp::new().await;
    let args = AccountSet {
        email: "keine-adresse".to_owned(),
        name: None,
    };
    assert!(cli::account_set(&app.db, args).await.is_err());
}
