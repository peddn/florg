//! Login by e-mail link, end to end against the HTTP API.

mod common;

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};
use chrono::{Duration, Utc};
use common::{Browser, EMAIL, PUBLIC_URL, TestApp, json};
use florg::{
    cli::{self, AccountSet},
    config::SmtpConfig,
    entity::{login_token, session},
    mail::Mailer,
};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};

#[tokio::test]
async fn login_by_link_signs_in_and_logout_signs_out() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    assert!(!app.is_authenticated(&mut browser).await);

    let response = app.submit_email(&mut browser, EMAIL).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await["delivery"], "mail");

    // The mail goes to the account and the link is built from the
    // configured public address (rules L2, L10).
    let mail = app.mails.lock().unwrap().last().unwrap().clone();
    assert_eq!(mail.to, EMAIL);
    let token = app.last_token();
    assert!(
        mail.body
            .contains(&format!("{PUBLIC_URL}/anmelden/bestaetigen#token={token}"))
    );

    // Following the link alone does not sign in (rule L6).
    assert!(!app.is_authenticated(&mut browser).await);

    let response = app.confirm(&mut browser, &token).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let session_cookie = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().to_owned())
        .find(|value| value.starts_with("florg_session="))
        .expect("session cookie is set");
    for attribute in ["HttpOnly", "Secure", "SameSite=Strict", "Path=/"] {
        assert!(session_cookie.contains(attribute), "missing {attribute}");
    }
    assert!(!browser.has("florg_login"), "binding cookie is removed");
    assert!(app.is_authenticated(&mut browser).await);

    let response = app.post(&mut browser, "/api/auth/logout").await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(!browser.has("florg_session"));
    assert!(!app.is_authenticated(&mut browser).await);
}

#[tokio::test]
async fn database_stores_only_hashes() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;
    app.confirm(&mut browser, &token).await;

    let tokens = login_token::Entity::find().all(&app.db).await.unwrap();
    assert_eq!(tokens.len(), 1);
    assert_ne!(tokens[0].token_hash, token);
    assert!(tokens[0].used_at.is_some());

    let (_, session_secret) = browser
        .cookies
        .iter()
        .find(|(name, _)| name == "florg_session")
        .unwrap();
    let sessions = session::Entity::find().all(&app.db).await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_ne!(&sessions[0].session_hash, session_secret);
}

#[tokio::test]
async fn link_works_only_once() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;
    // Keep the binding cookie, which a successful login removes.
    let mut second_attempt = browser.clone();

    assert_eq!(
        app.confirm(&mut browser, &token).await.status(),
        StatusCode::NO_CONTENT
    );
    let response = app.confirm(&mut second_attempt, &token).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(response).await["error"], "invalid_link");
}

#[tokio::test]
async fn link_works_only_in_the_requesting_browser() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;

    // Another browser without the binding cookie.
    let mut stranger = Browser::default();
    assert_eq!(
        app.confirm(&mut stranger, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
    // Another browser that requested its own link.
    let mut other = Browser::default();
    app.request_link(&mut other).await;
    assert_eq!(
        app.confirm(&mut other, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
    assert!(!app.is_authenticated(&mut stranger).await);
    assert!(!app.is_authenticated(&mut other).await);

    // The failed attempts have not used up the link.
    assert_eq!(
        app.confirm(&mut browser, &token).await.status(),
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn expired_link_is_rejected() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;

    let row = login_token::Entity::find()
        .one(&app.db)
        .await
        .unwrap()
        .unwrap();
    let mut row = row.into_active_model();
    row.expires_at = Set(Utc::now() - Duration::seconds(1));
    row.update(&app.db).await.unwrap();

    assert_eq!(
        app.confirm(&mut browser, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn unknown_token_is_rejected() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    app.request_link(&mut browser).await;

    assert_eq!(
        app.confirm(&mut browser, "not-a-token").await.status(),
        StatusCode::BAD_REQUEST
    );
}

/// Status, body and the cookies a response sets, without their random values.
async fn fingerprint(response: Response<Body>) -> (StatusCode, serde_json::Value, Vec<String>) {
    let status = response.status();
    let cookies = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| {
            let value = value.to_str().unwrap();
            let (name, rest) = value.split_once('=').unwrap();
            let attributes = rest.split_once(';').map(|(_, attributes)| attributes);
            format!("{name};{}", attributes.unwrap_or_default())
        })
        .collect();
    (status, json(response).await, cookies)
}

#[tokio::test]
async fn link_requests_are_limited_without_telling_the_browser() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let first = app.submit_email(&mut browser, EMAIL).await;
    for _ in 0..2 {
        app.request_link(&mut browser).await;
    }

    let limited = app.submit_email(&mut browser, EMAIL).await;
    assert_eq!(fingerprint(limited).await, fingerprint(first).await);
    assert_eq!(app.mail_count(), 3);
    assert_eq!(app.token_count().await, 3);
}

#[tokio::test]
async fn wrong_address_sends_nothing_and_looks_the_same() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let right = app.submit_email(&mut browser, EMAIL).await;
    assert_eq!(app.mail_count(), 1);

    let mut stranger = Browser::default();
    for email in ["fremd@example.org", "", "lehrer@example.org.evil.test"] {
        let wrong = app.submit_email(&mut stranger, email).await;
        assert_eq!(wrong.status(), StatusCode::OK);
        assert!(stranger.has("florg_login"), "binding cookie is set anyway");
        assert_eq!(app.mail_count(), 1, "no mail for {email:?}");
        assert_eq!(app.token_count().await, 1, "no token for {email:?}");
    }

    let wrong = app.submit_email(&mut stranger, "fremd@example.org").await;
    assert_eq!(fingerprint(wrong).await, fingerprint(right).await);
}

#[tokio::test]
async fn wrong_addresses_do_not_use_up_the_limit() {
    let app = TestApp::with_account().await;
    let mut stranger = Browser::default();
    for _ in 0..10 {
        app.submit_email(&mut stranger, "fremd@example.org").await;
    }

    // The owner of the account still gets a link.
    let mut browser = Browser::default();
    let token = app.request_link(&mut browser).await;
    assert_eq!(
        app.confirm(&mut browser, &token).await.status(),
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn address_is_matched_without_case_and_spaces() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();

    app.submit_email(&mut browser, "  Lehrer@Example.ORG ")
        .await;
    assert_eq!(app.mail_count(), 1);
    // The mail goes to the stored address, not to the entered spelling.
    assert_eq!(app.mails.lock().unwrap()[0].to, EMAIL);
}

#[tokio::test]
async fn link_request_without_account_sends_nothing_and_looks_the_same() {
    let with_account = TestApp::with_account().await;
    let right = with_account
        .submit_email(&mut Browser::default(), EMAIL)
        .await;

    let app = TestApp::new().await;
    let response = app.submit_email(&mut Browser::default(), EMAIL).await;
    assert_eq!(fingerprint(response).await, fingerprint(right).await);
    assert_eq!(app.mail_count(), 0);
}

#[tokio::test]
async fn link_request_needs_an_address_field() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();

    let response = app.post(&mut browser, "/api/auth/request-link").await;
    assert!(response.status().is_client_error());
    assert_eq!(app.mail_count(), 0);
}

#[tokio::test]
async fn failed_mail_does_not_use_up_the_limit() {
    // Nothing listens on this port, so sending fails at once.
    let smtp = SmtpConfig {
        host: "127.0.0.1".to_owned(),
        port: 9,
        user: "florg".to_owned(),
        password: "secret".to_owned(),
        from: "florg@example.org".to_owned(),
    };
    let mailer = Mailer::from_config(Some(&smtp)).unwrap();
    let app = TestApp::with_mailer(mailer, Arc::default()).await;
    app.set_email(EMAIL).await;

    let response = app.submit_email(&mut Browser::default(), EMAIL).await;
    // The browser cannot tell that the mail failed.
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(app.token_count().await, 0);
}

#[tokio::test]
async fn protected_endpoints_need_a_session() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();

    for uri in ["/api/auth/logout", "/api/auth/logout-all"] {
        let response = app.post(&mut browser, uri).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(json(response).await["error"], "unauthenticated");
    }

    // A made-up session identifier does not help.
    browser
        .cookies
        .push(("florg_session".to_owned(), "0".repeat(64)));
    let response = app.post(&mut browser, "/api/auth/logout").await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_ends_one_session_and_logout_all_ends_every_session() {
    let app = TestApp::with_account().await;
    let mut phone = app.sign_in().await;
    let mut laptop = app.sign_in().await;
    let mut tablet = app.sign_in().await;

    app.post(&mut phone, "/api/auth/logout").await;
    assert!(!app.is_authenticated(&mut phone).await);
    assert!(app.is_authenticated(&mut laptop).await);
    assert!(app.is_authenticated(&mut tablet).await);

    let response = app.post(&mut laptop, "/api/auth/logout-all").await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(!app.is_authenticated(&mut laptop).await);
    assert!(!app.is_authenticated(&mut tablet).await);
}

#[tokio::test]
async fn expired_session_is_signed_out() {
    let app = TestApp::with_account().await;
    let mut browser = app.sign_in().await;

    let row = session::Entity::find().one(&app.db).await.unwrap().unwrap();
    let mut row = row.into_active_model();
    row.expires_at = Set(Utc::now() - Duration::seconds(1));
    row.update(&app.db).await.unwrap();

    assert!(!app.is_authenticated(&mut browser).await);
}

#[tokio::test]
async fn session_in_use_is_extended() {
    let app = TestApp::with_account().await;
    let mut browser = app.sign_in().await;

    let before = session::Entity::find().one(&app.db).await.unwrap().unwrap();
    let mut row = before.clone().into_active_model();
    row.last_used_at = Set(before.last_used_at - Duration::hours(2));
    row.expires_at = Set(before.expires_at - Duration::hours(2));
    row.update(&app.db).await.unwrap();

    assert!(app.is_authenticated(&mut browser).await);
    let after = session::Entity::find().one(&app.db).await.unwrap().unwrap();
    assert!(after.expires_at >= before.expires_at);
    assert!(after.last_used_at >= before.last_used_at);
}

#[tokio::test]
async fn changing_the_address_ends_sessions_and_open_links() {
    let app = TestApp::with_account().await;
    let mut signed_in = app.sign_in().await;
    let mut waiting = Browser::default();
    let open_token = app.request_link(&mut waiting).await;

    // Setting the same address again changes nothing.
    app.set_email(EMAIL).await;
    assert!(app.is_authenticated(&mut signed_in).await);

    app.set_email("neu@example.org").await;
    assert!(!app.is_authenticated(&mut signed_in).await);
    assert_eq!(
        app.confirm(&mut waiting, &open_token).await.status(),
        StatusCode::BAD_REQUEST
    );

    // The old address no longer triggers a mail, the new one does.
    let mut browser = Browser::default();
    let before = app.mail_count();
    app.submit_email(&mut browser, EMAIL).await;
    assert_eq!(app.mail_count(), before);
    app.submit_email(&mut browser, "neu@example.org").await;
    assert_eq!(app.mail_count(), before + 1);
    assert_eq!(
        app.mails.lock().unwrap().last().unwrap().to,
        "neu@example.org"
    );
}

#[tokio::test]
async fn invalid_address_is_refused() {
    let app = TestApp::new().await;
    let args = AccountSet {
        email: "keine-adresse".to_owned(),
        name: None,
    };
    assert!(cli::account_set(&app.db, args).await.is_err());
}
