# florg

florg ("Fachlehrer Organizer") unterstützt bayerische Fachlehrer bei Jahresplanung, Wochenplanung, Unterrichtsnachweis, Schülerbeobachtung, Zeugnisbemerkungen, Noten und Materialkosten.

Das Projekt ist im Aufbau. Bisher gibt es das Grundgerüst, die Anmeldung, Schuljahre mit Ferien und Feiertagen, die Listen der Fächer und Klassen sowie die Schüler je Schuljahr mit CSV-Import.

- Konzept und Regeln: [`docs/konzept.md`](docs/konzept.md)
- Datenmodell: [`docs/datenmodell.md`](docs/datenmodell.md)

## Aufbau

| Verzeichnis | Inhalt |
| --- | --- |
| `backend/` | Rust: axum, SeaORM, SQLite |
| `frontend/` | Vue 3 (JavaScript): Vite, Pinia, Vue Router, Tailwind CSS |
| `docs/` | Konzept und Datenmodell |

## Voraussetzungen

- Rust (stable) mit `cargo`
- Node.js 22 oder neuer mit `npm`

Unter Arch Linux: `sudo pacman -S rust nodejs npm`

## Entwicklung

Backend und Frontend laufen in zwei Terminals.

Terminal 1, Backend auf http://127.0.0.1:3000:

```sh
cd backend
cargo run
```

Beim ersten Start entsteht die Datenbank `backend/florg.db`.

Terminal 2, Frontend auf http://localhost:5173:

```sh
cd frontend
npm ci
npm run dev
```

`npm ci` installiert genau die Versionen aus `package-lock.json` und ändert die Datei nicht. Nach jedem `git pull` mit neuen Abhängigkeiten genügt ein erneutes `npm ci`.

Öffne http://localhost:5173. Anfragen an `/api` leitet der Entwicklungsserver an das Backend weiter.

### Konto anlegen und anmelden

Jede Instanz hat genau ein Konto. Lege es einmal an:

```sh
cd backend
cargo run -- account set --email du@example.org
```

Ohne eingerichteten Mailversand verschickt florg keine Mail. Der Anmeldelink steht dann im Terminal des Backends:

1. Gib auf der Anmeldeseite die Adresse des Kontos ein und tippe auf "Anmeldelink anfordern".
2. Kopiere den Link aus dem Terminal des Backends in denselben Browser.
3. Bestätige mit "Jetzt anmelden".

Der Link gilt 15 Minuten, nur einmal und nur in dem Browser, der ihn angefordert hat.

Die Anmeldeseite antwortet immer gleich, auch bei falscher Adresse. Kommt kein Link, steht der Grund im Terminal des Backends.

## Wie im Betrieb starten

Im Betrieb liefert das Backend das gebaute Frontend selbst aus. Es läuft also nur ein Programm.

```sh
cd frontend
npm ci
npm run build
cd ../backend
cargo run --release
```

Öffne http://127.0.0.1:3000. Damit die Anmeldelinks auf diese Adresse zeigen, setze vorher in `backend/.env` den Wert `FLORG_PUBLIC_URL=http://127.0.0.1:3000`.

Auf einem Server gehört ein Reverse-Proxy mit HTTPS vor florg. `FLORG_PUBLIC_URL` ist dann die HTTPS-Adresse.

## Befehle

Das Backend kennt neben dem Serverstart zwei Befehle. In der Entwicklung stehen sie hinter `cargo run --`.

| Befehl | Wirkung |
| --- | --- |
| `florg` | Startet den Server |
| `florg account set --email <Adresse>` | Legt das Konto an oder ändert dessen Adresse. Eine Änderung beendet alle Sitzungen |
| `florg mail test` | Verschickt eine Testmail an die Adresse des Kontos |
| `florg demo-data` | Füllt eine leere Datenbank mit Beispieldaten. Nur für Entwicklung und Test |

## Beispieldaten

Zum Ausprobieren und Testen füllt ein Befehl die Datenbank mit Beispieldaten: das Schuljahr nach dem laufenden mit Ferien und Feiertagen, vier Fächer, acht Klassen und 140 Schüler mit erfundenen Namen. Jeder Lauf erzeugt dieselben Daten.

```sh
cd backend
cargo run -- demo-data
```

Enthält die Datenbank schon Fachdaten, bricht der Befehl ab. Mit `--reset` löscht er sie vorher:

```sh
cargo run -- demo-data --reset
```

- `--reset` löscht Schuljahre, unterrichtsfreie Zeiten, Fächer, Klassen und Schüler. Konto und Sitzungen bleiben erhalten, du bleibst also angemeldet.
- `--reset` ist gesperrt, solange SMTP-Werte gesetzt sind. Das schützt eine Instanz im Betrieb vor einem Versehen.
- Das Backend darf dabei weiterlaufen. Lade danach die Seite im Browser neu.

## Konfiguration

Das Backend liest Umgebungsvariablen, wahlweise aus `backend/.env`. Die Vorlage mit allen Werten ist [`backend/.env.example`](backend/.env.example).

Für den Mailversand setzt du die SMTP-Werte aus der Vorlage und prüfst sie mit `florg mail test`. Fehlen sie, schreibt florg den Anmeldelink ins Server-Protokoll. Unter systemd liest du es mit `journalctl`. Unvollständige SMTP-Werte verhindern den Start.

Das Protokoll zeigt Meldungen ab der Stufe `info`. Mit der Umgebungsvariable `RUST_LOG` lässt sich das ändern, zum Beispiel `RUST_LOG=debug`.

## Ferien und Feiertage

Beim Anlegen eines Schuljahres trägt florg Ferien und Feiertage ein.

- Feiertage werden berechnet (`backend/src/calendar.rs`).
- Ferien stehen in [`backend/data/school_holidays_bavaria.json`](backend/data/school_holidays_bavaria.json). Die Datei stammt aus der amtlichen Kalenderdatei des bayerischen Kultusministeriums und nennt Quelle und Abrufdatum. Sie reicht bis zum Schuljahr 2029/30 und wird in das Programm eingebaut.

Erscheint eine neue Ferienordnung, werden die neuen Schuljahre in dieser Datei ergänzt.

## Schüler als CSV importieren

Auf der Seite "Schüler" eines Schuljahres liest "CSV importieren" eine Liste ein, aus einer Datei oder als eingefügter Text.

```csv
Vorname;Nachname;Klasse;Anrede
Anna;Huber;7a;sie
Ben;Maier;7b;er
Kim;Lang;7b;
```

- Die erste Zeile nennt die Spalten. Pflicht ist nur `Vorname`. Die Reihenfolge ist frei, unbekannte Spalten werden übergangen.
- `Anrede` kennt "er", "sie" und leer (nur Vorname). Statt `Anrede` geht auch `Geschlecht` mit "m", "w" und "d". Gespeichert wird nur die Anrede.
- Trennzeichen ist Semikolon, Komma oder Tabulator. Dateien aus Excel funktionieren ebenso wie kopierte Zellen.
- Klassen, die es noch nicht gibt, werden angelegt.
- Vor dem Anlegen zeigt florg eine Vorschau. Schon vorhandene Schüler werden übersprungen. Solange eine Zeile fehlerhaft ist, wird nichts angelegt.

Die genauen Regeln stehen in [`docs/datenmodell.md`](docs/datenmodell.md), D-I1 bis D-I8.

## Prüfungen

Dieselben Prüfungen laufen bei jedem Pull Request automatisch.

```sh
cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

```sh
cd frontend
npm run lint
npm run build
```
