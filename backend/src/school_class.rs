//! Class names: a small list maintained by the teacher (concept, Z8a).

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
    entity::{enrollment, school_class},
    error::ApiError,
    validate::{clean_name, same_name},
};

/// Routes of this module. All of them need a session.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/classes", get(list).post(create))
        .route("/classes/{id}", put(update).delete(remove))
}

#[derive(Serialize)]
struct SchoolClass {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct SchoolClassInput {
    name: String,
}

impl From<school_class::Model> for SchoolClass {
    fn from(class: school_class::Model) -> Self {
        Self {
            id: class.id,
            name: class.name,
        }
    }
}

/// Sort key that puts "7a" before "10a": the leading number first, then
/// the rest without regard to case. Names without a number come last.
pub fn sort_key(name: &str) -> (u32, String) {
    let name = name.trim();
    let digits = name.chars().take_while(char::is_ascii_digit).count();
    let number = name[..digits].parse().unwrap_or(u32::MAX);
    (number, name[digits..].trim().to_lowercase())
}

/// All classes in display order.
pub async fn load_all(db: &impl ConnectionTrait) -> Result<Vec<school_class::Model>, ApiError> {
    let mut classes = school_class::Entity::find().all(db).await?;
    classes.sort_by_key(|class| sort_key(&class.name));
    Ok(classes)
}

/// Rejects a name that another class already uses.
async fn ensure_unique(
    db: &impl ConnectionTrait,
    name: &str,
    except_id: Option<i32>,
) -> Result<(), ApiError> {
    let taken = load_all(db)
        .await?
        .iter()
        .any(|other| Some(other.id) != except_id && same_name(&other.name, name));
    if taken {
        return Err(ApiError::conflict(
            "Eine Klasse mit diesem Namen gibt es schon.",
        ));
    }
    Ok(())
}

async fn find(db: &DatabaseConnection, id: i32) -> Result<school_class::Model, ApiError> {
    school_class::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<SchoolClass>>, ApiError> {
    let classes = load_all(&state.db).await?;
    Ok(Json(classes.into_iter().map(Into::into).collect()))
}

/// Creates a class after checking the rules of the data model.
pub async fn insert(
    db: &impl ConnectionTrait,
    name: &str,
) -> Result<school_class::Model, ApiError> {
    let name = clean_name(name)?;
    ensure_unique(db, &name, None).await?;
    let now = now();

    Ok(school_class::ActiveModel {
        name: Set(name),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<SchoolClassInput>,
) -> Result<(StatusCode, Json<SchoolClass>), ApiError> {
    let class = insert(&state.db, &input.name).await?;
    Ok((StatusCode::CREATED, Json(class.into())))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<SchoolClassInput>,
) -> Result<Json<SchoolClass>, ApiError> {
    let name = clean_name(&input.name)?;
    let class = find(&state.db, id).await?;
    ensure_unique(&state.db, &name, Some(id)).await?;

    let mut class = class.into_active_model();
    class.name = Set(name);
    class.updated_at = Set(now());
    let class = class.update(&state.db).await?;
    Ok(Json(class.into()))
}

/// Deletes a class unless an enrollment still names it.
async fn remove(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    find(&state.db, id).await?;
    let in_use = enrollment::Entity::find()
        .filter(enrollment::Column::SchoolClassId.eq(id))
        .count(&state.db)
        .await?;
    if in_use > 0 {
        return Err(ApiError::conflict(
            "Der Klasse sind noch Schüler zugeordnet. Ordne sie zuerst einer anderen Klasse zu.",
        ));
    }
    school_class::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_sort_by_number_then_letter() {
        let mut names = vec!["10a", "7b", "Ü2", "7A", "9", "10 b", "DFK"];
        names.sort_by_key(|name| sort_key(name));
        assert_eq!(names, ["7A", "7b", "9", "10a", "10 b", "DFK", "Ü2"]);
    }
}
