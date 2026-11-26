# florg – Projektanweisungen

florg ("Fachlehrer Organizer") ist eine Webanwendung für bayerische Fachlehrer. Das fachliche Regelwerk steht in `docs/konzept.md`, das Datenmodell in `docs/datenmodell.md`. Lies beide, bevor du etwas planst oder änderst.

## Arbeitsregeln

Diese Regeln gelten in jeder Sitzung und haben Vorrang vor Tempo.

1. **Entscheidungen hinterfragen.** Prüfe jede Entscheidung des Auftraggebers, bevor du sie umsetzt: Macht sie die Lösung unnötig kompliziert? Drohen Doppelentwicklungen nebeneinander? Widerspricht sie einer Regel in `docs/konzept.md`? Benenne Bedenken offen und schlage eine einfachere Alternative vor.
2. **Nie einfach losbauen.** Stelle geplante Änderungen zuerst vor: was geändert wird, warum, und welche Dateien betroffen sind. Setze erst nach ausdrücklicher Zustimmung um.
3. **Pull Requests.** Jede Änderung kommt auf einen eigenen Branch und als Pull Request. Der Auftraggeber testet und merged selbst. Schreibe nie direkt auf `main` und merge nie selbst. Nenne die Testschritte am Ende jeder Abschlussmeldung im Chat, auch wenn sie in der Beschreibung des Pull Requests stehen.
4. **Konzept vor Code.** `docs/konzept.md` ist die Grundlage. Weicht eine Umsetzung von einer Regel ab, wird zuerst das Konzept geändert, dann der Code.

## Technik

- Backend (`backend/`): Rust mit axum und SQLite über SeaORM. Nur verwenden, was auch PostgreSQL kann (siehe `docs/datenmodell.md`, Grundsätze). Schemaänderungen nur als neue Migration in `backend/src/migration/`.
- Kein handgeschriebenes SQL außerhalb von Migrationen und Verbindungsaufbau; Abfragen laufen über SeaORM.
- Neue API-Endpunkte brauchen eine Sitzung: Sie kommen in `backend/src/routes.rs` zu den geschützten Routen (`protected`). Offen bleiben nur `/api/health` und die Anmeldung selbst.
- Frontend (`frontend/`): Vue 3 in JavaScript (kein TypeScript) mit Pinia, Vue Router und Tailwind CSS, gebaut mit Vite.
- Bausteine mit Verhalten (Modal, Menü, Auswahl, Ziehen und Ablegen) kommen aus Reka UI und werden nicht selbst geschrieben. Seiten verwenden dafür die eigenen Komponenten in `frontend/src/components/` (z. B. `ModalDialog`, `ConfirmDialog`), nicht Reka UI direkt.
- Formulare zum Anlegen und Ändern sowie Rückfragen vor dem Löschen öffnen sich als Modal. Keine Rückfragen über `window.confirm`.
- Desktop zuerst: Planung und Verwaltung nutzen die Breite großer Bildschirme. Die Erfassung im Unterricht muss am Handy gut bedienbar sein, alle Seiten bleiben am Handy benutzbar.
- Oberfläche zuerst in Desktopbreite prüfen und zeigen, zusätzlich in Handybreite.
- Kein Offline-Betrieb, eine Instanz pro Lehrer
- Sicherheit hat Vorrang vor Komfort und Tempo. Anmeldung nur per E-Mail-Link (`docs/konzept.md`, Regeln L1 bis L13). Kryptografie, Zufallszahlen, Hashing, Cookies und Mailversand nur aus etablierten Bibliotheken, nie selbst schreiben (Regel L13).

## Beispieldaten

- `florg demo-data` füllt eine leere Datenbank mit Beispieldaten, `florg demo-data --reset` ersetzt vorhandene Fachdaten (`docs/konzept.md`, E1 bis E4). In der Entwicklung: `cargo run -- demo-data --reset`.
- Jeder Pull Request, der neue Fachdaten einführt, erweitert `backend/src/demo_data.rs` um passende Beispieldaten und das Löschen bei `--reset`.
- Testschritte gehen vom Stand nach `demo-data --reset` aus und nennen konkrete Einträge daraus.

## Prüfungen vor jedem Pull Request

- Backend: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
- Frontend: `npm run lint`, `npm run build`
- Abhängigkeiten des Frontends mit `npm ci` installieren, nicht mit `npm install`. `npm install` nur, um eine Abhängigkeit hinzuzufügen oder zu ändern.

## Sprache

- Code, Bezeichner und Kommentare: Englisch.
- Dokumentation, Oberfläche der Anwendung, Commit-Messages, Pull Requests und Kommunikation: Deutsch.
- Bezeichner für Fachbegriffe folgen dem Glossar in `docs/konzept.md`, Abschnitt 2. Neue Fachbegriffe werden zuerst dort eingetragen.
- Die Anwendung ist einsprachig deutsch. Keine Mehrsprachigkeit einbauen.
