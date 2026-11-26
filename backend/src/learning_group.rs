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
        .route("/learning-groups/{id}/memberships", post(add_membership))
        .route(
            "/group-memberships/{id}",
            put(update_membership).delete(remove_membership),
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

/// One period of a student in a learning group. A student who left and came
/// back has several, one row each. Rows follow the order of the student
/// list, periods of the same student by date.
#[derive(Serialize)]
struct Member {
    /// Id of the membership.
    id: i32,
    student_id: i32,
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
struct PeriodInput {
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
}

#[derive(Deserialize)]
struct MembershipInput {
    student_id: i32,
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

/// Makes students members of a learning group for the whole school year.
/// Students who have any period in the group already are skipped; a student
/// who returns gets a new period through `add_period`. Everyone must be enrolled in the school year of
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

/// First and last day a period covers. Missing dates stand for the limits
/// of the school year (data model, `group_membership`).
fn effective(
    year: &school_year::Model,
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
) -> (NaiveDate, NaiveDate) {
    (
        joined_on.unwrap_or(year.start_date),
        left_on.unwrap_or(year.end_date),
    )
}

/// Checks a period of a student in a group: within the school year, in
/// order, and without overlapping another period of the same student in
/// the same group. `except_id` is the membership being changed.
async fn check_period(
    db: &impl ConnectionTrait,
    group: &learning_group::Model,
    student_id: i32,
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
    except_id: Option<i32>,
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
    let (first, last) = effective(&year, joined_on, left_on);
    if first > last {
        return Err(ApiError::invalid("Der Eintritt liegt nach dem Austritt."));
    }

    let overlaps = memberships_of(db, vec![group.id])
        .await?
        .iter()
        .filter(|other| other.student_id == student_id && Some(other.id) != except_id)
        .any(|other| {
            let (other_first, other_last) = effective(&year, other.joined_on, other.left_on);
            first <= other_last && other_first <= last
        });
    if overlaps {
        return Err(ApiError::invalid(
            "Der Zeitraum überschneidet sich mit einem anderen Zeitraum dieses Schülers in der Lerngruppe.",
        ));
    }
    Ok(())
}

/// Changes the period of a membership.
pub async fn set_period(
    db: &impl ConnectionTrait,
    membership_id: i32,
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
) -> Result<group_membership::Model, ApiError> {
    let membership = find_membership(db, membership_id).await?;
    let group = find_group(db, membership.learning_group_id).await?;
    check_period(
        db,
        &group,
        membership.student_id,
        joined_on,
        left_on,
        Some(membership.id),
    )
    .await?;

    let mut membership = membership.into_active_model();
    membership.joined_on = Set(joined_on);
    membership.left_on = Set(left_on);
    membership.updated_at = Set(now());
    Ok(membership.update(db).await?)
}

/// Adds a period of a student in a group, e.g. when the student returns
/// after having left it. The student must be enrolled in the school year.
pub async fn add_period(
    db: &impl ConnectionTrait,
    group: &learning_group::Model,
    student_id: i32,
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
) -> Result<group_membership::Model, ApiError> {
    let enrolled = load_students(db, group.school_year_id)
        .await?
        .iter()
        .any(|student| student.id == student_id);
    if !enrolled {
        return Err(ApiError::invalid(
            "Der Schüler gehört nicht zum Schuljahr der Lerngruppe.",
        ));
    }
    check_period(db, group, student_id, joined_on, left_on, None).await?;

    let now = now();
    Ok(group_membership::ActiveModel {
        learning_group_id: Set(group.id),
        student_id: Set(student_id),
        joined_on: Set(joined_on),
        left_on: Set(left_on),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

async fn find_membership(
    db: &impl ConnectionTrait,
    id: i32,
) -> Result<group_membership::Model, ApiError> {
    group_membership::Entity::find_by_id(id)
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
    // Periods of each student, by date. A missing start comes first.
    let mut periods: HashMap<i32, Vec<group_membership::Model>> = HashMap::new();
    for membership in memberships_of(db, vec![id]).await? {
        periods
            .entry(membership.student_id)
            .or_default()
            .push(membership);
    }
    for list in periods.values_mut() {
        list.sort_by_key(|membership| membership.joined_on);
    }

    // The student list is already in display order.
    let members = load_students(db, group.school_year_id)
        .await?
        .into_iter()
        .flat_map(|student: StudentRow| {
            periods
                .remove(&student.id)
                .unwrap_or_default()
                .into_iter()
                .map(move |membership| Member {
                    id: membership.id,
                    student_id: student.id,
                    first_name: student.first_name.clone(),
                    last_name: student.last_name.clone(),
                    form_of_address: student.form_of_address,
                    school_class_name: student.school_class_name.clone(),
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
    // A student with several periods in a group counts once.
    let mut members: HashMap<i32, HashSet<i32>> = HashMap::new();
    let group_ids = groups.iter().map(|group| group.id).collect();
    for membership in memberships_of(&state.db, group_ids).await? {
        members
            .entry(membership.learning_group_id)
            .or_default()
            .insert(membership.student_id);
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
                member_count: members.get(&group.id).map_or(0, HashSet::len),
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

/// Adds another period of a student, e.g. after a return to the group.
async fn add_membership(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<MembershipInput>,
) -> Result<Json<GroupDetail>, ApiError> {
    let transaction = state.db.begin().await?;
    let group = find_group(&transaction, id).await?;
    add_period(
        &transaction,
        &group,
        input.student_id,
        input.joined_on,
        input.left_on,
    )
    .await?;
    let detail = load_detail(&transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

/// Changes since when and until when a period lasts.
async fn update_membership(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<PeriodInput>,
) -> Result<Json<GroupDetail>, ApiError> {
    let transaction = state.db.begin().await?;
    let membership = set_period(&transaction, id, input.joined_on, input.left_on).await?;
    let detail = load_detail(&transaction, membership.learning_group_id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

/// Removes one period. Other periods of the same student stay.
async fn remove_membership(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    find_membership(&state.db, id).await?;
    group_membership::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
