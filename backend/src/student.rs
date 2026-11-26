//! Students of a school year (concept Z8, Z8a; data model `student`,
//! `enrollment`).

use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    db::now,
    entity::{
        enrollment, school_class, school_year,
        student::{self, FormOfAddress},
    },
    error::ApiError,
    learning_group::remove_student_from_year,
    school_class::{load_all as load_classes, sort_key as class_sort_key},
    student_import,
    validate::MAX_NAME_LENGTH,
};

/// Routes of this module. All of them need a session.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/school-years/{year_id}/students", get(list).post(create))
        .route(
            "/school-years/{year_id}/students/import",
            post(student_import::import),
        )
        .route(
            "/school-years/{year_id}/students/{student_id}",
            put(update).delete(remove),
        )
}

/// A student as shown in the list of a school year.
#[derive(Debug, Clone, Serialize)]
pub struct StudentRow {
    /// Id of the student, not of the enrollment.
    pub id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub form_of_address: FormOfAddress,
    pub school_class_id: Option<i32>,
    pub school_class_name: Option<String>,
}

#[derive(Deserialize)]
struct StudentInput {
    first_name: String,
    last_name: Option<String>,
    form_of_address: FormOfAddress,
    school_class_id: Option<i32>,
}

/// Trims a first name and checks that it is neither empty nor overlong.
pub fn clean_first_name(first_name: &str) -> Result<String, ApiError> {
    let first_name = first_name.trim();
    if first_name.is_empty() {
        return Err(ApiError::invalid("Der Vorname fehlt."));
    }
    if first_name.chars().count() > MAX_NAME_LENGTH {
        return Err(ApiError::invalid("Der Vorname ist zu lang."));
    }
    Ok(first_name.to_owned())
}

/// Trims a last name. An empty one counts as missing.
pub fn clean_last_name(last_name: Option<&str>) -> Result<Option<String>, ApiError> {
    let Some(last_name) = last_name.map(str::trim).filter(|name| !name.is_empty()) else {
        return Ok(None);
    };
    if last_name.chars().count() > MAX_NAME_LENGTH {
        return Err(ApiError::invalid("Der Nachname ist zu lang."));
    }
    Ok(Some(last_name.to_owned()))
}

impl StudentInput {
    async fn validated(self, db: &impl ConnectionTrait) -> Result<Self, ApiError> {
        let first_name = clean_first_name(&self.first_name)?;
        let last_name = clean_last_name(self.last_name.as_deref())?;
        if let Some(class_id) = self.school_class_id {
            let exists = school_class::Entity::find_by_id(class_id)
                .one(db)
                .await?
                .is_some();
            if !exists {
                return Err(ApiError::invalid("Diese Klasse gibt es nicht."));
            }
        }
        Ok(Self {
            first_name,
            last_name,
            ..self
        })
    }
}

pub async fn ensure_year_exists(db: &impl ConnectionTrait, year_id: i32) -> Result<(), ApiError> {
    school_year::Entity::find_by_id(year_id)
        .one(db)
        .await?
        .map(|_| ())
        .ok_or(ApiError::NOT_FOUND)
}

/// Sort order of the list: class, last name, first name. Students without
/// a class come last.
fn sort_key(row: &StudentRow) -> (bool, (u32, String), String, String) {
    (
        row.school_class_name.is_none(),
        class_sort_key(row.school_class_name.as_deref().unwrap_or_default()),
        row.last_name.as_deref().unwrap_or_default().to_lowercase(),
        row.first_name.to_lowercase(),
    )
}

/// Loads the students of a school year in display order.
pub async fn load_rows(
    db: &impl ConnectionTrait,
    year_id: i32,
) -> Result<Vec<StudentRow>, ApiError> {
    let enrollments = enrollment::Entity::find()
        .filter(enrollment::Column::SchoolYearId.eq(year_id))
        .all(db)
        .await?;
    let student_ids: Vec<i32> = enrollments.iter().map(|entry| entry.student_id).collect();
    let students: HashMap<i32, student::Model> = student::Entity::find()
        .filter(student::Column::Id.is_in(student_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|student| (student.id, student))
        .collect();
    let classes: HashMap<i32, String> = load_classes(db)
        .await?
        .into_iter()
        .map(|class| (class.id, class.name))
        .collect();

    let mut rows: Vec<StudentRow> = enrollments
        .into_iter()
        .filter_map(|entry| {
            let student = students.get(&entry.student_id)?;
            Some(StudentRow {
                id: student.id,
                first_name: student.first_name.clone(),
                last_name: student.last_name.clone(),
                form_of_address: student.form_of_address,
                school_class_id: entry.school_class_id,
                school_class_name: entry
                    .school_class_id
                    .and_then(|id| classes.get(&id).cloned()),
            })
        })
        .collect();
    rows.sort_by_cached_key(sort_key);
    Ok(rows)
}

async fn load_row(
    db: &impl ConnectionTrait,
    year_id: i32,
    student_id: i32,
) -> Result<StudentRow, ApiError> {
    load_rows(db, year_id)
        .await?
        .into_iter()
        .find(|row| row.id == student_id)
        .ok_or(ApiError::NOT_FOUND)
}

/// Creates a student together with the enrollment in a school year.
pub async fn insert_student(
    db: &impl ConnectionTrait,
    year_id: i32,
    first_name: String,
    last_name: Option<String>,
    form_of_address: FormOfAddress,
    school_class_id: Option<i32>,
) -> Result<i32, ApiError> {
    let now = now();
    let student = student::ActiveModel {
        first_name: Set(first_name),
        last_name: Set(last_name),
        form_of_address: Set(form_of_address),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;
    enrollment::ActiveModel {
        student_id: Set(student.id),
        school_year_id: Set(year_id),
        school_class_id: Set(school_class_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(student.id)
}

async fn find_enrollment(
    db: &impl ConnectionTrait,
    year_id: i32,
    student_id: i32,
) -> Result<enrollment::Model, ApiError> {
    enrollment::Entity::find()
        .filter(enrollment::Column::SchoolYearId.eq(year_id))
        .filter(enrollment::Column::StudentId.eq(student_id))
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

async fn list(
    State(state): State<AppState>,
    Path(year_id): Path<i32>,
) -> Result<Json<Vec<StudentRow>>, ApiError> {
    ensure_year_exists(&state.db, year_id).await?;
    Ok(Json(load_rows(&state.db, year_id).await?))
}

async fn create(
    State(state): State<AppState>,
    Path(year_id): Path<i32>,
    Json(input): Json<StudentInput>,
) -> Result<(StatusCode, Json<StudentRow>), ApiError> {
    let transaction = state.db.begin().await?;
    ensure_year_exists(&transaction, year_id).await?;
    let input = input.validated(&transaction).await?;
    let student_id = insert_student(
        &transaction,
        year_id,
        input.first_name,
        input.last_name,
        input.form_of_address,
        input.school_class_id,
    )
    .await?;
    let row = load_row(&transaction, year_id, student_id).await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// Changes the student and the class in this school year.
async fn update(
    State(state): State<AppState>,
    Path((year_id, student_id)): Path<(i32, i32)>,
    Json(input): Json<StudentInput>,
) -> Result<Json<StudentRow>, ApiError> {
    let transaction = state.db.begin().await?;
    let entry = find_enrollment(&transaction, year_id, student_id).await?;
    let input = input.validated(&transaction).await?;
    let now = now();

    let student = student::Entity::find_by_id(student_id)
        .one(&transaction)
        .await?
        .ok_or(ApiError::NOT_FOUND)?;
    let mut student = student.into_active_model();
    student.first_name = Set(input.first_name);
    student.last_name = Set(input.last_name);
    student.form_of_address = Set(input.form_of_address);
    student.updated_at = Set(now);
    student.update(&transaction).await?;

    let mut entry = entry.into_active_model();
    entry.school_class_id = Set(input.school_class_id);
    entry.updated_at = Set(now);
    entry.update(&transaction).await?;

    let row = load_row(&transaction, year_id, student_id).await?;
    transaction.commit().await?;
    Ok(Json(row))
}

/// Removes a student from a school year, including the memberships in its
/// learning groups. A student without any school year left is deleted as
/// well (data model, `enrollment`).
async fn remove(
    State(state): State<AppState>,
    Path((year_id, student_id)): Path<(i32, i32)>,
) -> Result<StatusCode, ApiError> {
    let transaction = state.db.begin().await?;
    let entry = find_enrollment(&transaction, year_id, student_id).await?;
    remove_student_from_year(&transaction, year_id, student_id).await?;
    enrollment::Entity::delete_by_id(entry.id)
        .exec(&transaction)
        .await?;

    let remaining = enrollment::Entity::find()
        .filter(enrollment::Column::StudentId.eq(student_id))
        .count(&transaction)
        .await?;
    if remaining == 0 {
        student::Entity::delete_by_id(student_id)
            .exec(&transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
