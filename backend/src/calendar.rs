//! Calendar rules for Bavarian schools: public holidays, school holidays
//! and school days (data model, D-F1 to D-F6).

use std::sync::LazyLock;

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// School holidays from the official calendar of the Bavarian ministry of
/// education. The file names its source (D-F3).
const HOLIDAY_DATA_JSON: &str = include_str!("../data/school_holidays_bavaria.json");

/// Content of the holiday data file (D-F3a).
#[derive(Debug, Deserialize)]
pub struct HolidayData {
    pub source: String,
    pub retrieved: NaiveDate,
    pub school_years: Vec<SchoolYearTemplate>,
}

/// A school year of the data file. Serves as a template when a school year
/// is created (D-F3b).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SchoolYearTemplate {
    pub name: String,
    #[serde(rename(serialize = "start_date"))]
    pub first_school_day: NaiveDate,
    #[serde(rename(serialize = "end_date"))]
    pub last_school_day: NaiveDate,
    #[serde(skip_serializing)]
    pub holidays: Vec<Holiday>,
}

/// One period of school holidays.
#[derive(Debug, Clone, Deserialize)]
pub struct Holiday {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// A public holiday on a given date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicHoliday {
    pub name: &'static str,
    pub date: NaiveDate,
}

/// Returns the holiday data built into the program.
pub fn holiday_data() -> &'static HolidayData {
    static DATA: LazyLock<HolidayData> = LazyLock::new(|| {
        // The file is part of the program. A unit test parses it.
        serde_json::from_str(HOLIDAY_DATA_JSON).expect("holiday data file is valid")
    });
    &DATA
}

/// Returns all school holidays of the data file that lie completely
/// within the given period (D-F3b).
pub fn school_holidays_within(start: NaiveDate, end: NaiveDate) -> Vec<&'static Holiday> {
    holiday_data()
        .school_years
        .iter()
        .flat_map(|year| &year.holidays)
        .filter(|holiday| start <= holiday.start_date && holiday.end_date <= end)
        .collect()
}

/// Returns Easter Sunday of a year in the Gregorian calendar, computed
/// with the anonymous Gregorian algorithm (Meeus/Jones/Butcher).
pub fn easter_sunday(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .expect("the algorithm yields a date in March or April")
}

/// Returns the Day of Repentance and Prayer: the Wednesday before
/// 23 November. It is free of lessons at Bavarian schools (D-F2).
pub fn day_of_repentance(year: i32) -> NaiveDate {
    let mut date = fixed(year, 11, 22);
    while date.weekday() != Weekday::Wed {
        date -= Duration::days(1);
    }
    date
}

fn fixed(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("fixed holidays are valid dates")
}

/// Returns the public holidays of a calendar year that matter for Bavarian
/// schools, in order (D-F2). Regional holidays are left out (D-F5).
pub fn public_holidays(year: i32) -> Vec<PublicHoliday> {
    let easter = easter_sunday(year);
    let after_easter = |days: i64| easter + Duration::days(days);
    let mut holidays = vec![
        ("Neujahr", fixed(year, 1, 1)),
        ("Heilige Drei Könige", fixed(year, 1, 6)),
        ("Karfreitag", after_easter(-2)),
        ("Ostermontag", after_easter(1)),
        ("Tag der Arbeit", fixed(year, 5, 1)),
        ("Christi Himmelfahrt", after_easter(39)),
        ("Pfingstmontag", after_easter(50)),
        ("Fronleichnam", after_easter(60)),
        ("Tag der Deutschen Einheit", fixed(year, 10, 3)),
        ("Allerheiligen", fixed(year, 11, 1)),
        ("Buß- und Bettag", day_of_repentance(year)),
        ("1. Weihnachtstag", fixed(year, 12, 25)),
        ("2. Weihnachtstag", fixed(year, 12, 26)),
    ];
    holidays.sort_by_key(|(_, date)| *date);
    holidays
        .into_iter()
        .map(|(name, date)| PublicHoliday { name, date })
        .collect()
}

/// Returns all public holidays within the given period, in order. Holidays
/// on weekends are included (D-F2a).
pub fn public_holidays_within(start: NaiveDate, end: NaiveDate) -> Vec<PublicHoliday> {
    (start.year()..=end.year())
        .flat_map(public_holidays)
        .filter(|holiday| start <= holiday.date && holiday.date <= end)
        .collect()
}

/// Counts the school days of a period: Monday to Friday, without the days
/// that lie in one of the free periods (D-F6). Free periods are given as
/// first and last free day and may overlap.
pub fn count_school_days(
    start: NaiveDate,
    end: NaiveDate,
    free_periods: &[(NaiveDate, NaiveDate)],
) -> u32 {
    start
        .iter_days()
        .take_while(|day| *day <= end)
        .filter(|day| !matches!(day.weekday(), Weekday::Sat | Weekday::Sun))
        .filter(|day| {
            !free_periods
                .iter()
                .any(|(first, last)| first <= day && day <= last)
        })
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn easter_sunday_matches_known_dates() {
        let known = [
            (2000, 4, 23),
            (2008, 3, 23),
            (2011, 4, 24),
            (2024, 3, 31),
            (2025, 4, 20),
            (2026, 4, 5),
            (2027, 3, 28),
            (2028, 4, 16),
            (2029, 4, 1),
            (2030, 4, 21),
            (2038, 4, 25),
        ];
        for (year, month, day) in known {
            assert_eq!(easter_sunday(year), date(year, month, day), "{year}");
        }
    }

    #[test]
    fn day_of_repentance_matches_the_official_calendar() {
        // Dates from the calendar file of the ministry.
        let official = [
            (2025, 11, 19),
            (2026, 11, 18),
            (2027, 11, 17),
            (2028, 11, 22),
            (2029, 11, 21),
        ];
        for (year, month, day) in official {
            assert_eq!(day_of_repentance(year), date(year, month, day), "{year}");
        }
        // 22 November itself is a Wednesday in 2023.
        assert_eq!(day_of_repentance(2023), date(2023, 11, 22));
    }

    #[test]
    fn public_holidays_of_2027_are_complete_and_ordered() {
        let holidays = public_holidays(2027);
        let expected = [
            ("Neujahr", date(2027, 1, 1)),
            ("Heilige Drei Könige", date(2027, 1, 6)),
            ("Karfreitag", date(2027, 3, 26)),
            ("Ostermontag", date(2027, 3, 29)),
            ("Tag der Arbeit", date(2027, 5, 1)),
            ("Christi Himmelfahrt", date(2027, 5, 6)),
            ("Pfingstmontag", date(2027, 5, 17)),
            ("Fronleichnam", date(2027, 5, 27)),
            ("Tag der Deutschen Einheit", date(2027, 10, 3)),
            ("Allerheiligen", date(2027, 11, 1)),
            ("Buß- und Bettag", date(2027, 11, 17)),
            ("1. Weihnachtstag", date(2027, 12, 25)),
            ("2. Weihnachtstag", date(2027, 12, 26)),
        ];
        let actual: Vec<_> = holidays.iter().map(|h| (h.name, h.date)).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn public_holidays_within_spans_two_calendar_years() {
        let holidays = public_holidays_within(date(2026, 9, 15), date(2027, 7, 30));
        assert_eq!(holidays.len(), 13);
        assert_eq!(holidays.first().unwrap().name, "Tag der Deutschen Einheit");
        assert_eq!(holidays.last().unwrap().name, "Fronleichnam");
        // Included although it is a Saturday (D-F2a).
        assert!(holidays.iter().any(|h| h.date == date(2026, 10, 3)));
    }

    #[test]
    fn holiday_data_file_is_consistent() {
        let data = holiday_data();
        assert!(data.source.contains("BayMBl. 2022 Nr. 747"));
        let names: Vec<_> = data.school_years.iter().map(|y| y.name.as_str()).collect();
        assert_eq!(names, ["2026/27", "2027/28", "2028/29", "2029/30"]);

        for year in &data.school_years {
            assert!(
                year.first_school_day < year.last_school_day,
                "{}",
                year.name
            );
            assert_eq!(year.holidays.len(), 5, "{}", year.name);
            let mut previous_end = year.first_school_day;
            for holiday in &year.holidays {
                assert!(holiday.start_date <= holiday.end_date, "{}", holiday.name);
                assert!(previous_end < holiday.start_date, "{}", holiday.name);
                assert!(holiday.end_date < year.last_school_day, "{}", holiday.name);
                previous_end = holiday.end_date;
            }
        }
    }

    #[test]
    fn holiday_data_matches_the_published_dates_of_2026_27() {
        // As printed on the website of the ministry for 2026/2027.
        let year = &holiday_data().school_years[0];
        assert_eq!(year.first_school_day, date(2026, 9, 15));
        assert_eq!(year.last_school_day, date(2027, 7, 30));
        let published = [
            (
                "Unterrichtsfreie Tage um Allerheiligen",
                date(2026, 11, 2),
                date(2026, 11, 6),
            ),
            ("Weihnachtsferien", date(2026, 12, 24), date(2027, 1, 8)),
            ("Frühjahrsferien", date(2027, 2, 8), date(2027, 2, 12)),
            ("Osterferien", date(2027, 3, 22), date(2027, 4, 2)),
            ("Pfingstferien", date(2027, 5, 18), date(2027, 5, 28)),
        ];
        let actual: Vec<_> = year
            .holidays
            .iter()
            .map(|h| (h.name.as_str(), h.start_date, h.end_date))
            .collect();
        assert_eq!(actual, published);
    }

    #[test]
    fn school_days_skip_weekends_and_free_periods() {
        // Monday to Sunday, two weeks.
        let start = date(2026, 11, 2);
        let end = date(2026, 11, 15);
        assert_eq!(count_school_days(start, end, &[]), 10);
        // A free week and an overlapping single day count once.
        let free = [
            (date(2026, 11, 2), date(2026, 11, 6)),
            (date(2026, 11, 4), date(2026, 11, 4)),
            // A free Saturday changes nothing.
            (date(2026, 11, 14), date(2026, 11, 14)),
        ];
        assert_eq!(count_school_days(start, end, &free), 5);
        assert_eq!(count_school_days(start, start, &[]), 1);
    }
}
