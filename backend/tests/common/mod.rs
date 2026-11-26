//! Shared helpers for the integration tests: an application with its own
//! database, and a browser that keeps cookies.

// Every test file uses a different part of this module.
#![allow(dead_code)]

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, Ordering},
};

use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode, header},
};
use florg::{
    AppState,
    cli::{self, AccountSet},
    db,
    entity::login_token,
    mail::{CapturedMail, Mailer},
    routes,
};
use http_body_util::BodyExt;
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio_util::task::TaskTracker;
use tower::ServiceExt;

pub const PUBLIC_URL: &str = "https://florg.example.org";
pub const EMAIL: &str = "lehrer@example.org";

pub struct TestApp {
    pub app: Router,
    pub db: DatabaseConnection,
    pub mails: Arc<Mutex<Vec<CapturedMail>>>,
    pub background: TaskTracker,
    pub db_file: std::path::PathBuf,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        std::fs::remove_file(&self.db_file).ok();
    }
}

/// The cookies one browser holds.
#[derive(Default, Clone)]
pub struct Browser {
    pub cookies: Vec<(String, String)>,
}

impl Browser {
    pub fn header(&self) -> String {
        self.cookies
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Applies the `Set-Cookie` headers of a response.
    pub fn store(&mut self, response: &Response<Body>) {
        for value in response.headers().get_all(header::SET_COOKIE) {
            let pair = value.to_str().unwrap().split(';').next().unwrap();
            let (name, value) = pair.split_once('=').unwrap();
            self.cookies.retain(|(existing, _)| existing != name);
            if !value.is_empty() {
                self.cookies.push((name.to_owned(), value.to_owned()));
            }
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.cookies.iter().any(|(existing, _)| existing == name)
    }
}

impl TestApp {
    pub async fn new() -> Self {
        let mails = Arc::new(Mutex::new(Vec::new()));
        Self::with_mailer(Mailer::Capture(mails.clone()), mails).await
    }

    pub async fn with_mailer(mailer: Mailer, mails: Arc<Mutex<Vec<CapturedMail>>>) -> Self {
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
        let state = AppState::new(db.clone(), mailer, PUBLIC_URL);
        let background = state.background.clone();
        Self {
            app: routes::router(state, None),
            db,
            mails,
            background,
            db_file,
        }
    }

    pub async fn with_account() -> Self {
        let app = Self::new().await;
        app.set_email(EMAIL).await;
        app
    }

    pub async fn set_email(&self, email: &str) {
        let args = AccountSet {
            email: email.to_owned(),
            name: None,
        };
        cli::account_set(&self.db, args).await.unwrap();
    }

    pub async fn send(
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

    pub async fn get(&self, browser: &mut Browser, uri: &str) -> Response<Body> {
        self.send(browser, "GET", uri, None).await
    }

    pub async fn post(&self, browser: &mut Browser, uri: &str) -> Response<Body> {
        self.send(browser, "POST", uri, None).await
    }

    /// Submits the login form with `email` and waits until the backend has
    /// finished its background work.
    pub async fn submit_email(&self, browser: &mut Browser, email: &str) -> Response<Body> {
        let json = serde_json::json!({ "email": email });
        let response = self
            .send(browser, "POST", "/api/auth/request-link", Some(json))
            .await;
        self.background.close();
        self.background.wait().await;
        self.background.reopen();
        response
    }

    /// Requests a link with the right address and returns the token from
    /// the captured mail.
    pub async fn request_link(&self, browser: &mut Browser) -> String {
        let before = self.mail_count();
        let response = self.submit_email(browser, EMAIL).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(self.mail_count(), before + 1, "a mail was sent");
        self.last_token()
    }

    pub fn mail_count(&self) -> usize {
        self.mails.lock().unwrap().len()
    }

    pub async fn token_count(&self) -> usize {
        login_token::Entity::find()
            .all(&self.db)
            .await
            .unwrap()
            .len()
    }

    pub fn last_token(&self) -> String {
        let mails = self.mails.lock().unwrap();
        let body = &mails.last().expect("a mail was sent").body;
        let start = body.find("#token=").expect("mail contains a link") + "#token=".len();
        body[start..].split_whitespace().next().unwrap().to_owned()
    }

    pub async fn confirm(&self, browser: &mut Browser, token: &str) -> Response<Body> {
        let json = serde_json::json!({ "token": token });
        self.send(browser, "POST", "/api/auth/confirm", Some(json))
            .await
    }

    /// Runs the whole login and returns the signed-in browser.
    pub async fn sign_in(&self) -> Browser {
        let mut browser = Browser::default();
        let token = self.request_link(&mut browser).await;
        let response = self.confirm(&mut browser, &token).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        browser
    }

    pub async fn is_authenticated(&self, browser: &mut Browser) -> bool {
        let response = self.send(browser, "GET", "/api/auth/status", None).await;
        assert_eq!(response.status(), StatusCode::OK);
        json(response).await["authenticated"] == true
    }
}

pub async fn json(response: Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
