use std::{io::IsTerminal, process::ExitCode};

use anyhow::{Context, Result};
use clap::Parser;
use florg::{
    AppState, auth,
    cli::{self, AccountCommand, Cli, Command, MailCommand},
    config::Config,
    db,
    mail::{Delivery, Mailer},
    routes,
};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

/// Log level if `RUST_LOG` is not set. Warnings and the login link in log
/// delivery (rule L11a) must be visible without further configuration.
const DEFAULT_LOG_FILTER: &str = "info,sqlx=warn";

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Fehler: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    // A missing .env file is fine: defaults and real environment variables apply.
    dotenvy::dotenv().ok();
    init_logging();

    let config = Config::from_env()?;
    let db = db::connect(&config.database_url).await?;

    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(config, db).await,
        Command::Account(AccountCommand::Set(args)) => cli::account_set(&db, args).await,
        Command::Mail(MailCommand::Test) => cli::mail_test(&db, &config).await,
    }
}

fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        // No colour codes in files and the journal.
        .with_ansi(std::io::stdout().is_terminal())
        .init();
}

async fn serve(config: Config, db: DatabaseConnection) -> Result<()> {
    let mailer = Mailer::from_config(config.smtp.as_ref())?;
    if mailer.delivery() == Delivery::Log {
        warn!("no mail server configured: login links are written to this log, not sent");
    }
    if auth::find_account(&db).await?.is_none() {
        warn!("no account yet: create it with `florg account set --email <address>`");
    }
    auth::cleanup(&db).await?;

    let frontend_dir = config.frontend_dir.is_dir().then_some(&config.frontend_dir);
    match frontend_dir {
        Some(dir) => info!("serving frontend from {}", dir.display()),
        None => info!(
            "no built frontend at {}, serving the API only",
            config.frontend_dir.display()
        ),
    }

    let state = AppState::new(db, mailer, config.public_url.clone());
    let background = state.background.clone();
    let app = routes::router(state, frontend_dir.map(|dir| dir.as_path()));

    let listener = TcpListener::bind(config.bind)
        .await
        .with_context(|| format!("could not bind to {}", config.bind))?;
    info!("florg listening on http://{}", config.bind);
    info!("public address is {}", config.public_url);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    // Let mails that are still being sent go out.
    background.close();
    background.wait().await;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    info!("shutting down");
}
