//! Runtime configuration, read from environment variables.

use std::{env, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result, bail};

const DEFAULT_DATABASE_URL: &str = "sqlite://florg.db?mode=rwc";
const DEFAULT_BIND: &str = "127.0.0.1:3000";
const DEFAULT_FRONTEND_DIR: &str = "../frontend/dist";
/// The Vite development server.
const DEFAULT_PUBLIC_URL: &str = "http://localhost:5173";
/// Submission port with mandatory STARTTLS.
const DEFAULT_SMTP_PORT: u16 = 587;

#[derive(Debug, Clone)]
pub struct Config {
    /// SeaORM connection string.
    pub database_url: String,
    /// Address the HTTP server listens on.
    pub bind: SocketAddr,
    /// Directory with the built frontend. Served only if it exists.
    pub frontend_dir: PathBuf,
    /// Address under which users reach the application, without a trailing
    /// slash. Login links are built from it and from nothing else (rule L10).
    pub public_url: String,
    /// Mail server. `None` writes mails to the log instead (rule L11a).
    pub smtp: Option<SmtpConfig>,
}

#[derive(Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    /// Sender address, e.g. `florg <florg@example.org>`.
    pub from: String,
}

// Keeps the password out of debug output and logs.
impl std::fmt::Debug for SmtpConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmtpConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &"<redacted>")
            .field("from", &self.from)
            .finish()
    }
}

impl Config {
    /// Reads the configuration from `FLORG_*` environment variables and
    /// falls back to development defaults.
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| env::var(name).ok())
    }

    /// Like [`Config::from_env`], with an exchangeable source of variables.
    /// Empty values count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let get = |name: &str| lookup(name).filter(|value| !value.trim().is_empty());

        let database_url =
            get("FLORG_DATABASE_URL").unwrap_or_else(|| DEFAULT_DATABASE_URL.to_owned());
        let bind = get("FLORG_BIND")
            .unwrap_or_else(|| DEFAULT_BIND.to_owned())
            .parse()
            .context("FLORG_BIND muss eine Adresse wie 127.0.0.1:3000 sein")?;
        let frontend_dir = get("FLORG_FRONTEND_DIR")
            .unwrap_or_else(|| DEFAULT_FRONTEND_DIR.to_owned())
            .into();

        let public_url = get("FLORG_PUBLIC_URL")
            .unwrap_or_else(|| DEFAULT_PUBLIC_URL.to_owned())
            .trim_end_matches('/')
            .to_owned();
        if !(public_url.starts_with("https://") || public_url.starts_with("http://")) {
            bail!("FLORG_PUBLIC_URL muss mit https:// oder http:// beginnen");
        }

        Ok(Self {
            database_url,
            bind,
            frontend_dir,
            public_url,
            smtp: smtp_from_lookup(&get)?,
        })
    }
}

/// All SMTP values or none. A partial set is an error, so a typo cannot
/// silently switch a production instance to log delivery (rule L11a).
fn smtp_from_lookup(get: &impl Fn(&str) -> Option<String>) -> Result<Option<SmtpConfig>> {
    const REQUIRED: [&str; 4] = [
        "FLORG_SMTP_HOST",
        "FLORG_SMTP_USER",
        "FLORG_SMTP_PASSWORD",
        "FLORG_MAIL_FROM",
    ];

    let port = get("FLORG_SMTP_PORT");
    let values: Vec<Option<String>> = REQUIRED.iter().map(|name| get(name)).collect();

    if port.is_none() && values.iter().all(Option::is_none) {
        return Ok(None);
    }

    let missing: Vec<&str> = REQUIRED
        .iter()
        .zip(&values)
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| *name)
        .collect();
    if !missing.is_empty() {
        bail!(
            "Die SMTP-Einstellungen sind unvollständig, es fehlt: {}. Setze alle SMTP-Werte oder keinen.",
            missing.join(", ")
        );
    }

    let port = match port {
        Some(port) => port
            .parse()
            .context("FLORG_SMTP_PORT muss eine Portnummer wie 587 sein")?,
        None => DEFAULT_SMTP_PORT,
    };

    let mut values = values.into_iter().flatten();
    let mut next = || values.next().expect("all required values are present");
    Ok(Some(SmtpConfig {
        host: next(),
        port,
        user: next(),
        password: next(),
        from: next(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(vars: &[(&str, &str)]) -> Result<Config> {
        Config::from_lookup(|name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    fn no_smtp_values_mean_log_delivery() {
        let config = config(&[]).unwrap();
        assert!(config.smtp.is_none());
        assert_eq!(config.public_url, "http://localhost:5173");
    }

    #[test]
    fn complete_smtp_values_are_accepted() {
        let config = config(&[
            ("FLORG_SMTP_HOST", "mail.example.org"),
            ("FLORG_SMTP_USER", "florg"),
            ("FLORG_SMTP_PASSWORD", "secret"),
            ("FLORG_MAIL_FROM", "florg@example.org"),
        ])
        .unwrap();
        let smtp = config.smtp.unwrap();
        assert_eq!(smtp.port, 587);
        assert!(!format!("{smtp:?}").contains("secret"));
    }

    #[test]
    fn partial_smtp_values_are_an_error() {
        let error = config(&[
            ("FLORG_SMTP_HOST", "mail.example.org"),
            ("FLORG_SMTP_USER", "florg"),
            ("FLORG_SMTP_PASSWORD", ""),
        ])
        .unwrap_err()
        .to_string();
        assert!(error.contains("FLORG_SMTP_PASSWORD"));
        assert!(error.contains("FLORG_MAIL_FROM"));
        assert!(!error.contains("FLORG_SMTP_HOST,"));
    }

    #[test]
    fn public_url_loses_its_trailing_slash() {
        let config = config(&[("FLORG_PUBLIC_URL", "https://florg.example.org/")]).unwrap();
        assert_eq!(config.public_url, "https://florg.example.org");
    }
}
