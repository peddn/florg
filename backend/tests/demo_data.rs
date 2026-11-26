//! The command that fills the database with example data.

mod common;

use axum::http::StatusCode;
use chrono::NaiveDate;
use common::{TestApp, json};
use florg::demo_data::{self, Counts};
use serde_json::Value;

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 26).unwrap()
}

const EXPECTED: Counts = Counts {
    school_years: 1,
    // 5 holidays and 13 public holidays of 2027/28.
    free_periods: 18,
    subjects: 4,
    school_classes: 8,
    students: 140,
    learning_groups: 8,
    // Everyone with a class is in one group, one student in two.
    group_memberships: 139,
};

async fn get(app: &TestApp, browser: &mut common::Browser, uri: &str) -> Value {
    let response = app.get(browser, uri).await;
    assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
    json(response).await
}

#[tokio::test]
async fn empty_database_is_filled_with_the_next_school_year() {
    let app = TestApp::with_account().await;
    let summary = demo_data::run(&app.db, false, false, today())
        .await
        .unwrap();

    assert_eq!(summary.school_year, "2027/28");
    assert_eq!(summary.deleted, None);
    assert_eq!(summary.created, EXPECTED);

    // The data is usable through the API like data entered by hand.
    let mut browser = app.sign_in().await;
    let years = get(&app, &mut browser, "/api/school-years").await;
    assert_eq!(years[0]["name"], "2027/28");
    assert_eq!(years[0]["start_date"], "2027-09-14");
    let year_uri = format!("/api/school-years/{}", years[0]["id"]);
    let year = get(&app, &mut browser, &year_uri).await;
    assert!(year["school_days"].as_u64().unwrap() > 180);

    let students = get(&app, &mut browser, &format!("{year_uri}/students")).await;
    let students = students.as_array().unwrap();
    assert_eq!(students.len(), 140);
    // Sorted by class: "5a" first, the two students without a class last.
    assert_eq!(students[0]["school_class_name"], "5a");
    assert_eq!(students[137]["school_class_name"], "9b");
    assert_eq!(students[138]["school_class_name"], Value::Null);
    assert_eq!(students[139]["school_class_name"], Value::Null);
    let in_7a = students
        .iter()
        .filter(|student| student["school_class_name"] == "7a")
        .count();
    assert_eq!(in_7a, 18);

    let subjects = get(&app, &mut browser, "/api/subjects").await;
    assert_eq!(subjects.as_array().unwrap().len(), 4);

    let groups = get(&app, &mut browser, &format!("{year_uri}/learning-groups")).await;
    let names: Vec<_> = groups
        .as_array()
        .unwrap()
        .iter()
        .map(|group| {
            format!(
                "{} ({}, {})",
                group["name"].as_str().unwrap(),
                group["subject_short_name"].as_str().unwrap(),
                group["member_count"]
            )
        })
        .collect();
    assert_eq!(
        names,
        [
            "5a Werken (WG, 16)",
            "6a Werken (WG, 17)",
            "7a Technik (Te, 18)",
            "7b Technik (Te, 19)",
            "8a/b Technik Gruppe 1 (Te, 18)",
            "8a/b Technik Gruppe 2 (Te, 18)",
            "9a/b BLO-P (BLO-P, 16)",
            "9a/b Ernährung und Soziales (ES, 17)",
        ]
    );

    // One student left group 1 during the year and joined group 2 the
    // day after.
    let group_1 = get(
        &app,
        &mut browser,
        &format!("/api/learning-groups/{}", groups[4]["id"]),
    )
    .await;
    let left: Vec<_> = group_1["members"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|member| !member["left_on"].is_null())
        .collect();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0]["left_on"], "2028-02-15");
    let group_2 = get(
        &app,
        &mut browser,
        &format!("/api/learning-groups/{}", groups[5]["id"]),
    )
    .await;
    let joined: Vec<_> = group_2["members"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|member| !member["joined_on"].is_null())
        .collect();
    assert_eq!(joined.len(), 1);
    assert_eq!(joined[0]["id"], left[0]["id"]);
    assert_eq!(joined[0]["joined_on"], "2028-02-16");
}

#[tokio::test]
async fn every_run_creates_the_same_data() {
    let mut lists = Vec::new();
    for _ in 0..2 {
        let app = TestApp::with_account().await;
        demo_data::run(&app.db, false, false, today())
            .await
            .unwrap();
        let mut browser = app.sign_in().await;
        let students = get(&app, &mut browser, "/api/school-years/1/students").await;
        let names: Vec<_> = students
            .as_array()
            .unwrap()
            .iter()
            .map(|student| {
                format!(
                    "{} {} {} {}",
                    student["first_name"],
                    student["last_name"],
                    student["school_class_name"],
                    student["form_of_address"]
                )
            })
            .collect();
        lists.push(names);
    }
    assert_eq!(lists[0], lists[1]);
}

#[tokio::test]
async fn database_with_data_is_left_alone_without_reset() {
    let app = TestApp::with_account().await;
    demo_data::run(&app.db, false, false, today())
        .await
        .unwrap();
    let mut browser = app.sign_in().await;
    // A change by hand that must survive.
    let response = app
        .send(
            &mut browser,
            "POST",
            "/api/classes",
            Some(serde_json::json!({ "name": "10M" })),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let error = demo_data::run(&app.db, false, false, today())
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("--reset"), "{error}");

    let classes = get(&app, &mut browser, "/api/classes").await;
    assert_eq!(classes.as_array().unwrap().len(), 9);
    let students = get(&app, &mut browser, "/api/school-years/1/students").await;
    assert_eq!(students.as_array().unwrap().len(), 140);
}

#[tokio::test]
async fn reset_replaces_the_data_and_keeps_account_and_session() {
    let app = TestApp::with_account().await;
    demo_data::run(&app.db, false, false, today())
        .await
        .unwrap();
    let mut browser = app.sign_in().await;
    let response = app
        .send(
            &mut browser,
            "POST",
            "/api/classes",
            Some(serde_json::json!({ "name": "10M" })),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let summary = demo_data::run(&app.db, false, true, today()).await.unwrap();
    assert_eq!(
        summary.deleted,
        Some(Counts {
            school_classes: 9,
            ..EXPECTED
        })
    );
    assert_eq!(summary.created, EXPECTED);

    // Still signed in, and the change by hand is gone.
    assert!(app.is_authenticated(&mut browser).await);
    let classes = get(&app, &mut browser, "/api/classes").await;
    assert_eq!(classes.as_array().unwrap().len(), 8);
    let years = get(&app, &mut browser, "/api/school-years").await;
    assert_eq!(years.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn reset_is_refused_while_mail_delivery_is_configured() {
    let app = TestApp::with_account().await;
    demo_data::run(&app.db, false, false, today())
        .await
        .unwrap();

    let error = demo_data::run(&app.db, true, true, today())
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("Mailversand"), "{error}");

    // Nothing was deleted.
    let mut browser = app.sign_in().await;
    let students = get(&app, &mut browser, "/api/school-years/1/students").await;
    assert_eq!(students.as_array().unwrap().len(), 140);
}

#[tokio::test]
async fn empty_database_may_be_filled_even_with_mail_delivery() {
    // Without existing data nothing can be lost, so nothing is refused.
    let app = TestApp::with_account().await;
    let summary = demo_data::run(&app.db, true, true, today()).await.unwrap();
    assert_eq!(summary.deleted, None);
    assert_eq!(summary.created, EXPECTED);
}
