# florg

florg ("Fachlehrer Organizer") unterstützt bayerische Fachlehrer bei Jahresplanung, Wochenplanung, Unterrichtsnachweis, Schülerbeobachtung, Zeugnisbemerkungen, Noten und Materialkosten.

Das Projekt ist im Aufbau. Bisher gibt es nur das Grundgerüst ohne Fachfunktionen.

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

Öffne http://localhost:5173. Die Seite zeigt, ob das Backend erreichbar ist. Anfragen an `/api` leitet der Entwicklungsserver an das Backend weiter.

## Wie im Betrieb starten

Im Betrieb liefert das Backend das gebaute Frontend selbst aus. Es läuft also nur ein Programm.

```sh
cd frontend
npm install
npm run build
cd ../backend
cargo run --release
```

Öffne http://127.0.0.1:3000.

## Konfiguration

Das Backend liest Umgebungsvariablen, wahlweise aus `backend/.env`. Die Vorlage mit allen Werten ist [`backend/.env.example`](backend/.env.example).

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
