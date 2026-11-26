use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use florg::{AppState, db, routes};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn health_reports_ok_with_reachable_database() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    let app = routes::router(AppState { db }, None);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["database"], "ok");
}

#[tokio::test]
async fn unknown_api_path_is_not_found() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    let app = routes::router(AppState { db }, None);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn frontend_is_served_but_never_for_unknown_api_paths() {
    let dir = std::env::temp_dir().join(format!("florg-test-frontend-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><title>florg</title>",
    )
    .unwrap();

    let db = db::connect("sqlite::memory:").await.unwrap();
    let app = routes::router(AppState { db }, Some(&dir));

    let get = |uri: &'static str| {
        let app = app.clone();
        async move {
            app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap()
        }
    };

    // Client-side routes fall back to index.html.
    assert_eq!(get("/").await.status(), StatusCode::OK);
    assert_eq!(get("/some/client/route").await.status(), StatusCode::OK);
    // API paths do not.
    assert_eq!(
        get("/api/does-not-exist").await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(get("/api/health").await.status(), StatusCode::OK);
}
