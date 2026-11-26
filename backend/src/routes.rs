//! HTTP routes.

use std::path::Path;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{AppState, auth, learning_group, school_class, school_year, student, subject};

/// Builds the application router.
///
/// Everything under `/api` is the JSON API. If `frontend_dir` exists, all
/// other paths serve the built frontend, falling back to `index.html` so the
/// Vue router can handle client-side routes.
pub fn router(state: AppState, frontend_dir: Option<&Path>) -> Router {
    // Unknown API paths answer 404 and never fall through to the frontend.
    // Everything except the health check and the login itself needs a
    // session. New endpoints belong into `protected`.
    let protected = auth::http::protected_routes()
        .merge(school_year::routes())
        .merge(subject::routes())
        .merge(school_class::routes())
        .merge(student::routes())
        .merge(learning_group::routes());
    let protected = auth::http::with_session_check(protected, state.clone());
    let api = Router::new()
        .route("/health", get(health))
        .merge(auth::http::public_routes())
        .merge(protected)
        .fallback(|| async { StatusCode::NOT_FOUND });

    let mut app = Router::new().nest("/api", api);

    if let Some(dir) = frontend_dir {
        let index = ServeFile::new(dir.join("index.html"));
        app = app.fallback_service(ServeDir::new(dir).fallback(index));
    }

    app.layer(TraceLayer::new_for_http()).with_state(state)
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    database: &'static str,
    version: &'static str,
}

/// Reports whether the server runs and can reach its database.
async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    let database_ok = state.db.ping().await.is_ok();

    let body = Health {
        status: if database_ok { "ok" } else { "degraded" },
        database: if database_ok { "ok" } else { "unreachable" },
        version: env!("CARGO_PKG_VERSION"),
    };
    let code = if database_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (code, Json(body))
}
