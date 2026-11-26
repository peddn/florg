# florg

florg ("Fachlehrer Organizer") unterstützt bayerische Fachlehrer bei Jahresplanung, Wochenplanung, Unterrichtsnachweis, Schülerbeobachtung, Zeugnisbemerkungen, Noten und Materialkosten.

Das Projekt ist im Aufbau. Bisher gibt es das Grundgerüst und die Anmeldung, noch ohne Fachfunktionen.

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
npm install
npm run dev
```

Öffne http://localhost:5173. Anfragen an `/api` leitet der Entwicklungsserver an das Backend weiter.

### Konto anlegen und anmelden

Jede Instanz hat genau ein Konto. Lege es einmal an:

```sh
cd backend
cargo run -- account set --email du@example.org
```

Ohne eingerichteten Mailversand verschickt florg keine Mail. Der Anmeldelink steht dann im Terminal des Backends:

1. Tippe auf der Anmeldeseite auf "Anmeldelink anfordern".
2. Kopiere den Link aus dem Terminal des Backends in denselben Browser.
3. Bestätige mit "Jetzt anmelden".

Der Link gilt 15 Minuten, nur einmal und nur in dem Browser, der ihn angefordert hat.

## Wie im Betrieb starten

Im Betrieb liefert das Backend das gebaute Frontend selbst aus. Es läuft also nur ein Programm.

```sh
cd frontend
npm install
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

## Konfiguration

Das Backend liest Umgebungsvariablen, wahlweise aus `backend/.env`. Die Vorlage mit allen Werten ist [`backend/.env.example`](backend/.env.example).

Für den Mailversand setzt du die SMTP-Werte aus der Vorlage und prüfst sie mit `florg mail test`. Fehlen sie, schreibt florg den Anmeldelink ins Server-Protokoll. Unter systemd liest du es mit `journalctl`. Unvollständige SMTP-Werte verhindern den Start.

Das Protokoll zeigt Meldungen ab der Stufe `info`. Mit der Umgebungsvariable `RUST_LOG` lässt sich das ändern, zum Beispiel `RUST_LOG=debug`.

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
