//! Subjects and class names against the HTTP API.

mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use common::{Browser, TestApp, json};
use serde_json::{Value, json as value};

struct Session {
    app: TestApp,
    browser: Browser,
}

impl Session {
    async fn new() -> Self {
        let app = TestApp::with_account().await;
        let browser = app.sign_in().await;
        Self { app, browser }
    }

    async fn request(&mut self, method: &str, uri: &str, body: Option<Value>) -> Response<Body> {
        self.app.send(&mut self.browser, method, uri, body).await
    }

    async fn get(&mut self, uri: &str) -> Value {
        let response = self.request("GET", uri, None).await;
        assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
        json(response).await
    }

    async fn create(&mut self, uri: &str, body: Value) -> Value {
        let response = self.request("POST", uri, Some(body)).await;
        assert_eq!(response.status(), StatusCode::CREATED, "POST {uri}");
        json(response).await
    }

    async fn names(&mut self, uri: &str) -> Vec<String> {
        self.get(uri)
            .await
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["name"].as_str().unwrap().to_owned())
            .collect()
    }
}

async fn assert_error(response: Response<Body>, status: StatusCode, code: &str) {
    assert_eq!(response.status(), status);
    let body = json(response).await;
    assert_eq!(body["error"], code);
    assert!(!body["message"].as_str().unwrap().is_empty());
}

fn subject(name: &str, short_name: &str) -> Value {
    value!({ "name": name, "short_name": short_name })
}

fn class(name: &str) -> Value {
    value!({ "name": name })
}

#[tokio::test]
async fn master_data_endpoints_need_a_session() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();

    for (method, uri) in [
        ("GET", "/api/subjects"),
        ("POST", "/api/subjects"),
        ("PUT", "/api/subjects/1"),
        ("DELETE", "/api/subjects/1"),
        ("GET", "/api/classes"),
        ("POST", "/api/classes"),
        ("PUT", "/api/classes/1"),
        ("DELETE", "/api/classes/1"),
    ] {
        let body = Some(value!({ "name": "x", "short_name": "x" }));
        let response = app.send(&mut browser, method, uri, body).await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri}"
        );
    }
}

#[tokio::test]
async fn subjects_can_be_created_changed_and_deleted() {
    let mut session = Session::new().await;
    assert!(session.names("/api/subjects").await.is_empty());

    let technik = session
        .create("/api/subjects", subject("  Technik ", " Te "))
        .await;
    assert_eq!(technik["name"], "Technik");
    assert_eq!(technik["short_name"], "Te");
    session
        .create("/api/subjects", subject("ernährung und Soziales", "ES"))
        .await;
    // Sorted by name without regard to case.
    assert_eq!(
        session.names("/api/subjects").await,
        ["ernährung und Soziales", "Technik"]
    );

    let uri = format!("/api/subjects/{}", technik["id"]);
    let response = session
        .request("PUT", &uri, Some(subject("Werken", "We")))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await["short_name"], "We");

    let response = session.request("DELETE", &uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        session.names("/api/subjects").await,
        ["ernährung und Soziales"]
    );
    let response = session.request("DELETE", &uri, None).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    let response = session
        .request("PUT", &uri, Some(subject("Werken", "We")))
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn invalid_subjects_are_refused() {
    let mut session = Session::new().await;
    let technik = session
        .create("/api/subjects", subject("Technik", "Te"))
        .await;
    let werken = session
        .create("/api/subjects", subject("Werken", "We"))
        .await;

    let invalid = [
        subject(" ", "Te2"),
        subject("Informatik", " "),
        subject("Informatik", "ZuLangesKürzel"),
    ];
    for body in invalid {
        let response = session.request("POST", "/api/subjects", Some(body)).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }

    // Name and short name are unique regardless of case.
    for body in [subject("technik", "T"), subject("Textil", "TE")] {
        let response = session
            .request("POST", "/api/subjects", Some(body.clone()))
            .await;
        assert_error(response, StatusCode::CONFLICT, "conflict").await;
        let uri = format!("/api/subjects/{}", werken["id"]);
        let response = session.request("PUT", &uri, Some(body)).await;
        assert_error(response, StatusCode::CONFLICT, "conflict").await;
    }

    // Keeping the own name and short name is no conflict.
    let uri = format!("/api/subjects/{}", technik["id"]);
    let response = session
        .request("PUT", &uri, Some(subject("TECHNIK", "te")))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(session.names("/api/subjects").await.len(), 2);
}

#[tokio::test]
async fn classes_can_be_created_changed_and_deleted() {
    let mut session = Session::new().await;
    for name in ["10a", " 7b ", "7a", "DFK", "9"] {
        session.create("/api/classes", class(name)).await;
    }
    // "7a" before "10a", names without a number last.
    assert_eq!(
        session.names("/api/classes").await,
        ["7a", "7b", "9", "10a", "DFK"]
    );

    let classes = session.get("/api/classes").await;
    let uri = format!("/api/classes/{}", classes[0]["id"]);
    let response = session.request("PUT", &uri, Some(class("8a"))).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        session.names("/api/classes").await,
        ["7b", "8a", "9", "10a", "DFK"]
    );

    let response = session.request("DELETE", &uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        session.names("/api/classes").await,
        ["7b", "9", "10a", "DFK"]
    );
    let response = session.request("DELETE", &uri, None).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn invalid_classes_are_refused() {
    let mut session = Session::new().await;
    let first = session.create("/api/classes", class("7a")).await;
    let second = session.create("/api/classes", class("7b")).await;

    let response = session
        .request("POST", "/api/classes", Some(class("  ")))
        .await;
    assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;

    let response = session
        .request("POST", "/api/classes", Some(class(" 7A ")))
        .await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;
    let uri = format!("/api/classes/{}", second["id"]);
    let response = session.request("PUT", &uri, Some(class("7A"))).await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;

    // Changing only the case of the own name is allowed.
    let uri = format!("/api/classes/{}", first["id"]);
    let response = session.request("PUT", &uri, Some(class("7A"))).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(session.names("/api/classes").await, ["7A", "7b"]);
}
