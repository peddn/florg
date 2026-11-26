//! School years and school-free periods against the HTTP API.

mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use common::{Browser, TestApp, json};
use serde_json::{Value, json as value};

struct Session {
    app: TestApp,
    browser: Browser,
}

impl Session {
    async fn new() -> Self {
        let app = TestApp::with_account().await;
        let browser = app.sign_in().await;
        Self { app, browser }
    }

    async fn request(&mut self, method: &str, uri: &str, body: Option<Value>) -> Response<Body> {
        self.app.send(&mut self.browser, method, uri, body).await
    }

    async fn get(&mut self, uri: &str) -> Value {
        let response = self.request("GET", uri, None).await;
        assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
        json(response).await
    }

    /// Creates the school year 2026/27 with the dates of its template.
    async fn create_2026_27(&mut self) -> Value {
        let body = value!({
            "name": "2026/27", "start_date": "2026-09-15", "end_date": "2027-07-30"
        });
        let response = self.request("POST", "/api/school-years", Some(body)).await;
        assert_eq!(response.status(), StatusCode::CREATED);
        json(response).await
    }
}

fn period<'a>(year: &'a Value, name: &str) -> &'a Value {
    year["free_periods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|period| period["name"] == name)
        .unwrap_or_else(|| panic!("free period {name} exists"))
}

fn period_input(name: &str, kind: &str, start: &str, end: &str) -> Option<Value> {
    Some(value!({ "name": name, "kind": kind, "start_date": start, "end_date": end }))
}

async fn assert_error(response: Response<Body>, status: StatusCode, code: &str) {
    assert_eq!(response.status(), status);
    let body = json(response).await;
    assert_eq!(body["error"], code);
    assert!(!body["message"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn school_year_endpoints_need_a_session() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();
    let input = value!({ "name": "x", "start_date": "2026-09-15", "end_date": "2027-07-30" });

    for (method, uri) in [
        ("GET", "/api/school-years"),
        ("GET", "/api/school-years/templates"),
        ("POST", "/api/school-years"),
        ("GET", "/api/school-years/1"),
        ("PUT", "/api/school-years/1"),
        ("DELETE", "/api/school-years/1"),
        ("POST", "/api/school-years/1/free-periods"),
        ("PUT", "/api/free-periods/1"),
        ("DELETE", "/api/free-periods/1"),
    ] {
        let response = app
            .send(&mut browser, method, uri, Some(input.clone()))
            .await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri}"
        );
    }
}

#[tokio::test]
async fn templates_come_from_the_holiday_data_file() {
    let mut session = Session::new().await;
    let templates = session.get("/api/school-years/templates").await;

    assert_eq!(templates.as_array().unwrap().len(), 4);
    assert_eq!(
        templates[0],
        value!({ "name": "2026/27", "start_date": "2026-09-15", "end_date": "2027-07-30" })
    );
}

#[tokio::test]
async fn new_school_year_is_prefilled_with_holidays_and_public_holidays() {
    let mut session = Session::new().await;
    let year = session.create_2026_27().await;

    assert_eq!(year["name"], "2026/27");
    let periods = year["free_periods"].as_array().unwrap();
    let count = |kind: &str| periods.iter().filter(|p| p["kind"] == kind).count();
    assert_eq!(count("holiday"), 5);
    assert_eq!(count("public_holiday"), 13);
    assert_eq!(count("other"), 0);

    let christmas = period(&year, "Weihnachtsferien");
    assert_eq!(christmas["start_date"], "2026-12-24");
    assert_eq!(christmas["end_date"], "2027-01-08");
    assert_eq!(period(&year, "Buß- und Bettag")["start_date"], "2026-11-18");
    // Entered although they fall on a Saturday or into the holidays (D-F2a).
    assert_eq!(
        period(&year, "Tag der Deutschen Einheit")["start_date"],
        "2026-10-03"
    );
    assert_eq!(period(&year, "Neujahr")["start_date"], "2027-01-01");

    // Sorted by date.
    let starts: Vec<_> = periods
        .iter()
        .map(|p| p["start_date"].as_str().unwrap())
        .collect();
    assert!(starts.is_sorted());

    // Counted independently from the published dates.
    assert_eq!(year["school_days"], 185);

    // The stored year is the same as the returned one.
    let id = year["id"].as_i64().unwrap();
    assert_eq!(session.get(&format!("/api/school-years/{id}")).await, year);
    let list = session.get("/api/school-years").await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["name"], "2026/27");
}

#[tokio::test]
async fn school_year_outside_the_data_file_gets_public_holidays_only() {
    let mut session = Session::new().await;
    let body = value!({ "name": "2035/36", "start_date": "2035-09-11", "end_date": "2036-07-25" });
    let response = session
        .request("POST", "/api/school-years", Some(body))
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let year = json(response).await;

    let periods = year["free_periods"].as_array().unwrap();
    assert!(!periods.is_empty());
    assert!(periods.iter().all(|p| p["kind"] == "public_holiday"));
}

#[tokio::test]
async fn invalid_school_years_are_refused() {
    let mut session = Session::new().await;
    session.create_2026_27().await;

    let cases = [
        (
            value!({ "name": "2026/27", "start_date": "2026-09-15", "end_date": "2027-07-30" }),
            StatusCode::CONFLICT,
            "conflict",
        ),
        (
            value!({ "name": "  ", "start_date": "2026-09-15", "end_date": "2027-07-30" }),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid",
        ),
        (
            value!({ "name": "rückwärts", "start_date": "2027-07-30", "end_date": "2026-09-15" }),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid",
        ),
        (
            value!({ "name": "zu lang", "start_date": "2026-09-15", "end_date": "2028-07-30" }),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid",
        ),
    ];
    for (body, status, code) in cases {
        let response = session
            .request("POST", "/api/school-years", Some(body))
            .await;
        assert_error(response, status, code).await;
    }
    assert_eq!(
        session
            .get("/api/school-years")
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // A date that does not exist never reaches the rules.
    let body = value!({ "name": "x", "start_date": "2026-02-30", "end_date": "2027-07-30" });
    let response = session
        .request("POST", "/api/school-years", Some(body))
        .await;
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn free_periods_can_be_added_changed_and_deleted() {
    let mut session = Session::new().await;
    let year = session.create_2026_27().await;
    let id = year["id"].as_i64().unwrap();
    let year_uri = format!("/api/school-years/{id}");

    // A school-internal free day on a Friday costs one school day.
    let response = session
        .request(
            "POST",
            &format!("{year_uri}/free-periods"),
            period_input("  Pädagogischer Tag ", "other", "2026-10-09", "2026-10-09"),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json(response).await;
    assert_eq!(created["name"], "Pädagogischer Tag");
    let period_uri = format!("/api/free-periods/{}", created["id"]);
    assert_eq!(session.get(&year_uri).await["school_days"], 184);

    // Extended to the Thursday before: one more day.
    let response = session
        .request(
            "PUT",
            &period_uri,
            period_input("Pädagogische Tage", "other", "2026-10-08", "2026-10-09"),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let year = session.get(&year_uri).await;
    assert_eq!(year["school_days"], 183);
    assert_eq!(
        period(&year, "Pädagogische Tage")["start_date"],
        "2026-10-08"
    );

    let response = session.request("DELETE", &period_uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(session.get(&year_uri).await["school_days"], 185);

    let response = session.request("DELETE", &period_uri, None).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn invalid_free_periods_are_refused() {
    let mut session = Session::new().await;
    let year = session.create_2026_27().await;
    let uri = format!("/api/school-years/{}/free-periods", year["id"]);
    let existing = format!("/api/free-periods/{}", year["free_periods"][0]["id"]);

    let cases = [
        period_input("", "other", "2026-10-09", "2026-10-09"),
        period_input("rückwärts", "other", "2026-10-09", "2026-10-08"),
        period_input("zu früh", "other", "2026-09-14", "2026-09-15"),
        period_input("zu spät", "other", "2027-07-30", "2027-07-31"),
    ];
    for body in cases {
        let response = session.request("POST", &uri, body.clone()).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
        let response = session.request("PUT", &existing, body).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }

    // An unknown kind never reaches the rules.
    let body = period_input("x", "vacation", "2026-10-09", "2026-10-09");
    let response = session.request("POST", &uri, body).await;
    assert!(response.status().is_client_error());

    let body = period_input("x", "other", "2026-10-09", "2026-10-09");
    let response = session
        .request("POST", "/api/school-years/999/free-periods", body)
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn school_year_can_be_changed_unless_free_periods_fall_outside() {
    let mut session = Session::new().await;
    let year = session.create_2026_27().await;
    let uri = format!("/api/school-years/{}", year["id"]);

    // Starting on the Monday before adds one school day.
    let body =
        value!({ "name": "Schuljahr 26/27", "start_date": "2026-09-14", "end_date": "2027-07-30" });
    let response = session.request("PUT", &uri, Some(body)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let changed = json(response).await;
    assert_eq!(changed["name"], "Schuljahr 26/27");
    assert_eq!(changed["school_days"], 186);
    assert_eq!(changed["free_periods"], year["free_periods"]);

    // Keeping the own name is no conflict.
    let body =
        value!({ "name": "Schuljahr 26/27", "start_date": "2026-09-15", "end_date": "2027-07-30" });
    let response = session.request("PUT", &uri, Some(body)).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Starting after the first public holiday would leave it outside.
    let body =
        value!({ "name": "Schuljahr 26/27", "start_date": "2026-10-05", "end_date": "2027-07-30" });
    let response = session.request("PUT", &uri, Some(body)).await;
    assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    assert_eq!(session.get(&uri).await["start_date"], "2026-09-15");

    // Another year may not take an existing name.
    let body = value!({ "name": "2027/28", "start_date": "2027-09-14", "end_date": "2028-07-28" });
    let response = session
        .request("POST", "/api/school-years", Some(body))
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let other = format!("/api/school-years/{}", json(response).await["id"]);
    let body =
        value!({ "name": "Schuljahr 26/27", "start_date": "2027-09-14", "end_date": "2028-07-28" });
    let response = session.request("PUT", &other, Some(body)).await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;

    // Latest year first.
    let list = session.get("/api/school-years").await;
    assert_eq!(list[0]["name"], "2027/28");
    assert_eq!(list[1]["name"], "Schuljahr 26/27");
}

#[tokio::test]
async fn deleting_a_school_year_deletes_its_free_periods() {
    let mut session = Session::new().await;
    let year = session.create_2026_27().await;
    let uri = format!("/api/school-years/{}", year["id"]);
    let period_uri = format!("/api/free-periods/{}", year["free_periods"][0]["id"]);

    let response = session.request("DELETE", &uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = session.request("GET", &uri, None).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    let body = period_input("x", "other", "2026-10-09", "2026-10-09");
    let response = session.request("PUT", &period_uri, body).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    assert!(
        session
            .get("/api/school-years")
            .await
            .as_array()
            .unwrap()
            .is_empty()
    );

    // The name is free again.
    session.create_2026_27().await;
}
