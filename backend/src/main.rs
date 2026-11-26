use anyhow::{Context, Result};
use florg::{AppState, config::Config, db, routes};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // A missing .env file is fine: defaults and real environment variables apply.
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    let db = db::connect(&config.database_url).await?;

    let frontend_dir = config.frontend_dir.is_dir().then_some(&config.frontend_dir);
    match frontend_dir {
        Some(dir) => info!("serving frontend from {}", dir.display()),
        None => info!(
            "no built frontend at {}, serving the API only",
            config.frontend_dir.display()
        ),
    }

    let app = routes::router(AppState { db }, frontend_dir.map(|dir| dir.as_path()));

    let listener = TcpListener::bind(config.bind)
        .await
        .with_context(|| format!("could not bind to {}", config.bind))?;
    info!("florg listening on http://{}", config.bind);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    info!("shutting down");
}
