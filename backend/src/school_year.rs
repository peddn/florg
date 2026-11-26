//! School years and their school-free periods (concept Z1, Z1a; data
//! model D-F1 to D-F6).

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
};
use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

use crate::{
    AppState, calendar,
    db::now,
    entity::{
        school_free_period::{self, Kind},
        school_year,
    },
    error::ApiError,
};

/// Longest school year in days. Guards against typos in the year.
const MAX_SCHOOL_YEAR_DAYS: i64 = 400;
const MAX_NAME_LENGTH: usize = 100;

/// Routes of this module. All of them need a session.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/school-years", get(list).post(create))
        .route("/school-years/templates", get(templates))
        .route("/school-years/{id}", get(detail).put(update).delete(remove))
        .route("/school-years/{id}/free-periods", post(create_period))
        .route(
            "/free-periods/{id}",
            put(update_period).delete(remove_period),
        )
}

#[derive(Serialize)]
struct SchoolYearSummary {
    id: i32,
    name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Serialize)]
struct SchoolYearDetail {
    id: i32,
    name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    /// Number of school days (D-F6).
    school_days: u32,
    free_periods: Vec<FreePeriod>,
}

#[derive(Serialize)]
struct FreePeriod {
    id: i32,
    name: String,
    kind: Kind,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Deserialize)]
struct SchoolYearInput {
    name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Deserialize)]
struct FreePeriodInput {
    name: String,
    kind: Kind,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl From<school_year::Model> for SchoolYearSummary {
    fn from(year: school_year::Model) -> Self {
        Self {
            id: year.id,
            name: year.name,
            start_date: year.start_date,
            end_date: year.end_date,
        }
    }
}

impl From<school_free_period::Model> for FreePeriod {
    fn from(period: school_free_period::Model) -> Self {
        Self {
            id: period.id,
            name: period.name,
            kind: period.kind,
            start_date: period.start_date,
            end_date: period.end_date,
        }
    }
}

/// Trims a name and checks that it is neither empty nor overlong.
fn clean_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::invalid("Der Name fehlt."));
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(ApiError::invalid("Der Name ist zu lang."));
    }
    Ok(name.to_owned())
}

impl SchoolYearInput {
    fn validated(self) -> Result<Self, ApiError> {
        let name = clean_name(&self.name)?;
        if self.start_date > self.end_date {
            return Err(ApiError::invalid(
                "Der erste Schultag liegt nach dem letzten Schultag.",
            ));
        }
        if (self.end_date - self.start_date).num_days() > MAX_SCHOOL_YEAR_DAYS {
            return Err(ApiError::invalid(
                "Ein Schuljahr darf höchstens 400 Tage lang sein.",
            ));
        }
        Ok(Self { name, ..self })
    }
}

impl FreePeriodInput {
    fn validated(self, year: &school_year::Model) -> Result<Self, ApiError> {
        let name = clean_name(&self.name)?;
        if self.start_date > self.end_date {
            return Err(ApiError::invalid(
                "Der erste freie Tag liegt nach dem letzten freien Tag.",
            ));
        }
        if self.start_date < year.start_date || self.end_date > year.end_date {
            return Err(ApiError::invalid(
                "Die unterrichtsfreie Zeit liegt nicht vollständig im Schuljahr.",
            ));
        }
        Ok(Self { name, ..self })
    }
}

async fn find_year(db: &impl ConnectionTrait, id: i32) -> Result<school_year::Model, ApiError> {
    school_year::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

async fn find_periods(
    db: &impl ConnectionTrait,
    school_year_id: i32,
) -> Result<Vec<school_free_period::Model>, ApiError> {
    Ok(school_free_period::Entity::find()
        .filter(school_free_period::Column::SchoolYearId.eq(school_year_id))
        .order_by_asc(school_free_period::Column::StartDate)
        .order_by_asc(school_free_period::Column::EndDate)
        .order_by_asc(school_free_period::Column::Id)
        .all(db)
        .await?)
}

/// Rejects a name that another school year already uses.
async fn ensure_name_is_free(
    db: &impl ConnectionTrait,
    name: &str,
    except_id: Option<i32>,
) -> Result<(), ApiError> {
    let mut query = school_year::Entity::find().filter(school_year::Column::Name.eq(name));
    if let Some(id) = except_id {
        query = query.filter(school_year::Column::Id.ne(id));
    }
    if query.count(db).await? > 0 {
        return Err(ApiError::conflict(
            "Ein Schuljahr mit diesem Namen gibt es schon.",
        ));
    }
    Ok(())
}

async fn load_detail(db: &impl ConnectionTrait, id: i32) -> Result<SchoolYearDetail, ApiError> {
    let year = find_year(db, id).await?;
    let periods = find_periods(db, id).await?;
    let ranges: Vec<_> = periods
        .iter()
        .map(|period| (period.start_date, period.end_date))
        .collect();

    Ok(SchoolYearDetail {
        id: year.id,
        name: year.name,
        start_date: year.start_date,
        end_date: year.end_date,
        school_days: calendar::count_school_days(year.start_date, year.end_date, &ranges),
        free_periods: periods.into_iter().map(FreePeriod::from).collect(),
    })
}

/// Lists all school years, the latest first.
async fn list(State(state): State<AppState>) -> Result<Json<Vec<SchoolYearSummary>>, ApiError> {
    let years = school_year::Entity::find()
        .order_by_desc(school_year::Column::StartDate)
        .all(&state.db)
        .await?;
    Ok(Json(years.into_iter().map(Into::into).collect()))
}

/// Lists the school years of the holiday data file as templates (D-F3b).
async fn templates() -> Json<&'static [calendar::SchoolYearTemplate]> {
    Json(&calendar::holiday_data().school_years)
}

/// Creates a school year and fills in its holidays and public holidays
/// (D-F1).
async fn create(
    State(state): State<AppState>,
    Json(input): Json<SchoolYearInput>,
) -> Result<(StatusCode, Json<SchoolYearDetail>), ApiError> {
    let input = input.validated()?;
    let now = now();

    let transaction = state.db.begin().await?;
    ensure_name_is_free(&transaction, &input.name, None).await?;
    let year = school_year::ActiveModel {
        name: Set(input.name),
        start_date: Set(input.start_date),
        end_date: Set(input.end_date),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&transaction)
    .await?;

    let holidays = calendar::school_holidays_within(year.start_date, year.end_date)
        .into_iter()
        .map(|holiday| {
            (
                holiday.name.clone(),
                Kind::Holiday,
                holiday.start_date,
                holiday.end_date,
            )
        });
    let public_holidays = calendar::public_holidays_within(year.start_date, year.end_date)
        .into_iter()
        .map(|holiday| {
            (
                holiday.name.to_owned(),
                Kind::PublicHoliday,
                holiday.date,
                holiday.date,
            )
        });
    let periods: Vec<_> = holidays
        .chain(public_holidays)
        .map(
            |(name, kind, start_date, end_date)| school_free_period::ActiveModel {
                school_year_id: Set(year.id),
                name: Set(name),
                kind: Set(kind),
                start_date: Set(start_date),
                end_date: Set(end_date),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            },
        )
        .collect();
    if !periods.is_empty() {
        school_free_period::Entity::insert_many(periods)
            .exec(&transaction)
            .await?;
    }

    let detail = load_detail(&transaction, year.id).await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

/// Returns a school year with its free periods and its number of school
/// days.
async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<SchoolYearDetail>, ApiError> {
    Ok(Json(load_detail(&state.db, id).await?))
}

/// Changes name and dates of a school year. Its free periods must still
/// lie within it afterwards.
async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<SchoolYearInput>,
) -> Result<Json<SchoolYearDetail>, ApiError> {
    let input = input.validated()?;

    let transaction = state.db.begin().await?;
    let year = find_year(&transaction, id).await?;
    ensure_name_is_free(&transaction, &input.name, Some(id)).await?;

    let outside = find_periods(&transaction, id)
        .await?
        .iter()
        .any(|period| period.start_date < input.start_date || period.end_date > input.end_date);
    if outside {
        return Err(ApiError::invalid(
            "Mindestens eine unterrichtsfreie Zeit läge dann außerhalb des Schuljahres. Ändere oder lösche sie zuerst.",
        ));
    }

    let mut year = year.into_active_model();
    year.name = Set(input.name);
    year.start_date = Set(input.start_date);
    year.end_date = Set(input.end_date);
    year.updated_at = Set(now());
    year.update(&transaction).await?;

    let detail = load_detail(&transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(detail))
}

/// Deletes a school year together with its free periods.
async fn remove(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    let transaction = state.db.begin().await?;
    find_year(&transaction, id).await?;
    delete_periods_of(&transaction, id).await?;
    school_year::Entity::delete_by_id(id)
        .exec(&transaction)
        .await?;
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

// Deleted explicitly instead of relying on the cascade of the foreign key,
// so the result does not depend on a database setting.
async fn delete_periods_of(db: &impl ConnectionTrait, school_year_id: i32) -> Result<(), ApiError> {
    school_free_period::Entity::delete_many()
        .filter(school_free_period::Column::SchoolYearId.eq(school_year_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Adds a free period to a school year.
async fn create_period(
    State(state): State<AppState>,
    Path(school_year_id): Path<i32>,
    Json(input): Json<FreePeriodInput>,
) -> Result<(StatusCode, Json<FreePeriod>), ApiError> {
    let year = find_year(&state.db, school_year_id).await?;
    let input = input.validated(&year)?;
    let now = now();

    let period = school_free_period::ActiveModel {
        school_year_id: Set(year.id),
        name: Set(input.name),
        kind: Set(input.kind),
        start_date: Set(input.start_date),
        end_date: Set(input.end_date),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(period.into())))
}

async fn find_period(
    db: &DatabaseConnection,
    id: i32,
) -> Result<school_free_period::Model, ApiError> {
    school_free_period::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NOT_FOUND)
}

/// Changes a free period.
async fn update_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<FreePeriodInput>,
) -> Result<Json<FreePeriod>, ApiError> {
    let period = find_period(&state.db, id).await?;
    let year = find_year(&state.db, period.school_year_id).await?;
    let input = input.validated(&year)?;

    let mut period = period.into_active_model();
    period.name = Set(input.name);
    period.kind = Set(input.kind);
    period.start_date = Set(input.start_date);
    period.end_date = Set(input.end_date);
    period.updated_at = Set(now());
    let period = period.update(&state.db).await?;
    Ok(Json(period.into()))
}

/// Deletes a free period.
async fn remove_period(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    find_period(&state.db, id).await?;
    school_free_period::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
