[English](README.md) · **Română** · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)

# **TURBOCURSA**

Versiune: 1.5 · Ultima actualizare: 12 noiembrie 2026

## Ce este

TURBOCURSA este un joc arcade de curse pseudo-3D scris în Rust, în regim clean-room: motor, grafică, muzică și nume originale, fără cod, resurse sau mărci ale vreunui joc existent. Tot ce vezi și auzi este generat dintr-un seed: copaci, stânci, nori, nebuloase, aurore și bannere Mandelbrot la punctele de control, plus coloană sonoră și sunet de motor procedurale. Biblioteca de bază nu are dependențe la rulare.

Nouăsprezece peisaje generate: pădure, deșert, amurg, copaci-lume colosali cu coroanele ascunse în nori, platouri roșii stratificate, insule plutitoare cu cascade și o planetă cu inel, un lac-oglindă sub două luni, o tundră cu aurore și ninsoare, un vulcan cu vene de lavă, penaj de fum fractal și fulgere recursive, un abis luminat cu corali și alge, un cosmos cu nebuloasă Julia și drum curcubeu, o cetate neon din piramide Sierpinski și blocuri Menger, și o junglă luminiscentă de miceliu. Fiecare etapă înlănțuie un peisaj clasic și trei mai noi, într-o ordine dată de seed. Încă șase lumi, posibile și imposibile: o lume inversată cu un al doilea lanț muntos atârnat de cer, un deșert-prismă sub benzi de aurore spectrale, o lume mecanică de roți dințate uriașe, o mare de nori, o gaură neagră cu un inel de acreție lentilat și un canion fractal cu straturi dungate.

## Utilizare

- Nativ: `cargo run --release --features native --bin turbo` (adaugă `--versus` pentru 2 jucători, `--daily` pentru pista zilei, `--seed N` pentru o pistă fixă, `--lang xx` pentru limbă).
- Web: `./build_web.sh` produce un singur fișier offline `dist/turbocursa.html` (comenzi tactile, selector de limbă, scor maxim). Parametri: `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Comenzi: săgeți sau WASD, Space = turbo, Enter = reîncepe, M = fără sunet. Jucătorul 2: WASD + Shift stânga.
- Interfață și documentație: English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. Textul din joc folosește un font pixelat; diacriticele sunt desenate ca un semn deasupra literei.
- Moduri: `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (întrecere cu un rival calculator), `--versus` (2 jucători); web: `?mode=NUME`. · `--career` (divizii, campionate și cupe, cu salvare)
- Demo: `--demo` pune un pilot AI să parcurgă toate modurile pe toate nivelurile de dificultate, etapă după etapă (mașina demo e protejată și nu consumă combustibil; nu se salvează nimic)

## Moduri de joc

- **Campanie** — Învinge cronometrul prin patru peisaje generate; punctele de control adaugă timp.
- **Contra Cronometru** — Fără trafic, cronometrul numără în sus: termină cât poți de repede.
- **Infinit** — Ture fără sfârșit: fiecare tură e o etapă nou generată, +25 s.
- **Ora De Vârf** — Trafic triplu; fiecare depășire îți aduce 0,8 secunde.
- **Vânătoare De Orbe** — Urmează trasee fractale de orbe; fiecare adaugă timp și combo.
- **Slalom Prin Inele** — Treci prin inele: fiecare trecere ridică multiplicatorul, o ratare costă o secundă.
- **Inversare** — Direcția se inversează șase secunde din fiecare paisprezece.
- **Întuneric** — Întuneric: se vede doar conul farurilor, peisajul e doar o bănuială.
- **Zen** — Fără ceas, fără trafic, fără ciocniri: doar conduci prin peisaje fără sfârșit.
- **Contra CPU** — Întrece un rival condus de calculator pe aceeași etapă, ecran împărțit.
- **2 Jucători** — Doi oameni, o etapă, ecran împărțit: primul la sosire câștigă.

## Carieră

Pornește cu `--career` (web: `?mode=career`). Progresul se salvează automat (fișierul `~/.turbocursa_career` în varianta nativă, `localStorage` în browser).
- **Divizii** — Bronz, Argint, Aur, Platină, Diamant. Fiecare sezon e un campionat de șase etape cu 12 piloți: primii trei urcă, ultimii trei coboară; campionul diviziei Diamant primește invitație la Cupa Mondială.
- **Cupe cu runde eliminatorii** — Timpul tău e comparat cu rivali simulați, cu atât mai puternici cu cât cupa și divizia sunt mai înalte; cei mai lenți sunt eliminați la fiecare rundă (16 → 8 → 4 → finală la cupa națională; până la 32 → 24 → 16 → 8 → 4 → finală la Cupa Mondială).
- **Calificări multiple** — Două șanse de *repesaj* pe cupă: dacă ești eliminat, cei mai buni dintre eliminați mai concurează o dată, iar primii doi revin.
- **Scara** — Ajungi în finala **cupei naționale** ca să te califici la **cupa continentală**; în finala ei, la **cupa intercontinentală**; în finala ei, la **Cupa Mondială**.
- **Fiecare țară** — 196 țări pe șase continente; poți intra și în cupa națională a oricărei alte țări (cupe în străinătate) și aduni câte un trofeu pentru fiecare în vitrina de trofee.
- **Clasamente** — un clasament mondial de 300 de piloți plus tine, clasamentul națiunilor, tabelul campionatului și statisticile tale.

## Mașini pe țări

Fiecare țară are propria mașină de curse: o livrea pictată din culorile naționale (aproximative, artă originală) cu număr de înmatriculare și un profil de comportament (viteză maximă, accelerație, aderență, 2-4 încărcări turbo). Profilul e un buget cu sumă zero, deci nicio țară nu are o mașină mai bună în toate privințele. O alegi cu `--car RO` (sau `--car random`), `?car=RO` în web sau o răsfoiești în Garajul din carieră; în carieră se folosește automat mașina țării tale.

## Dificultate, vehicule, obstacole și bonusuri

- **5 niveluri de dificultate** — Ușor, Normal, Greu, Expert, Nebun: modifică timpul de start și bonusurile de checkpoint (×2,0 … ×1,6), cantitatea și viteza traficului, câte obstacole și bonusuri sunt pe drum și cât te costă o ciocnire. În carieră, nivelul face și rivalii simulați mai puternici sau mai slabi.
- **19 clase de vehicule** pentru mașina ta: mașină sport, berlină, micromașină, camion, tir, moped, motocicletă, triciclu, tuk-tuk, quad (ATV), buggy, monopost, mașină de raliu, pickup, furgonetă, autobuz, tractor, limuzină și monster truck. Fiecare păstrează culorile și profilul țării și adaugă propriul compromis: viteză maximă, accelerație, aderență, lățime, rezistență la ciocniri și ceas. Vehiculele cu două roți sunt înguste, agile și fragile; autobuzul, tractorul și monster truck-ul sunt late și rezistente; monopostul e cel mai rapid și cel mai fragil.
- **Trafic amestecat**: mașini, micromașini, camioane, tiruri, autobuze, tractoare, motociclete și triciclete; cu cât nivelul e mai mare, cu atât mai multe vehicule grele.
- **Pe drum**: obstacole (conuri, pete de ulei, bariere, gropi, bălți, noroi) și bonusuri (+4 s, turbo în plus, impuls de viteză, scut, monede, trafic înghețat, scor ×2). Efectele active apar ca bare colorate sub ceas.
- **Ecran de selecție** la începutul fiecărui joc: ← → vehicul, ↑ ↓ dificultate, Enter pentru start, Backspace pentru a reveni la el. `--class` / `--level` / `--nopick` (nativ) sau `?class=…&level=…` / cele două selectoare (web) îl sar; în carieră folosește liniile *Dificultate* / *Vehicul* din meniu.

## Lume, sisteme și unelte

- **Vreme și teren** — 11 tipuri de vreme (senin, ploaie, furtună, ceață, ninsoare, viscol, furtună de nisip, grindină, polei, caniculă, cădere de cenușă) și 6 terenuri (asfalt, lac înghețat, câmpie de sare, câmp de lavă, dune, mlaștină) schimbă aderența, vizibilitatea și aspectul; lasă-le pe Auto sau alege-le (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Zi și noapte** — zori, zi, amurg sau noapte (`--time`, `?time=`), cu faruri, stele și greieri.
- **Viață pe marginea drumului** — poduri, tunele, viaducte și pasuri; treceri la nivel cu trenuri; animale; rampe și plăci de turbo; poliție care urmărește vitezomanii (dacă ești prins, ești descalificat).
- **Combustibil și avarii** — combustibilul se consumă și benzinăriile îl reumplu; loviturile avariază caroseria și motorul (viteză maximă mai mică, accelerație mai slabă, direcție mai moale), iar benzinăriile le repară.
- **Cutie de viteze, anvelope și setup** — cutie automată sau manuală (E / Q schimbă, T comută); anvelope de șosea, ploaie, zăpadă și off-road și patru setup-uri (echilibrat, sprint, downforce, economic): `--tyre`, `--setup`.
- **Rang 0–999 și joc responsabil** — rangul crește cu experiența și scalează dificultatea (`--rank N` îl forțează); se numără zilele la rând, un memento sugerează o pauză după o sesiune lungă, iar un mesaj spune când ajunge pe azi.
- **Insigne și colecții** — 25 de insigne și o colecție de clase, vremi, terenuri, momente ale zilei și țări conduse.
- **Provocarea zilei** — `--daily` / `?daily`: aceeași etapă, același vehicul și aceleași condiții pentru toți, cu un cod scurt de rezultat pentru comparație.
- **Fantomă** — cea mai bună cursă pe o etapă este redată ca o fantomă cyan cu diferența de timp (salvată în `~/.turbocursa_ghost_*` sau `localStorage`; util mai ales cu seed fix sau pe etapa zilei).
- **Raliu** — `--rally` / `?mode=rally`: cinci etape de la zori până noaptea, cu vreme și teren schimbătoare; avariile, combustibilul și uzura anvelopelor se păstrează; minutele din parcul de service se adaugă la timp; clasamentul final se calculează față de 15 echipaje simulate.
- **Mod foto** — P cameră liberă (săgețile o mută), H arată sau ascunde HUD-ul, C salvează un PNG (`turbocursa_photo_N.png`; în browser se descarcă).
- **Comenzi** — tastatură, butoane tactile, gamepad în browser (manetele sau crucea direcțională virează, RT/A accelerație, LT/B frână, X turbo, Start enter, Select înapoi, LB/RB trepte, Y manual) și virare prin înclinarea telefonului (butonul ⟲).
- **Accesibilitate** — filtre pentru daltonism (`--colorblind protan|deutan|tritan|contrast`), asistențe care țin curba, recentrează mașina și înmoaie zidurile (`--assist 1..3`), mod lent (`--slow 70` = 70% din viteza maximă) și interfață mare (`--large`).
- **Sunet** — motor procedural cu trepte, vânt, ploaie, furtună, grindină, sirene, claxon de tren, gheață, bip de combustibil scăzut, motor avariat și sunete de animale.
- **Creator de etape și coduri** — `--design` (web: `?mode=design`): alegi seed-ul, modul, vehiculul, nivelul, vremea, terenul, ora, cele patru peisaje, cât de sinuos și deluros e drumul și cât de lung; un grafic arată profilul drumului. Fiecare etapă are un cod scurt cu verificare `TCS1-…`: conduce-l cu `--code COD` sau `?code=COD`; în versiunea web apasă L pentru a-l afișa și a copia un link.
- **Orașe și reguli de circulație** — fiecare traseu traversează orașe cu bulevarde și străzi: fațade, trotuare, treceri de pietoni, semafoare (verde, galben, roșu), pietoni și limită de viteză. Trecerea pe roșu, viteza peste limită sau necedarea trecerii unui pieton cheamă poliția (pe Easy o amendă costă timp); lovirea unui pieton costă timp și avarii.
- **Condusul responsabil se plătește** — oprirea la roșu, cedarea trecerii, respectarea limitei și traversarea unui oraș fără abateri aduc scor, timp și un multiplicator până la x3 („GOOD DRIVER” / „ȘOFER BUN”, „ORAȘ CURAT”); orice abatere întrerupe seria. Faptul că lași o ambulanță sau o autospecială de pompieri cu sirena pornită să depășească aduce bonus, blocarea ei este abatere.
- **Ambuteiaje, ocoliri și RadarMap** — unele trasee au un ambuteiaj: o coloană oprită cu un culoar șerpuit prin care poți răzbi încet (lovirea ei este accident). O bretea spre dreapta înainte de ambuteiaj duce pe o stradă liberă; RadarMap din HUD (hartă cu direcția în sus) arată drumul din față, orașele, ambuteiajul în roșu, ocolirea în verde, semafoarele și timpul de traversare pe drumul principal față de ieșire.
- **Vehicule de urgență și elicoptere** — ambulanțe și autospeciale de pompieri depășesc cu girofarurile pornite; o scenă de accident (un elicopter care aterizează pe drum pe măsură ce te apropii, o ambulanță și o autospecială) blochează banda din dreapta și o lasă liberă pe cea din stânga; un elicopter de monitorizare a traficului planează deasupra orașelor și ambuteiajelor.
- **Misiuni, poligon, parcări** — preia un colet sau un pasager și lasă-l în golful de destinație înainte să expire timpul; traseul are poligon (pantă, inele de slalom, rampă) și parcări anunțate de portaluri, fiecare cu bonus.
- **Vehicule de serviciu deblocabile** — mașina de poliție (10 orașe fără greșeli), ambulanța (8 livrări), autospeciala de pompieri (3 provocări zilnice și 4 bonusuri de poligon sau parcare), elicopterul (celelalte trei plus 7 provocări zilnice). Pot încălca regulile, traficul le face loc, iar elicopterul zboară peste trafic și aterizează când merge încet.
- **Evenimente și extra** — campionate de 7, 10 sau 15 etape și turul de opt etape al unei țări (`--champ`, `--tour`, cod `TCC1-…`); provocare săptămânală cu un modificator care se schimbă (`--weekly`); import și export de fantome (`--ghost`, `--ghost-export`); pluton vizibil de 8–16 rivali cu grilă, grilă inversată și tur de calificare (`--pack`, `--inverted`, `--qualify`); ecran împărțit pentru 3–4 jucători cu completare de la calculator (`--players`); bifurcații (rapidă la stânga, panoramică la dreapta) cu elicopterul care arată drumul; urmărire cu mașina de poliție; lanț de depășiri de la 200 până la 2000; garaj unde monedele cumpără îmbunătățiri (`--garage`, `--buy`); stiluri de muzică, mph, scheme de taste și coduri secrete de vopsea (`--music`, `--mph`, `--keys`, `--secret`); sirene pentru mașina de poliție, ambulanță și autospecială. Totul se poate regla din pagina **EXTRAS** a ecranului de start (Backspace pe ecranul vehiculului). Un câmp de cod (⌨ în browser) primește un campionat, o etapă, o fantomă, un cuvânt de vopsea, DAILY sau WEEKLY.

## Limite

- Versiunea WebAssembly și ieșirea audio nativă (`--features audio`, cpal) nu au putut fi compilate sau testate în mediul de dezvoltare; logica pe care o învelesc este acoperită de teste. Sunetul sintetizat a fost verificat numeric, nu la ureche.
- Pistele sunt generate: dificultatea variază cu seed-ul (un bot cu 70% accelerație termină 60 din 60 de etape, cu 55% tot 60 din 60, dar cu doar 0,6 s de rezervă).
- Nu este o copie a vreunui joc comercial și nu are legătură cu vreunul.
- Viteză: pe mașina de dezvoltare, build-ul nativ release durează cca 13 ms pe cadru (percentila 99: 23 ms), deci pe mașini mai lente poate scădea sub 60 fps; varianta din browser nu a fost măsurată.
- Țări: sunt incluse 196 (cele 193 de state membre ONU plus Vaticanul, Palestina și Taiwanul); culorile steagurilor de pe mașini sunt aproximative, iar numele țărilor din meniuri sunt doar în engleză (selectorul de mașini din web folosește numele traduse de browser).
- Cariera: rivalii sunt simulați pornind de la timpul de referință al pilotului automat. Curba de dificultate a fost verificată cu piloți sintetici (ritmul de referință câștigă cupa națională din divizia Bronz și pierde Cupa Mondială din Diamant), nu cu jucători reali, iar pagina web a fost testată doar cu un motor simulat.
- Nivelurile de dificultate, cele 19 clase de vehicule, obstacolele și bonusurile au fost echilibrate doar cu șoferi simulați și teste automate, nu cu jucători reali; compromisurile sunt plauzibile, nu calibrate. Sprite-urile vehiculelor sunt desene pixel simple, iar bannerul de checkpoint arată tot „+30” la orice nivel.
- Versiunea web a rulat cu motorul WebAssembly real doar în Chromium fără ecran (nu pe telefoane reale), sunetul nativ nu a fost auzit pe un dispozitiv real, iar gamepadul nativ (Linux, `/dev/input/js0`) a fost testat doar prin decodor, nu cu un gamepad fizic; pe alte sisteme versiunea nativă nu are încă gamepad.
- Echilibrul combustibilului, poliției, animalelor, avariilor și rivalilor din raliu a fost verificat doar cu piloți simulați, nu cu jucători reali.
- Regulile de oraș, ambuteiaj, ocolire și vehicule de urgență au fost echilibrate și testate doar cu șoferi simulați, nu cu jucători reali.
- Echilibrul misiunilor, poligonului și parcărilor și pragurile de deblocare au fost reglate doar cu șoferi simulați, nu cu jucători reali.
- Evenimentele și extra-urile noi au fost verificate doar cu șoferi simulați și teste automate: sunetul (stilurile de muzică, sirenele) nu a fost ascultat, nimic nu a fost încercat pe un gamepad fizic sau pe telefon, echilibrul nu a fost jucat de oameni, iar clasamentele online lipsesc pentru că ar cere un server.

## Licență

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).
