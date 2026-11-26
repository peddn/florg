//! Learning groups of a school year and their members (concept Z8, Z8a;
//! data model `learning_group`, `group_membership`).

use std::collections::{HashMap, HashSet};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
};
use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    db::now,
    entity::{group_membership, learning_group, school_year, student::FormOfAddress, subject},
    error::ApiError,
    school_class::sort_key,
    student::{StudentRow, load_rows as load_students},
    validate::{clean_name, same_name},
};

/// Routes of this module. All of them need a session.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/school-years/{year_id}/learning-groups",
            get(list).post(create),
        )
        .route(
            "/learning-groups/{id}",
            get(detail).put(update).delete(remove),
        )
        .route("/learning-groups/{id}/members", post(add_members))
        .route(
            "/learning-groups/{id}/members/{student_id}",
            put(update_member).delete(remove_member),
        )
}

#[derive(Serialize)]
struct GroupSummary {
    id: i32,
    name: String,
    subject_id: i32,
    subject_name: String,
    subject_short_name: String,
    member_count: usize,
}

#[derive(Serialize)]
struct GroupDetail {
    id: i32,
    school_year_id: i32,
    name: String,
    subject_id: i32,
    subject_name: String,
    subject_short_name: String,
    members: Vec<Member>,
}

/// A student in a learning group, in the order of the student list.
#[derive(Serialize)]
struct Member {
    /// Id of the student.
    id: i32,
    first_name: String,
    last_name: Option<String>,
    form_of_address: FormOfAddress,
    school_class_name: Option<String>,
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
}

#[derive(Deserialize)]
struct GroupInput {
    name: String,
    subject_id: i32,
}

#[derive(Deserialize)]
struct MembersInput {
    student_ids: Vec<i32>,
}

#[derive(Deserialize)]
struct MemberInput {
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
}

async fn find_year(db: &impl ConnectionTrait, id: i32) -> Result<school_year::Model, ApiError> {
    school_year::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

async fn find_group(db: &impl ConnectionTrait, id: i32) -> Result<learning_group::Model, ApiError> {
    learning_group::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

async fn find_subject(db: &impl ConnectionTrait, id: i32) -> Result<subject::Model, ApiError> {
    subject::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::invalid("Dieses Fach gibt es nicht."))
}

async fn groups_of_year(
    db: &impl ConnectionTrait,
    year_id: i32,
) -> Result<Vec<learning_group::Model>, ApiError> {
    Ok(learning_group::Entity::find()
        .filter(learning_group::Column::SchoolYearId.eq(year_id))
        .all(db)
        .await?)
}

async fn memberships_of(
    db: &impl ConnectionTrait,
    group_ids: Vec<i32>,
) -> Result<Vec<group_membership::Model>, ApiError> {
    Ok(group_membership::Entity::find()
        .filter(group_membership::Column::LearningGroupId.is_in(group_ids))
        .all(db)
        .await?)
}

/// Rejects a name that another learning group of the school year uses.
async fn ensure_name_is_free(
    db: &impl ConnectionTrait,
    year_id: i32,
    name: &str,
    except_id: Option<i32>,
) -> Result<(), ApiError> {
    let taken = groups_of_year(db, year_id)
        .await?
        .iter()
        .any(|other| Some(other.id) != except_id && same_name(&other.name, name));
    if taken {
        return Err(ApiError::conflict(
            "Eine Lerngruppe mit diesem Namen gibt es in diesem Schuljahr schon.",
        ));
    }
    Ok(())
}

/// Creates a learning group after checking the rules of the data model.
pub async fn insert(
    db: &impl ConnectionTrait,
    year_id: i32,
    name: &str,
    subject_id: i32,
) -> Result<learning_group::Model, ApiError> {
    let name = clean_name(name)?;
    find_year(db, year_id).await?;
    find_subject(db, subject_id).await?;
    ensure_name_is_free(db, year_id, &name, None).await?;
    let now = now();

    Ok(learning_group::ActiveModel {
        school_year_id: Set(year_id),
        subject_id: Set(subject_id),
        name: Set(name),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

/// Makes students members of a learning group. Students who already are
/// members are skipped. Everyone must be enrolled in the school year of
/// the group.
pub async fn insert_members(
    db: &impl ConnectionTrait,
    group: &learning_group::Model,
    student_ids: &[i32],
) -> Result<(), ApiError> {
    let enrolled: HashSet<i32> = load_students(db, group.school_year_id)
        .await?
        .into_iter()
        .map(|student| student.id)
        .collect();
    if student_ids.iter().any(|id| !enrolled.contains(id)) {
        return Err(ApiError::invalid(
            "Mindestens ein Schüler gehört nicht zum Schuljahr der Lerngruppe.",
        ));
    }

    let mut members: HashSet<i32> = memberships_of(db, vec![group.id])
        .await?
        .into_iter()
        .map(|membership| membership.student_id)
        .collect();
    let now = now();
    for student_id in student_ids {
        // `insert` is false for members and for repeated ids.
        if !members.insert(*student_id) {
            continue;
        }
        group_membership::ActiveModel {
            learning_group_id: Set(group.id),
            student_id: Set(*student_id),
            joined_on: Set(None),
            left_on: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

/// Checks and stores the period of a membership.
pub async fn set_member_period(
    db: &impl ConnectionTrait,
    group: &learning_group::Model,
    student_id: i32,
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
) -> Result<(), ApiError> {
    let year = find_year(db, group.school_year_id).await?;
    let outside = |date: Option<NaiveDate>| {
        date.is_some_and(|date| date < year.start_date || date > year.end_date)
    };
    if outside(joined_on) || outside(left_on) {
        return Err(ApiError::invalid(
            "Eintritt und Austritt müssen im Schuljahr liegen.",
        ));
    }
    if let (Some(joined_on), Some(left_on)) = (joined_on, left_on)
        && joined_on > left_on
    {
        return Err(ApiError::invalid("Der Eintritt liegt nach dem Austritt."));
    }

    let membership = find_membership(db, group.id, student_id).await?;
    let mut membership = membership.into_active_model();
    membership.joined_on = Set(joined_on);
    membership.left_on = Set(left_on);
    membership.updated_at = Set(now());
    membership.update(db).await?;
    Ok(())
}

async fn find_membership(
    db: &impl ConnectionTrait,
    group_id: i32,
    student_id: i32,
) -> Result<group_membership::Model, ApiError> {
    group_membership::Entity::find()
        .filter(group_membership::Column::LearningGroupId.eq(group_id))
        .filter(group_membership::Column::StudentId.eq(student_id))
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

/// Ends all memberships of a student in the learning groups of a school
/// year. Called when the student is removed from that school year.
pub async fn remove_student_from_year(
    db: &impl ConnectionTrait,
    year_id: i32,
    student_id: i32,
) -> Result<(), ApiError> {
    let group_ids: Vec<i32> = groups_of_year(db, year_id)
        .await?
        .into_iter()
        .map(|group| group.id)
        .collect();
    group_membership::Entity::delete_many()
        .filter(group_membership::Column::LearningGroupId.is_in(group_ids))
        .filter(group_membership::Column::StudentId.eq(student_id))
        .exec(db)
        .await?;
    Ok(())
}

async fn load_detail(db: &impl ConnectionTrait, id: i32) -> Result<GroupDetail, ApiError> {
    let group = find_group(db, id).await?;
    let subject = subject::Entity::find_by_id(group.subject_id)
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)?;
    let memberships: HashMap<i32, group_membership::Model> = memberships_of(db, vec![id])
        .await?
        .into_iter()
        .map(|membership| (membership.student_id, membership))
        .collect();

    // The student list is already in display order.
    let members = load_students(db, group.school_year_id)
        .await?
        .into_iter()
        .filter_map(|student: StudentRow| {
            let membership = memberships.get(&student.id)?;
            Some(Member {
                id: student.id,
                first_name: student.first_name,
                last_name: student.last_name,
                form_of_address: student.form_of_address,
                school_class_name: student.school_class_name,
                joined_on: membership.joined_on,
                left_on: membership.left_on,
            })
        })
        .collect();

    Ok(GroupDetail {
        id: group.id,
        school_year_id: group.school_year_id,
        name: group.name,
        subject_id: subject.id,
        subject_name: subject.name,
        subject_short_name: subject.short_name,
        members,
    })
}

/// Lists the learning groups of a school year, sorted like classes.
async fn list(
    State(state): State<AppState>,
    Path(year_id): Path<i32>,
) -> Result<Json<Vec<GroupSummary>>, ApiError> {
    find_year(&state.db, year_id).await?;
    let mut groups = groups_of_year(&state.db, year_id).await?;
    groups.sort_by_cached_key(|group| sort_key(&group.name));

    let subjects: HashMap<i32, subject::Model> = subject::Entity::find()
        .all(&state.db)
        .await?
        .into_iter()
        .map(|subject| (subject.id, subject))
        .collect();
    let mut member_counts: HashMap<i32, usize> = HashMap::new();
    let group_ids = groups.iter().map(|group| group.id).collect();
    for membership in memberships_of(&state.db, group_ids).await? {
        *member_counts
            .entry(membership.learning_group_id)
            .or_default() += 1;
    }

    let summaries = groups
        .into_iter()
        .filter_map(|group| {
            let subject = subjects.get(&group.subject_id)?;
            Some(GroupSummary {
                id: group.id,
                name: group.name,
                subject_id: subject.id,
                subject_name: subject.name.clone(),
                subject_short_name: subject.short_name.clone(),
                member_count: member_counts.get(&group.id).copied().unwrap_or_default(),
            })
        })
        .collect();
    Ok(Json(summaries))
}

async fn create(
    State(state): State<AppState>,
    Path(year_id): Path<i32>,
    Json(input): Json<GroupInput>,
) -> Result<(StatusCode, Json<GroupDetail>), ApiError> {
    let transaction = state.db.begin().await?;
    let group = insert(&transaction, year_id, &input.name, input.subject_id).await?;
    let detail = load_detail(&transaction, group.id).await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<GroupDetail>, ApiError> {
    Ok(Json(load_detail(&state.db, id).await?))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<GroupInput>,
) -> Result<Json<GroupDetail>, ApiError> {
    let name = clean_name(&input.name)?;
    let transaction = state.db.begin().await?;
    let group = find_group(&transaction, id).await?;
    find_subject(&transaction, input.subject_id).await?;
    ensure_name_is_free(&transaction, group.school_year_id, &name, Some(id)).await?;

    let mut group = group.into_active_model();
    group.name = Set(name);
    group.subject_id = Set(input.subject_id);
    group.updated_at = Set(now());
    group.update(&transaction).await?;

    let detail = load_detail(&transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

/// Deletes a learning group together with its memberships.
async fn remove(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    let transaction = state.db.begin().await?;
    find_group(&transaction, id).await?;
    group_membership::Entity::delete_many()
        .filter(group_membership::Column::LearningGroupId.eq(id))
        .exec(&transaction)
        .await?;
    learning_group::Entity::delete_by_id(id)
        .exec(&transaction)
        .await?;
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_members(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<MembersInput>,
) -> Result<Json<GroupDetail>, ApiError> {
    let transaction = state.db.begin().await?;
    let group = find_group(&transaction, id).await?;
    insert_members(&transaction, &group, &input.student_ids).await?;
    let detail = load_detail(&transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

/// Changes since when and until when a student belongs to the group.
async fn update_member(
    State(state): State<AppState>,
    Path((id, student_id)): Path<(i32, i32)>,
    Json(input): Json<MemberInput>,
) -> Result<Json<GroupDetail>, ApiError> {
    let transaction = state.db.begin().await?;
    let group = find_group(&transaction, id).await?;
    set_member_period(
        &transaction,
        &group,
        student_id,
        input.joined_on,
        input.left_on,
    )
    .await?;
    let detail = load_detail(&transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

async fn remove_member(
    State(state): State<AppState>,
    Path((id, student_id)): Path<(i32, i32)>,
) -> Result<StatusCode, ApiError> {
    let membership = find_membership(&state.db, id, student_id).await?;
    group_membership::Entity::delete_by_id(membership.id)
        .exec(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
