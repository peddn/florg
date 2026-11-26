//! CSV import of the students of a school year (data model, D-I1 to
//! D-I8). The file is read with the `csv` crate.

use std::collections::{HashMap, HashSet};

use axum::{
    Json,
    extract::{Path, State},
};
use csv::{ReaderBuilder, StringRecord, Trim};
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set, TransactionTrait};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    db::now,
    entity::{school_class, student::FormOfAddress},
    error::ApiError,
    school_class::load_all as load_classes,
    student::{
        StudentRow, clean_first_name, clean_last_name, ensure_year_exists, insert_student,
        load_rows,
    },
    validate::{MAX_NAME_LENGTH, same_name},
};

/// Most data rows a file may have (D-I8).
const MAX_ROWS: usize = 1000;
/// Largest accepted text, far above any real class list.
const MAX_BYTES: usize = 1_000_000;

#[derive(Deserialize)]
pub struct ImportInput {
    /// The content of the CSV file.
    csv: String,
    /// Only show what would happen (D-I7).
    dry_run: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum RowStatus {
    /// Will be created.
    New,
    /// Already exists in the school year or earlier in the file (D-I6).
    Duplicate,
    Error,
}

#[derive(Debug, Serialize)]
pub struct ImportRow {
    /// Line in the file, counted from 1.
    pub line: u64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub school_class: Option<String>,
    pub form_of_address: Option<FormOfAddress>,
    pub status: RowStatus,
    /// German explanation of an error.
    pub message: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ImportPreview {
    /// German name of the detected delimiter.
    pub delimiter: &'static str,
    /// Columns of the file that florg does not use.
    pub ignored_columns: Vec<String>,
    /// Classes that do not exist yet and are created by the import (D-I5).
    pub new_classes: Vec<String>,
    pub rows: Vec<ImportRow>,
    pub new_count: usize,
    pub duplicate_count: usize,
    pub error_count: usize,
    /// Whether the students were created. Always false for a dry run and
    /// when a row has an error.
    pub imported: bool,
}

/// Positions of the known columns in the file.
#[derive(Default)]
struct Columns {
    first_name: Option<usize>,
    last_name: Option<usize>,
    school_class: Option<usize>,
    form_of_address: Option<usize>,
    gender: Option<usize>,
    ignored: Vec<String>,
}

impl Columns {
    fn from_header(header: &StringRecord) -> Self {
        let mut columns = Self::default();
        for (index, name) in header.iter().enumerate() {
            let slot = match name.trim().to_lowercase().as_str() {
                "vorname" | "rufname" => &mut columns.first_name,
                "nachname" | "familienname" => &mut columns.last_name,
                "klasse" => &mut columns.school_class,
                "anrede" => &mut columns.form_of_address,
                "geschlecht" => &mut columns.gender,
                _ => {
                    if !name.trim().is_empty() {
                        columns.ignored.push(name.trim().to_owned());
                    }
                    continue;
                }
            };
            // The first column of a kind wins.
            slot.get_or_insert(index);
        }
        columns
    }
}

/// Picks the delimiter that occurs most often in the header line (D-I3).
fn detect_delimiter(text: &str) -> (u8, &'static str) {
    let header = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();
    let candidates = [(b';', "Semikolon"), (b',', "Komma"), (b'\t', "Tabulator")];
    let mut best = candidates[0];
    let mut best_count = 0;
    for candidate in candidates {
        let count = header.bytes().filter(|byte| *byte == candidate.0).count();
        if count > best_count {
            best = candidate;
            best_count = count;
        }
    }
    best
}

fn parse_form_of_address(value: &str) -> Result<Option<FormOfAddress>, &'static str> {
    match value.trim().to_lowercase().as_str() {
        "" => Ok(None),
        "er" => Ok(Some(FormOfAddress::He)),
        "sie" => Ok(Some(FormOfAddress::She)),
        "nur vorname" | "vorname" | "-" => Ok(Some(FormOfAddress::NameOnly)),
        _ => Err("Die Anrede ist unbekannt. Erlaubt sind \"er\", \"sie\" oder leer."),
    }
}

fn parse_gender(value: &str) -> Result<Option<FormOfAddress>, &'static str> {
    match value.trim().to_lowercase().as_str() {
        "" => Ok(None),
        "m" | "männlich" | "maennlich" => Ok(Some(FormOfAddress::He)),
        "w" | "weiblich" => Ok(Some(FormOfAddress::She)),
        "d" | "divers" => Ok(Some(FormOfAddress::NameOnly)),
        _ => Err("Das Geschlecht ist unbekannt. Erlaubt sind \"m\", \"w\", \"d\" oder leer."),
    }
}

/// What makes two students the same for the import (D-I6).
fn duplicate_key(
    first_name: &str,
    last_name: Option<&str>,
    school_class: Option<&str>,
) -> (String, String, String) {
    let normalize = |value: &str| value.trim().to_lowercase();
    (
        normalize(first_name),
        normalize(last_name.unwrap_or_default()),
        normalize(school_class.unwrap_or_default()),
    )
}

/// Returns the line, counted from 1, of the record that starts at or after
/// `offset`. The reader reports the offset before any empty lines in front
/// of a record, so those are skipped here.
fn line_of(text: &str, offset: usize) -> u64 {
    let bytes = text.as_bytes();
    let mut start = offset.min(bytes.len());
    while bytes.get(start) == Some(&b'\n') {
        start += 1;
    }
    1 + bytes[..start].iter().filter(|byte| **byte == b'\n').count() as u64
}

/// Reads the file and decides for every row what the import would do.
/// Errors that concern the whole file are returned as `Err`.
pub fn preview(
    text: &str,
    classes: &[school_class::Model],
    existing: &[StudentRow],
) -> Result<ImportPreview, ApiError> {
    if text.len() > MAX_BYTES {
        return Err(ApiError::invalid("Die Datei ist zu groß."));
    }
    // One kind of line ending, so lines can be counted reliably.
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let text = text.as_str();
    let (delimiter, delimiter_name) = detect_delimiter(text);
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(text.as_bytes());

    let unreadable = || ApiError::invalid("Die Datei lässt sich nicht als CSV lesen.");
    let header = reader.headers().map_err(|_| unreadable())?.clone();
    let columns = Columns::from_header(&header);
    let Some(first_name_column) = columns.first_name else {
        return Err(ApiError::invalid(
            "Die Spalte \"Vorname\" fehlt. Die erste Zeile muss die Spalten nennen.",
        ));
    };

    let mut seen: HashSet<_> = existing
        .iter()
        .map(|row| {
            duplicate_key(
                &row.first_name,
                row.last_name.as_deref(),
                row.school_class_name.as_deref(),
            )
        })
        .collect();
    // New classes in the spelling of their first occurrence.
    let mut new_classes: Vec<String> = Vec::new();
    let mut rows = Vec::new();

    for record in reader.records() {
        let record = record.map_err(|_| unreadable())?;
        if rows.len() == MAX_ROWS {
            return Err(ApiError::invalid(
                "Die Datei hat mehr als 1000 Zeilen. Teile sie auf.",
            ));
        }
        let line = record
            .position()
            .map_or(0, |position| line_of(text, position.byte() as usize));
        let field = |column: Option<usize>| column.and_then(|index| record.get(index));

        let raw_first_name = field(Some(first_name_column)).unwrap_or_default();
        let raw_class = field(columns.school_class)
            .map(str::trim)
            .filter(|name| !name.is_empty());

        // Collect the first problem of the row, if any.
        let mut message = None;
        let first_name = clean_first_name(raw_first_name).unwrap_or_else(|_| {
            message = Some(if raw_first_name.trim().is_empty() {
                "Der Vorname fehlt."
            } else {
                "Der Vorname ist zu lang."
            });
            raw_first_name.trim().to_owned()
        });
        let last_name = clean_last_name(field(columns.last_name)).unwrap_or_else(|_| {
            message.get_or_insert("Der Nachname ist zu lang.");
            None
        });
        if raw_class.is_some_and(|name| name.chars().count() > MAX_NAME_LENGTH) {
            message.get_or_insert("Der Name der Klasse ist zu lang.");
        }

        // `Anrede` wins over `Geschlecht` if both are filled (D-I4).
        let from_address = field(columns.form_of_address).map(parse_form_of_address);
        let from_gender = field(columns.gender).map(parse_gender);
        let mut form_of_address = Some(FormOfAddress::NameOnly);
        let mut unreadable_address = false;
        for parsed in [from_gender, from_address].into_iter().flatten() {
            match parsed {
                Ok(Some(value)) => form_of_address = Some(value),
                Ok(None) => {}
                Err(problem) => {
                    message.get_or_insert(problem);
                    unreadable_address = true;
                }
            }
        }
        // The preview shows no form of address it could not read.
        if unreadable_address {
            form_of_address = None;
        }

        // Use the existing spelling of a class, or the first new one.
        let school_class = raw_class.map(|name| {
            if let Some(class) = classes.iter().find(|class| same_name(&class.name, name)) {
                return class.name.clone();
            }
            if let Some(known) = new_classes.iter().find(|known| same_name(known, name)) {
                return known.clone();
            }
            if message.is_none() {
                new_classes.push(name.to_owned());
            }
            name.to_owned()
        });

        let status = if message.is_some() {
            RowStatus::Error
        } else {
            let key = duplicate_key(&first_name, last_name.as_deref(), school_class.as_deref());
            if seen.insert(key) {
                RowStatus::New
            } else {
                RowStatus::Duplicate
            }
        };
        rows.push(ImportRow {
            line,
            first_name,
            last_name,
            school_class,
            form_of_address,
            status,
            message,
        });
    }

    if rows.is_empty() {
        return Err(ApiError::invalid(
            "Die Datei enthält keine Schüler. Unter der Kopfzeile steht nichts.",
        ));
    }

    // Classes that are only named by duplicates need not be created.
    new_classes.retain(|name| {
        rows.iter().any(|row| {
            row.status == RowStatus::New
                && row
                    .school_class
                    .as_deref()
                    .is_some_and(|class| same_name(class, name))
        })
    });

    let count = |status| rows.iter().filter(|row| row.status == status).count();
    Ok(ImportPreview {
        delimiter: delimiter_name,
        ignored_columns: columns.ignored,
        new_classes,
        new_count: count(RowStatus::New),
        duplicate_count: count(RowStatus::Duplicate),
        error_count: count(RowStatus::Error),
        rows,
        imported: false,
    })
}

/// Creates the classes and students a preview calls new.
async fn apply(
    db: &impl ConnectionTrait,
    year_id: i32,
    preview: &ImportPreview,
    classes: &[school_class::Model],
) -> Result<(), ApiError> {
    let now = now();
    let mut class_ids: HashMap<String, i32> = classes
        .iter()
        .map(|class| (class.name.to_lowercase(), class.id))
        .collect();
    for name in &preview.new_classes {
        let class = school_class::ActiveModel {
            name: Set(name.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await?;
        class_ids.insert(name.to_lowercase(), class.id);
    }

    for row in preview
        .rows
        .iter()
        .filter(|row| row.status == RowStatus::New)
    {
        let class_id = row
            .school_class
            .as_ref()
            .and_then(|name| class_ids.get(&name.to_lowercase()).copied());
        insert_student(
            db,
            year_id,
            row.first_name.clone(),
            row.last_name.clone(),
            row.form_of_address.unwrap_or(FormOfAddress::NameOnly),
            class_id,
        )
        .await?;
    }
    Ok(())
}

/// Shows what a CSV file would create, or creates it (D-I7).
pub async fn import(
    State(state): State<AppState>,
    Path(year_id): Path<i32>,
    Json(input): Json<ImportInput>,
) -> Result<Json<ImportPreview>, ApiError> {
    let transaction = state.db.begin().await?;
    ensure_year_exists(&transaction, year_id).await?;
    let classes = load_classes(&transaction).await?;
    let existing = load_rows(&transaction, year_id).await?;
    let mut preview = preview(&input.csv, &classes, &existing)?;

    if !input.dry_run && preview.error_count == 0 {
        apply(&transaction, year_id, &preview, &classes).await?;
        transaction.commit().await?;
        preview.imported = true;
    }
    Ok(Json(preview))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn class(id: i32, name: &str) -> school_class::Model {
        school_class::Model {
            id,
            name: name.to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn existing(first_name: &str, last_name: &str, class: Option<&str>) -> StudentRow {
        StudentRow {
            id: 1,
            first_name: first_name.to_owned(),
            last_name: Some(last_name.to_owned()),
            form_of_address: FormOfAddress::NameOnly,
            school_class_id: class.map(|_| 1),
            school_class_name: class.map(str::to_owned),
        }
    }

    fn statuses(preview: &ImportPreview) -> Vec<RowStatus> {
        preview.rows.iter().map(|row| row.status).collect()
    }

    #[test]
    fn semicolon_file_with_all_columns() {
        let text = "Vorname;Nachname;Klasse;Anrede\nAnna;Huber;7a;sie\nBen;Maier;7b;er\nKim;;;\n";
        let preview = preview(text, &[class(1, "7a")], &[]).unwrap();

        assert_eq!(preview.delimiter, "Semikolon");
        assert_eq!(preview.new_count, 3);
        assert_eq!(preview.new_classes, ["7b"]);
        let anna = &preview.rows[0];
        assert_eq!(anna.line, 2);
        assert_eq!(anna.last_name.as_deref(), Some("Huber"));
        assert_eq!(anna.school_class.as_deref(), Some("7a"));
        assert_eq!(anna.form_of_address, Some(FormOfAddress::She));
        let kim = &preview.rows[2];
        assert_eq!(kim.last_name, None);
        assert_eq!(kim.school_class, None);
        assert_eq!(kim.form_of_address, Some(FormOfAddress::NameOnly));
    }

    #[test]
    fn comma_and_tab_are_detected_and_column_order_is_free() {
        let comma = "klasse,NACHNAME,Vorname\n7a,\"Huber, geb. Maier\",Anna\n";
        let preview_comma = preview(comma, &[], &[]).unwrap();
        assert_eq!(preview_comma.delimiter, "Komma");
        assert_eq!(
            preview_comma.rows[0].last_name.as_deref(),
            Some("Huber, geb. Maier")
        );

        let tab = "Vorname\tNachname\nAnna\tHuber\n";
        let preview_tab = preview(tab, &[], &[]).unwrap();
        assert_eq!(preview_tab.delimiter, "Tabulator");
        assert_eq!(preview_tab.rows[0].first_name, "Anna");
    }

    #[test]
    fn single_column_and_windows_line_endings() {
        let preview = preview("Vorname\r\nAnna\r\n\r\nBen\r\n", &[], &[]).unwrap();
        assert_eq!(preview.new_count, 2);
        assert_eq!(preview.rows[1].first_name, "Ben");
        // Lines are counted as an editor shows them: the header is line 1.
        assert_eq!(preview.rows[0].line, 2);
        assert_eq!(preview.rows[1].line, 4);

        // A value in quotes may span lines; the next row is counted right.
        let text = "Vorname;Nachname\n\"Anna\";\"Huber\nLang\"\nBen;Maier\n";
        let spanning = super::preview(text, &[], &[]).unwrap();
        assert_eq!(spanning.rows[0].line, 2);
        assert_eq!(spanning.rows[1].line, 4);
    }

    #[test]
    fn gender_column_becomes_form_of_address() {
        let text = "Vorname;Geschlecht\nA;m\nB;W\nC;d\nD;\nE;weiblich\n";
        let preview = preview(text, &[], &[]).unwrap();
        let forms: Vec<_> = preview
            .rows
            .iter()
            .map(|row| row.form_of_address.unwrap())
            .collect();
        assert_eq!(
            forms,
            [
                FormOfAddress::He,
                FormOfAddress::She,
                FormOfAddress::NameOnly,
                FormOfAddress::NameOnly,
                FormOfAddress::She,
            ]
        );
    }

    #[test]
    fn form_of_address_wins_over_gender() {
        let text = "Vorname;Geschlecht;Anrede\nA;m;sie\nB;m;\n";
        let preview = preview(text, &[], &[]).unwrap();
        assert_eq!(preview.rows[0].form_of_address, Some(FormOfAddress::She));
        assert_eq!(preview.rows[1].form_of_address, Some(FormOfAddress::He));
    }

    #[test]
    fn unknown_columns_are_ignored_and_named() {
        let text = "Vorname;Geburtsdatum;Klasse;Ort\nAnna;01.02.2013;7a;Hof\n";
        let preview = preview(text, &[], &[]).unwrap();
        assert_eq!(preview.ignored_columns, ["Geburtsdatum", "Ort"]);
        assert_eq!(preview.rows[0].school_class.as_deref(), Some("7a"));
    }

    #[test]
    fn rows_with_problems_are_errors() {
        let long = "x".repeat(101);
        let text = format!(
            "Vorname;Nachname;Klasse;Anrede;Geschlecht\n;Huber;7a;;\nAnna;Huber;7a;es;\nBen;Maier;7a;;x\n{long};M;7a;;\nCem;{long};7a;;\nDana;K;{long};;\nEva;Lang;7a;;\n"
        );
        let preview = preview(&text, &[], &[]).unwrap();

        assert_eq!(preview.error_count, 6);
        assert_eq!(preview.new_count, 1);
        assert_eq!(preview.rows[0].message, Some("Der Vorname fehlt."));
        assert!(preview.rows[1].message.unwrap().contains("Anrede"));
        assert_eq!(preview.rows[1].form_of_address, None);
        assert!(preview.rows[2].message.unwrap().contains("Geschlecht"));
        assert_eq!(preview.rows[3].message, Some("Der Vorname ist zu lang."));
        assert_eq!(preview.rows[4].message, Some("Der Nachname ist zu lang."));
        assert_eq!(
            preview.rows[5].message,
            Some("Der Name der Klasse ist zu lang.")
        );
        // Only the class of the valid row is created.
        assert_eq!(preview.new_classes, ["7a"]);
    }

    #[test]
    fn existing_and_repeated_students_are_duplicates() {
        let text =
            "Vorname;Nachname;Klasse\nanna;HUBER;7A\nAnna;Huber;7b\nBen;Maier;7a\nBen;Maier;7a\n";
        let preview = preview(
            text,
            &[class(1, "7a")],
            &[existing("Anna", "Huber", Some("7a"))],
        )
        .unwrap();

        assert_eq!(
            statuses(&preview),
            [
                RowStatus::Duplicate,
                RowStatus::New,
                RowStatus::New,
                RowStatus::Duplicate
            ]
        );
        // The class keeps its existing spelling.
        assert_eq!(preview.rows[0].school_class.as_deref(), Some("7a"));
        assert_eq!(preview.new_classes, ["7b"]);
    }

    #[test]
    fn new_classes_keep_their_first_spelling_and_are_listed_once() {
        let text = "Vorname;Klasse\nA;8a\nB;8A\nC; 8a \n";
        let preview = preview(text, &[], &[]).unwrap();
        assert_eq!(preview.new_classes, ["8a"]);
        assert!(
            preview
                .rows
                .iter()
                .all(|row| row.school_class.as_deref() == Some("8a"))
        );
    }

    #[test]
    fn files_without_usable_content_are_refused() {
        for text in ["", "Nachname;Klasse\nHuber;7a\n", "Vorname;Nachname\n"] {
            assert!(preview(text, &[], &[]).is_err(), "{text:?}");
        }
        let mut too_many = String::from("Vorname\n");
        for index in 0..=MAX_ROWS {
            too_many.push_str(&format!("S{index}\n"));
        }
        assert!(preview(&too_many, &[], &[]).is_err());
    }
}
