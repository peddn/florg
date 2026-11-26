//! Command line: the server and the operating commands (rule L12).

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel, Set};

use crate::{
    auth,
    config::Config,
    db,
    entity::account,
    mail::{self, Delivery, Mailer},
};

#[derive(Parser)]
#[command(name = "florg", version, about = "Fachlehrer Organizer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Startet den Server. Das ist auch die Vorgabe ohne Befehl.
    Serve,
    /// Verwaltet das Konto der Instanz.
    #[command(subcommand)]
    Account(AccountCommand),
    /// Prüft den Mailversand.
    #[command(subcommand)]
    Mail(MailCommand),
}

#[derive(Subcommand)]
pub enum AccountCommand {
    /// Legt das Konto an oder ändert dessen Adresse.
    Set(AccountSet),
}

#[derive(Args)]
pub struct AccountSet {
    /// Adresse, an die Anmeldelinks gehen.
    #[arg(long)]
    pub email: String,
    /// Name für Ausgaben.
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Subcommand)]
pub enum MailCommand {
    /// Verschickt eine Testmail an die Adresse des Kontos.
    Test,
}

/// Creates the account or changes its address. Changing the address ends
/// all sessions and withdraws open login links.
pub async fn account_set(db: &DatabaseConnection, args: AccountSet) -> Result<()> {
    let email = args.email.trim().to_owned();
    if email.parse::<lettre::Address>().is_err() {
        bail!("\"{email}\" ist keine gültige E-Mail-Adresse.");
    }
    let now = db::now();

    match auth::find_account(db).await? {
        None => {
            account::ActiveModel {
                email: Set(email.clone()),
                display_name: Set(args.name),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(db)
            .await?;
            println!("Konto angelegt. Anmeldelinks gehen an {email}.");
        }
        Some(existing) => {
            let account_id = existing.id;
            let email_changed = existing.email != email;
            let mut account = existing.into_active_model();
            account.email = Set(email.clone());
            if args.name.is_some() {
                account.display_name = Set(args.name);
            }
            account.updated_at = Set(now);
            account.update(db).await?;

            if email_changed {
                auth::revoke_all(db, account_id).await?;
                println!(
                    "Adresse geändert. Anmeldelinks gehen jetzt an {email}. \
                     Alle Sitzungen wurden beendet."
                );
            } else {
                println!("Konto aktualisiert. Die Adresse {email} ist unverändert.");
            }
        }
    }
    Ok(())
}

/// Sends a test mail to the address of the account.
pub async fn mail_test(db: &DatabaseConnection, config: &Config) -> Result<()> {
    let mailer = Mailer::from_config(config.smtp.as_ref())?;
    if mailer.delivery() == Delivery::Log {
        bail!(
            "Der Mailversand ist nicht eingerichtet. Setze die SMTP-Werte \
             (siehe backend/.env.example)."
        );
    }
    let account = auth::find_account(db).await?.context(
        "Es ist noch kein Konto angelegt. Lege es mit \"florg account set --email …\" an.",
    )?;

    let (subject, body) = mail::test_mail();
    mailer
        .send(&account.email, subject, &body)
        .await
        .context("Die Testmail konnte nicht verschickt werden")?;
    println!("Testmail an {} verschickt.", account.email);
    Ok(())
}
