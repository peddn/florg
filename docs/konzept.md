# florg – Konzept und Regelwerk

Stand: 26.11.2026

florg ("Fachlehrer Organizer") unterstützt bayerische Fachlehrer bei Jahresplanung, Wochenplanung, Unterrichtsnachweis, Schülerbeobachtung, kompetenzorientierten Zeugnisbemerkungen, Noten und Materialkosten. Dieses Dokument hält Konzept und Regeln fest. Code entsteht erst, wenn das Regelwerk steht.

Der Hauptnutzen ist Zeitersparnis: Der Unterrichtsnachweis entsteht aus der Planung, und der Stundenabschluss dauert höchstens etwa eine Minute.

## 1 Rahmenentscheidungen

| Thema | Entscheidung |
| --- | --- |
| Zielgruppe | Fachlehrer an Förderschule, Mittelschule und Realschule; generischer Ansatz, nicht auf ein Fach festgelegt |
| Start | Förderschwerpunkt Lernen, Technik in Jahrgangsstufe 7 |
| Lizenz | Open Source |
| Betrieb | Eine Instanz pro Lehrer und Schule auf einem privat gemieteten Server; ein Konto, keine Rollen. Wer an mehreren Schulen unterrichtet, betreibt mehrere Instanzen |
| Anmeldung | Nur per Link an die hinterlegte E-Mail-Adresse, kein Passwort |
| Sicherheit | Hat Vorrang vor Komfort und Tempo |
| Geräte | Hauptnutzung am Desktop. Die Erfassung im Unterricht (Anwesenheit, Beobachtung) muss am Handy gut bedienbar sein |
| Offline | Nicht nötig |
| Backend | Rust mit axum; SQLite über SeaORM als Zwischenschicht, ein späterer Wechsel zu PostgreSQL bleibt möglich |
| Frontend | Vue.js in JavaScript (kein TypeScript) mit Pinia und Router, Tailwind CSS, gebaut mit Vite. Bausteine mit Verhalten (Modal, Menü, Auswahl) kommen aus Reka UI, das Aussehen aus Tailwind |
| Auslieferung | Das Backend liefert das gebaute Frontend aus; im Betrieb läuft ein Programm |
| Prüfung | GitHub Actions bauen, testen und linten jeden Pull Request |
| Mengengerüst | 8 bis 10 Gruppen, je bis 16 Schüler (mehr möglich), etwa 130 bis 160 Schüler je Instanz |
| Ausgaben | Zeugnisbemerkungen, Noten, Leistungsübersichten, Kostenkalkulationen |

Die Module bauen aufeinander auf: Zeitgerüst → Jahresplanung → Wochenplanung und Unterrichtsnachweis → Beobachtung → Zeugnisbemerkungen. Noten und Kosten hängen an den Lerneinheiten der Jahresplanung.

## 2 Begriffe

| Begriff | Im Code | Bedeutung |
| --- | --- | --- |
| Schuljahr | `school_year` | Zeitraum mit Schultagen, Ferien und Feiertagen |
| Lerngruppe | `learning_group` | Die Schüler, die gemeinsam unterrichtet werden. Nicht gleich Klasse: Gruppen können geteilt oder klassenübergreifend sein |
| Stundenplan | `timetable` | Wochenraster je Lerngruppe mit Gültigkeitszeitraum |
| Unterrichtsblock | `lesson_block` | Aufeinanderfolgende Stunden einer Gruppe an einem Tag. Kleinste Einheit für Nachweis und Bewertung |
| Katalog | `catalog` | Ein Lehrplan in einer bestimmten Version, als Baum aus Abschnitten und Kompetenzen |
| Kompetenz | `competency` | Eine Kompetenzerwartung aus einem Katalog oder eine selbst angelegte |
| Abschnitt | `section` | Knoten im Katalog, der Kompetenzen bündelt (z. B. "1.5.2 Arbeiten"). Zugleich Satzgruppe fürs Zeugnis |
| Inhalt | `content_item` | Stoff, Material oder Verfahren aus einem Lehrplan (z. B. Bohren, Massivholz). Wird nicht bewertet |
| Themenbereich | `topic_area` | Eigene Gliederung des Jahres, z. B. nach Material |
| Lerneinheit | `learning_unit` | Werkstück, Kurs oder Themenschwerpunkt innerhalb eines Themenbereichs |
| Beobachtung | `observation` | Sternebewertung (1 bis 5) eines Schülers zu einer Kompetenz in einem Block |
| Notiz | `note` | Freitext zu einem Schüler, optional an Block oder Kompetenz gebunden |
| Zeugnisart | `report_type` | Konfigurierbare Art von Zeugnis mit Zeitraum und Zielzahl an Sätzen |
| Satzbaustein | `phrase` | Satz zu einem Abschnitt in einer Stufe für eine Zeugnisart |
| Leistungsnachweis | `assessment` | Benotete Leistung eines frei definierbaren Typs |
| Rohmaterial | `material` | Material mit Einheit und Preis pro Einheit |
| Stückliste | `bill_of_materials` | Materialmengen je Schüler für ein Werkstück |
| Schüler | `student` | Mitglied einer oder mehrerer Lerngruppen |
| Note | `grade` | Ganze Note von 1 bis 6 |
| Kriterienraster | `rubric` | Teilkriterien mit Punkten für einen Leistungsnachweis |
| Konto | `account` | Das eine Benutzerkonto der Instanz |
| Anmelde-Token | `login_token` | Einmaliger, kurz gültiger Schlüssel in einem Anmeldelink |
| Sitzung | `session` | Angemeldeter Zustand auf einem Gerät |
| Unterrichtsfreie Zeit | `school_free_period` | Ferien, Feiertag oder sonstiger freier Tag |
| Fach | `subject` | Unterrichtsfach einer Lerngruppe |
| Klasse | `school_class` | Bezeichnung einer Klasse, z. B. 7a. Gilt über Schuljahre hinweg und wird einem Schüler je Schuljahr zugeordnet |
| Schuljahreseintrag | `enrollment` | Ein Schüler in einem Schuljahr, optional mit Klasse |
| Stundenplaneintrag | `timetable_entry` | Eine Stunde im Wochenraster eines Stundenplans |
| Unterrichtsnachweis | `lesson_record` | Dokumentation eines gehaltenen Unterrichtsblocks |
| Fehlzeit | `absence` | Abwesenheit eines Schülers in einem Unterrichtsblock |
| Zeugnistext | `report_text` | Fertiger Fließtext je Schüler, Fach und Zeugnisart |
| Nachweis-Typ | `assessment_type` | Frei anlegbare Art von Leistungsnachweis |
| Notenschlüssel | `grading_scale` | Umrechnung von Punkten in Noten |

Die Spalte "Im Code" ist verbindlich. Bezeichner im Code folgen diesem Glossar. Neue Fachbegriffe werden zuerst hier eingetragen.

## 3 Kompetenzkatalog

### 3.1 Quellen

| Katalog | Fassung | Verwendung |
| --- | --- | --- |
| Förderschule Lernen: Berufs- und Lebensorientierung – Praxis 7–9 | gültig ab Schuljahr 2026/27 | Kompetenzen für den Start |
| Förderschule Lernen: BLO – Praxis Technik 7–9 | gültig bis Schuljahr 2025/26 | Vorgängerfassung, nur zur Referenz |
| Mittelschule: Technik M7, M8, M9 | LehrplanPLUS | Inhalte (Material, Verfahren) und zusätzliche Kompetenzen |

Alle drei stammen aus der PDF-Sammlung von lehrplanplus.bayern.de (Abruf 23.11.2026).

### 3.2 Regeln

- K1: Ein Katalog ist ein Baum beliebiger Tiefe. Die Lehrpläne sind mal zwei-, mal dreistufig gegliedert.
- K2: Jeder Katalog hat eine Version mit Gültigkeitszeitraum. Eine Lerngruppe ist je Schuljahr an genau eine Version gebunden.
- K3: Jede Kompetenz hat eine feste ID aus Katalog, Abschnitt und Position. Der Text ist kein Schlüssel, weil derselbe Wortlaut mehrfach vorkommt (z. B. "dokumentieren den Arbeitsprozess" in 1.4.3 und 1.5.3).
- K4: Kompetenzen sind nicht an eine Jahrgangsstufe gebunden. Der Lehrplan beschreibt das Ergebnis eines dreijährigen Lernprozesses.
- K5: Je Lerngruppe und Schuljahr wählt der Lehrer aus, welche Kompetenzen angestrebt werden. Abgewählte Kompetenzen erscheinen nirgends.
- K6: Entwicklungsbezogene Kompetenzen (Motorik und Wahrnehmung, Denken und Lernstrategien, Kommunikation und Sprache, Emotionen und soziales Handeln) bewertet die Klassleitung. Sie sind nicht Teil des Startkatalogs und werden nicht bewertet.
- K7: Eigene Kompetenzen und eigene Abschnitte können angelegt werden.
- K8: Kataloge werden als Datei importiert und stehen nicht fest im Code.
- K9: Inhalte aus dem Mittelschul-Lehrplan sind eine eigene Ebene. Themenbereiche und Lerneinheiten können auf sie verweisen. Bewertet werden nur Kompetenzen.
- K10: Eine Lerneinheit kann Kompetenzen aus mehreren Katalogen mischen.
- K11 (später): Kompetenzen können als verwandt verknüpft werden, damit eine Beobachtung nicht mehrfach erfasst werden muss.

### 3.3 Startkatalog Technik, Jahrgangsstufe 7

60 fachliche Kompetenzen in 13 Abschnitten aus BLO-Praxis (Fassung ab 2026/27):

| Bereich | Abschnitt | Kompetenzen |
| --- | --- | --- |
| 1.2 Verantwortungsvoll arbeiten | 1.2.1 Gesund leben, sicher arbeiten | 6 |
| | 1.2.2 Zusammen arbeiten | 5 |
| | 1.2.3 Nachhaltig agieren | 4 |
| 1.3 Fachspezifisch arbeiten | 1.3.1 Skizzieren und zeichnen | 4 |
| | 1.3.2 Ein Vorhaben umsetzen | 6 |
| | 1.3.3 Mit Medien arbeiten | 6 |
| 1.5 Technik vertiefen | 1.5.1 Planen | 5 |
| | 1.5.2 Arbeiten | 3 |
| | 1.5.3 Dokumentieren, Präsentieren und Reflektieren | 4 |
| 5 Methodenkompetenzen | 5.1 Fachspezifische Methoden gewinnen und anwenden | 9 |
| | 5.2 Sich über Arbeitsprozesse und Arbeitsergebnisse austauschen | 3 |
| | 5.3 Über Arbeitsprozesse und Ergebnisse reflektieren | 2 |
| | 5.4 Eignung für Berufsfelder einschätzen | 3 |

Nicht im Startkatalog: 1.1 und alle weiteren entwicklungsbezogenen Abschnitte, 1.4 Ernährung und Soziales, Lernbereich 2 (Berufsfelderkundung, ab Jahrgangsstufe 8), Lernbereich 3 (Projekt) und Lernbereich 4 (Berufswahlentscheidung).

### 3.4 Ausblick ab Jahrgangsstufe 8

- Lernbereich 2 enthält 14 Berufsfeld-Module, darunter Bautechnik, Elektrotechnik, Fahrzeugtechnik, Farbtechnik und Raumgestaltung, Holztechnik sowie Metalltechnik und Recycling.
- Jedes Modul hat zwei Abschnitte: "Grundlagen und Begriffe" und "Berufsspezifisch arbeiten". Ein Modul wird als Lerneinheit vom Typ Kurs abgebildet und bringt zwei Satzgruppen mit.
- Der Lehrplan verlangt pro Schuljahr mindestens ein Modul aus Bereich A und eines aus Bereich B. Das betrifft die Schule, nicht den einzelnen Lehrer. florg zeigt die Kennzeichnung A oder B an und prüft nichts.

## 4 Modul Zeitgerüst

- Z1: Ein Schuljahr hat Beginn, Ende, Ferien und Feiertage. Daraus ergeben sich die Schultage.
- Z1a: Feiertage werden berechnet. Ferien kommen aus einer mitgelieferten Datendatei auf Grundlage der amtlichen Ferienordnung (derzeit bis Schuljahr 2029/30). Beim Anlegen eines Schuljahres füllt florg beides vor, alles bleibt von Hand änderbar.
- Z2: Der Stundenplan ordnet Wochentag und Stunde einer Lerngruppe und einem Fach zu. Er hat einen Gültigkeitszeitraum, weil er sich unterjährig ändern kann.
- Z2a: Ein Stundenplan kann im Wechsel von A- und B-Woche laufen. Gezählt werden nur Schulwochen, also Wochen mit mindestens einem Schultag.
- Z3: Aus Schultagen und Stundenplan erzeugt florg alle Unterrichtsblöcke des Schuljahres.
- Z4: Aufeinanderfolgende Stunden derselben Gruppe am selben Tag bilden automatisch einen Block. Ein Block hat eine Dauer in Unterrichtsstunden.
- Z5: Ein Block lässt sich von Hand teilen, etwa wenn die zweite Stunde ausfällt.
- Z6: Blöcke können als Ausfall markiert werden (Krankheit, Wandertag, Praktikum), mit optionalem Grund.
- Z7: Das verfügbare Stundenbudget je Gruppe ist die Summe der Dauern aller nicht ausgefallenen Blöcke. Es rechnet sich laufend neu.
- Z8: Eine Lerngruppe besteht aus Schülern, die aus verschiedenen Klassen stammen können. Jeder Schüler hat Vorname und Anrede (er, sie oder nur Vorname). Nachname und Klasse sind optional.
- Z8a: Entscheidend ist die Zugehörigkeit zur Lerngruppe. Die Klasse ist ein Merkmal des Schülers im Schuljahr: eine Bezeichnung aus einer gepflegten Liste. Ein Schüler kann ohne Klasse in einer Lerngruppe sein.
- Z8b: Gespeichert wird die Anrede, nicht das Geschlecht. Mehr braucht florg für die Zeugnisbemerkungen nicht (S7).
- Z9: Schüler behalten ihre Historie über Schuljahre hinweg.

## 5 Modul Jahresplanung

- J1: Die Hierarchie ist Themenbereich → Lerneinheit → Unterrichtsblöcke.
- J2: Themenbereiche sind die eigene Ordnung des Lehrers, z. B. nach Material (Holz, Metall, Kunststoff, Elektro). Sie müssen nicht der Lehrplangliederung folgen.
- J3: Eine Lerneinheit hat einen Typ (Werkstück, Kurs, Themenschwerpunkt, frei erweiterbar) und ein Stundenkontingent.
- J4: Kompetenzen können dem Themenbereich und der Lerneinheit zugeordnet werden. Die Kompetenzliste einer Lerneinheit ist die Vereinigung aus beiden.
- J5: florg zeigt je Gruppe geplante gegen verfügbare Stunden.
- J6: Lerneinheiten werden in Reihenfolge auf die Blöcke der Gruppe verteilt.
- J7: Lerneinheiten sind als Vorlage speicherbar, mit Kompetenzen, Stückliste und Kriterienraster. Vorlagen lassen sich für andere Gruppen und Schuljahre wiederverwenden und als Datei austauschen.
- J8: florg zeigt, welche ausgewählten Kompetenzen im Jahr noch keiner Lerneinheit zugeordnet sind.

## 6 Modul Wochenplanung und Unterrichtsnachweis

- W1: Die Wochenansicht entsteht aus Stundenplan und Jahresplanung. Sie zeigt je Block Gruppe, Lerneinheit und geplantes Stundenthema.
- W2: Je Block kann ein Stundenthema und eine kurze Planungsnotiz hinterlegt werden.
- W3: Der Unterrichtsnachweis ist aus der Planung vorausgefüllt. Im Normalfall genügt ein Tipp auf "wie geplant gehalten".
- W4: Abweichungen werden im Nachweis editiert: anderes Thema, Ausfall, Vertretung.
- W5: Verschiebt sich ein Block oder fällt er aus, rücken die folgenden Blöcke der Lerneinheit auf Wunsch nach.
- W6: Der Nachweis enthält Datum, Gruppe, Dauer, Thema, Lerneinheit, Fehlende und optional eine Bemerkung.
- W7: Der Nachweis ist je Gruppe und Zeitraum als Liste ausgebbar.

## 7 Modul Beobachtung

- B1: Bewertet wird einmal je Block, nicht je Einzelstunde.
- B2: Zuerst wird die Anwesenheit erfasst. Fehlende Schüler erhalten keine Bewertung, auch keine implizite.
- B3: Je Block sind höchstens 3 Fokus-Kompetenzen aktiv. Die Lerneinheit liefert die Kandidaten.
- B4: Die Skala hat 5 Sterne. Der Standardwert ist 3.
- B5: Erfassung nach dem Ausnahmeprinzip: Der Lehrer tippt nur bei abweichenden Schülern auf mehr oder weniger Sterne.
- B6: Erst der Abschluss des Blocks speichert die Beobachtungen, auch die unveränderten 3 Sterne. So bleibt "unauffällig" von "nicht beobachtet" unterscheidbar.
- B7: Die Ansicht zeigt eine Kompetenz oben und darunter die Schülerliste mit Sternen. Wischen wechselt zur nächsten Kompetenz.
- B8: Notizen sind je Schüler jederzeit möglich, optional an Block oder Kompetenz gebunden. Die Spracheingabe der Handytastatur genügt.
- B9: Abgeschlossene Blöcke lassen sich nachträglich korrigieren.
- B10: Beobachtungen laufen über Schuljahre weiter und bleiben dem Schüler zugeordnet.

## 8 Modul Zeugnisbemerkungen

- S1: Satzbausteine hängen am Abschnitt, nicht an der einzelnen Kompetenz. Für den Startkatalog sind das 13 Satzgruppen.
- S2: Je Abschnitt gibt es 5 Stufen, deckungsgleich mit den Sternen.
- S3: Zeugnisarten sind konfigurierbar. Vorbelegt sind Zwischenzeugnis und Jahreszeugnis. Jede hat Namen, Beobachtungszeitraum und Zielzahl an Sätzen (vorerst beide 5 bis 6).
- S4: Jede Zeugnisart hat eigene Texte je Abschnitt und Stufe. florg wandelt keine Zeitformen automatisch um, weil die Unterschiede schulspezifisch sind.
- S5: Die Sätze einer Zeugnisart lassen sich in eine andere kopieren und dort anpassen.
- S6: Umfang für den Start: 13 Abschnitte × 5 Stufen × 2 Zeugnisarten = 130 Sätze.
- S7: Platzhalter sind Vorname, Pronomen (er/sie) und Possessivform (sein/ihr). Der erste Satz des Textes nennt den Vornamen, die folgenden nutzen das Pronomen. Bei Anrede "nur Vorname" wird immer der Vorname gesetzt.
- S8: Die Stufe eines Abschnitts ist der gerundete Durchschnitt der Sterne aller Beobachtungen zu seinen Kompetenzen im Zeitraum. Alle Beobachtungen zählen gleich, jüngere werden nicht stärker gewichtet.
- S9: Ein Abschnitt wird nur vorgeschlagen, wenn im Zeitraum mindestens 3 Beobachtungen vorliegen (einstellbar).
- S10: florg schlägt so viele Abschnitte vor, wie die Zielzahl verlangt, beginnend mit den meisten Beobachtungen. Der Lehrer kann Abschnitte tauschen und Stufen überschreiben.
- S11: Die Reihenfolge der Sätze folgt der Katalogreihenfolge und ist änderbar.
- S12: Das Ergebnis ist ein editierbarer Fließtext. Die Notizen zum Schüler stehen beim Bearbeiten daneben.
- S13: Der fertige Text wird je Schüler, Fach und Zeugnisart gespeichert und als Text kopierbar oder als Liste je Gruppe ausgegeben. florg erzeugt nicht das amtliche Zeugnisformular.
- S14: Satzsammlungen lassen sich als Datei exportieren und importieren.
- S15 (optional): Für einzelne Kompetenzen kann ein eigener Satz hinterlegt werden, der bei 1 oder 5 Sternen vorgeschlagen wird.

## 9 Modul Noten

- N1: Nachweis-Typen sind frei anlegbar (z. B. Werkstück, praktische Probe, schriftliche Probe, Mitarbeit). Jeder Typ hat Namen und Standardgewicht.
- N2: Ein Leistungsnachweis hat Gruppe, Typ, Titel, Datum und Gewicht. Das Gewicht kommt vom Typ und ist im Einzelfall änderbar.
- N3: Ein Leistungsnachweis kann mit einer Lerneinheit verknüpft sein.
- N4: Bewertungsart "Note direkt": Der Lehrer trägt eine ganze Note von 1 bis 6 ein.
- N5: Bewertungsart "Punkte": Der Nachweis hat eine Maximalpunktzahl. Ein Notenschlüssel rechnet Punkte in eine ganze Note um. Notenschlüssel sind als Vorlage speicherbar.
- N6: Kriterienraster: Ein Nachweis kann Teilkriterien mit Punkten haben (bei einem Werkstück z. B. Maßhaltigkeit, Oberfläche, Funktion, Arbeitsweise). Die Summe der Kriterien ergibt die Punktzahl. Raster sind als Vorlage speicherbar.
- N7: Es gibt nur ganze Noten, keine Tendenzen.
- N8: Der Schnitt ist das gewichtete Mittel aller Nachweise im Zeitraum, mit zwei Nachkommastellen.
- N9: Die Zeugnisnote setzt immer der Lehrer. Der Schnitt ist ein Vorschlag und wird nie automatisch übernommen.
- N10: Fehlende erhalten "nicht teilgenommen". Das zählt nicht in den Schnitt und kann nachgetragen werden.
- N11: Sterne fließen nicht in Noten ein. Sie werden bei der Notenfindung daneben angezeigt.
- N12: Der Zeitraum hängt an der Zeugnisart. Vorbelegung: Zwischenzeugnis bis zum Halbjahr, Jahreszeugnis über das ganze Schuljahr.
- N13: Ausgaben sind die Notenliste je Gruppe und die Leistungsübersicht je Schüler mit Nachweisen, Schnitt, Sterneverlauf je Abschnitt und Notizen.

## 10 Modul Kosten

- M1: Rohmaterial hat Namen, Einheit (m, m², Stück, kg, Liter), Preis pro Einheit und einen Verschnittzuschlag (ja/nein mit Prozentsatz, Standard 10 %).
- M2: Der Verschnitt ist eine Eigenschaft des Materials. Stückware wie Schrauben hat keinen, Platten- und Stangenmaterial schon.
- M3: Preise gelten je Schuljahr. Eine Preisänderung verändert alte Kalkulationen nicht.
- M4: Ein Werkstück (Lerneinheit) hat eine Stückliste: Material × Menge je Schüler.
- M5: Optional hat ein Werkstück eine Pauschale für Kleinmaterial (Leim, Schleifpapier, Öl).
- M6: Kosten je Werkstück und Schüler = Summe aus Menge × Preis × (1 + Verschnittsatz) für Materialien mit Zuschlag, sonst Menge × Preis, zuzüglich Pauschale.
- M7: Alle Auswertungen verwenden dieselbe Formel. Es gibt keine getrennte Netto-Rechnung.
- M8: Auswertungen: je Werkstück, je Schüler (Summe seiner Werkstücke), je Gruppe (Werkstückkosten × Schülerzahl), je Klasse und je Schuljahr.
- M9: Die Bedarfsliste zeigt je Material die reine Menge für das Schuljahr inklusive Verschnitt. Sie rundet nicht auf Einkaufseinheiten.
- M10: Materialgeld von Schülern wird nicht verwaltet.

## 11 Querschnittsthemen

### Anmeldung und Sicherheit

- L1: Es gibt nur einen Anmeldeweg: einen Link per E-Mail. Passwörter gibt es nicht.
- L2: Die Anmeldeseite hat ein Eingabefeld für die E-Mail-Adresse. florg verschickt nur dann einen Link, wenn die eingegebene Adresse die des Kontos ist. Groß- und Kleinschreibung spielen keine Rolle. Der Link geht immer an die im Konto hinterlegte Adresse.
- L2a: Die Antwort auf eine Anforderung ist immer dieselbe und kommt immer gleich schnell: bei passender Adresse, bei falscher Adresse, bei erreichter Grenze (L7), bei fehlendem Konto und bei einem Fehler des Mailversands. So verrät die Anmeldeseite weder die hinterlegte Adresse noch den Zustand der Instanz. Die Ursache steht im Server-Protokoll, die eingegebene Adresse nicht.
- L3: Ein Anmelde-Token ist zufällig mit mindestens 128 Bit, einmal gültig und läuft nach 15 Minuten ab.
- L4: In der Datenbank steht nur der Hash des Tokens.
- L5: Der Link funktioniert nur in dem Browser, der ihn angefordert hat.
- L6: Der Link meldet nicht direkt an, sondern führt auf eine Seite mit Bestätigungsknopf. So verbrauchen Mail-Scanner das Token nicht.
- L7: Anforderungen von Links sind begrenzt, damit niemand das Postfach flutet: höchstens 3 Links in 15 Minuten. Es zählen nur Anforderungen mit der Adresse des Kontos. Wer die Adresse nicht kennt, löst keine Mail aus und kann die Grenze nicht ausschöpfen.
- L8: Eine Sitzung gilt 30 Tage ab der letzten Nutzung. Das Sitzungs-Cookie ist `HttpOnly`, `Secure` und `SameSite`.
- L9: "Abmelden" ist der Standard, steht in der Kopfzeile und beendet nur die Sitzung des aktuellen Geräts. "Überall abmelden" ist die zweite Option, steht auf der Seite "Konto" und beendet sofort alle Sitzungen. Eine Liste einzelner Geräte gibt es nicht.
- L10: Die Anwendung ist nur über HTTPS erreichbar. Im Betrieb übernimmt das ein vorgeschalteter Reverse-Proxy; florg selbst lauscht nur lokal. Die öffentliche Adresse steht in der Konfiguration (`FLORG_PUBLIC_URL`). Anmeldelinks werden nur aus ihr gebildet, nie aus Angaben der Anfrage.
- L11: Die SMTP-Zugangsdaten stehen in einer Konfigurationsdatei oder in Umgebungsvariablen auf dem Server, nie in der Datenbank und nicht in der Oberfläche. Die Verbindung zum Mailserver ist verschlüsselt.
- L11a: Sind keine SMTP-Werte gesetzt, schreibt florg die Mail mit dem Anmeldelink ins Server-Protokoll, statt sie zu verschicken. Das dient der Entwicklung und als Notweg, wenn der Mailversand ausfällt. L5 gilt unverändert: Der Link aus dem Protokoll funktioniert nur in dem Browser, der ihn angefordert hat. florg warnt beim Start, und die Anmeldeseite weist darauf hin. Unvollständige SMTP-Werte sind ein Startfehler.
- L12: Zwei Befehle auf dem Server gehören zum Betrieb: Konto anlegen oder dessen Adresse ändern, Testmail verschicken. Eine Adressänderung beendet alle Sitzungen und verwirft offene Anmeldelinks. Einen Befehl, der einen Anmeldelink ausgibt, gibt es nicht.
- L13: Kryptografie, Zufallszahlen, Hashing, Cookies und Mailversand kommen aus etablierten Bibliotheken und werden nicht selbst geschrieben. Sitzungen liegen in der eigenen Tabelle `session`; selbst geschrieben sind dabei nur das Nachschlagen und die Ablaufprüfung.

### Entwicklung und Test

- E1: Der Befehl `florg demo-data` füllt eine leere Datenbank mit plausiblen Beispieldaten für das Schuljahr nach dem laufenden. Von selbst legt florg nie Beispieldaten an.
- E2: Die Beispieldaten sind bei jedem Lauf dieselben und entstehen über dieselben Regeln wie Eingaben in der Oberfläche. Mit jedem neuen Teil der Anwendung kommen dessen Daten dazu.
- E3: Enthält die Datenbank schon Fachdaten, bricht der Befehl ab. Mit dem Zusatz `--reset` löscht er vorher alle Fachdaten. Konto, Sitzungen und offene Anmeldelinks bleiben erhalten.
- E4: `--reset` verweigert den Dienst, wenn der Mailversand eingerichtet ist. So trifft er keine Instanz im Betrieb.

### Datenschutz

- Die Daten liegen auf einem vom Lehrer privat gemieteten Server. Ob das für Schülerdaten zulässig ist, klärt jeder Nutzer mit Schulleitung oder Datenschutzbeauftragtem. Dieses Dokument trifft dazu keine rechtliche Aussage.
- florg unterstützt datensparsames Arbeiten: Kürzel statt Klarnamen als Option, verschlüsselte Übertragung, vollständiger Export und Löschung je Schüler, Gruppe und Schuljahr.

### Import und Export

- Kataloge, Satzsammlungen, Vorlagen für Lerneinheiten, Notenschlüssel und Kriterienraster sind als Datei austauschbar.
- Schüler eines Schuljahres lassen sich als CSV einlesen, aus einer Datei oder als eingefügter Text. Vor dem Anlegen zeigt florg eine Vorschau. Die Regeln stehen in `docs/datenmodell.md` (D-I1 bis D-I8).
- Ob die Lehrplantexte des ISB im Open-Source-Repository mitgeliefert werden dürfen, ist ungeklärt. Bis zur Klärung bleibt der Katalog eine getrennte Importdatei.

### Bedienung

- Planung, Verwaltung und Zeugnisphase sind für den Desktop ausgelegt und nutzen dessen Breite, etwa mit Tabellen. Die Erfassung im Unterricht ist für das Handy ausgelegt. Alle Seiten bleiben am Handy benutzbar.
- Formulare zum Anlegen und Ändern sowie Rückfragen vor dem Löschen öffnen sich als Modal über der Seite.
- Eine Änderung wird erst angezeigt, wenn der Server sie bestätigt hat. Das gilt überall, auch bei der Erfassung im Unterricht. Solange die Anfrage läuft, zeigt das auslösende Element einen Kreisel und ist gesperrt; der Rest der Seite bleibt bedienbar. Bei einem Fehler bleibt der alte Stand stehen, und eine Meldung erscheint. Die Oberfläche zeigt nie einen Wert vorab an, den sie später zurücknehmen müsste.
- Die Statusleiste zeigt "Speichert …", solange eine Änderung unterwegs ist. Kreisel und Hinweis erscheinen erst nach kurzer Verzögerung, damit bei schneller Verbindung nichts flackert.
- Listen kommen ohne Blättern aus. Jede Liste ist durch ihren Zusammenhang begrenzt, etwa auf ein Schuljahr oder eine Lerngruppe (siehe Mengengerüst), und lässt sich filtern und durchsuchen.
- Die Kopfzeile hat zwei Zeilen. Die erste gilt allgemein: links "Schuljahre", "Fächer" und "Klassen", rechts "Konto" und "Abmelden". Die zweite gilt für das aktive Schuljahr: links dessen Auswahl, daneben die Seiten dieses Schuljahres. Gibt es noch kein Schuljahr, entfällt die zweite Zeile.
- Aktiv ist das Schuljahr, das zuletzt gewählt wurde. Ohne Wahl ist es das Schuljahr, in dem das heutige Datum liegt, sonst das jüngste.
- Eine Statusleiste am unteren Bildschirmrand zeigt rechts, ob das Backend erreichbar ist.
- Ausgaben sind als Text kopierbar und als PDF druckbar.

## 12 Nicht im Umfang

- Anmeldung mit Passwort
- Passkeys (später möglich, wenn der E-Mail-Link im Alltag stört)
- Eigener Mailserver und SMTP-Einstellungen in der Oberfläche
- Mehrbenutzerbetrieb, Rollen und Rechte
- Offline-Betrieb und Synchronisation
- Bewertung entwicklungsbezogener Kompetenzen
- Verwaltung von Materialgeld
- Notizen mit Markierung "für Klassleitung"
- Stärkere Gewichtung jüngerer Beobachtungen
- Aufrunden auf Einkaufseinheiten
- Erzeugen amtlicher Zeugnisformulare
- Noten mit Tendenzen

## 13 Offene Punkte

1. Lizenz der Lehrplantexte: Darf der Katalog mitgeliefert werden? Anfrage beim ISB nötig.
2. Mindestzahl an Beobachtungen je Abschnitt (S9): Ist 3 der richtige Standardwert?
3. Verwandte Kompetenzen (K11): nötig für den Start oder später?
4. Format der Importdateien für Kataloge und Satzsammlungen.
5. Die 130 Satzbausteine für den Startkatalog müssen geschrieben werden.
6. Welche Kompetenzen des Startkatalogs werden für die 7. Klasse tatsächlich ausgewählt?
7. Datenmodell: steht in `docs/datenmodell.md`, im Detail bisher für Stufe 1.
8. Wer die Adresse des Kontos kennt, kann die Grenze aus L7 ausschöpfen, so neue Anmeldungen blockieren und Mails auslösen; bestehende Sitzungen sind nicht betroffen. Eine Adresse, die Schüler nicht kennen, erschwert das. Ob zusätzlich bekannte Geräte ein eigenes Kontingent bekommen, ist offen.

## 14 Umsetzungsreihenfolge

Es gibt keinen Zeitdruck. Die Anwendung wird 2026 nicht produktiv genutzt. Jede Stufe ist für sich nutzbar und wird über einen oder mehrere Pull Requests umgesetzt.

| Stufe | Inhalt | Ergebnis |
| --- | --- | --- |
| 1 | Grundgerüst, Login, Schuljahr, Lerngruppen, Schüler, Stundenplan, Unterrichtsblöcke | Alle Blöcke des Jahres und das Stundenbudget sind sichtbar |
| 2 | Katalogimport, Themenbereiche, Lerneinheiten, Kompetenzzuordnung | Die Jahresplanung steht |
| 3 | Wochenansicht und Unterrichtsnachweis | Der Nachweis entsteht aus der Planung |
| 4 | Anwesenheit, Sternebewertung, Notizen | Die Beobachtungen laufen |
| 5 | Satzbausteine und Zeugnistexte | Zeugnisbemerkungen entstehen aus den Beobachtungen |
| 6 | Noten mit Kriterienraster | Die Notenverwaltung ist komplett |
| 7 | Rohmaterial, Stücklisten, Kostenauswertung | Die Kostenkalkulation ist komplett |

Das Datenmodell wird grob über alle Stufen entworfen und nur für die jeweils nächste Stufe im Detail festgelegt.

### Aufteilung von Stufe 1

Stufe 1 wäre als ein einziger Pull Request zu groß zum Testen. Sie wird deshalb in mehreren Pull Requests umgesetzt. Jeder ist für sich prüfbar.

| Nr. | Inhalt | Was danach testbar ist | Stand |
| --- | --- | --- | --- |
| A | Grundgerüst: Projektstruktur, Backend und Frontend starten, Datenbank wird angelegt | Die Seite lädt und zeigt, dass das Backend antwortet | erledigt |
| B | Anmeldung: Konto, E-Mail-Link, Sitzung, die zwei Server-Befehle | Anmelden per Link, abmelden, überall abmelden | erledigt |
| C | Schuljahr und unterrichtsfreie Zeiten, Feiertagsberechnung, Ferien-Datendatei | Schuljahr anlegen, Ferien und Feiertage sind vorgefüllt | erledigt |
| D1 | Zweite Menüzeile mit Auswahl des Schuljahres, Fächer, Klassen | Schuljahr umschalten, Fächer und Klassen pflegen | erledigt |
| D2 | Schüler mit Schuljahreseintrag, CSV-Import | Schüler eines Schuljahres pflegen und als Liste einlesen | erledigt |
| D3 | Lerngruppen mit Mitgliedern | Lerngruppen anlegen und Schüler zuordnen | offen |
| E | Stundenplan mit A/B-Wochen, Blockerzeugung, Stundenbudget | Alle Blöcke des Jahres und das Budget je Gruppe | offen |

## 15 Arbeitsregeln für die Umsetzung

Diese Regeln gelten für die Zusammenarbeit beim Programmieren.

- A1: Entscheidungen des Auftraggebers werden immer hinterfragt. Vor der Umsetzung wird geprüft, ob eine Entscheidung die Lösung unnötig verkompliziert, ob Doppelentwicklungen nebeneinander drohen oder ob sie bestehenden Regeln widerspricht. Bedenken werden offen benannt, mit einer einfacheren Alternative.
- A2: Es wird nie einfach losgebaut. Geplante Änderungen werden zuerst vorgestellt: was geändert wird, warum, und welche Dateien betroffen sind. Umgesetzt wird erst nach ausdrücklicher Zustimmung.
- A3: Gearbeitet wird mit Pull Requests. Jede Änderung kommt auf einen eigenen Branch und als Pull Request ins Repository. Der Auftraggeber testet und merged selbst. Es wird nie direkt auf den Hauptbranch geschrieben und nie selbst gemerged.
- A4: Dieses Dokument ist die Grundlage. Weicht eine Umsetzung von einer Regel ab, wird zuerst das Dokument geändert, dann der Code.
- A5: Code, Bezeichner und Kommentare sind englisch. Dokumentation, Oberfläche, Commit-Messages, Pull Requests und Kommunikation sind deutsch. Die Anwendung ist einsprachig deutsch.
