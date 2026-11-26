//! Outgoing mail (rules L11, L11a).

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, Result};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tracing::warn;

use crate::config::SmtpConfig;

/// Port for SMTP over implicit TLS. Every other port uses STARTTLS.
const IMPLICIT_TLS_PORT: u16 = 465;
const SMTP_TIMEOUT: Duration = Duration::from_secs(20);

/// How a mail left the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Sent through the mail server.
    Mail,
    /// Written to the server log because no mail server is configured.
    Log,
}

/// A mail kept in memory by [`Mailer::Capture`].
#[derive(Debug, Clone)]
pub struct CapturedMail {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Clone)]
pub enum Mailer {
    /// Sends over an encrypted SMTP connection.
    Smtp {
        transport: Box<AsyncSmtpTransport<Tokio1Executor>>,
        from: Mailbox,
    },
    /// Writes mails to the log (rule L11a).
    Log,
    /// Keeps mails in memory. For automated tests only.
    Capture(Arc<Mutex<Vec<CapturedMail>>>),
}

impl Mailer {
    pub fn from_config(smtp: Option<&SmtpConfig>) -> Result<Self> {
        let Some(smtp) = smtp else {
            return Ok(Self::Log);
        };

        let from = smtp
            .from
            .parse()
            .context("FLORG_MAIL_FROM muss eine Adresse wie florg@example.org sein")?;
        // Both variants refuse to send over an unencrypted connection.
        let builder = if smtp.port == IMPLICIT_TLS_PORT {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)
        }
        .context("FLORG_SMTP_HOST ist kein gültiger Hostname")?;
        let transport = builder
            .port(smtp.port)
            .credentials(Credentials::new(smtp.user.clone(), smtp.password.clone()))
            .timeout(Some(SMTP_TIMEOUT))
            .build();

        Ok(Self::Smtp {
            transport: Box::new(transport),
            from,
        })
    }

    /// Whether mails are sent or only written to the log.
    pub fn delivery(&self) -> Delivery {
        match self {
            Self::Smtp { .. } | Self::Capture(_) => Delivery::Mail,
            Self::Log => Delivery::Log,
        }
    }

    /// Sends a plain text mail.
    pub async fn send(&self, to: &str, subject: &str, body: &str) -> Result<Delivery> {
        match self {
            Self::Smtp { transport, from } => {
                let message = Message::builder()
                    .from(from.clone())
                    .to(to.parse().context("invalid recipient address")?)
                    .subject(subject)
                    .header(ContentType::TEXT_PLAIN)
                    .body(body.to_owned())
                    .context("could not build mail")?;
                transport
                    .send(message)
                    .await
                    .context("could not send mail")?;
            }
            Self::Log => {
                warn!("no mail server configured, mail to {to} follows:\n{subject}\n\n{body}");
            }
            Self::Capture(mails) => {
                mails.lock().expect("mail capture lock").push(CapturedMail {
                    to: to.to_owned(),
                    subject: subject.to_owned(),
                    body: body.to_owned(),
                });
            }
        }
        Ok(self.delivery())
    }
}

/// Subject and text of the mail with a login link.
pub fn login_mail(link: &str) -> (&'static str, String) {
    let body = format!(
        "Hallo,\n\n\
         mit diesem Link meldest du dich bei florg an:\n\n\
         {link}\n\n\
         Der Link gilt 15 Minuten und nur einmal. Er funktioniert nur in dem \
         Browser, in dem du ihn angefordert hast.\n\n\
         Falls du keinen Link angefordert hast, kannst du diese Mail \
         ignorieren. Ohne deinen Browser ist der Link wertlos.\n"
    );
    ("Dein Anmeldelink für florg", body)
}

/// Subject and text of the test mail (rule L12).
pub fn test_mail() -> (&'static str, String) {
    (
        "Testmail von florg",
        "Hallo,\n\ndiese Testmail zeigt, dass der Mailversand von florg funktioniert.\n".to_owned(),
    )
}
