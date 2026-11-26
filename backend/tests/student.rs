//! Students, enrollments and the CSV import against the HTTP API.

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

    async fn create(&mut self, uri: &str, body: Value) -> Value {
        let response = self.request("POST", uri, Some(body)).await;
        assert_eq!(response.status(), StatusCode::CREATED, "POST {uri}");
        json(response).await
    }

    /// Creates a school year and returns the address of its students.
    async fn school_year(&mut self, name: &str, start: &str, end: &str) -> String {
        let body = value!({ "name": name, "start_date": start, "end_date": end });
        let year = self.create("/api/school-years", body).await;
        format!("/api/school-years/{}/students", year["id"])
    }

    async fn year_2026_27(&mut self) -> String {
        self.school_year("2026/27", "2026-09-15", "2027-07-30")
            .await
    }

    async fn class(&mut self, name: &str) -> i64 {
        let class = self.create("/api/classes", value!({ "name": name })).await;
        class["id"].as_i64().unwrap()
    }

    async fn import(&mut self, students_uri: &str, csv: &str, dry_run: bool) -> Value {
        let body = value!({ "csv": csv, "dry_run": dry_run });
        let response = self
            .request("POST", &format!("{students_uri}/import"), Some(body))
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        json(response).await
    }

    /// "Nachname Vorname (Klasse)" of every student, in list order.
    async fn listed(&mut self, students_uri: &str) -> Vec<String> {
        self.get(students_uri)
            .await
            .as_array()
            .unwrap()
            .iter()
            .map(|student| {
                format!(
                    "{} {} ({})",
                    student["last_name"].as_str().unwrap_or("-"),
                    student["first_name"].as_str().unwrap(),
                    student["school_class_name"].as_str().unwrap_or("-")
                )
            })
            .collect()
    }
}

fn student(first_name: &str, last_name: Option<&str>, address: &str, class: Option<i64>) -> Value {
    value!({
        "first_name": first_name,
        "last_name": last_name,
        "form_of_address": address,
        "school_class_id": class,
    })
}

async fn assert_error(response: Response<Body>, status: StatusCode, code: &str) {
    assert_eq!(response.status(), status);
    let body = json(response).await;
    assert_eq!(body["error"], code);
    assert!(!body["message"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn student_endpoints_need_a_session() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();

    for (method, uri) in [
        ("GET", "/api/school-years/1/students"),
        ("POST", "/api/school-years/1/students"),
        ("POST", "/api/school-years/1/students/import"),
        ("PUT", "/api/school-years/1/students/1"),
        ("DELETE", "/api/school-years/1/students/1"),
    ] {
        let body = Some(value!({ "csv": "Vorname\nA\n", "dry_run": false }));
        let response = app.send(&mut browser, method, uri, body).await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri}"
        );
    }
}

#[tokio::test]
async fn students_can_be_created_changed_and_removed() {
    let mut session = Session::new().await;
    let uri = session.year_2026_27().await;
    let class_7a = session.class("7a").await;
    let class_10a = session.class("10a").await;
    assert!(session.listed(&uri).await.is_empty());

    let anna = session
        .create(
            &uri,
            student("  Anna ", Some(" Huber "), "she", Some(class_10a)),
        )
        .await;
    assert_eq!(anna["first_name"], "Anna");
    assert_eq!(anna["last_name"], "Huber");
    assert_eq!(anna["form_of_address"], "she");
    assert_eq!(anna["school_class_name"], "10a");
    session
        .create(&uri, student("Ben", Some("Maier"), "he", Some(class_7a)))
        .await;
    session
        .create(
            &uri,
            student("Zoe", Some("Adam"), "name_only", Some(class_7a)),
        )
        .await;
    // Working with an abbreviation only, without a class.
    let kim = session
        .create(&uri, student("K.", Some("  "), "name_only", None))
        .await;
    assert_eq!(kim["last_name"], Value::Null);

    // By class ("7a" before "10a"), then last name; without a class last.
    assert_eq!(
        session.listed(&uri).await,
        [
            "Adam Zoe (7a)",
            "Maier Ben (7a)",
            "Huber Anna (10a)",
            "- K. (-)"
        ]
    );

    let anna_uri = format!("{uri}/{}", anna["id"]);
    let response = session
        .request(
            "PUT",
            &anna_uri,
            Some(student("Anne", Some("Huber-Lang"), "name_only", None)),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let changed = json(response).await;
    assert_eq!(changed["first_name"], "Anne");
    assert_eq!(changed["school_class_id"], Value::Null);
    // Without a class she moves to the end, behind the student without a
    // last name.
    assert_eq!(
        session.listed(&uri).await,
        [
            "Adam Zoe (7a)",
            "Maier Ben (7a)",
            "- K. (-)",
            "Huber-Lang Anne (-)"
        ]
    );

    let response = session.request("DELETE", &anna_uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(session.listed(&uri).await.len(), 3);
    let response = session.request("DELETE", &anna_uri, None).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn invalid_students_are_refused() {
    let mut session = Session::new().await;
    let uri = session.year_2026_27().await;
    let existing = session
        .create(&uri, student("Anna", None, "she", None))
        .await;
    let existing_uri = format!("{uri}/{}", existing["id"]);
    let long = "x".repeat(101);

    let invalid = [
        student(" ", Some("Huber"), "she", None),
        student(&long, None, "she", None),
        student("Anna", Some(&long), "she", None),
        student("Anna", None, "she", Some(999)),
    ];
    for body in invalid {
        let response = session.request("POST", &uri, Some(body.clone())).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
        let response = session.request("PUT", &existing_uri, Some(body)).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }

    // An unknown form of address never reaches the rules.
    let response = session
        .request("POST", &uri, Some(student("Anna", None, "it", None)))
        .await;
    assert!(response.status().is_client_error());

    for unknown in [
        "/api/school-years/999/students".to_owned(),
        format!("{uri}/999"),
    ] {
        let method = if unknown.ends_with("/students") {
            "POST"
        } else {
            "PUT"
        };
        let response = session
            .request(method, &unknown, Some(student("Anna", None, "she", None)))
            .await;
        assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    }
    let response = session
        .request("GET", "/api/school-years/999/students", None)
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    assert_eq!(session.listed(&uri).await.len(), 1);
}

#[tokio::test]
async fn students_belong_to_their_school_year_only() {
    let mut session = Session::new().await;
    let first = session.year_2026_27().await;
    let second = session
        .school_year("2027/28", "2027-09-14", "2028-07-28")
        .await;
    let anna = session
        .create(&first, student("Anna", None, "she", None))
        .await;

    assert_eq!(session.listed(&first).await.len(), 1);
    assert!(session.listed(&second).await.is_empty());
    // A student of one school year cannot be changed through another.
    let response = session
        .request(
            "PUT",
            &format!("{second}/{}", anna["id"]),
            Some(student("X", None, "she", None)),
        )
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn school_year_and_class_in_use_cannot_be_deleted() {
    let mut session = Session::new().await;
    let uri = session.year_2026_27().await;
    let year_uri = uri.trim_end_matches("/students").to_owned();
    let class_7a = session.class("7a").await;
    let class_uri = format!("/api/classes/{class_7a}");
    let anna = session
        .create(&uri, student("Anna", None, "she", Some(class_7a)))
        .await;

    let response = session.request("DELETE", &year_uri, None).await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;
    let response = session.request("DELETE", &class_uri, None).await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;
    // Renaming a class in use is fine and shows in the list.
    let response = session
        .request("PUT", &class_uri, Some(value!({ "name": "7c" })))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(session.listed(&uri).await, ["- Anna (7c)"]);

    let response = session
        .request("DELETE", &format!("{uri}/{}", anna["id"]), None)
        .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = session.request("DELETE", &class_uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = session.request("DELETE", &year_uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

const CSV: &str = "Vorname;Nachname;Klasse;Geschlecht\n\
                   Anna;Huber;7a;w\n\
                   Ben;Maier;7b;m\n\
                   Kim;Lang;7b;d\n";

#[tokio::test]
async fn csv_preview_creates_nothing() {
    let mut session = Session::new().await;
    let uri = session.year_2026_27().await;
    session.class("7a").await;

    let preview = session.import(&uri, CSV, true).await;
    assert_eq!(preview["imported"], false);
    assert_eq!(preview["delimiter"], "Semikolon");
    assert_eq!(preview["new_count"], 3);
    assert_eq!(preview["duplicate_count"], 0);
    assert_eq!(preview["error_count"], 0);
    assert_eq!(preview["new_classes"], value!(["7b"]));
    assert_eq!(preview["rows"][0]["status"], "new");
    assert_eq!(preview["rows"][0]["form_of_address"], "she");
    assert_eq!(preview["rows"][2]["form_of_address"], "name_only");

    assert!(session.listed(&uri).await.is_empty());
    assert_eq!(
        session.get("/api/classes").await.as_array().unwrap().len(),
        1
    );
}

#[tokio::test]
async fn csv_import_creates_students_and_missing_classes() {
    let mut session = Session::new().await;
    let uri = session.year_2026_27().await;
    session.class("7a").await;

    let result = session.import(&uri, CSV, false).await;
    assert_eq!(result["imported"], true);
    assert_eq!(result["new_count"], 3);
    assert_eq!(
        session.listed(&uri).await,
        ["Huber Anna (7a)", "Lang Kim (7b)", "Maier Ben (7b)"]
    );
    let classes = session.get("/api/classes").await;
    let names: Vec<_> = classes
        .as_array()
        .unwrap()
        .iter()
        .map(|class| class["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["7a", "7b"]);
    let students = session.get(&uri).await;
    assert_eq!(students[0]["form_of_address"], "she");
    assert_eq!(students[2]["form_of_address"], "he");

    // Importing the same file again changes nothing (D-I6).
    let again = session.import(&uri, CSV, false).await;
    assert_eq!(again["imported"], true);
    assert_eq!(again["new_count"], 0);
    assert_eq!(again["duplicate_count"], 3);
    assert_eq!(again["new_classes"], value!([]));
    assert_eq!(session.listed(&uri).await.len(), 3);

    // The same students may be imported into another school year.
    let other = session
        .school_year("2027/28", "2027-09-14", "2028-07-28")
        .await;
    let result = session.import(&other, CSV, false).await;
    assert_eq!(result["new_count"], 3);
    assert_eq!(session.listed(&uri).await.len(), 3);
}

#[tokio::test]
async fn csv_import_with_an_error_creates_nothing() {
    let mut session = Session::new().await;
    let uri = session.year_2026_27().await;
    let csv = "Vorname;Klasse;Anrede\nAnna;7a;sie\n;7a;er\nBen;8c;es\n";

    let result = session.import(&uri, csv, false).await;
    assert_eq!(result["imported"], false);
    assert_eq!(result["new_count"], 1);
    assert_eq!(result["error_count"], 2);
    assert_eq!(result["rows"][1]["status"], "error");
    assert_eq!(result["rows"][1]["line"], 3);
    assert_eq!(result["rows"][1]["message"], "Der Vorname fehlt.");

    assert!(session.listed(&uri).await.is_empty());
    assert!(
        session
            .get("/api/classes")
            .await
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn unusable_csv_files_are_refused() {
    let mut session = Session::new().await;
    let uri = session.year_2026_27().await;

    for csv in ["", "Nachname\nHuber\n", "Vorname;Nachname\n"] {
        let body = value!({ "csv": csv, "dry_run": true });
        let response = session
            .request("POST", &format!("{uri}/import"), Some(body))
            .await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }

    let body = value!({ "csv": "Vorname\nAnna\n", "dry_run": true });
    let response = session
        .request("POST", "/api/school-years/999/students/import", Some(body))
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}
