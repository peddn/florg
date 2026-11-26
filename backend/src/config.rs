//! Runtime configuration, read from environment variables.

use std::{env, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result};

const DEFAULT_DATABASE_URL: &str = "sqlite://florg.db?mode=rwc";
const DEFAULT_BIND: &str = "127.0.0.1:3000";
const DEFAULT_FRONTEND_DIR: &str = "../frontend/dist";

#[derive(Debug, Clone)]
pub struct Config {
    /// SeaORM connection string.
    pub database_url: String,
    /// Address the HTTP server listens on.
    pub bind: SocketAddr,
    /// Directory with the built frontend. Served only if it exists.
    pub frontend_dir: PathBuf,
}

impl Config {
    /// Reads the configuration from `FLORG_*` environment variables and
    /// falls back to development defaults.
    pub fn from_env() -> Result<Self> {
        let database_url =
            env::var("FLORG_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned());
        let bind = env::var("FLORG_BIND")
            .unwrap_or_else(|_| DEFAULT_BIND.to_owned())
            .parse()
            .context("FLORG_BIND must be an address like 127.0.0.1:3000")?;
        let frontend_dir = env::var("FLORG_FRONTEND_DIR")
            .unwrap_or_else(|_| DEFAULT_FRONTEND_DIR.to_owned())
            .into();

        Ok(Self {
            database_url,
            bind,
            frontend_dir,
        })
    }
}
