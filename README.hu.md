[English](README.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · **Magyar**

# **TURBOCURSA**

Verzió: 1.5 · Utolsó frissítés: 2026. november 12.

## Mi ez

A TURBOCURSA egy Rustban, „clean-room” módszerrel írt pszeudo-3D arcade autóverseny: saját motor, grafika, zene és név, egyetlen létező játék kódja, erőforrása vagy védjegye nélkül. Minden, amit látsz és hallasz, egy magból (seed) jön létre: fák, sziklák, felhők, ködök, sarki fény és Mandelbrot-zászlók az ellenőrzőpontokon, valamint procedurális zene és motorhang. Az alapkönyvtárnak nincs futásidejű függősége.

Tizenkilenc generált táj: erdő, sivatag, alkony, a felhőkben rejtőző koronájú óriás világfák, vörös rétegzett mesák, lebegő szigetek vízesésekkel és gyűrűs bolygóval, tükörtó két hold alatt, sarki fényes, havas tundra, lávaerekkel teli vulkán fraktál füstoszloppal és rekurzív villámokkal, napfényes mélység korallokkal és hínárral, kozmosz Julia-ködvel és szivárványúttal, Sierpinski-piramisokból és Menger-blokkokból álló neonváros, valamint világító micélium-dzsungel. Minden szakasz egy klasszikus és három újabb tájat fűz össze a seed által meghatározott sorrendben. Hat újabb világ, lehetséges és lehetetlen: egy fordított világ, ahol egy második hegylánc lóg le az égről, egy prizma-sivatag spektrális sarki fénysávok alatt, egy óraműves világ óriási fogaskerekekkel, egy felhőtenger, egy fekete lyuk gravitációsan elhajlított akkréciós gyűrűvel és egy fraktál kanyon sávos rétegekkel.

## Használat

- Natív: `cargo run --release --features native --bin turbo` (`--versus` 2 játékoshoz, `--daily` a nap szakaszához, `--seed N` rögzített szakaszhoz, `--lang xx` a nyelvhez).
- Web: a `./build_web.sh` egyetlen offline fájlt készít: `dist/turbocursa.html` (érintésvezérlés, nyelvválasztó, egyéni rekord). Paraméterek: `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Vezérlés: nyilak vagy WASD, Szóköz = turbó, Enter = újra, M = néma. 2. játékos: WASD + bal Shift.
- Felület és dokumentáció: English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. A játékszöveg pixelbetűt használ; az ékezeteket a betű fölé rajzolt jel jelzi.
- Módok: `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (verseny számítógépes riválissal), `--versus` (2 játékos); web: `?mode=NÉV`. · `--career` (divíziók, bajnokságok és kupák, mentéssel)

## Játékmódok

- **Kampány** — Győzd le az órát négy generált tájon át; az ellenőrzőpontok időt adnak.
- **Időfutam** — Forgalom nélkül, az óra felfelé számol: érj célba minél gyorsabban.
- **Végtelen** — Végtelen körök: minden kör új, generált szakasz, +25 mp.
- **Csúcsforgalom** — Háromszoros forgalom; minden előzés 0,8 másodpercet ad.
- **Gömbvadászat** — Kövesd a gömbök fraktál nyomvonalát; mindegyik időt és kombót ad.
- **Gyűrűslalom** — Hajts át a gyűrűkön: minden találat növeli a szorzót, egy hiba egy másodpercbe kerül.
- **Fordított** — A kormányzás tizennégy másodpercből hatra megfordul.
- **Sötét** — Sötétség: csak a fényszóró kúpja látszik, a táj csupán szóbeszéd.
- **Zen** — Óra, forgalom és ütközés nélkül: csak vezetsz a végtelen tájakon.
- **CPU Ellen** — Versenyezz egy számítógép által vezérelt riválissal ugyanazon a szakaszon, osztott képernyőn.
- **2 Játékos** — Két ember, egy szakasz, osztott képernyő: az nyer, aki előbb ér célba.

## Karrier

Indítás: `--career` (web: `?mode=career`). Az előrehaladás automatikusan mentődik (natívan a `~/.turbocursa_career` fájl, böngészőben `localStorage`).
- **Divíziók** — Bronz, Ezüst, Arany, Platina, Gyémánt. Minden szezon hat fordulós bajnokság 12 versenyzővel: az első három feljut, az utolsó három kiesik; a Gyémánt bajnoka szabadkártyát kap a Világkupára.
- **Kieséses kupák** — Az idődet szimulált riválisokhoz mérik, akik a kupával és a divízióval erősödnek; a leglassabbak minden körben kiesnek (16 → 8 → 4 → döntő a nemzeti kupában; akár 32 → 24 → 16 → 8 → 4 → döntő a Világkupában).
- **Többszörös kvalifikáció** — Kupánként két *vigaszág* esély: ha kiesel, a legjobb kiesettek újra futnak, és az első kettő visszatér.
- **A létra** — Jutás a **nemzeti kupa** döntőjébe kvalifikál a **kontinentális kupára**; annak döntője az **interkontinentális kupára**; annak döntője a **Világkupára**.
- **Minden ország** — 196 ország hat földrészen; bármely más ország nemzeti kupájában is indulhatsz (külföldi kupák), és mindegyikért trófeát gyűjthetsz a vitrinben.
- **Rangsorok** — 300 versenyzős világranglista veled együtt, nemzetek rangsora, a bajnoki tabella és a statisztikáid.

## Autók országonként

Minden országnak saját versenyautója van: nemzeti színekkel festett (közelítő, saját grafika) felirat rendszámmal, és vezetési profil (végsebesség, gyorsulás, tapadás, 2-4 turbótöltés). A profil nulla összegű keret, így egyik országnak sincs mindenben jobb autója. Választás: `--car RO` (vagy `--car random`), webes `?car=RO`, vagy böngészd a karrier Garázsában; a karrierben automatikusan az országod autója szerepel.

## Nehézség, járművek, akadályok és bónuszok

- **5 nehézségi szint** — Könnyű, Normál, Nehéz, Profi, Őrült: skálázzák a kezdő időt és az ellenőrzőpont-bónuszokat (×1,25 … ×0,68), a forgalom sűrűségét és sebességét, az úton lévő akadályok és bónuszok számát és azt, mennyit veszítesz egy ütközésnél. Karrierben a szint a szimulált riválisokat is erősebbé vagy gyengébbé teszi.
- **19 járműosztály** a saját autódnak: sportautó, szedán, mikroautó, teherautó, nyergesvontató (TIR), segédmotoros kerékpár, motorkerékpár, trike, tuk-tuk, quad, buggy, formaautó, raliautó, pickup, furgon, busz, traktor, limuzin és monster truck. Mindegyik megtartja az ország színeit és menettulajdonságait, és hozza a saját kompromisszumát: végsebesség, gyorsulás, tapadás, szélesség, ütközésállóság és óra. A kétkerekűek keskenyek, fürgék és törékenyek; a busz, a traktor és a monster truck szélesek és szívósak; a formaautó a leggyorsabb és a legtörékenyebb.
- **Vegyes forgalom**: autók, mikroautók, teherautók, nyergesvontatók, buszok, traktorok, motorosok és trike-ok; minél magasabb a szint, annál több nehézjármű.
- **Az úton**: akadályok (bóják, olajfoltok, korlátok, kátyúk, pocsolyák, sár) és bónuszok (+4 mp, extra turbó, sebességlöket, pajzs, érmék, megállított forgalom, ×2 pont). Az aktív hatások színes sávokként látszanak az óra alatt.
- **Választóképernyő** minden játék elején: ← → jármű, ↑ ↓ nehézség, Enter a rajthoz, Backspace a visszatéréshez. A `--class` / `--level` / `--nopick` (natív) vagy a `?class=…&level=…` / a két választó (web) átugorja; karrierben a menü *Nehézség* / *Jármű* sorait használd.

## Világ, rendszerek és eszközök

- **Időjárás és útfelületek** — 11 időjárás (derült, eső, vihar, köd, hó, hóvihar, homokvihar, jégeső, zabola, hőhullám, hamueső) és 6 útfelület (aszfalt, elfagyott tó, sósivatag, lávamező, dűnék, mocsár) változtatja a tapadást, a látást és a megjelenést; hagyd Auto-n vagy válaszd ki (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Nappal és éjszaka** — hajnal, nappal, alkonyat vagy éjszaka (`--time`, `?time=`), fényszórókkal, csillagokkal és tücskökkel.
- **Élet az út mentén** — hidak, alagutak, viaduktok és hegyi hágók; vasúti átjárók vonatokkal; állatok; rámpák és turbólapok; rendőrség, amely üldözi a száguldozókat (aki elfogják, kizárják).
- **Üzemanyag és sérülések** — az üzemanyag fogy, a benzinkutak újratöltik; az ütközések rongálják a karosszériát és a motort (kisebb végsebesség, gyengébb gyorsulás, lazább kormányzás), a kutak megjavítják őket.
- **Váltó, gumik és beállítás** — automata vagy kézi váltó (E / Q vált, T átkapcsol); országúti, esős, havas és terepgumik és négy beállítás (kiegyensúlyozott, sprint, leszorítóerő, takarékos): `--tyre`, `--setup`.
- **Rang 0–999 és felelős játék** — a rang a tapasztalattal nő és skálázza a nehézséget (`--rank N` felülírja); a napi sorozatot számoljuk, hosszú játék után emlékeztető javasol szünetet, és üzenet jelzi, mikor elég mára.
- **Jelvények és gyűjtemények** — 25 jelvény és gyűjtemény a vezetett osztályokról, időjárásokról, útfelületekről, napszakokról és országokról.
- **Napi kihívás** — `--daily` / `?daily`: mindenkinek ugyanaz a szakasz, jármű és körülmény, rövid eredménykóddal az összehasonlításhoz.
- **Szellem** — a szakaszon elért legjobb futamod ciánszínű szellemként ismétlődik az időkülönbséggel (mentés: `~/.turbocursa_ghost_*` vagy `localStorage`; leginkább rögzített maggal vagy a nap szakaszán hasznos).
- **Rally** — `--rally` / `?mode=rally`: öt szakasz hajnaltól éjszakáig, változó időjárással és útfelülettel; a sérülés, az üzemanyag és a gumikopás átvihető; a szervizparkban töltött percek hozzáadódnak az időhöz; a végső helyezést 15 szimulált csapattal szemben számoljuk.
- **Fotómód** — P szabad kamera (a nyilak mozgatják), H megjeleníti vagy elrejti a HUD-ot, C PNG-t ment (`turbocursa_photo_N.png`; böngészőben letöltődik).
- **Vezérlés** — billentyűzet, érintőgombok, játékvezérlő a böngészőben (kar vagy iránykereszt kormányoz, RT/A gáz, LT/B fék, X turbó, Start enter, Select vissza, LB/RB fokozatok, Y kézi) és döntéses kormányzás telefonon (⟲ gomb).
- **Hozzáférhetőség** — színtévesztés elleni szűrők (`--colorblind protan|deutan|tritan|contrast`), segédek, amelyek tartják a kanyart, középre húzzák az autót és puhítják a falakat (`--assist 1..3`), lassú mód (`--slow 70` = a végsebesség 70%-a) és nagy felület (`--large`).
- **Hang** — procedurális motor fokozatokkal, széllel, esővel, viharral, jégesővel, szirénákkal, vonatkürttel, jéggel, alacsony üzemanyag jelzéssel, sérült motorral és állathangokkal.
- **Szakaszeditor és kódok** — `--design` (web: `?mode=design`): válaszd ki a magot, a módot, a járművet, a szintet, az időjárást, az útfelületet, a napszakot, a négy tájat, az út kanyarosságát, dombosságát és hosszát; diagram mutatja a profilt. Minden szakasznak van egy rövid, ellenőrzőszámos kódja `TCS1-…`: vezesd `--code KÓD` vagy `?code=KÓD` megadásával; a webes változatban az L megjeleníti és linket másol.
- **Városok és közlekedési szabályok** — minden szakasz városokon vezet át, sugárutakkal és utcákkal: homlokzatok, járdák, zebrák, lámpák (zöld, sárga, piros), gyalogosok és sebességkorlátozás. A piros lámpa áthajtása, a sebességtúllépés vagy a gyalogos elsőbbségének megsértése rendőrt hív (Easy fokozaton a bírság időbe kerül); gyalogos elütése időt és sérülést jelent.
- **A felelős vezetés kifizetődik** — a piros lámpánál megállás, az elsőbbségadás, a korlátozás betartása és a szabálysértés nélkül átszelt város pontot, időt és legfeljebb x3-os szorzót hoz („JÓ SOFŐR”, „TISZTA VÁROS”); minden szabálysértés nullázza a sorozatot. Ha bekapcsolt szirénával előz mentő vagy tűzoltóautó és helyet adsz, az is jutalom, az akadályozásuk szabálysértés.
- **Dugók, kerülők és RadarMap** — egyes szakaszokon dugó van: álló sor kanyargó résszel, amelyen lassan átóvakodhatsz (a sorba rohanni baleset). A dugó előtti jobb oldali lehajtó szabad utcára vezet; a HUD RadarMap térképe (haladási irány felfelé) mutatja az előttünk lévő utat, a városokat, a dugót pirosan, a kerülőt zölden, a lámpákat és a főúton, illetve a kijáraton való áthaladás idejét.
- **Mentők, tűzoltók és helikopterek** — mentők és tűzoltóautók villogó fénnyel előznek; egy baleseti helyszín (egy helikopter, amely közeledtedre leszáll az útra, egy mentő és egy tűzoltóautó) elzárja a jobb sávot, a bal szabadon marad; egy forgalomfigyelő helikopter lebeg a városok és dugók felett.
- **Megbízások, gyakorlópálya, parkolók** — vegyél fel egy csomagot vagy utast, és tedd le a megállóban az idő lejárta előtt; az úton gyakorlópálya (lejtő, szlalomgyűrűk, rámpa) és hidakkal jelzett parkolók vannak, mindegyik bónusszal.
- **Feloldható szolgálati járművek** — rendőrautó (10 hibátlan város), mentő (8 kiszállítás), tűzoltóautó (3 napi kihívás és 4 pálya- vagy parkolóbónusz), helikopter (a másik három plusz 7 kihívás). Megszeghetik a szabályokat, a forgalom kitér, a helikopter átrepül a forgalom felett, lassan pedig leszáll.
- **Események és extrák** — 7, 10 vagy 15 szakaszos bajnokságok és egy ország nyolcszakaszos körútja (`--champ`, `--tour`, `TCC1-…` kód); heti kihívás váltakozó módosítóval (`--weekly`); szellemek importja és exportja (`--ghost`, `--ghost-export`); látható, 8–16 riválisból álló mezőny rajtrácssal, fordított rajtrácssal és időmérő körrel (`--pack`, `--inverted`, `--qualify`); 3–4 játékos osztott képernyőn gépi kitöltéssel (`--players`); elágazások (balra gyors, jobbra festői), ahol a helikopter mutatja az utat; üldözés a rendőrautóval; előzési lánc 200-tól 2000-ig; garázs, ahol az érmékkel fejlesztéseket vehetsz (`--garage`, `--buy`); zenei stílusok, mph, billentyűkiosztások és titkos fényezéskódok (`--music`, `--mph`, `--keys`, `--secret`); szirénák a rendőrautónak, a mentőnek és a tűzoltóautónak. Mindez a kezdőképernyő **EXTRAS** oldalán állítható (Backspace a járműválasztó képernyőn). Egy kódmező (⌨ a böngészőben) bajnokságot, szakaszt, szellemet, fényezés szót, DAILY-t vagy WEEKLY-t fogad.

## Korlátok

- A WebAssembly-változatot és a natív hangkimenetet (`--features audio`, cpal) a fejlesztői környezetben nem lehetett lefordítani és kipróbálni; az általuk burkolt logikát tesztek fedik. A szintetizált hangot számszerűen ellenőriztem, fülre nem.
- A pályák generáltak: a nehézség a seedtől függ (egy 70%-os gázzal vezető bot 60 szakaszból 60-et teljesít, 55%-nál is 60-at, de csak 0,6 s tartalékkal).
- Nem másolata egyetlen kereskedelmi játéknak sem, és nem áll kapcsolatban velük.
- Sebesség: a fejlesztői gépen egy natív release build kb. 13 ms/képkocka (99. percentilis: 23 ms), így lassabb gépeken 60 fps alá eshet; a böngészős változatot nem mértem.
- Országok: 196 szerepel (az ENSZ 193 tagja, plusz a Vatikán, Palesztina és Tajvan); az autókon a zászlószínek közelítőek, a menükben az országnevek csak angolul vannak (a webes választó a böngésző fordított neveit használja).
- Karrier: a riválisokat az autopilóta referenciaidejéből szimuláljuk. A nehézségi görbét szintetikus versenyzőkkel ellenőriztük (a referenciatempó megnyeri a bronz nemzeti kupát, de elveszti a gyémánt Világkupát), nem valódi játékosokkal, a weboldalt pedig csak szimulált motorral próbáltuk ki.
- A nehézségi szinteket, a 19 járműosztályt, az akadályokat és a bónuszokat csak szimulált vezetőkkel és automata tesztekkel hangoltam, valódi játékosokkal nem; a kompromisszumok hihetők, de nem kalibráltak. A sprite-ok egyszerű pixelrajzok, az ellenőrzőpont-felirat pedig minden szinten „+30”-at mutat.
- A webes változat a valódi WebAssembly motorral csak fej nélküli Chromiumban futott (valódi telefonokon nem), a natív hangot soha nem hallgattuk meg valódi eszközön, a natív játékvezérlőt (Linux, `/dev/input/js0`) pedig csak a dekóderén keresztül teszteltük, fizikai vezérlővel nem; más rendszereken a natív változatnak még nincs játékvezérlője.
- Az üzemanyag, a rendőrség, az állatok, a sérülések és a rally-riválisok egyensúlyát csak szimulált pilótákkal ellenőriztük, valódi játékosokkal nem.
- A város, a dugó, a kerülő és a segélyjárművek szabályait csak szimulált vezetőkkel hangoltuk és teszteltük, valódi játékosokkal nem.
- A megbízások, a gyakorlópálya és a parkolók egyensúlyát és a feloldási küszöböket csak szimulált pilótákkal hangoltam, valódi játékosokkal nem.
- Az új eseményeket és extrákat csak szimulált sofőrökkel és automatikus ellenőrzésekkel teszteltem: a hangot (zenei stílusok, szirénák) nem hallgattam meg, semmit sem próbáltam valódi játékvezérlőn vagy telefonon, az egyensúlyt nem játszották valódi emberek, az online ranglisták pedig hiányoznak, mert szerver kell hozzájuk.

## Licenc

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).
