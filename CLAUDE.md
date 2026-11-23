# florg – Projektanweisungen

florg ("Fachlehrer Organizer") ist eine Webanwendung für bayerische Fachlehrer. Das fachliche Regelwerk steht in `docs/konzept.md`, das Datenmodell in `docs/datenmodell.md`. Lies beide, bevor du etwas planst oder änderst.

## Arbeitsregeln

Diese Regeln gelten in jeder Sitzung und haben Vorrang vor Tempo.

1. **Entscheidungen hinterfragen.** Prüfe jede Entscheidung des Auftraggebers, bevor du sie umsetzt: Macht sie die Lösung unnötig kompliziert? Drohen Doppelentwicklungen nebeneinander? Widerspricht sie einer Regel in `docs/konzept.md`? Benenne Bedenken offen und schlage eine einfachere Alternative vor.
2. **Nie einfach losbauen.** Stelle geplante Änderungen zuerst vor: was geändert wird, warum, und welche Dateien betroffen sind. Setze erst nach ausdrücklicher Zustimmung um.
3. **Pull Requests.** Jede Änderung kommt auf einen eigenen Branch und als Pull Request. Der Auftraggeber testet und merged selbst. Schreibe nie direkt auf `main` und merge nie selbst.
4. **Konzept vor Code.** `docs/konzept.md` ist die Grundlage. Weicht eine Umsetzung von einer Regel ab, wird zuerst das Konzept geändert, dann der Code.

## Technik

- Backend: Rust mit SQLite über eine Zwischenschicht. Nur verwenden, was auch PostgreSQL kann (siehe `docs/datenmodell.md`, Grundsätze).
- Frontend: Vue.js mit Pinia und Router, Tailwind CSS
- Mobile first, kein Offline-Betrieb, eine Instanz pro Lehrer
- Sicherheit hat Vorrang vor Komfort und Tempo. Anmeldung nur per E-Mail-Link (`docs/konzept.md`, Regeln L1 bis L13). Kryptografie, Zufallszahlen, Hashing und Sitzungsverwaltung nur aus etablierten Bibliotheken, nie selbst schreiben.

## Sprache

- Code, Bezeichner und Kommentare: Englisch.
- Dokumentation, Oberfläche der Anwendung, Commit-Messages, Pull Requests und Kommunikation: Deutsch.
- Bezeichner für Fachbegriffe folgen dem Glossar in `docs/konzept.md`, Abschnitt 2. Neue Fachbegriffe werden zuerst dort eingetragen.
- Die Anwendung ist einsprachig deutsch. Keine Mehrsprachigkeit einbauen.
