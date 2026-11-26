//! Example data for development and tests (concept, E1 to E4).
//!
//! The data is the same on every run: names come from fixed lists, without
//! any randomness. Everything is created through the same functions the
//! endpoints use, so example data cannot break a rule of the data model.
//!
//! Every pull request that introduces new kinds of data extends `fill` and
//! `clear` here.

use anyhow::{Result, bail};
use std::collections::HashMap;

use chrono::{Duration, NaiveDate};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    TransactionTrait,
};

use crate::{
    calendar::{self, SchoolYearTemplate},
    entity::{
        enrollment, group_membership, learning_group, school_class, school_free_period,
        school_year, student, student::FormOfAddress, subject,
    },
    learning_group::{add_period, insert as insert_group, insert_members, set_period},
    school_class::insert as insert_class,
    school_year::insert_with_free_periods,
    student::insert_student,
    subject::insert as insert_subject,
};

const SUBJECTS: [(&str, &str); 4] = [
    ("Technik", "Te"),
    ("Werken und Gestalten", "WG"),
    ("Ernährung und Soziales", "ES"),
    ("Berufs- und Lebensorientierung – Praxis", "BLO-P"),
];

/// Classes with their number of students.
const CLASSES: [(&str, usize); 8] = [
    ("5a", 16),
    ("6a", 17),
    ("7a", 18),
    ("7b", 19),
    ("8a", 17),
    ("8b", 18),
    ("9a", 16),
    ("9b", 17),
];

/// Which part of a class belongs to a learning group.
#[derive(Clone, Copy)]
enum Part {
    All,
    /// The first half in list order, rounded up.
    FirstHalf,
    SecondHalf,
}

/// Name of a learning group, short name of its subject, and the classes it
/// draws its students from.
type GroupSpec = (&'static str, &'static str, &'static [(&'static str, Part)]);

/// The learning groups. Some span two classes, split in halves.
const GROUPS: [GroupSpec; 8] = [
    ("5a Werken", "WG", &[("5a", Part::All)]),
    ("6a Werken", "WG", &[("6a", Part::All)]),
    ("7a Technik", "Te", &[("7a", Part::All)]),
    ("7b Technik", "Te", &[("7b", Part::All)]),
    (
        "8a/b Technik Gruppe 1",
        "Te",
        &[("8a", Part::FirstHalf), ("8b", Part::FirstHalf)],
    ),
    (
        "8a/b Technik Gruppe 2",
        "Te",
        &[("8a", Part::SecondHalf), ("8b", Part::SecondHalf)],
    ),
    (
        "9a/b Ernährung und Soziales",
        "ES",
        &[("9a", Part::FirstHalf), ("9b", Part::FirstHalf)],
    ),
    (
        "9a/b BLO-P",
        "BLO-P",
        &[("9a", Part::SecondHalf), ("9b", Part::SecondHalf)],
    ),
];

/// One student changes from the first to the second of these groups and
/// later back, as an example of changes during the school year: the last
/// day in the first group and the first day back, counted in days after the
/// first school day.
const GROUP_CHANGE: (&str, &str, i64, i64) =
    ("8a/b Technik Gruppe 1", "8a/b Technik Gruppe 2", 154, 223);

/// Students that have not been assigned to a class yet.
const STUDENTS_WITHOUT_CLASS: usize = 2;

/// Every n-th student is addressed by first name only (concept S7).
const NAME_ONLY_EVERY: usize = 12;

const FIRST_NAMES: [(&str, FormOfAddress); 40] = [
    ("Anna", FormOfAddress::She),
    ("Ben", FormOfAddress::He),
    ("Clara", FormOfAddress::She),
    ("David", FormOfAddress::He),
    ("Elif", FormOfAddress::She),
    ("Finn", FormOfAddress::He),
    ("Greta", FormOfAddress::She),
    ("Hasan", FormOfAddress::He),
    ("Ida", FormOfAddress::She),
    ("Jonas", FormOfAddress::He),
    ("Katharina", FormOfAddress::She),
    ("Leon", FormOfAddress::He),
    ("Mia", FormOfAddress::She),
    ("Noah", FormOfAddress::He),
    ("Olivia", FormOfAddress::She),
    ("Paul", FormOfAddress::He),
    ("Rosa", FormOfAddress::She),
    ("Samuel", FormOfAddress::He),
    ("Theresa", FormOfAddress::She),
    ("Umut", FormOfAddress::He),
    ("Valentina", FormOfAddress::She),
    ("Wladimir", FormOfAddress::He),
    ("Yasmin", FormOfAddress::She),
    ("Zoran", FormOfAddress::He),
    ("Amelie", FormOfAddress::She),
    ("Bastian", FormOfAddress::He),
    ("Chiara", FormOfAddress::She),
    ("Emil", FormOfAddress::He),
    ("Franziska", FormOfAddress::She),
    ("Georg", FormOfAddress::He),
    ("Hanna", FormOfAddress::She),
    ("Ivan", FormOfAddress::He),
    ("Johanna", FormOfAddress::She),
    ("Kilian", FormOfAddress::He),
    ("Leyla", FormOfAddress::She),
    ("Maximilian", FormOfAddress::He),
    ("Nele", FormOfAddress::She),
    ("Oskar", FormOfAddress::He),
    ("Sophie", FormOfAddress::She),
    ("Tobias", FormOfAddress::He),
];

const LAST_NAMES: [&str; 47] = [
    "Aigner",
    "Bauer",
    "Celik",
    "Dietrich",
    "Eder",
    "Fischer",
    "Gruber",
    "Huber",
    "Ivanov",
    "Jäger",
    "Kaya",
    "Lang",
    "Maier",
    "Novak",
    "Obermeier",
    "Petrov",
    "Reiter",
    "Schmid",
    "Thoma",
    "Unger",
    "Vogel",
    "Wagner",
    "Yilmaz",
    "Ziegler",
    "Auer",
    "Brandl",
    "Demir",
    "Egger",
    "Friedl",
    "Graf",
    "Hofmann",
    "Kraus",
    "Lechner",
    "Moser",
    "Neumann",
    "Pichler",
    "Rieger",
    "Schuster",
    "Steiner",
    "Wimmer",
    "Zimmermann",
    "Kovac",
    "Hartl",
    "Seidl",
    "Arslan",
    "Berger",
    "Lindner",
];

/// How many rows of each kind exist or were touched.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub school_years: u64,
    pub free_periods: u64,
    pub subjects: u64,
    pub school_classes: u64,
    pub students: u64,
    pub learning_groups: u64,
    pub group_memberships: u64,
}

impl Counts {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl std::fmt::Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Schuljahre, {} unterrichtsfreie Zeiten, {} Fächer, {} Klassen, {} Schüler, \
             {} Lerngruppen mit {} Mitgliedschaften",
            self.school_years,
            self.free_periods,
            self.subjects,
            self.school_classes,
            self.students,
            self.learning_groups,
            self.group_memberships
        )
    }
}

/// What a run of the command did.
#[derive(Debug)]
pub struct Summary {
    /// Name of the school year the example data belongs to.
    pub school_year: String,
    /// What `--reset` deleted before filling. `None` without a reset.
    pub deleted: Option<Counts>,
    pub created: Counts,
}

async fn count(db: &impl ConnectionTrait) -> Result<Counts> {
    Ok(Counts {
        school_years: school_year::Entity::find().count(db).await?,
        free_periods: school_free_period::Entity::find().count(db).await?,
        subjects: subject::Entity::find().count(db).await?,
        school_classes: school_class::Entity::find().count(db).await?,
        students: student::Entity::find().count(db).await?,
        learning_groups: learning_group::Entity::find().count(db).await?,
        group_memberships: group_membership::Entity::find().count(db).await?,
    })
}

/// Deletes all domain data. Account, sessions and login tokens stay (E3).
async fn clear(db: &impl ConnectionTrait) -> Result<()> {
    // Dependent rows first.
    group_membership::Entity::delete_many().exec(db).await?;
    learning_group::Entity::delete_many().exec(db).await?;
    enrollment::Entity::delete_many().exec(db).await?;
    student::Entity::delete_many().exec(db).await?;
    school_free_period::Entity::delete_many().exec(db).await?;
    school_year::Entity::delete_many().exec(db).await?;
    school_class::Entity::delete_many().exec(db).await?;
    subject::Entity::delete_many().exec(db).await?;
    Ok(())
}

/// The school year after the one that contains `today`, taken from the
/// holiday data file (E1). Falls back to its last school year.
fn next_school_year(today: NaiveDate) -> &'static SchoolYearTemplate {
    let years = &calendar::holiday_data().school_years;
    years
        .iter()
        .find(|year| year.first_school_day > today)
        .or(years.last())
        .expect("the holiday data file contains school years")
}

/// First name, last name and form of address of the n-th example student.
/// The step widths share no factor with the list lengths, so names repeat
/// as late as possible and never within the example data.
fn example_student(index: usize) -> (&'static str, &'static str, FormOfAddress) {
    let (first_name, form_of_address) = FIRST_NAMES[(index * 7) % FIRST_NAMES.len()];
    let last_name = LAST_NAMES[(index * 5) % LAST_NAMES.len()];
    let form_of_address = if index % NAME_ONLY_EVERY == NAME_ONLY_EVERY - 1 {
        FormOfAddress::NameOnly
    } else {
        form_of_address
    };
    (first_name, last_name, form_of_address)
}

async fn fill(db: &impl ConnectionTrait, today: NaiveDate) -> Result<String> {
    let template = next_school_year(today);
    let year = insert_with_free_periods(
        db,
        template.name.clone(),
        template.first_school_day,
        template.last_school_day,
    )
    .await?;

    let mut subject_ids = HashMap::new();
    for (name, short_name) in SUBJECTS {
        let subject = insert_subject(db, name, short_name).await?;
        subject_ids.insert(short_name, subject.id);
    }

    // Students of each class in the order they were created.
    let mut students_of: HashMap<&str, Vec<i32>> = HashMap::new();
    let mut index = 0;
    for (class_name, size) in CLASSES {
        let class = insert_class(db, class_name).await?;
        for _ in 0..size {
            let (first_name, last_name, form_of_address) = example_student(index);
            let student_id = insert_student(
                db,
                year.id,
                first_name.to_owned(),
                Some(last_name.to_owned()),
                form_of_address,
                Some(class.id),
            )
            .await?;
            students_of.entry(class_name).or_default().push(student_id);
            index += 1;
        }
    }
    for _ in 0..STUDENTS_WITHOUT_CLASS {
        let (first_name, last_name, form_of_address) = example_student(index);
        insert_student(
            db,
            year.id,
            first_name.to_owned(),
            Some(last_name.to_owned()),
            form_of_address,
            None,
        )
        .await?;
        index += 1;
    }

    let mut groups = HashMap::new();
    for (name, subject, sources) in GROUPS {
        let group = insert_group(db, year.id, name, subject_ids[subject]).await?;
        let mut members = Vec::new();
        for (class_name, part) in sources {
            let students = &students_of[class_name];
            let half = students.len().div_ceil(2);
            members.extend_from_slice(match part {
                Part::All => students,
                Part::FirstHalf => &students[..half],
                Part::SecondHalf => &students[half..],
            });
        }
        insert_members(db, &group, &members).await?;
        groups.insert(name, (group, members));
    }

    // The first member of one group changes to another and later returns.
    let (from, to, left_after, back_after) = GROUP_CHANGE;
    let (from_group, from_members) = &groups[from];
    let (to_group, _) = &groups[to];
    let student_id = from_members[0];
    let left_on = year.start_date + Duration::days(left_after);
    let back_on = year.start_date + Duration::days(back_after);
    let first_period = group_membership::Entity::find()
        .filter(group_membership::Column::LearningGroupId.eq(from_group.id))
        .filter(group_membership::Column::StudentId.eq(student_id))
        .one(db)
        .await?
        .expect("the student was just added");
    set_period(db, first_period.id, None, Some(left_on)).await?;
    add_period(
        db,
        to_group,
        student_id,
        Some(left_on + Duration::days(1)),
        Some(back_on - Duration::days(1)),
    )
    .await?;
    add_period(db, from_group, student_id, Some(back_on), None).await?;

    Ok(year.name)
}

/// Fills the database with example data.
///
/// A database that already holds domain data is left alone unless `reset`
/// is set, and a reset is refused while mail delivery is configured, which
/// marks an instance in operation (E3, E4). Either everything happens or
/// nothing.
pub async fn run(
    db: &DatabaseConnection,
    mail_configured: bool,
    reset: bool,
    today: NaiveDate,
) -> Result<Summary> {
    let transaction = db.begin().await?;
    let existing = count(&transaction).await?;

    let deleted = if existing.is_empty() {
        None
    } else if !reset {
        bail!(
            "Die Datenbank enthält schon Daten ({existing}). Beispieldaten kommen nur in eine \
             leere Datenbank. Mit \"--reset\" werden die vorhandenen Fachdaten vorher gelöscht."
        );
    } else if mail_configured {
        bail!(
            "\"--reset\" löscht alle Fachdaten und ist gesperrt, solange der Mailversand \
             eingerichtet ist (SMTP-Werte gesetzt). Das schützt eine Instanz im Betrieb."
        );
    } else {
        clear(&transaction).await?;
        Some(existing)
    };

    let school_year = fill(&transaction, today).await?;
    let created = count(&transaction).await?;
    transaction.commit().await?;

    Ok(Summary {
        school_year,
        deleted,
        created,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn next_school_year_follows_the_current_one() {
        // During 2026/27 and in the summer holidays before it.
        assert_eq!(next_school_year(date(2026, 11, 26)).name, "2027/28");
        assert_eq!(next_school_year(date(2027, 7, 30)).name, "2027/28");
        assert_eq!(next_school_year(date(2026, 8, 20)).name, "2026/27");
        // On its first day a school year is the current one.
        assert_eq!(next_school_year(date(2027, 9, 14)).name, "2028/29");
        // Beyond the data file the last school year is used.
        assert_eq!(next_school_year(date(2031, 1, 1)).name, "2029/30");
    }

    #[test]
    fn example_students_are_all_different() {
        let total: usize =
            CLASSES.iter().map(|(_, size)| size).sum::<usize>() + STUDENTS_WITHOUT_CLASS;
        assert_eq!(total, 140);

        let names: HashSet<_> = (0..total)
            .map(|index| {
                let (first_name, last_name, _) = example_student(index);
                (first_name, last_name)
            })
            .collect();
        assert_eq!(names.len(), total);
    }

    #[test]
    fn example_students_use_every_form_of_address() {
        let forms: Vec<_> = (0..140).map(|index| example_student(index).2).collect();
        let count = |form| forms.iter().filter(|value| **value == form).count();
        assert_eq!(count(FormOfAddress::NameOnly), 11);
        assert!(count(FormOfAddress::He) > 50);
        assert!(count(FormOfAddress::She) > 50);
    }
}
