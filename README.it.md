[English](README.md) · [Română](README.ro.md) · [Français](README.fr.md) · **Italiano** · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)

# **TURBOCURSA**

Versione: 1.5 · Ultimo aggiornamento: 12 novembre 2026

## Che cos'è

TURBOCURSA è un gioco di corse arcade pseudo-3D scritto in Rust in modalità clean-room: motore, grafica, musica e nome originali, senza codice, risorse o marchi di alcun gioco esistente. Tutto ciò che vedi e senti è generato da un seed: alberi, rocce, nuvole, nebulose, aurore e striscioni di Mandelbrot ai checkpoint, più colonna sonora e suono del motore procedurali. La libreria di base non ha dipendenze a runtime.

Diciannove paesaggi generati: foresta, deserto, tramonto, colossali alberi-mondo con la chioma tra le nuvole, mesa rosse stratificate, isole fluttuanti con cascate e un pianeta ad anelli, un lago specchio sotto due lune, una tundra con aurore e neve, un vulcano con vene di lava, pennacchio di fumo frattale e fulmini ricorsivi, un abisso illuminato con coralli e alghe, un cosmo con nebulosa di Julia e strada arcobaleno, una cittadella neon di piramidi di Sierpinski e blocchi di Menger, e una giungla di micelio luminosa. Ogni tappa concatena un paesaggio classico e tre più recenti, in un ordine dato dal seed. Altri sei mondi, possibili e impossibili: un mondo capovolto con una seconda catena montuosa appesa al cielo, un deserto-prisma sotto bande di aurore spettrali, un mondo a orologeria di ingranaggi giganti, un mare di nuvole, un buco nero con un anello di accrescimento deformato dalla lente gravitazionale e un canyon frattale a strati rigati.

## Uso

- Nativo: `cargo run --release --features native --bin turbo` (aggiungi `--versus` per 2 giocatori, `--daily` per la tappa del giorno, `--seed N` per una tappa fissa, `--lang xx` per la lingua).
- Web: `./build_web.sh` produce un solo file offline `dist/turbocursa.html` (comandi touch, selettore di lingua, miglior punteggio). Parametri: `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Comandi: frecce o WASD, Spazio = turbo, Invio = riprova, M = muto. Giocatore 2: WASD + Maiusc sinistro.
- Interfaccia e documentazione: English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. Il testo di gioco usa un font a pixel; gli accenti sono disegnati come un segno sopra la lettera.
- Modalità: `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (gara contro un rivale al computer), `--versus` (2 giocatori); web: `?mode=NOME`. · `--career` (divisioni, campionati e coppe, con salvataggio)

## Modalità di gioco

- **Campagna** — Batti il tempo attraverso quattro paesaggi generati; i checkpoint aggiungono tempo.
- **Cronometro** — Niente traffico, il tempo sale: arriva il più in fretta possibile.
- **Infinito** — Giri senza fine: ogni giro è una tappa generata nuova, +25 s.
- **Ora Di Punta** — Traffico triplo; ogni sorpasso regala 0,8 secondi.
- **Caccia Alle Sfere** — Segui scie frattali di sfere; ognuna aggiunge tempo e combo.
- **Slalom A Anelli** — Attraversa gli anelli: ogni passaggio alza il moltiplicatore, un errore costa un secondo.
- **Inversione** — Lo sterzo si inverte per sei secondi ogni quattordici.
- **Buio Totale** — Buio: si vede solo il cono dei fari, il paesaggio è una voce.
- **Zen** — Niente tempo, traffico o incidenti: guidi solo tra paesaggi infiniti.
- **Contro CPU** — Sfida un rivale guidato dal computer sulla stessa tappa, schermo diviso.
- **2 Giocatori** — Due umani, una tappa, schermo diviso: vince chi arriva prima.

## Carriera

Avvia con `--career` (web: `?mode=career`). I progressi si salvano da soli (file `~/.turbocursa_career` in nativo, `localStorage` nel browser).
- **Divisioni** — Bronzo, Argento, Oro, Platino, Diamante. Ogni stagione è un campionato di sei giornate con 12 piloti: i primi tre salgono, gli ultimi tre scendono; il campione del Diamante ottiene un invito alla Coppa del Mondo.
- **Coppe a eliminazione** — Il tuo tempo è confrontato con rivali simulati, più forti quanto più alta è la coppa e la divisione; i più lenti escono a ogni turno (16 → 8 → 4 → finale nella coppa nazionale; fino a 32 → 24 → 16 → 8 → 4 → finale nella Coppa del Mondo).
- **Qualificazioni multiple** — Due possibilità di *ripescaggio* per coppa: se esci, i migliori eliminati corrono di nuovo e i primi due rientrano.
- **La scala** — Raggiungi la finale della **coppa nazionale** per qualificarti alla **coppa continentale**; la sua finale apre la **coppa intercontinentale**; la sua finale, la **Coppa del Mondo**.
- **Ogni paese** — 196 paesi su sei continenti; puoi iscriverti anche alla coppa nazionale di qualsiasi altro paese (coppe all'estero) e conquistare un trofeo per ciascuno nella bacheca.
- **Classifiche** — una classifica mondiale di 300 piloti più te, la classifica delle nazioni, la tabella del campionato e le tue statistiche.

## Auto per paese

Ogni paese ha la sua auto da corsa: una livrea dipinta con i colori nazionali (approssimativi, arte originale) con targa, e un profilo di guida (velocità massima, accelerazione, tenuta, 2-4 cariche turbo). Il profilo è un budget a somma zero: nessun paese ha un'auto migliore in tutto. Si sceglie con `--car RO` (o `--car random`), `?car=RO` sul web, oppure si sfogliano nel Garage della carriera; in carriera si usa in automatico l'auto del tuo paese.

## Difficoltà, veicoli, ostacoli e bonus

- **5 livelli di difficoltà** — Facile, Normale, Difficile, Esperto, Folle: modificano il tempo iniziale e i bonus dei checkpoint (×1,25 … ×0,68), quantità e velocità del traffico, quanti ostacoli e bonus ci sono sulla strada e quanto costa uno scontro. In carriera il livello rende più forti o più deboli anche i rivali simulati.
- **19 classi di veicoli** per la tua auto: auto sportiva, berlina, citycar, camion, autoarticolato (TIR), motorino, motocicletta, trike, tuk-tuk, quad, buggy, monoposto, auto da rally, pick-up, furgone, autobus, trattore, limousine e monster truck. Ognuna mantiene livrea e profilo del paese e aggiunge il proprio compromesso: velocità massima, accelerazione, aderenza, larghezza, resistenza agli urti e cronometro. Le due ruote sono strette, agili e fragili; autobus, trattore e monster truck sono larghi e robusti; la monoposto è la più veloce e la più fragile.
- **Traffico misto**: auto, citycar, camion, autoarticolati, autobus, trattori, moto e trike; più il livello è alto, più veicoli pesanti.
- **Sulla strada**: ostacoli (coni, macchie d'olio, barriere, buche, pozzanghere, fango) e bonus (+4 s, turbo extra, spinta di velocità, scudo, monete, traffico fermo, punteggio ×2). Gli effetti attivi appaiono come barre colorate sotto il cronometro.
- **Schermata di selezione** all'inizio di ogni partita: ← → veicolo, ↑ ↓ difficoltà, Invio per partire, Backspace per tornarci. `--class` / `--level` / `--nopick` (nativo) o `?class=…&level=…` / i due selettori (web) la saltano; in carriera usa le righe *Difficoltà* / *Veicolo* dell'hub.

## Mondo, sistemi e strumenti

- **Meteo e fondi** — 11 meteo (sereno, pioggia, temporale, nebbia, neve, bufera di neve, tempesta di sabbia, grandine, ghiaccio nero, canicola, pioggia di cenere) e 6 fondi (asfalto, lago ghiacciato, distesa di sale, campo di lava, dune, palude) cambiano aderenza, visibilità e aspetto; lasciali su Auto o sceglili (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Giorno e notte** — alba, giorno, tramonto o notte (`--time`, `?time=`), con fari, stelle e grilli.
- **Vita lungo la strada** — ponti, gallerie, viadotti e passi di montagna; passaggi a livello con treni; animali; rampe e pedane turbo; polizia che insegue i centauri della velocità (se ti prendono, sei squalificato).
- **Carburante e danni** — il carburante cala e le stazioni lo rifanno; gli urti danneggiano carrozzeria e motore (meno velocità massima, accelerazione più debole, sterzo più molle) e le stazioni li riparano.
- **Cambio, gomme e assetto** — cambio automatico o manuale (E / Q cambiano marcia, T commuta); gomme strada, pioggia, neve e fuoristrada e quattro assetti (bilanciato, sprint, deportanza, economia): `--tyre`, `--setup`.
- **Rango 0–999 e gioco responsabile** — il rango cresce con l'esperienza e adatta la difficoltà (`--rank N` per forzarlo); si contano i giorni di fila, un promemoria suggerisce una pausa dopo una sessione lunga e un avviso dice quando per oggi basta.
- **Distintivi e collezioni** — 25 distintivi e una collezione di classi, meteo, fondi, momenti del giorno e paesi guidati.
- **Sfida del giorno** — `--daily` / `?daily`: stessa tappa, stesso veicolo e stesse condizioni per tutti, con un breve codice del risultato da confrontare.
- **Fantasma** — la tua corsa migliore su una tappa viene riproposta come fantasma ciano con il distacco (salvato in `~/.turbocursa_ghost_*` o `localStorage`; utile soprattutto con seme fisso o con la tappa del giorno).
- **Rally** — `--rally` / `?mode=rally`: cinque tappe dall'alba alla notte con meteo e fondo variabili; danni, carburante e usura delle gomme restano; i minuti nel parco assistenza si sommano al tempo; la classifica finale è calcolata contro 15 equipaggi simulati.
- **Modalità foto** — P telecamera libera (le frecce la muovono), H mostra o nasconde l'HUD, C salva un PNG (`turbocursa_photo_N.png`; nel browser viene scaricato).
- **Comandi** — tastiera, pulsanti touch, gamepad nel browser (stick o croce direzionale sterzano, RT/A gas, LT/B freno, X turbo, Start invio, Select indietro, LB/RB marce, Y manuale) e sterzo a inclinazione sul telefono (pulsante ⟲).
- **Accessibilità** — filtri per il daltonismo (`--colorblind protan|deutan|tritan|contrast`), aiuti che tengono la curva, ricentrano l'auto e ammorbidiscono i muri (`--assist 1..3`), modalità lenta (`--slow 70` = 70% della velocità massima) e interfaccia grande (`--large`).
- **Suono** — motore procedurale con marce, vento, pioggia, temporale, grandine, sirene, clacson del treno, ghiaccio, bip di carburante basso, motore danneggiato e versi di animali.
- **Progettista di tappe e codici** — `--design` (web: `?mode=design`): scegli seme, modalità, veicolo, livello, meteo, fondo, ora, i quattro paesaggi, quanto è tortuosa e collinosa la strada e quanto è lunga; un grafico mostra il profilo. Ogni tappa ha un breve codice con controllo `TCS1-…`: guidala con `--code CODICE` o `?code=CODICE`; nella versione web premi L per mostrarlo e copiare un link.
- **Città e codice della strada** — ogni tappa attraversa città con viali e strade: facciate, marciapiedi, strisce pedonali, semafori (verde, giallo, rosso), pedoni e limite di velocità. Passare col rosso, superare il limite o non dare la precedenza a un pedone chiama la polizia (in Easy una multa costa tempo); investire un pedone costa tempo e danni.
- **Guidare con responsabilità paga** — fermarsi al rosso, dare la precedenza, rispettare il limite e attraversare una città senza infrazioni fruttano punti, tempo e un moltiplicatore fino a x3 («BUON AUTISTA», «CITTÀ PULITA»); ogni infrazione azzera la serie. Lasciar passare un'ambulanza o un'autopompa con la sirena accesa dà un bonus, ostacolarla è un'infrazione.
- **Code, deviazioni e RadarMap** — alcune tappe hanno una coda: una fila ferma con un corridoio sinuoso da percorrere piano (urtarla è un incidente). Una corsia d'uscita a destra prima della coda porta su una strada libera; il RadarMap nell'HUD (mappa con la direzione in alto) mostra la strada davanti, le città, la coda in rosso, la deviazione in verde, i semafori e il tempo di attraversamento sulla strada principale rispetto all'uscita.
- **Veicoli di soccorso ed elicotteri** — ambulanze e autopompe sorpassano coi lampeggianti accesi; una scena d'incidente (un elicottero che atterra sulla strada mentre ti avvicini, un'ambulanza e un'autopompa) blocca la corsia di destra e lascia libera quella di sinistra; un elicottero di monitoraggio del traffico sorvola città e code.
- **Missioni, campo prove, parcheggi** — carica un pacco o un passeggero e lascialo nella baia prima che scada il tempo; il percorso ha un campo prove (pendenza, anelli di slalom, rampa) e parcheggi annunciati da portali, ognuno con un bonus.
- **Veicoli di servizio da sbloccare** — auto della polizia (10 città senza errori), ambulanza (8 consegne), autopompa (3 sfide del giorno e 4 bonus di campo prove o parcheggio), elicottero (gli altri tre più 7 sfide). Possono violare le regole, il traffico si sposta, e l'elicottero sorvola il traffico e atterra a bassa velocità.
- **Eventi ed extra** — campionati di 7, 10 o 15 tappe e tour di otto tappe di un paese (`--champ`, `--tour`, codice `TCC1-…`); sfida settimanale con un modificatore a rotazione (`--weekly`); import ed export dei fantasmi (`--ghost`, `--ghost-export`); gruppo visibile di 8–16 rivali con griglia, griglia invertita e giro di qualifica (`--pack`, `--inverted`, `--qualify`); schermo diviso per 3–4 giocatori con completamento CPU (`--players`); bivi (veloce a sinistra, panoramico a destra) con l'elicottero che indica la strada; inseguimento con l'auto della polizia; catena di sorpassi da 200 a 2000; garage dove le monete comprano potenziamenti (`--garage`, `--buy`); stili musicali, mph, schemi di tasti e codici segreti di vernice (`--music`, `--mph`, `--keys`, `--secret`); sirene per l'auto della polizia, l'ambulanza e l'autopompa. Tutto si imposta dalla pagina **EXTRAS** della schermata iniziale (Backspace sulla schermata del veicolo). Un campo codice (⌨ nel browser) accetta un campionato, una tappa, un fantasma, una parola di vernice, DAILY o WEEKLY.

## Limiti

- La versione WebAssembly e l'uscita audio nativa (`--features audio`, cpal) non hanno potuto essere compilate né provate nell'ambiente di sviluppo; la logica che incapsulano è coperta da test. Il suono sintetizzato è stato verificato numericamente, non all'orecchio.
- I tracciati sono generati: la difficoltà varia col seed (un bot al 70% di acceleratore completa 60 tappe su 60, al 55% anch'esso 60 su 60, ma con soli 0,6 s di margine).
- Non è la copia di alcun gioco commerciale e non è affiliato ad alcuno.
- Velocità: sulla macchina di sviluppo una build nativa release impiega circa 13 ms per frame (99° percentile: 23 ms), quindi macchine più lente possono scendere sotto i 60 fps; la versione browser non è stata misurata.
- Paesi: ne sono inclusi 196 (i 193 membri ONU più Vaticano, Palestina e Taiwan); i colori delle bandiere sulle auto sono approssimativi e i nomi dei paesi nei menu sono solo in inglese (il selettore web usa i nomi tradotti dal browser).
- Carriera: i rivali sono simulati a partire dal tempo di riferimento del pilota automatico. La curva di difficoltà è stata verificata con piloti sintetici (il ritmo di riferimento vince la coppa nazionale in Bronzo e perde la Coppa del Mondo in Diamante), non con giocatori reali, e la pagina web è stata provata solo con un motore simulato.
- Livelli di difficoltà, le 19 classi di veicoli, ostacoli e bonus sono stati bilanciati solo con piloti simulati e test automatici, non con giocatori reali; i compromessi sono plausibili, non calibrati. Gli sprite sono semplici disegni a pixel e il banner del checkpoint mostra ancora «+30» a ogni livello.
- La versione web è stata eseguita con il vero motore WebAssembly solo in Chromium headless (non su telefoni reali), l'audio nativo non è mai stato ascoltato su un dispositivo reale e il gamepad nativo (Linux, `/dev/input/js0`) è stato provato solo tramite il suo decodificatore, non con un gamepad fisico; sugli altri sistemi la versione nativa non ha ancora il gamepad.
- Il bilanciamento di carburante, polizia, animali, danni e rivali del rally è stato verificato solo con piloti simulati, non con giocatori reali.
- Le regole di città, coda, deviazione e veicoli di soccorso sono state bilanciate e provate solo con piloti simulati, non con giocatori reali.
- L'equilibrio di missioni, campo prove e parcheggi e le soglie di sblocco sono stati tarati solo con piloti simulati, non con giocatori reali.
- I nuovi eventi ed extra sono stati provati solo con piloti simulati e controlli automatici: l'audio (stili musicali, sirene) non è stato ascoltato, nulla è stato provato su un gamepad fisico o su un telefono, il bilanciamento non è stato giocato da persone reali e le classifiche online mancano perché richiedono un server.

## Licenza

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).
