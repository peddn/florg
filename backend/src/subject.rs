//! Subjects: a small list maintained by the teacher.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    db::now,
    entity::{learning_group, subject},
    error::ApiError,
    validate::{clean_name, same_name},
};

/// Longest short name, so it fits into a timetable cell.
const MAX_SHORT_NAME_LENGTH: usize = 10;

/// Routes of this module. All of them need a session.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects", get(list).post(create))
        .route("/subjects/{id}", put(update).delete(remove))
}

#[derive(Serialize)]
struct Subject {
    id: i32,
    name: String,
    short_name: String,
}

#[derive(Deserialize)]
struct SubjectInput {
    name: String,
    short_name: String,
}

impl From<subject::Model> for Subject {
    fn from(subject: subject::Model) -> Self {
        Self {
            id: subject.id,
            name: subject.name,
            short_name: subject.short_name,
        }
    }
}

impl SubjectInput {
    fn validated(self) -> Result<Self, ApiError> {
        let name = clean_name(&self.name)?;
        let short_name = self.short_name.trim().to_owned();
        if short_name.is_empty() {
            return Err(ApiError::invalid("Das Kürzel fehlt."));
        }
        if short_name.chars().count() > MAX_SHORT_NAME_LENGTH {
            return Err(ApiError::invalid(
                "Das Kürzel darf höchstens 10 Zeichen lang sein.",
            ));
        }
        Ok(Self { name, short_name })
    }
}

/// All subjects, sorted by name without regard to case.
async fn load_all(db: &impl ConnectionTrait) -> Result<Vec<subject::Model>, ApiError> {
    let mut subjects = subject::Entity::find().all(db).await?;
    subjects.sort_by_key(|subject| subject.name.to_lowercase());
    Ok(subjects)
}

/// Rejects a name or short name that another subject already uses.
async fn ensure_unique(
    db: &impl ConnectionTrait,
    input: &SubjectInput,
    except_id: Option<i32>,
) -> Result<(), ApiError> {
    for other in load_all(db).await? {
        if Some(other.id) == except_id {
            continue;
        }
        if same_name(&other.name, &input.name) {
            return Err(ApiError::conflict(
                "Ein Fach mit diesem Namen gibt es schon.",
            ));
        }
        if same_name(&other.short_name, &input.short_name) {
            return Err(ApiError::conflict(
                "Ein Fach mit diesem Kürzel gibt es schon.",
            ));
        }
    }
    Ok(())
}

async fn find(db: &DatabaseConnection, id: i32) -> Result<subject::Model, ApiError> {
    subject::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Subject>>, ApiError> {
    let subjects = load_all(&state.db).await?;
    Ok(Json(subjects.into_iter().map(Into::into).collect()))
}

/// Creates a subject after checking the rules of the data model.
pub async fn insert(
    db: &impl ConnectionTrait,
    name: &str,
    short_name: &str,
) -> Result<subject::Model, ApiError> {
    let input = SubjectInput {
        name: name.to_owned(),
        short_name: short_name.to_owned(),
    }
    .validated()?;
    ensure_unique(db, &input, None).await?;
    let now = now();

    Ok(subject::ActiveModel {
        name: Set(input.name),
        short_name: Set(input.short_name),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<SubjectInput>,
) -> Result<(StatusCode, Json<Subject>), ApiError> {
    let subject = insert(&state.db, &input.name, &input.short_name).await?;
    Ok((StatusCode::CREATED, Json(subject.into())))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<SubjectInput>,
) -> Result<Json<Subject>, ApiError> {
    let input = input.validated()?;
    let subject = find(&state.db, id).await?;
    ensure_unique(&state.db, &input, Some(id)).await?;

    let mut subject = subject.into_active_model();
    subject.name = Set(input.name);
    subject.short_name = Set(input.short_name);
    subject.updated_at = Set(now());
    let subject = subject.update(&state.db).await?;
    Ok(Json(subject.into()))
}

/// Deletes a subject unless a learning group still uses it.
async fn remove(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    find(&state.db, id).await?;
    let in_use = learning_group::Entity::find()
        .filter(learning_group::Column::SubjectId.eq(id))
        .count(&state.db)
        .await?;
    if in_use > 0 {
        return Err(ApiError::conflict(
            "Das Fach wird noch von einer Lerngruppe genutzt.",
        ));
    }
    subject::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
