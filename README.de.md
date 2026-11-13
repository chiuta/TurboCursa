[English](README.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · **Deutsch** · [Magyar](README.hu.md)

# **TURBOCURSA**

Version: 1.5 · Letzte Aktualisierung: 12. November 2026

## Was es ist

TURBOCURSA ist ein Pseudo-3D-Arcade-Rennspiel in Rust, im Clean-Room-Verfahren entwickelt: eigene Engine, Grafik, Musik und eigener Name, ohne Code, Assets oder Marken eines bestehenden Spiels. Alles, was man sieht und hört, wird aus einem Seed erzeugt: Bäume, Felsen, Wolken, Nebel, Polarlichter und Mandelbrot-Banner an den Kontrollpunkten sowie prozedurale Musik und Motorgeräusche. Die Kernbibliothek hat keine Laufzeitabhängigkeiten.

Neunzehn erzeugte Landschaften: Wald, Wüste, Abenddämmerung, kolossale Weltenbäume mit Kronen in den Wolken, rote geschichtete Tafelberge, schwebende Inseln mit Wasserfällen und ein Ringplanet, ein Spiegelsee unter zwei Monden, eine Tundra mit Polarlicht und Schnee, ein Vulkan mit Lavaadern, fraktaler Rauchsäule und rekursiven Blitzen, ein sonnendurchfluteter Abgrund mit Korallen und Tang, ein Kosmos mit Julia-Nebel und Regenbogenstraße, eine Neonstadt aus Sierpinski-Pyramiden und Menger-Blöcken sowie ein leuchtender Myzel-Dschungel. Jede Etappe verkettet eine klassische und drei neuere Landschaften in einer vom Seed bestimmten Reihenfolge. Sechs weitere Welten, mögliche und unmögliche: eine umgekehrte Welt mit einer zweiten Bergkette, die vom Himmel hängt, eine Prismenwüste unter spektralen Polarlichtbändern, eine Uhrwerkwelt aus riesigen Zahnrädern, ein Wolkenmeer, ein Schwarzes Loch mit gravitationsverzerrtem Akkretionsring und ein fraktaler Canyon aus gebänderten Schichten.

## Verwendung

- Nativ: `cargo run --release --features native --bin turbo` (mit `--versus` für 2 Spieler, `--daily` für die Tagesetappe, `--seed N` für eine feste Etappe, `--lang xx` für die Sprache).
- Web: `./build_web.sh` erzeugt eine einzelne Offline-Datei `dist/turbocursa.html` (Touch-Steuerung, Sprachauswahl, persönliche Bestleistung). Parameter: `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Steuerung: Pfeiltasten oder WASD, Leertaste = Turbo, Enter = neu starten, M = stumm. Spieler 2: WASD + linke Umschalttaste.
- Oberfläche und Dokumentation: English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. Der Spieltext nutzt eine Pixelschrift; Akzente werden als Zeichen über dem Buchstaben gezeichnet.
- Modi: `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (Rennen gegen einen Computergegner), `--versus` (2 Spieler); Web: `?mode=NAME`. · `--career` (Divisionen, Meisterschaften und Pokale, mit Speicherstand)
- Demo: `--demo` lässt einen KI-Piloten alle Modi auf allen Schwierigkeitsstufen fahren, Etappe für Etappe (das Demo-Auto ist geschützt und verbraucht keinen Kraftstoff; nichts wird gespeichert)

## Spielmodi

- **Kampagne** — Schlage die Uhr durch vier erzeugte Landschaften; Kontrollpunkte geben Zeit.
- **Zeitfahren** — Kein Verkehr, die Uhr zählt aufwärts: so schnell wie möglich ins Ziel.
- **Endlos** — Endlose Runden: jede Runde ist eine neu erzeugte Etappe, +25 s.
- **Stau** — Dreifacher Verkehr; jedes Überholen bringt 0,8 Sekunden.
- **Orbenjagd** — Folge fraktalen Orbenspuren; jede bringt Zeit und Kombo.
- **Ringslalom** — Fahre durch die Ringe: jeder Treffer erhöht den Kombo-Faktor, ein Fehler kostet eine Sekunde.
- **Umkehr** — Die Lenkung kehrt sich sechs von vierzehn Sekunden um.
- **Dunkelheit** — Dunkelheit: nur der Scheinwerferkegel ist sichtbar, die Landschaft ist ein Gerücht.
- **Zen** — Keine Uhr, kein Verkehr, keine Unfälle: einfach durch endlose Landschaften fahren.
- **Gegen CPU** — Fahre gegen einen computergesteuerten Rivalen auf derselben Etappe, geteilter Bildschirm.
- **2 Spieler** — Zwei Menschen, eine Etappe, geteilter Bildschirm: Wer zuerst im Ziel ist, gewinnt.

## Karriere

Start mit `--career` (Web: `?mode=career`). Der Fortschritt wird automatisch gespeichert (Datei `~/.turbocursa_career` nativ, `localStorage` im Browser).
- **Divisionen** — Bronze, Silber, Gold, Platin, Diamant. Jede Saison ist eine Meisterschaft über sechs Spieltage mit 12 Fahrern: die ersten drei steigen auf, die letzten drei ab; der Diamant-Meister erhält eine Wildcard für den Weltpokal.
- **Pokale mit K.-o.-Runden** — Deine Zeit wird mit simulierten Rivalen verglichen, die mit Pokal und Division stärker werden; die Langsamsten scheiden in jeder Runde aus (16 → 8 → 4 → Finale im Nationalpokal; bis zu 32 → 24 → 16 → 8 → 4 → Finale im Weltpokal).
- **Mehrfache Qualifikation** — Zwei *Hoffnungslauf*-Chancen pro Pokal: Wer ausscheidet, fährt mit den besten Ausgeschiedenen noch einmal, die ersten zwei kehren zurück.
- **Die Leiter** — Erreiche das Finale des **Nationalpokals**, um dich für den **Kontinentalpokal** zu qualifizieren; dessen Finale öffnet den **Interkontinentalpokal**; dessen Finale den **Weltpokal**.
- **Jedes Land** — 196 Länder auf sechs Kontinenten; du kannst auch am Nationalpokal jedes anderen Landes teilnehmen (Auslandspokale) und sammelst für jedes eine Trophäe im Trophäenschrank.
- **Ranglisten** — eine Weltrangliste mit 300 Fahrern plus dir, eine Nationenwertung, die Tabelle der Meisterschaft und deine Statistik.

## Autos je Land

Jedes Land hat sein eigenes Rennauto: eine Lackierung in den Landesfarben (angenähert, eigene Grafik) mit Kennzeichen und ein Fahrprofil (Höchsttempo, Beschleunigung, Haftung, 2-4 Turbo-Ladungen). Das Profil ist ein Nullsummen-Budget: Kein Land hat ein in allem besseres Auto. Auswahl mit `--car RO` (oder `--car random`), `?car=RO` im Web oder blättere in der Karriere-Garage; in der Karriere wird automatisch das Auto deines Landes benutzt.

## Schwierigkeit, Fahrzeuge, Hindernisse und Boni

- **5 Schwierigkeitsgrade** — Leicht, Normal, Schwer, Experte, Wahnsinn: Sie skalieren Startzeit und Checkpoint-Boni (×2,0 … ×1,6), Menge und Tempo des Verkehrs, wie viele Hindernisse und Boni auf der Straße liegen und wie viel Geschwindigkeit ein Unfall kostet. In der Karriere machen sie auch die simulierten Rivalen stärker oder schwächer.
- **19 Fahrzeugklassen** für dein Auto: Sportwagen, Limousine, Kleinstwagen, LKW, Sattelzug (TIR), Moped, Motorrad, Trike, Tuk-Tuk, Quad, Buggy, Formelwagen, Rallyeauto, Pickup, Transporter, Bus, Traktor, Stretchlimo und Monster Truck. Jede behält Lackierung und Profil des Landes und bringt ihren Kompromiss mit: Höchstgeschwindigkeit, Beschleunigung, Haftung, Breite, Unfallfestigkeit und Uhr. Zweiräder sind schmal, wendig und empfindlich; Bus, Traktor und Monster Truck sind breit und robust; der Formelwagen ist der schnellste und zerbrechlichste.
- **Gemischter Verkehr**: Autos, Kleinstwagen, LKW, Sattelzüge, Busse, Traktoren, Motorräder und Trikes; je höher der Grad, desto mehr Schwerverkehr.
- **Auf der Straße**: Hindernisse (Pylonen, Ölflecken, Absperrungen, Schlaglöcher, Pfützen, Schlamm) und Boni (+4 s, Extra-Turbo, Temposchub, Schild, Münzen, Verkehr stoppt, Punkte ×2). Aktive Effekte erscheinen als farbige Balken unter der Uhr.
- **Auswahlbildschirm** zu Beginn jedes Spiels: ← → Fahrzeug, ↑ ↓ Schwierigkeit, Enter zum Start, Rücktaste, um zurückzukehren. `--class` / `--level` / `--nopick` (nativ) oder `?class=…&level=…` / die beiden Auswahlfelder (Web) überspringen ihn; in der Karriere nutze die Zeilen *Schwierigkeit* / *Fahrzeug* im Hub.

## Welt, Systeme und Werkzeuge

- **Wetter und Untergründe** — 11 Wetterlagen (klar, Regen, Gewitter, Nebel, Schnee, Schneesturm, Sandsturm, Hagel, Glatteis, Hitzewelle, Ascheregen) und 6 Untergründe (Asphalt, gefrorener See, Salzwüste, Lavafeld, Dünen, Moor) verändern Haftung, Sicht und Aussehen; auf Auto lassen oder wählen (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Tag und Nacht** — Morgengrauen, Tag, Abenddämmerung oder Nacht (`--time`, `?time=`), mit Scheinwerfern, Sternen und Grillen.
- **Leben am Straßenrand** — Brücken, Tunnel, Viadukte und Bergpässe; Bahnübergänge mit Zügen; Tiere; Rampen und Turbo-Felder; Polizei, die Raser verfolgt (wer erwischt wird, wird disqualifiziert).
- **Sprit und Schäden** — der Sprit geht zur Neige und Tankstellen füllen ihn auf; Unfälle beschädigen Karosserie und Motor (weniger Höchstgeschwindigkeit, schwächere Beschleunigung, schwammigere Lenkung) und Tankstellen reparieren sie.
- **Getriebe, Reifen und Setup** — Automatik oder manuelles Getriebe (E / Q schalten, T wechselt); Straßen-, Regen-, Schnee- und Geländereifen und vier Setups (ausgewogen, Sprint, Abtrieb, sparsam): `--tyre`, `--setup`.
- **Rang 0–999 und verantwortungsvolles Spielen** — der Rang steigt mit der Erfahrung und skaliert die Schwierigkeit (`--rank N` überschreibt ihn); Tage in Folge werden gezählt, eine Erinnerung schlägt nach langer Sitzung eine Pause vor und ein Hinweis sagt, wann es für heute genug ist.
- **Abzeichen und Sammlungen** — 25 Abzeichen und eine Sammlung der gefahrenen Klassen, Wetterlagen, Untergründe, Tageszeiten und Länder.
- **Tagesherausforderung** — `--daily` / `?daily`: dieselbe Etappe, dasselbe Fahrzeug und dieselben Bedingungen für alle, mit einem kurzen Ergebniscode zum Vergleichen.
- **Geist** — deine beste Fahrt auf einer Etappe wird als cyanfarbener Geist mit Zeitabstand wiedergegeben (gespeichert in `~/.turbocursa_ghost_*` oder `localStorage`; am nützlichsten mit festem Startwert oder der Tagesetappe).
- **Rallye** — `--rally` / `?mode=rally`: fünf Etappen vom Morgengrauen bis zur Nacht mit wechselndem Wetter und Untergrund; Schäden, Sprit und Reifenverschleiß werden mitgenommen; Minuten im Servicepark zählen zur Zeit; die Endplatzierung wird gegen 15 simulierte Teams berechnet.
- **Fotomodus** — P freie Kamera (Pfeile bewegen sie), H blendet das HUD ein oder aus, C speichert ein PNG (`turbocursa_photo_N.png`; im Browser wird es heruntergeladen).
- **Steuerung** — Tastatur, Touch-Tasten, Gamepad im Browser (Sticks oder Steuerkreuz lenken, RT/A Gas, LT/B Bremse, X Turbo, Start Enter, Select zurück, LB/RB Gänge, Y manuell) und Neigungslenkung am Handy (Taste ⟲).
- **Barrierefreiheit** — Filter für Farbsehschwäche (`--colorblind protan|deutan|tritan|contrast`), Hilfen, die Kurven halten, das Auto zentrieren und Wände abmildern (`--assist 1..3`), ein langsamer Modus (`--slow 70` = 70 % der Höchstgeschwindigkeit) und eine große Oberfläche (`--large`).
- **Ton** — prozeduraler Motor mit Gängen, Wind, Regen, Gewitter, Hagel, Sirenen, Zughupe, Eis, Spritwarnton, beschädigtem Motor und Tierlauten.
- **Etappen-Designer und Codes** — `--design` (Web: `?mode=design`): Startwert, Modus, Fahrzeug, Stufe, Wetter, Untergrund, Tageszeit, die vier Landschaften, Kurvigkeit, Hügeligkeit und Länge wählen; ein Diagramm zeigt das Profil. Jede Etappe hat einen kurzen geprüften Code `TCS1-…`: fahre ihn mit `--code CODE` oder `?code=CODE`; in der Web-Version zeigt L ihn an und kopiert einen Link.
- **Städte und Verkehrsregeln** — jede Etappe führt durch Städte mit Boulevards und Straßen: Fassaden, Gehwege, Zebrastreifen, Ampeln (grün, gelb, rot), Fußgänger und ein Tempolimit. Rotlichtverstoß, zu schnelles Fahren oder fehlende Vorfahrt für Fußgänger rufen die Polizei (auf Easy kostet ein Bußgeld Zeit); einen Fußgänger anzufahren kostet Zeit und Schäden.
- **Verantwortungsvolles Fahren lohnt sich** — an Rot halten, Vorrang gewähren, das Limit einhalten und eine Stadt ohne Verstoß durchqueren bringen Punkte, Zeit und einen Multiplikator bis x3 („GUT GEFAHREN“, „SAUBERE STADT“); jeder Verstoß setzt die Serie zurück. Einen Krankenwagen oder ein Feuerwehrauto mit Sirene überholen zu lassen, wird belohnt, sie zu behindern ist ein Verstoß.
- **Staus, Umleitungen und RadarMap** — manche Etappen haben einen Stau: eine stehende Schlange mit einer gewundenen Gasse, durch die man langsam schleichen kann (sie zu rammen ist ein Unfall). Eine Ausfahrt rechts vor dem Stau führt auf eine freie Straße; die RadarMap im HUD (Karte mit Fahrtrichtung oben) zeigt die Strecke voraus, Städte, den Stau in Rot, die Umleitung in Grün, die Ampeln und die Zeit für die Durchfahrt auf der Hauptstraße gegenüber der Ausfahrt.
- **Einsatzfahrzeuge und Hubschrauber** — Krankenwagen und Feuerwehrautos überholen mit Blaulicht; eine Unfallstelle (ein Hubschrauber, der bei deiner Annäherung auf der Straße landet, ein Krankenwagen und ein Feuerwehrauto) sperrt die rechte Spur und lässt die linke frei; ein Verkehrshubschrauber kreist über Städten und Staus.
- **Aufträge, Testgelände, Parkplätze** — nimm ein Paket oder einen Fahrgast auf und setze ihn in der Bucht ab, bevor die Zeit abläuft; die Strecke hat ein Testgelände (Steigung, Slalomringe, Rampe) und Parkplätze, durch Schilderbrücken angekündigt, jeweils mit Bonus.
- **Freischaltbare Dienstfahrzeuge** — Polizeiauto (10 saubere Städte), Krankenwagen (8 Lieferungen), Feuerwehrauto (3 Tagesrennen und 4 Testgelände- oder Parkbonus), Hubschrauber (die anderen drei plus 7 Tagesrennen). Sie dürfen Regeln brechen, der Verkehr weicht aus, und der Hubschrauber fliegt über den Verkehr und landet bei niedriger Geschwindigkeit.
- **Ereignisse und Extras** — Meisterschaften mit 7, 10 oder 15 Etappen und die Acht-Etappen-Tour durch ein Land (`--champ`, `--tour`, Code `TCC1-…`); Wochenherausforderung mit wechselndem Modifikator (`--weekly`); Import und Export von Geistern (`--ghost`, `--ghost-export`); sichtbares Feld mit 8–16 Rivalen samt Startaufstellung, umgekehrter Aufstellung und Qualifikationsrunde (`--pack`, `--inverted`, `--qualify`); Splitscreen für 3–4 Spieler mit CPU-Auffüllung (`--players`); Weggabelungen (schnell links, landschaftlich rechts), bei denen der Hubschrauber den Weg zeigt; Verfolgungsjagd mit dem Polizeiwagen; Überholkette von 200 bis 2000; Garage, in der Münzen Upgrades kaufen (`--garage`, `--buy`); Musikstile, mph, Tastenbelegungen und geheime Lackcodes (`--music`, `--mph`, `--keys`, `--secret`); Sirenen für Polizeiwagen, Krankenwagen und Löschfahrzeug. Alles lässt sich auf der Seite **EXTRAS** des Startbildschirms einstellen (Rücktaste auf dem Fahrzeugbild). Ein Codefeld (⌨ im Browser) nimmt eine Meisterschaft, eine Etappe, einen Geist, ein Lackwort, DAILY oder WEEKLY an.

## Grenzen

- Die WebAssembly-Version und die native Audioausgabe (`--features audio`, cpal) konnten in der Entwicklungsumgebung weder gebaut noch getestet werden; die darin verpackte Logik ist durch Tests abgedeckt. Der synthetisierte Klang wurde numerisch geprüft, nicht nach Gehör.
- Die Strecken werden erzeugt: Die Schwierigkeit hängt vom Seed ab (ein Bot mit 70 % Gas schafft 60 von 60 Etappen, mit 55 % ebenfalls 60 von 60, aber nur mit 0,6 s Reserve).
- Keine Kopie eines kommerziellen Spiels und mit keinem verbunden.
- Geschwindigkeit: Auf dem Entwicklungsrechner braucht ein nativer Release-Build etwa 13 ms pro Bild (99. Perzentil: 23 ms); langsamere Rechner können unter 60 fps fallen; die Browser-Version wurde nicht gemessen.
- Länder: 196 sind enthalten (die 193 UN-Mitglieder plus Vatikan, Palästina und Taiwan); die Flaggenfarben auf den Autos sind angenähert, und die Ländernamen in den Menüs sind nur englisch (die Web-Auswahl nutzt die vom Browser übersetzten Namen).
- Karriere: Die Rivalen werden aus der Referenzzeit des Autopiloten simuliert. Die Schwierigkeitskurve wurde mit synthetischen Fahrern geprüft (Referenztempo gewinnt den Nationalpokal in Bronze und verliert den Weltpokal in Diamant), nicht mit echten Spielern; die Webseite wurde nur mit einer simulierten Engine getestet.
- Schwierigkeitsgrade, die 19 Fahrzeugklassen, Hindernisse und Boni wurden nur mit simulierten Fahrern und automatischen Tests abgestimmt, nicht mit echten Spielern; die Kompromisse sind plausibel, nicht kalibriert. Die Sprites sind schlichte Pixelzeichnungen, und das Checkpoint-Banner zeigt auf jeder Stufe weiterhin „+30“.
- Die Web-Version lief mit der echten WebAssembly-Engine nur in Chromium ohne Fenster (nicht auf echten Handys), der native Ton wurde nie auf einem echten Gerät gehört, und das native Gamepad (Linux, `/dev/input/js0`) wurde nur über seinen Decoder getestet, nicht mit einem echten Gamepad; auf anderen Systemen hat die native Version noch kein Gamepad.
- Die Balance von Sprit, Polizei, Tieren, Schäden und Rallye-Rivalen wurde nur mit simulierten Fahrern geprüft, nicht mit echten Spielern.
- Die Regeln für Stadt, Stau, Umleitung und Einsatzfahrzeuge wurden nur mit simulierten Fahrern abgestimmt und getestet, nicht mit echten Spielern.
- Balance von Aufträgen, Testgelände und Parkplätzen sowie die Freischalt-Schwellen wurden nur mit simulierten Fahrern abgestimmt, nicht mit echten Spielern.
- Die neuen Ereignisse und Extras wurden nur mit simulierten Fahrern und automatischen Prüfungen getestet: der Ton (Musikstile, Sirenen) wurde nicht angehört, nichts wurde mit einem echten Gamepad oder Telefon ausprobiert, die Balance wurde nicht von echten Menschen gespielt, und Online-Bestenlisten fehlen, weil sie einen Server brauchen.

## Lizenz

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).
