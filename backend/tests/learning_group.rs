//! Learning groups and their members against the HTTP API.

mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use common::{Browser, TestApp, json};
use serde_json::{Value, json as value};

/// A signed-in session with one school year, two subjects and five
/// students: three in "7a", one in "7b", one without a class.
struct Fixture {
    app: TestApp,
    browser: Browser,
    year: i64,
    technik: i64,
    werken: i64,
    /// Student ids by first name.
    anna: i64,
    ben: i64,
    cem: i64,
    dana: i64,
    eva: i64,
}

impl Fixture {
    async fn new() -> Self {
        let app = TestApp::with_account().await;
        let browser = app.sign_in().await;
        let mut fixture = Self {
            app,
            browser,
            year: 0,
            technik: 0,
            werken: 0,
            anna: 0,
            ben: 0,
            cem: 0,
            dana: 0,
            eva: 0,
        };
        fixture.year = fixture
            .school_year("2026/27", "2026-09-15", "2027-07-30")
            .await;
        fixture.technik = fixture.subject("Technik", "Te").await;
        fixture.werken = fixture.subject("Werken", "We").await;
        let class_7a = fixture.id("/api/classes", value!({ "name": "7a" })).await;
        let class_7b = fixture.id("/api/classes", value!({ "name": "7b" })).await;
        let year = fixture.year;
        fixture.anna = fixture.student(year, "Anna", "Huber", Some(class_7a)).await;
        fixture.ben = fixture.student(year, "Ben", "Maier", Some(class_7a)).await;
        fixture.cem = fixture.student(year, "Cem", "Aksoy", Some(class_7a)).await;
        fixture.dana = fixture.student(year, "Dana", "Lang", Some(class_7b)).await;
        fixture.eva = fixture.student(year, "Eva", "Zorn", None).await;
        fixture
    }

    async fn request(&mut self, method: &str, uri: &str, body: Option<Value>) -> Response<Body> {
        self.app.send(&mut self.browser, method, uri, body).await
    }

    async fn get(&mut self, uri: &str) -> Value {
        let response = self.request("GET", uri, None).await;
        assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
        json(response).await
    }

    /// Creates something and returns its id.
    async fn id(&mut self, uri: &str, body: Value) -> i64 {
        let response = self.request("POST", uri, Some(body)).await;
        assert_eq!(response.status(), StatusCode::CREATED, "POST {uri}");
        json(response).await["id"].as_i64().unwrap()
    }

    async fn school_year(&mut self, name: &str, start: &str, end: &str) -> i64 {
        let body = value!({ "name": name, "start_date": start, "end_date": end });
        self.id("/api/school-years", body).await
    }

    async fn subject(&mut self, name: &str, short_name: &str) -> i64 {
        let body = value!({ "name": name, "short_name": short_name });
        self.id("/api/subjects", body).await
    }

    async fn student(&mut self, year: i64, first: &str, last: &str, class: Option<i64>) -> i64 {
        let body = value!({
            "first_name": first, "last_name": last,
            "form_of_address": "name_only", "school_class_id": class,
        });
        self.id(&format!("/api/school-years/{year}/students"), body)
            .await
    }

    fn groups_uri(&self) -> String {
        format!("/api/school-years/{}/learning-groups", self.year)
    }

    async fn group(&mut self, name: &str, subject: i64) -> String {
        let uri = self.groups_uri();
        let id = self
            .id(&uri, value!({ "name": name, "subject_id": subject }))
            .await;
        format!("/api/learning-groups/{id}")
    }

    async fn add(&mut self, group_uri: &str, students: &[i64]) -> Response<Body> {
        let body = value!({ "student_ids": students });
        self.request("POST", &format!("{group_uri}/members"), Some(body))
            .await
    }

    /// Address of the period of a student, the only one or the `index`-th.
    async fn period_uri(&mut self, group_uri: &str, student: i64, index: usize) -> String {
        let detail = self.get(group_uri).await;
        let id = detail["members"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|member| member["student_id"] == student)
            .nth(index)
            .unwrap_or_else(|| panic!("period {index} of student {student} exists"))["id"]
            .clone();
        format!("/api/group-memberships/{id}")
    }

    /// First names of the members, in list order.
    async fn members(&mut self, group_uri: &str) -> Vec<String> {
        self.get(group_uri).await["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|member| member["first_name"].as_str().unwrap().to_owned())
            .collect()
    }

    /// "Name (Fach, Mitglieder)" of every group, in list order.
    async fn groups(&mut self) -> Vec<String> {
        let uri = self.groups_uri();
        self.get(&uri)
            .await
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
            .collect()
    }
}

async fn assert_error(response: Response<Body>, status: StatusCode, code: &str) {
    assert_eq!(response.status(), status);
    let body = json(response).await;
    assert_eq!(body["error"], code);
    assert!(!body["message"].as_str().unwrap().is_empty());
}

fn group_input(name: &str, subject: i64) -> Option<Value> {
    Some(value!({ "name": name, "subject_id": subject }))
}

fn period(joined_on: Option<&str>, left_on: Option<&str>) -> Option<Value> {
    Some(value!({ "joined_on": joined_on, "left_on": left_on }))
}

#[tokio::test]
async fn learning_group_endpoints_need_a_session() {
    let app = TestApp::with_account().await;
    let mut browser = Browser::default();

    for (method, uri) in [
        ("GET", "/api/school-years/1/learning-groups"),
        ("POST", "/api/school-years/1/learning-groups"),
        ("GET", "/api/learning-groups/1"),
        ("PUT", "/api/learning-groups/1"),
        ("DELETE", "/api/learning-groups/1"),
        ("POST", "/api/learning-groups/1/members"),
        ("POST", "/api/learning-groups/1/memberships"),
        ("PUT", "/api/group-memberships/1"),
        ("DELETE", "/api/group-memberships/1"),
    ] {
        let body = Some(value!({ "name": "x", "subject_id": 1, "student_ids": [1] }));
        let response = app.send(&mut browser, method, uri, body).await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri}"
        );
    }
}

#[tokio::test]
async fn learning_groups_can_be_created_changed_and_deleted() {
    let mut f = Fixture::new().await;
    assert!(f.groups().await.is_empty());

    let (technik, werken) = (f.technik, f.werken);
    f.group("10a Technik", technik).await;
    let group = f.group("  7a/b Technik Gruppe 1 ", technik).await;
    // Sorted like classes: "7…" before "10…".
    assert_eq!(
        f.groups().await,
        ["7a/b Technik Gruppe 1 (Te, 0)", "10a Technik (Te, 0)"]
    );
    let detail = f.get(&group).await;
    assert_eq!(detail["name"], "7a/b Technik Gruppe 1");
    assert_eq!(detail["subject_name"], "Technik");
    assert_eq!(detail["school_year_id"], f.year);
    assert_eq!(detail["members"], value!([]));

    let response = f
        .request("PUT", &group, group_input("7a/b Werken", werken))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await["subject_short_name"], "We");

    let response = f.request("DELETE", &group, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(f.groups().await, ["10a Technik (Te, 0)"]);
    for method in ["GET", "DELETE"] {
        let response = f.request(method, &group, None).await;
        assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    }
    let response = f.request("PUT", &group, group_input("x", technik)).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn invalid_learning_groups_are_refused() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let uri = f.groups_uri();
    let first = f.group("7a Technik", technik).await;
    let second = f.group("7b Technik", technik).await;

    for body in [group_input("  ", technik), group_input("7c Technik", 999)] {
        let response = f.request("POST", &uri, body.clone()).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
        let response = f.request("PUT", &first, body).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }

    // The name is unique within the school year, regardless of case.
    let response = f
        .request("POST", &uri, group_input("7A TECHNIK", technik))
        .await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;
    let response = f
        .request("PUT", &second, group_input("7a technik", technik))
        .await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;
    let response = f
        .request("PUT", &first, group_input("7A Technik", technik))
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    // Another school year may use the same name.
    let other_year = f.school_year("2027/28", "2027-09-14", "2028-07-28").await;
    let other_uri = format!("/api/school-years/{other_year}/learning-groups");
    let response = f
        .request("POST", &other_uri, group_input("7a Technik", technik))
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = f
        .request(
            "POST",
            "/api/school-years/999/learning-groups",
            group_input("x", technik),
        )
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    assert_eq!(f.groups().await.len(), 2);
}

#[tokio::test]
async fn members_can_be_added_and_removed() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let group = f.group("Technik Gruppe 1", technik).await;
    let (anna, ben, cem, dana, eva) = (f.anna, f.ben, f.cem, f.dana, f.eva);

    let response = f.add(&group, &[dana, anna]).await;
    assert_eq!(response.status(), StatusCode::OK);
    // In the order of the student list: by class, then last name.
    assert_eq!(f.members(&group).await, ["Anna", "Dana"]);
    assert_eq!(f.get(&group).await["members"][1]["school_class_name"], "7b");

    // Members and repeated ids are skipped. A student without a class may
    // join as well (Z8a).
    let response = f.add(&group, &[anna, eva, cem, cem]).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(f.members(&group).await, ["Cem", "Anna", "Dana", "Eva"]);
    assert_eq!(f.groups().await, ["Technik Gruppe 1 (Te, 4)"]);

    let member = f.period_uri(&group, anna, 0).await;
    let response = f.request("DELETE", &member, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(f.members(&group).await, ["Cem", "Dana", "Eva"]);
    let response = f.request("DELETE", &member, None).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    let _ = ben;
}

#[tokio::test]
async fn only_students_of_the_school_year_can_join() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let group = f.group("Technik", technik).await;
    let other_year = f.school_year("2027/28", "2027-09-14", "2028-07-28").await;
    let stranger = f.student(other_year, "Finn", "Graf", None).await;
    let anna = f.anna;

    for students in [vec![stranger], vec![anna, 999]] {
        let response = f.add(&group, &students).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }
    // Nothing was added, not even the valid student of the second attempt.
    assert!(f.members(&group).await.is_empty());

    let response = f.add("/api/learning-groups/999", &[anna]).await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn membership_can_be_limited_to_a_period() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let group = f.group("Technik", technik).await;
    let (anna, ben) = (f.anna, f.ben);
    f.add(&group, &[anna]).await;
    let member = f.period_uri(&group, anna, 0).await;

    let detail = f.get(&group).await;
    assert_eq!(detail["members"][0]["joined_on"], Value::Null);
    assert_eq!(detail["members"][0]["left_on"], Value::Null);

    let response = f
        .request(
            "PUT",
            &member,
            period(Some("2026-11-02"), Some("2027-02-12")),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let detail = json(response).await;
    assert_eq!(detail["members"][0]["joined_on"], "2026-11-02");
    assert_eq!(detail["members"][0]["left_on"], "2027-02-12");

    let invalid = [
        period(Some("2026-09-14"), None),
        period(None, Some("2027-07-31")),
        period(Some("2027-03-01"), Some("2027-02-28")),
    ];
    for body in invalid {
        let response = f.request("PUT", &member, body).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }
    assert_eq!(f.get(&group).await["members"][0]["left_on"], "2027-02-12");

    // A single day and the limits of the school year are allowed.
    for body in [
        period(Some("2026-09-15"), Some("2026-09-15")),
        period(Some("2026-09-15"), Some("2027-07-30")),
        period(None, None),
    ] {
        let response = f.request("PUT", &member, body).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
    assert_eq!(f.get(&group).await["members"][0]["joined_on"], Value::Null);

    let response = f
        .request("PUT", "/api/group-memberships/999", period(None, None))
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
    let _ = ben;
}

#[tokio::test]
async fn student_can_leave_and_return() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let group = f.group("Technik", technik).await;
    let (anna, ben) = (f.anna, f.ben);
    f.add(&group, &[anna, ben]).await;
    let memberships = format!("{group}/memberships");

    // Anna leaves before the winter holidays and comes back after Easter.
    let first = f.period_uri(&group, anna, 0).await;
    let response = f
        .request("PUT", &first, period(None, Some("2026-12-23")))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = Some(value!({ "student_id": anna, "joined_on": "2027-04-12", "left_on": null }));
    let response = f.request("POST", &memberships, body).await;
    assert_eq!(response.status(), StatusCode::OK);
    let detail = json(response).await;

    // Two rows for Anna, by date, and she still counts once.
    let rows: Vec<_> = detail["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|member| {
            format!(
                "{} {} {}",
                member["first_name"].as_str().unwrap(),
                member["joined_on"].as_str().unwrap_or("-"),
                member["left_on"].as_str().unwrap_or("-")
            )
        })
        .collect();
    assert_eq!(rows, ["Anna - 2026-12-23", "Anna 2027-04-12 -", "Ben - -"]);
    assert_eq!(f.groups().await, ["Technik (Te, 2)"]);

    // Adding Anna the usual way changes nothing: she has periods already.
    f.add(&group, &[anna]).await;
    assert_eq!(f.members(&group).await, ["Anna", "Anna", "Ben"]);

    // Removing one period keeps the other.
    let second = f.period_uri(&group, anna, 1).await;
    let response = f.request("DELETE", &second, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(f.members(&group).await, ["Anna", "Ben"]);
}

#[tokio::test]
async fn periods_of_a_student_must_not_overlap() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let group = f.group("Technik", technik).await;
    let (anna, ben) = (f.anna, f.ben);
    f.add(&group, &[anna, ben]).await;
    let memberships = format!("{group}/memberships");
    let first = f.period_uri(&group, anna, 0).await;
    f.request("PUT", &first, period(None, Some("2026-12-23")))
        .await;

    let add = |joined_on: &str, left_on: Option<&str>| {
        Some(value!({ "student_id": anna, "joined_on": joined_on, "left_on": left_on }))
    };
    // Overlapping the first period, including its last day.
    for body in [
        add("2026-12-23", None),
        add("2026-10-01", Some("2026-10-31")),
    ] {
        let response = f.request("POST", &memberships, body).await;
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    }
    // Ben is in the group for the whole year: no room for another period.
    let body = Some(value!({ "student_id": ben, "joined_on": "2027-05-03", "left_on": null }));
    let response = f.request("POST", &memberships, body).await;
    assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;

    // The day after is fine.
    let response = f
        .request("POST", &memberships, add("2026-12-24", None))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    // Extending the first period into the second is refused.
    let response = f
        .request("PUT", &first, period(None, Some("2027-01-15")))
        .await;
    assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    // Changing a period may keep its own dates.
    let response = f
        .request("PUT", &first, period(None, Some("2026-12-20")))
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    // Only students of the school year, and only existing groups.
    let other_year = f.school_year("2027/28", "2027-09-14", "2028-07-28").await;
    let stranger = f.student(other_year, "Finn", "Graf", None).await;
    let body = Some(value!({ "student_id": stranger, "joined_on": null, "left_on": null }));
    let response = f.request("POST", &memberships, body.clone()).await;
    assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, "invalid").await;
    let response = f
        .request("POST", "/api/learning-groups/999/memberships", body)
        .await;
    assert_error(response, StatusCode::NOT_FOUND, "not_found").await;
}

#[tokio::test]
async fn deleting_a_group_keeps_its_students() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let group = f.group("Technik", technik).await;
    let other = f.group("Werken", technik).await;
    let (anna, ben) = (f.anna, f.ben);
    f.add(&group, &[anna, ben]).await;
    f.add(&other, &[anna]).await;

    let response = f.request("DELETE", &group, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let students = f
        .get(&format!("/api/school-years/{}/students", f.year))
        .await;
    assert_eq!(students.as_array().unwrap().len(), 5);
    // Memberships in other groups are untouched.
    assert_eq!(f.members(&other).await, ["Anna"]);
}

#[tokio::test]
async fn removing_a_student_from_the_school_year_ends_the_memberships() {
    let mut f = Fixture::new().await;
    let technik = f.technik;
    let first = f.group("Technik 1", technik).await;
    let second = f.group("Technik 2", technik).await;
    let (anna, ben) = (f.anna, f.ben);
    f.add(&first, &[anna, ben]).await;
    f.add(&second, &[anna]).await;

    let student = format!("/api/school-years/{}/students/{anna}", f.year);
    let response = f.request("DELETE", &student, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    assert_eq!(f.members(&first).await, ["Ben"]);
    assert!(f.members(&second).await.is_empty());
    assert_eq!(f.groups().await, ["Technik 1 (Te, 1)", "Technik 2 (Te, 0)"]);
}

#[tokio::test]
async fn subject_and_school_year_in_use_cannot_be_deleted() {
    let mut f = Fixture::new().await;
    let (technik, werken) = (f.technik, f.werken);
    let empty_year = f.school_year("2027/28", "2027-09-14", "2028-07-28").await;
    let year_uri = format!("/api/school-years/{empty_year}");
    let response = f
        .request(
            "POST",
            &format!("{year_uri}/learning-groups"),
            group_input("Technik", technik),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let group = format!("/api/learning-groups/{}", json(response).await["id"]);

    let subject_uri = format!("/api/subjects/{technik}");
    let response = f.request("DELETE", &subject_uri, None).await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;
    let response = f.request("DELETE", &year_uri, None).await;
    assert_error(response, StatusCode::CONFLICT, "conflict").await;
    // A subject no group uses can go.
    let response = f
        .request("DELETE", &format!("/api/subjects/{werken}"), None)
        .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = f.request("DELETE", &group, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = f.request("DELETE", &year_uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = f.request("DELETE", &subject_uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
