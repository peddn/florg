# florg – Datenmodell

Stand: 26.11.2026

Dieses Dokument leitet das Datenmodell aus `docs/konzept.md` ab. Teil 1 zeigt alle Objekte grob über alle sieben Stufen. Teil 2 legt Tabellen und Felder nur für Stufe 1 fest. Die Details der späteren Stufen entstehen jeweils vor deren Umsetzung.

Bezeichner folgen dem Glossar in `docs/konzept.md`, Abschnitt 2.

## Grundsätze

- D1: Die Datenbank ist SQLite. Der Zugriff läuft über SeaORM als Zwischenschicht, damit ein späterer Wechsel zu PostgreSQL möglich bleibt.
- D2: Das Schema nutzt nur, was beide Datenbanken können: keine SQLite-Sonderfunktionen, keine dynamische Typisierung, Fremdschlüssel sind eingeschaltet.
- D3: Jede Tabelle hat einen ganzzahligen Primärschlüssel `id`.
- D4: Ein Datum wird als ISO-Text `JJJJ-MM-TT` gespeichert, ein Zeitpunkt als ISO-Text in UTC. Beides lässt sich in PostgreSQL verlustfrei in `DATE` und `TIMESTAMPTZ` überführen.
- D5: Jede Tabelle hat `created_at` und `updated_at`. Sie sind in den Feldlisten unten nicht wiederholt.
- D6: Aufzählungswerte (z. B. Status) werden als englischer Text gespeichert und im Code als Enum geführt.
- D7: Schemaänderungen laufen ausschließlich über Migrationen.
- D8: Es gibt genau ein Konto je Instanz. Tabellen tragen deshalb keinen Besitzer.

## Teil 1: Gesamtmodell (grob)

```mermaid
erDiagram
    account ||--o{ login_token : fordert_an
    account ||--o{ session : hat
    school_year ||--o{ school_free_period : hat
    school_year ||--o{ school_class : hat
    school_year ||--o{ learning_group : hat
    school_year ||--o{ timetable : hat
    subject ||--o{ learning_group : unterrichtet_in
    school_class ||--o{ class_membership : enthaelt
    student ||--o{ class_membership : gehoert_zu
    learning_group ||--o{ group_membership : enthaelt
    student ||--o{ group_membership : gehoert_zu
    timetable ||--o{ timetable_entry : besteht_aus
    learning_group ||--o{ timetable_entry : belegt
    learning_group ||--o{ lesson_block : hat

    catalog ||--o{ section : gliedert
    section ||--o{ section : enthaelt
    section ||--o{ competency : buendelt
    catalog ||--o{ content_item : nennt
    learning_group ||--o{ topic_area : plant
    topic_area ||--o{ learning_unit : enthaelt
    learning_unit }o--o{ competency : zielt_auf
    learning_unit ||--o{ lesson_block : belegt

    lesson_block ||--o| lesson_record : dokumentiert
    lesson_block ||--o{ absence : verzeichnet
    lesson_block ||--o{ observation : liefert
    student ||--o{ observation : erhaelt
    competency ||--o{ observation : betrifft
    student ||--o{ note : erhaelt

    report_type ||--o{ phrase : hat
    section ||--o{ phrase : hat
    student ||--o{ report_text : erhaelt
    report_type ||--o{ report_text : gilt_fuer

    assessment_type ||--o{ assessment : typisiert
    learning_group ||--o{ assessment : hat
    assessment ||--o| rubric : nutzt
    assessment ||--o{ assessment_result : ergibt
    student ||--o{ assessment_result : erhaelt

    material ||--o{ material_price : kostet
    learning_unit ||--o{ bill_of_materials_item : braucht
    material ||--o{ bill_of_materials_item : geht_ein
```

### Objekte je Stufe

| Stufe | Objekte | Dockt an |
| --- | --- | --- |
| 1 Zeitgerüst | `account`, `login_token`, `session`, `school_year`, `school_free_period`, `subject`, `school_class`, `student`, `class_membership`, `learning_group`, `group_membership`, `timetable`, `timetable_entry`, `lesson_block` | – |
| 2 Jahresplanung | `catalog`, `section`, `competency`, `content_item`, `topic_area`, `learning_unit` sowie die Zuordnungen von Kompetenzen zu Lerngruppe, Themenbereich und Lerneinheit | `learning_group`, `lesson_block` |
| 3 Unterrichtsnachweis | `lesson_record`, `absence` | `lesson_block`, `student` |
| 4 Beobachtung | `observation`, `note` | `lesson_block`, `student`, `competency` |
| 5 Zeugnisbemerkungen | `report_type`, `phrase`, `report_text` | `section`, `student`, `learning_group` |
| 6 Noten | `assessment_type`, `assessment`, `grading_scale`, `rubric` mit Kriterien, `assessment_result` mit Kriterienpunkten, Zeugnisnote | `learning_group`, `learning_unit`, `student`, `report_type` |
| 7 Kosten | `material`, `material_price`, `bill_of_materials_item` | `learning_unit`, `school_year` |

### Andockpunkte, die Stufe 1 schon berücksichtigt

- `lesson_block` ist der Anker für Nachweis, Anwesenheit und Beobachtung (Regeln B1, W3). Deshalb bekommt jeder Block eine stabile `id` und wird nie gelöscht, sobald Daten an ihm hängen (Regel D-B4 unten).
- `learning_group` trägt später die Katalogversion (K2), die Kompetenzauswahl (K5), die Themenbereiche und die Leistungsnachweise.
- `student` ist unabhängig vom Schuljahr, damit Beobachtungen über Jahre weiterlaufen (Z9, B10). Die Zugehörigkeit zu Klasse und Lerngruppe ist je Schuljahr eigens gespeichert.
- `lesson_block.learning_unit_id` kommt in Stufe 2 als neues, optionales Feld dazu. Stufe 1 braucht es nicht.

## Teil 2: Stufe 1 im Detail

### account

Das eine Konto der Instanz.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `email` | Text | ja | Adresse, an die Anmeldelinks gehen |
| `display_name` | Text | nein | Name für Ausgaben |

Es gibt kein Passwort. Die Anmeldung läuft nur über einen Link per E-Mail (siehe `docs/konzept.md`, Abschnitt 11, Regeln L1 bis L13).

### login_token

Ein angeforderter Anmeldelink.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `account_id` | Fremdschlüssel | ja | Konto |
| `token_hash` | Text | ja | Hash des Tokens, eindeutig. Das Token selbst wird nie gespeichert |
| `browser_binding_hash` | Text | ja | Hash eines Werts, der nur im anfordernden Browser liegt |
| `expires_at` | Zeitpunkt | ja | Ablauf, 15 Minuten nach Anforderung |
| `used_at` | Zeitpunkt | nein | Zeitpunkt der Einlösung. Gesetzt bedeutet: verbraucht |

Abgelaufene und verbrauchte Token werden regelmäßig gelöscht.

### session

Eine angemeldete Sitzung auf einem Gerät.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `account_id` | Fremdschlüssel | ja | Konto |
| `session_hash` | Text | ja | Hash der Sitzungskennung, eindeutig. Die Kennung selbst liegt nur im Cookie |
| `last_used_at` | Zeitpunkt | ja | Letzte Nutzung |
| `expires_at` | Zeitpunkt | ja | Ablauf, 30 Tage nach der letzten Nutzung |

"Abmelden" löscht die Sitzung des aktuellen Geräts, "Überall abmelden" alle Sitzungen des Kontos (Regel L9).

### school_year

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `name` | Text | ja | z. B. "2026/27", eindeutig |
| `start_date` | Datum | ja | Erster Schultag |
| `end_date` | Datum | ja | Letzter Schultag |

### school_free_period

Ferien, Feiertage und andere unterrichtsfreie Tage. Ein einzelner Tag hat gleiches Start- und Enddatum.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `school_year_id` | Fremdschlüssel | ja | Schuljahr |
| `name` | Text | ja | z. B. "Weihnachtsferien" |
| `kind` | Text | ja | `holiday`, `public_holiday` oder `other` |
| `start_date` | Datum | ja | Erster freier Tag |
| `end_date` | Datum | ja | Letzter freier Tag |

### subject

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `name` | Text | ja | z. B. "Berufs- und Lebensorientierung – Praxis" |
| `short_name` | Text | ja | z. B. "BLO-P", für die mobile Ansicht |

### school_class

Eine Klasse in einem Schuljahr. Die "7a" des nächsten Jahres ist ein neuer Datensatz.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `school_year_id` | Fremdschlüssel | ja | Schuljahr |
| `name` | Text | ja | z. B. "7a", eindeutig je Schuljahr |
| `grade_level` | Ganzzahl | ja | Jahrgangsstufe |

### student

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `first_name` | Text | ja | Vorname oder Kürzel |
| `last_name` | Text | nein | Nachname; leer bei Arbeit mit Kürzeln |
| `form_of_address` | Text | ja | `he`, `she` oder `name_only` (Regel S7) |

### class_membership

Welcher Schüler in welchem Schuljahr in welcher Klasse ist.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `student_id` | Fremdschlüssel | ja | Schüler |
| `school_class_id` | Fremdschlüssel | ja | Klasse; das Schuljahr ergibt sich aus der Klasse |

Ein Schüler ist je Schuljahr in höchstens einer Klasse.

### learning_group

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `school_year_id` | Fremdschlüssel | ja | Schuljahr |
| `subject_id` | Fremdschlüssel | ja | Fach |
| `name` | Text | ja | z. B. "7a/b Technik Gruppe 1", eindeutig je Schuljahr |

### group_membership

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `learning_group_id` | Fremdschlüssel | ja | Lerngruppe |
| `student_id` | Fremdschlüssel | ja | Schüler |
| `joined_on` | Datum | nein | Leer bedeutet: seit Schuljahresbeginn |
| `left_on` | Datum | nein | Leer bedeutet: bis Schuljahresende |

Ein Schüler kann in mehreren Lerngruppen sein. Die Datumsfelder decken Wechsel während des Schuljahres ab.

### timetable

Ein Stundenplan mit Gültigkeitszeitraum. Ändert sich der Plan während des Jahres, wird ein neuer Stundenplan angelegt, der den alten ablöst.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `school_year_id` | Fremdschlüssel | ja | Schuljahr |
| `name` | Text | ja | z. B. "ab September" |
| `valid_from` | Datum | ja | Erster Tag der Gültigkeit |
| `valid_to` | Datum | nein | Leer bedeutet: bis Schuljahresende |
| `cycle_length` | Ganzzahl | ja | 1 = jede Woche gleich, 2 = Wechsel von A- und B-Woche |
| `cycle_anchor_date` | Datum | nein | Ein Tag in einer Schulwoche, die A-Woche ist. Pflicht bei `cycle_length` 2 |

Die Gültigkeitszeiträume der Stundenpläne eines Schuljahres überschneiden sich nicht.

### timetable_entry

Eine einzelne Stunde im Wochenraster.

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `timetable_id` | Fremdschlüssel | ja | Stundenplan |
| `learning_group_id` | Fremdschlüssel | ja | Lerngruppe |
| `weekday` | Ganzzahl | ja | 1 = Montag bis 5 = Freitag |
| `period` | Ganzzahl | ja | Nummer der Unterrichtsstunde am Tag |
| `cycle_week` | Ganzzahl | nein | Leer = jede Woche, 1 = nur A-Woche, 2 = nur B-Woche |

Je Stundenplan, Wochentag, Stunde und Woche im Wechsel gibt es höchstens einen Eintrag. Ein Eintrag für jede Woche schließt Einträge für A oder B auf demselben Platz aus.

### lesson_block

| Feld | Typ | Pflicht | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | ja | Primärschlüssel |
| `learning_group_id` | Fremdschlüssel | ja | Lerngruppe |
| `date` | Datum | ja | Unterrichtstag |
| `first_period` | Ganzzahl | ja | Erste Stunde des Blocks |
| `duration` | Ganzzahl | ja | Dauer in Unterrichtsstunden, mindestens 1 |
| `status` | Text | ja | `planned`, `held` oder `cancelled` |
| `cancel_reason` | Text | nein | Grund bei Ausfall |
| `origin` | Text | ja | `generated` (aus dem Stundenplan) oder `manual` (von Hand angelegt oder geändert) |

Je Lerngruppe, Datum und erster Stunde gibt es höchstens einen Block.

### Regeln zum Vorfüllen der unterrichtsfreien Zeiten

- D-F1: Beim Anlegen eines Schuljahres füllt florg `school_free_period` vor. Danach sind es normale Einträge, die geändert, gelöscht und ergänzt werden können.
- D-F2: Feiertage werden berechnet und nicht als Daten mitgeliefert. Sie haben ein festes Datum oder hängen am Osterdatum. Der Buß- und Bettag ist der Mittwoch vor dem 23. November und an bayerischen Schulen unterrichtsfrei.
- D-F3: Ferien kommen aus einer Datendatei im Repository, abgeschrieben aus der amtlichen Ferienordnung mit Quellenangabe. Die aktuelle Ferienordnung reicht bis zum Schuljahr 2029/30.
- D-F4: Fehlt ein Schuljahr in der Datendatei, werden die Ferien von Hand eingetragen. Schulinterne freie Tage werden immer von Hand eingetragen.
- D-F5: Mariä Himmelfahrt und das Augsburger Friedensfest gelten nur regional. Beide liegen in den Sommerferien und werden deshalb nicht gesondert behandelt.

### Regeln zur Erzeugung der Blöcke

- D-B1: Für jeden Schultag eines Schuljahres, der in keiner unterrichtsfreien Zeit liegt, erzeugt florg aus dem an diesem Tag gültigen Stundenplan die Blöcke (Regel Z3).
- D-B1a: Eine Schulwoche ist eine Kalenderwoche von Montag bis Freitag mit mindestens einem Schultag. Wochen, die ganz in unterrichtsfreie Zeiten fallen, sind keine Schulwochen.
- D-B1b: Bei einem Stundenplan mit A/B-Wechsel zählen nur Schulwochen. Die Schulwoche des Ankerdatums ist eine A-Woche, die nächste Schulwoche eine B-Woche, dann wieder A. Ferien unterbrechen die Zählung, verschieben sie aber nicht: Nach den Ferien folgt die Woche, die vorher nicht dran war.
- D-B2: Aufeinanderfolgende Stunden derselben Lerngruppe am selben Tag ergeben einen Block mit entsprechender Dauer (Z4).
- D-B3: Ändern sich Stundenplan oder unterrichtsfreie Zeiten, werden nur Blöcke neu erzeugt, die in der Zukunft liegen, den Status `planned` haben und den Ursprung `generated`.
- D-B4: Blöcke mit Status `held` oder `cancelled`, Blöcke mit Ursprung `manual` und Blöcke, an denen Daten hängen, werden nie automatisch geändert oder gelöscht. Passt ein solcher Block nicht mehr zum Stundenplan, zeigt florg das als Hinweis an.
- D-B5: Teilen eines Blocks (Z5) verkürzt den bestehenden Block und legt für den Rest einen neuen an. Beide erhalten den Ursprung `manual`.
- D-B6: Das Stundenbudget einer Lerngruppe ist die Summe von `duration` über alle Blöcke, deren Status nicht `cancelled` ist (Z7). Es wird berechnet und nicht gespeichert.

## Bewusst weggelassen in Stufe 1

- **Schule als eigenes Objekt.** Eine Instanz bildet eine Schule ab. Wer an mehreren Schulen unterrichtet, betreibt mehrere Instanzen.
- **Gerätebezeichnung an der Sitzung.** "Überall abmelden" genügt.
- **Uhrzeiten der Stunden.** Stunden sind nummeriert. Uhrzeiten können später als eigene kleine Tabelle dazukommen, ohne bestehende Tabellen zu ändern.
- **Samstage.** Nicht vorgesehen.
- **Wechsel mit mehr als zwei Wochen.** `cycle_length` lässt nur 1 und 2 zu.
- **Passwörter.** Es gibt keine, also auch kein Zurücksetzen.
- **SMTP-Zugangsdaten.** Sie stehen in der Konfiguration auf dem Server und nicht in der Datenbank.

## Offene Punkte zu Stufe 1

1. Format der Ferien-Datendatei. Es wird mit der Umsetzung von Stufe 1 festgelegt.
2. Zählt eine Woche mit nur einem oder zwei Schultagen als Schulwoche für den A/B-Wechsel? Regel D-B1a sagt ja. Das ist an der Schule zu prüfen.
