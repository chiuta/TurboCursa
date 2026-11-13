//! Interface language for the in-game HUD (8 languages, as on chiuta.ro).
//! Strings are upper-case for the 3x5 pixel font; diacritics are drawn as an accent pixel (see font.rs).
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En = 0,
    Ro,
    Fr,
    It,
    Es,
    Pt,
    De,
    Hu,
}

pub const LANGS: [(Lang, &str, &str); 8] = [
    (Lang::En, "en", "English"),
    (Lang::Ro, "ro", "Română"),
    (Lang::Fr, "fr", "Français"),
    (Lang::It, "it", "Italiano"),
    (Lang::Es, "es", "Español"),
    (Lang::Pt, "pt", "Português"),
    (Lang::De, "de", "Deutsch"),
    (Lang::Hu, "hu", "Magyar"),
];

static CUR: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    CUR.store(l as u8, Ordering::Relaxed);
}

pub fn lang() -> Lang {
    LANGS[(CUR.load(Ordering::Relaxed) as usize).min(7)].0
}

impl Lang {
    /// "ro", "ro_RO.UTF-8", "pt-BR", "HU" ... -> Lang
    pub fn parse(s: &str) -> Option<Lang> {
        let p = s.trim().to_ascii_lowercase();
        let p = p.split(['_', '-', '.']).next().unwrap_or("");
        LANGS.iter().find(|(_, c, _)| *c == p).map(|(l, _, _)| *l)
    }
    pub fn from_index(i: u32) -> Lang {
        LANGS[(i as usize).min(7)].0
    }
}

/// Message keys.
#[derive(Clone, Copy)]
pub enum S {
    Time,
    Kmh,
    Turbo,
    TimeUp,
    Score,
    PressEnter,
    StageClear,
    Passed,
    Winner,
    SecondPlace,
    Draw,
    Rematch,
    First,
    Second,
    Go,
    Checkpoint,
    StageClearBang,
    Lap,
    Orbs,
    Combo,
    Inverted,
    Normal,
    Ring,
}

const N: usize = 23;
type Row = [&'static str; N];

// order: Time Kmh Turbo TimeUp Score PressEnter StageClear Passed({} = count) Winner SecondPlace Draw Rematch First Second Go Checkpoint StageClearBang
const EN: Row = [
    "TIME",
    "KM/H",
    "TURBO",
    "TIME UP",
    "SCORE",
    "PRESS ENTER",
    "STAGE CLEAR",
    "PASSED {} CARS",
    "WINNER!",
    "2ND PLACE",
    "DRAW",
    "ENTER: REMATCH",
    "1ST",
    "2ND",
    "GO!",
    "CHECKPOINT",
    "STAGE CLEAR!",
    "LAP",
    "ORBS",
    "COMBO",
    "CONTROLS INVERTED!",
    "NORMAL",
    "RING!",
];
const RO: Row = [
    "TIMP",
    "KM/H",
    "TURBO",
    "TIMP EXPIRAT",
    "SCOR",
    "APASĂ ENTER",
    "ETAPĂ TERMINATĂ",
    "AI DEPĂȘIT {} MAȘINI",
    "CÂȘTIGĂTOR!",
    "LOCUL 2",
    "EGALITATE",
    "ENTER: REVANȘĂ",
    "LOC 1",
    "LOC 2",
    "START!",
    "PUNCT DE CONTROL",
    "ETAPĂ TERMINATĂ!",
    "TURA",
    "ORBE",
    "COMBO",
    "COMENZI INVERSATE!",
    "NORMAL",
    "INEL!",
];
const FR: Row = [
    "TEMPS",
    "KM/H",
    "TURBO",
    "TEMPS ÉCOULÉ",
    "SCORE",
    "APPUYEZ SUR ENTRÉE",
    "ÉTAPE TERMINÉE",
    "{} VOITURES DÉPASSÉES",
    "GAGNANT!",
    "2E PLACE",
    "ÉGALITÉ",
    "ENTRÉE: REVANCHE",
    "1ER",
    "2E",
    "DÉPART!",
    "POINT DE CONTRÔLE",
    "ÉTAPE TERMINÉE!",
    "TOUR",
    "ORBES",
    "COMBO",
    "COMMANDES INVERSÉES!",
    "NORMAL",
    "ANNEAU!",
];
const IT: Row = [
    "TEMPO",
    "KM/H",
    "TURBO",
    "TEMPO SCADUTO",
    "PUNTI",
    "PREMI INVIO",
    "TAPPA COMPLETATA",
    "{} AUTO SUPERATE",
    "VINCITORE!",
    "SECONDO POSTO",
    "PAREGGIO",
    "INVIO: RIVINCITA",
    "PRIMO",
    "SECONDO",
    "VIA!",
    "CHECKPOINT",
    "TAPPA COMPLETATA!",
    "GIRO",
    "SFERE",
    "COMBO",
    "COMANDI INVERTITI!",
    "NORMALE",
    "ANELLO!",
];
const ES: Row = [
    "TIEMPO",
    "KM/H",
    "TURBO",
    "TIEMPO AGOTADO",
    "PUNTOS",
    "PULSA ENTER",
    "ETAPA COMPLETADA",
    "{} COCHES ADELANTADOS",
    "GANADOR!",
    "SEGUNDO PUESTO",
    "EMPATE",
    "ENTER: REVANCHA",
    "PRIMERO",
    "SEGUNDO",
    "YA!",
    "PUNTO DE CONTROL",
    "ETAPA COMPLETADA!",
    "VUELTA",
    "ORBES",
    "COMBO",
    "CONTROLES INVERTIDOS!",
    "NORMAL",
    "ANILLO!",
];
const PT: Row = [
    "TEMPO",
    "KM/H",
    "TURBO",
    "TEMPO ESGOTADO",
    "PONTOS",
    "PRESSIONE ENTER",
    "ETAPA CONCLUÍDA",
    "{} CARROS ULTRAPASSADOS",
    "VENCEDOR!",
    "SEGUNDO LUGAR",
    "EMPATE",
    "ENTER: REVANCHE",
    "PRIMEIRO",
    "SEGUNDO",
    "JÁ!",
    "PONTO DE CONTROLO",
    "ETAPA CONCLUÍDA!",
    "VOLTA",
    "ESFERAS",
    "COMBO",
    "COMANDOS INVERTIDOS!",
    "NORMAL",
    "ANEL!",
];
const DE: Row = [
    "ZEIT",
    "KM/H",
    "TURBO",
    "ZEIT ABGELAUFEN",
    "PUNKTE",
    "ENTER DRÜCKEN",
    "ETAPPE GESCHAFFT",
    "{} AUTOS ÜBERHOLT",
    "SIEGER!",
    "PLATZ 2",
    "UNENTSCHIEDEN",
    "ENTER: REVANCHE",
    "PLATZ 1",
    "PLATZ 2",
    "LOS!",
    "KONTROLLPUNKT",
    "ETAPPE GESCHAFFT!",
    "RUNDE",
    "ORBEN",
    "KOMBO",
    "STEUERUNG UMGEKEHRT!",
    "NORMAL",
    "RING!",
];
const HU: Row = [
    "IDŐ",
    "KM/H",
    "TURBÓ",
    "LEJÁRT AZ IDŐ",
    "PONT",
    "NYOMJ ENTERT",
    "SZAKASZ TELJESÍTVE",
    "{} AUTÓ ELŐZVE",
    "GYŐZTES!",
    "2. HELY",
    "DÖNTETLEN",
    "ENTER: ÚJRA",
    "1. HELY",
    "2. HELY",
    "RAJT!",
    "ELLENŐRZŐPONT",
    "SZAKASZ TELJESÍTVE!",
    "KÖR",
    "GÖMB",
    "KOMBÓ",
    "FORDÍTOTT IRÁNYÍTÁS!",
    "NORMÁL",
    "GYŰRŰ!",
];

const TABLE: [&Row; 8] = [&EN, &RO, &FR, &IT, &ES, &PT, &DE, &HU];

pub fn tr_in(l: Lang, k: S) -> &'static str {
    TABLE[l as usize][k as usize]
}

pub fn tr(k: S) -> &'static str {
    tr_in(lang(), k)
}

/// "PASSED {} CARS" with the number filled in.
pub fn passed(n: u32) -> String {
    tr(S::Passed).replace("{}", &n.to_string())
}

/// Game banners are stored as stable English keys; map them to the current language.
pub fn banner(key: &str) -> &str {
    match key {
        "GO!" => tr(S::Go),
        "CHECKPOINT" => tr(S::Checkpoint),
        "STAGE CLEAR!" => tr(S::StageClearBang),
        "LAP" => tr(S::Lap),
        "INVERT" => tr(S::Inverted),
        "NORMAL" => tr(S::Normal),
        "RING" => tr(S::Ring),
        other => ITEM_BANNERS
            .iter()
            .find(|(k, _)| *k == other)
            .map_or(other, |(_, v)| v[(lang() as usize).min(7)]),
    }
}

/// Item banner in a given language (the same table `banner` uses); `None` for an unknown key.
pub fn item_banner_in(l: Lang, key: &str) -> Option<&'static str> {
    ITEM_BANNERS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v[(l as usize).min(7)])
}

/// Banners for obstacles and bonuses: en, ro, fr, it, es, pt, de, hu.
const ITEM_BANNERS: [(&str, [&str; 8]); 101] = [
    (
        "ENTER CODE",
        [
            "CODE", "COD", "CODE", "CODICE", "CÓDIGO", "CÓDIGO", "CODE", "KÓD",
        ],
    ),
    (
        "CODE HINT",
        [
            "TYPE A CODE",
            "SCRIE UN COD",
            "SAISIR UN CODE",
            "SCRIVI UN CODICE",
            "ESCRIBE UN CÓDIGO",
            "ESCREVA UM CÓDIGO",
            "CODE EINGEBEN",
            "ÍRJ BE EGY KÓDOT",
        ],
    ),
    (
        "CODE OK",
        [
            "CODE ACCEPTED",
            "COD ACCEPTAT",
            "CODE ACCEPTÉ",
            "CODICE ACCETTATO",
            "CÓDIGO ACEPTADO",
            "CÓDIGO ACEITE",
            "CODE AKZEPTIERT",
            "KÓD ELFOGADVA",
        ],
    ),
    (
        "CODE INVALID",
        [
            "INVALID CODE",
            "COD INVALID",
            "CODE INVALIDE",
            "CODICE NON VALIDO",
            "CÓDIGO NO VÁLIDO",
            "CÓDIGO INVÁLIDO",
            "UNGÜLTIGER CODE",
            "ÉRVÉNYTELEN KÓD",
        ],
    ),
    (
        "SOLO RACE",
        [
            "SOLO RACE",
            "CURSĂ SOLO",
            "COURSE SOLO",
            "GARA SINGOLA",
            "CARRERA SOLO",
            "CORRIDA SOLO",
            "EINZELRENNEN",
            "EGYÉNI FUTAM",
        ],
    ),
    (
        "VERSUS MODES",
        [
            "VERSUS MODES",
            "MODURI VERSUS",
            "MODES VERSUS",
            "MODI VERSUS",
            "MODOS VERSUS",
            "MODOS VERSUS",
            "VERSUS-MODI",
            "VERSUS MÓDOK",
        ],
    ),
    (
        "NATIVE ONLY",
        [
            "NATIVE ONLY",
            "DOAR NATIV",
            "NATIF SEULEMENT",
            "SOLO NATIVO",
            "SOLO NATIVO",
            "SÓ NATIVO",
            "NUR NATIV",
            "CSAK NATÍV",
        ],
    ),
    (
        "STOCK MODE",
        [
            "STOCK MODE",
            "MOD STANDARD",
            "MODE STANDARD",
            "MODO STANDARD",
            "MODO ESTÁNDAR",
            "MODO PADRÃO",
            "STANDARDMODUS",
            "ALAPMÓD",
        ],
    ),
    (
        "BACK KEY",
        [
            "BACK", "ÎNAPOI", "RETOUR", "INDIETRO", "VOLVER", "VOLTAR", "ZURÜCK", "VISSZA",
        ],
    ),
    (
        "ROMANIA",
        [
            "ROMANIA",
            "ROMÂNIA",
            "ROUMANIE",
            "ROMANIA",
            "RUMANÍA",
            "ROMÉNIA",
            "RUMÄNIEN",
            "ROMÁNIA",
        ],
    ),
    (
        "FRANCE",
        [
            "FRANCE",
            "FRANȚA",
            "FRANCE",
            "FRANCIA",
            "FRANCIA",
            "FRANÇA",
            "FRANKREICH",
            "FRANCIAORSZÁG",
        ],
    ),
    (
        "ITALY",
        [
            "ITALY",
            "ITALIA",
            "ITALIE",
            "ITALIA",
            "ITALIA",
            "ITÁLIA",
            "ITALIEN",
            "OLASZORSZÁG",
        ],
    ),
    (
        "SPAIN",
        [
            "SPAIN",
            "SPANIA",
            "ESPAGNE",
            "SPAGNA",
            "ESPAÑA",
            "ESPANHA",
            "SPANIEN",
            "SPANYOLORSZÁG",
        ],
    ),
    (
        "PORTUGAL",
        [
            "PORTUGAL",
            "PORTUGALIA",
            "PORTUGAL",
            "PORTOGALLO",
            "PORTUGAL",
            "PORTUGAL",
            "PORTUGAL",
            "PORTUGÁLIA",
        ],
    ),
    (
        "GERMANY",
        [
            "GERMANY",
            "GERMANIA",
            "ALLEMAGNE",
            "GERMANIA",
            "ALEMANIA",
            "ALEMANHA",
            "DEUTSCHLAND",
            "NÉMETORSZÁG",
        ],
    ),
    (
        "HUNGARY",
        [
            "HUNGARY",
            "UNGARIA",
            "HONGRIE",
            "UNGHERIA",
            "HUNGRÍA",
            "HUNGRIA",
            "UNGARN",
            "MAGYARORSZÁG",
        ],
    ),
    (
        "FINLAND",
        [
            "FINLAND",
            "FINLANDA",
            "FINLANDE",
            "FINLANDIA",
            "FINLANDIA",
            "FINLÂNDIA",
            "FINNLAND",
            "FINNORSZÁG",
        ],
    ),
    (
        "EXTRAS",
        [
            "EXTRAS", "EXTRA", "EXTRAS", "EXTRA", "EXTRAS", "EXTRAS", "EXTRAS", "EXTRÁK",
        ],
    ),
    (
        "ON",
        [
            "ON", "PORNIT", "ACTIVÉ", "ATTIVO", "SÍ", "LIGADO", "AN", "BE",
        ],
    ),
    (
        "OFF",
        ["OFF", "OPRIT", "NON", "NO", "NO", "NÃO", "AUS", "KI"],
    ),
    (
        "MAX",
        [
            "MAX", "MAXIM", "MAX", "MASSIMO", "MÁXIMO", "MÁXIMO", "MAX", "MAX",
        ],
    ),
    (
        "MUSIC",
        [
            "MUSIC", "MUZICĂ", "MUSIQUE", "MUSICA", "MÚSICA", "MÚSICA", "MUSIK", "ZENE",
        ],
    ),
    (
        "MUSIC NORMAL",
        [
            "NORMAL", "NORMALĂ", "NORMALE", "NORMALE", "NORMAL", "NORMAL", "NORMAL", "NORMÁL",
        ],
    ),
    (
        "MUSIC CALM",
        [
            "CALM",
            "LINIȘTITĂ",
            "CALME",
            "CALMA",
            "CALMA",
            "CALMA",
            "RUHIG",
            "NYUGODT",
        ],
    ),
    (
        "MUSIC DRIVE",
        [
            "DRIVE", "DE DRUM", "ROUTE", "STRADA", "RUTA", "ESTRADA", "FAHRT", "ÚTON",
        ],
    ),
    (
        "SPEED UNIT",
        [
            "SPEED UNIT",
            "UNITATE VITEZĂ",
            "UNITÉ VITESSE",
            "UNITÀ VELOCITÀ",
            "UNIDAD VELOCIDAD",
            "UNIDADE VELOCIDADE",
            "TEMPOEINHEIT",
            "SEBESSÉG EGYSÉGE",
        ],
    ),
    (
        "KEYS",
        [
            "KEYS",
            "TASTE",
            "TOUCHES",
            "TASTI",
            "TECLAS",
            "TECLAS",
            "TASTEN",
            "BILLENTYŰK",
        ],
    ),
    (
        "PACK",
        [
            "PACK", "PLUTON", "PELOTON", "GRUPPO", "PELOTÓN", "PELOTÃO", "FELD", "MEZŐNY",
        ],
    ),
    (
        "INVERTED GRID",
        [
            "INVERTED GRID",
            "GRILĂ INVERSATĂ",
            "GRILLE INVERSÉE",
            "GRIGLIA INVERTITA",
            "PARRILLA INVERTIDA",
            "GRELHA INVERTIDA",
            "UMGEKEHRTE AUFSTELLUNG",
            "FORDÍTOTT RAJTRÁCS",
        ],
    ),
    (
        "PLAYERS",
        [
            "PLAYERS",
            "JUCĂTORI",
            "JOUEURS",
            "GIOCATORI",
            "JUGADORES",
            "JOGADORES",
            "SPIELER",
            "JÁTÉKOSOK",
        ],
    ),
    (
        "GARAGE",
        [
            "GARAGE", "GARAJ", "GARAGE", "GARAGE", "GARAJE", "GARAGEM", "GARAGE", "GARÁZS",
        ],
    ),
    (
        "ENGINE",
        [
            "ENGINE", "MOTOR", "MOTEUR", "MOTORE", "MOTOR", "MOTOR", "MOTOR", "MOTOR",
        ],
    ),
    (
        "GEARBOX",
        [
            "GEARBOX",
            "CUTIE DE VITEZE",
            "BOÎTE DE VITESSES",
            "CAMBIO",
            "CAJA DE CAMBIOS",
            "CAIXA DE VELOCIDADES",
            "GETRIEBE",
            "SEBESSÉGVÁLTÓ",
        ],
    ),
    (
        "TYRES",
        [
            "TYRES",
            "ANVELOPE",
            "PNEUS",
            "PNEUMATICI",
            "NEUMÁTICOS",
            "PNEUS",
            "REIFEN",
            "GUMIK",
        ],
    ),
    (
        "TURBO",
        [
            "TURBO", "TURBO", "TURBO", "TURBO", "TURBO", "TURBO", "TURBO", "TURBÓ",
        ],
    ),
    (
        "EXTRAS HINT",
        [
            "ARROWS CHANGE · ENTER BUYS · BACKSPACE BACK",
            "← → SCHIMBĂ · ENTER CUMPĂRĂ · BACKSPACE ÎNAPOI",
            "← → CHANGER · ENTRÉE ACHÈTE · RETOUR ARRIÈRE",
            "← → CAMBIA · INVIO ACQUISTA · INDIETRO",
            "← → CAMBIAR · INTRO COMPRA · VOLVER",
            "← → ALTERAR · ENTER COMPRA · VOLTAR",
            "← → ÄNDERN · ENTER KAUFT · ZURÜCK",
            "← → VÁLTÁS · ENTER VÁSÁROL · VISSZA",
        ],
    ),
    (
        "COINS",
        [
            "COINS", "MONEDE", "PIÈCES", "MONETE", "MONEDAS", "MOEDAS", "MÜNZEN", "ÉRMÉK",
        ],
    ),
    (
        "POS",
        [
            "POS", "LOC.", "POS.", "POS.", "POS.", "POS.", "PLATZ", "HELY",
        ],
    ),
    (
        "PODIUM!",
        [
            "PODIUM!", "PODIUM!", "PODIUM !", "PODIO!", "¡PODIO!", "PÓDIO!", "PODEST!", "DOBOGÓ!",
        ],
    ),
    (
        "FORK AHEAD",
        [
            "FORK AHEAD",
            "BIFURCAȚIE",
            "FOURCHE",
            "BIVIO",
            "BIFURCACIÓN",
            "BIFURCAÇÃO",
            "GABELUNG",
            "ELÁGAZÁS",
        ],
    ),
    (
        "QUICK ROUTE",
        [
            "QUICK ROUTE",
            "RUTĂ RAPIDĂ",
            "ROUTE RAPIDE",
            "PERCORSO VELOCE",
            "RUTA RÁPIDA",
            "ROTA RÁPIDA",
            "SCHNELLE ROUTE",
            "GYORS ÚT",
        ],
    ),
    (
        "SCENIC ROUTE",
        [
            "SCENIC ROUTE",
            "RUTĂ PANORAMICĂ",
            "ROUTE PANORAMIQUE",
            "PERCORSO PANORAMICO",
            "RUTA PANORÁMICA",
            "ROTA PANORÂMICA",
            "PANORAMAROUTE",
            "PANORÁMAÚT",
        ],
    ),
    (
        "CATCH UP",
        [
            "CATCH UP",
            "ajunge-l!",
            "RATTRAPE-LE !",
            "RAGGIUNGILO!",
            "¡ALCÁNZALO!",
            "ALCANCE-O!",
            "HOL EIN!",
            "érd utol!",
        ],
    ),
    (
        "RAM HIM!",
        [
            "RAM HIM!",
            "LOVEȘTE-L!",
            "ÉPERONNE-LE !",
            "SPERONALO!",
            "¡EMBÍSTELO!",
            "ABALROE-O!",
            "RAMME IHN!",
            "ÜTKÖZZ BELE!",
        ],
    ),
    (
        "ARRESTED!",
        [
            "ARRESTED!",
            "ARESTAT!",
            "ARRÊTÉ !",
            "ARRESTATO!",
            "¡ARRESTADO!",
            "PRESO!",
            "FESTGENOMMEN!",
            "ELKAPVA!",
        ],
    ),
    (
        "ESCAPED!",
        [
            "ESCAPED!",
            "A SCĂPAT!",
            "ÉVADÉ !",
            "EVASO!",
            "¡HA ESCAPADO!",
            "FUGIU!",
            "ENTKOMMEN!",
            "ELMENEKÜLT!",
        ],
    ),
    (
        "CHAIN",
        [
            "CHAIN",
            "LANȚ",
            "CHAÎNE",
            "SERIE",
            "RACHA",
            "SEQUÊNCIA",
            "KETTE",
            "SOROZAT",
        ],
    ),
    (
        "PROVING GROUND",
        [
            "PROVING GROUND",
            "POLIGON",
            "PISTE D'ESSAI",
            "POLIGONO",
            "POLÍGONO",
            "POLÍGONO",
            "TESTGELÄNDE",
            "PRÓBAPÁLYA",
        ],
    ),
    (
        "PARKING BONUS",
        [
            "PARKING BONUS",
            "BONUS PARCARE",
            "BONUS PARKING",
            "BONUS PARCHEGGIO",
            "BONO APARCAR",
            "BÓNUS ESTACIONAR",
            "PARK-BONUS",
            "PARKOLÓ BÓNUSZ",
        ],
    ),
    (
        "PARKED!",
        [
            "PARKED!",
            "PARCAT!",
            "GARÉ !",
            "PARCHEGGIATO!",
            "APARCADO!",
            "ESTACIONADO!",
            "GEPARKT!",
            "LEPARKOLT!",
        ],
    ),
    (
        "PERFECT PARKING!",
        [
            "PERFECT PARKING!",
            "PARCARE PERFECTĂ!",
            "CRÉNEAU PARFAIT !",
            "PARCHEGGIO PERFETTO!",
            "APARCADO PERFECTO!",
            "ESTACIONAMENTO PERFEITO!",
            "PERFEKT GEPARKT!",
            "TÖKÉLETES PARKOLÁS!",
        ],
    ),
    (
        "TRACK CLEAR!",
        [
            "TRACK CLEAR!",
            "POLIGON TERMINAT!",
            "PISTE RÉUSSIE !",
            "PISTA COMPLETA!",
            "PISTA SUPERADA!",
            "PISTA CONCLUÍDA!",
            "STRECKE GESCHAFFT!",
            "PÁLYA TELJESÍTVE!",
        ],
    ),
    (
        "PICK UP",
        [
            "PICK UP",
            "PRELUARE",
            "À RÉCUPÉRER",
            "RITIRO",
            "RECOGER",
            "RECOLHA",
            "ABHOLEN",
            "FELVÉTEL",
        ],
    ),
    (
        "DROP OFF",
        [
            "DROP OFF",
            "LIVRARE",
            "À LIVRER",
            "CONSEGNA",
            "ENTREGAR",
            "ENTREGA",
            "ABLIEFERN",
            "LEADÁS",
        ],
    ),
    (
        "PARCEL LOADED",
        [
            "PARCEL LOADED",
            "COLET ÎNCĂRCAT",
            "COLIS CHARGÉ",
            "PACCO CARICATO",
            "PAQUETE CARGADO",
            "ENCOMENDA CARREGADA",
            "PAKET GELADEN",
            "CSOMAG FELVÉVE",
        ],
    ),
    (
        "PASSENGER ON BOARD",
        [
            "PASSENGER ON BOARD",
            "PASAGER LA BORD",
            "PASSAGER À BORD",
            "PASSEGGERO A BORDO",
            "PASAJERO A BORDO",
            "PASSAGEIRO A BORDO",
            "FAHRGAST AN BORD",
            "UTAS A FEDÉLZETEN",
        ],
    ),
    (
        "DELIVERED!",
        [
            "DELIVERED!",
            "LIVRAT!",
            "LIVRÉ !",
            "CONSEGNATO!",
            "ENTREGADO!",
            "ENTREGUE!",
            "GELIEFERT!",
            "LEADVA!",
        ],
    ),
    (
        "TOO LATE",
        [
            "TOO LATE",
            "PREA TÂRZIU",
            "TROP TARD",
            "TROPPO TARDI",
            "DEMASIADO TARDE",
            "TARDE DEMAIS",
            "ZU SPÄT",
            "TÚL KÉSŐN",
        ],
    ),
    (
        "MISSED",
        [
            "MISSED", "RATAT", "MANQUÉ", "PERSO", "PERDIDO", "PERDIDO", "VERPASST", "ELMARADT",
        ],
    ),
    (
        "EMERGENCY",
        [
            "EMERGENCY",
            "URGENȚĂ",
            "URGENCE",
            "EMERGENZA",
            "EMERGENCIA",
            "EMERGÊNCIA",
            "NOTFALL",
            "SÜRGŐS",
        ],
    ),
    (
        "ACCIDENT",
        [
            "ACCIDENT",
            "ACCIDENT",
            "ACCIDENT",
            "INCIDENTE",
            "ACCIDENTE",
            "ACIDENTE",
            "UNFALL",
            "BALESET",
        ],
    ),
    (
        "MOVE OVER!",
        [
            "MOVE OVER!",
            "FĂ LOC!",
            "RANGEZ-VOUS !",
            "FATE SPAZIO!",
            "APÁRTATE!",
            "ABRA PASSAGEM!",
            "PLATZ MACHEN!",
            "HELYET!",
        ],
    ),
    (
        "MAIN",
        [
            "MAIN", "DRUM", "PRINC.", "PRINC.", "PRINC.", "PRINC.", "HAUPT", "FŐÚT",
        ],
    ),
    (
        "EXIT",
        [
            "EXIT", "IEȘIRE", "SORTIE", "USCITA", "SALIDA", "SAÍDA", "AUSFAHRT", "KIHAJTÓ",
        ],
    ),
    (
        "CITY",
        [
            "CITY", "ORAȘ", "VILLE", "CITTÀ", "CIUDAD", "CIDADE", "STADT", "VÁROS",
        ],
    ),
    (
        "SPEEDING!",
        [
            "SPEEDING!",
            "VITEZĂ!",
            "EXCÈS !",
            "TROPPO VELOCE!",
            "EXCESO!",
            "EXCESSO!",
            "ZU SCHNELL!",
            "TÚLSEBESSÉG!",
        ],
    ),
    (
        "RED LIGHT!",
        [
            "RED LIGHT!",
            "ROȘU!",
            "ROUGE !",
            "ROSSO!",
            "ROJO!",
            "VERMELHO!",
            "ROT!",
            "PIROS!",
        ],
    ),
    (
        "PEDESTRIAN!",
        [
            "PEDESTRIAN!",
            "PIETON!",
            "PIÉTON !",
            "PEDONE!",
            "PEATÓN!",
            "PEÃO!",
            "FUSSGÄNGER!",
            "GYALOGOS!",
        ],
    ),
    (
        "DETOUR",
        [
            "DETOUR",
            "OCOLIRE",
            "DÉVIATION",
            "DEVIAZIONE",
            "DESVÍO",
            "DESVIO",
            "UMLEITUNG",
            "KERÜLŐ",
        ],
    ),
    (
        "JAM AHEAD",
        [
            "JAM AHEAD",
            "AGLOMERAȚIE",
            "BOUCHON",
            "CODA",
            "ATASCO",
            "ENGARRAFAMENTO",
            "STAU",
            "DUGÓ",
        ],
    ),
    (
        "FINE",
        [
            "FINE", "AMENDĂ", "AMENDE", "MULTA", "MULTA", "MULTA", "BUSSGELD", "BÍRSÁG",
        ],
    ),
    (
        "GOOD DRIVER",
        [
            "GOOD DRIVER",
            "ȘOFER BUN",
            "BON CONDUCTEUR",
            "BUON AUTISTA",
            "BUEN CONDUCTOR",
            "BOM MOTORISTA",
            "GUT GEFAHREN",
            "JÓ SOFŐR",
        ],
    ),
    (
        "CLEAN CITY",
        [
            "CLEAN CITY",
            "ORAȘ CURAT",
            "VILLE PROPRE",
            "CITTÀ PULITA",
            "CIUDAD LIMPIA",
            "CIDADE LIMPA",
            "SAUBERE STADT",
            "TISZTA VÁROS",
        ],
    ),
    (
        "REPAIR",
        [
            "REPAIR",
            "REPARAȚII",
            "RÉPARATION",
            "RIPARAZIONE",
            "REPARACIÓN",
            "REPARO",
            "REPARATUR",
            "JAVÍTÁS",
        ],
    ),
    (
        "TRAIN!",
        [
            "TRAIN!", "TREN!", "TRAIN !", "TRENO!", "TREN!", "TREM!", "ZUG!", "VONAT!",
        ],
    ),
    (
        "CLOSE CALL",
        [
            "CLOSE CALL",
            "PE LÂNGĂ!",
            "DE JUSTESSE",
            "PER UN SOFFIO",
            "POR POCO",
            "POR POUCO",
            "KNAPP",
            "HAIR!",
        ],
    ),
    (
        "SERVICE",
        [
            "SERVICE",
            "SERVICE",
            "SERVICE",
            "OFFICINA",
            "TALLER",
            "OFICINA",
            "WERKSTATT",
            "SZERVIZ",
        ],
    ),
    (
        "ANIMAL!",
        [
            "ANIMAL!", "ANIMAL!", "ANIMAL !", "ANIMALE!", "ANIMAL!", "ANIMAL!", "TIER!", "ÁLLAT!",
        ],
    ),
    (
        "JUMP",
        [
            "JUMP", "SALT", "SAUT", "SALTO", "SALTO", "SALTO", "SPRUNG", "UGRÁS",
        ],
    ),
    (
        "BIG AIR",
        [
            "BIG AIR",
            "SALT URIAȘ",
            "GRAND SAUT",
            "SALTO GRANDE",
            "SALTO GRANDE",
            "SALTO GRANDE",
            "WEITSPRUNG",
            "NAGY UGRÁS",
        ],
    ),
    (
        "LAND",
        [
            "LAND",
            "ATERIZARE",
            "ATTERRI",
            "ATTERRATO",
            "ATERRIZA",
            "POUSO",
            "LANDUNG",
            "LANDOLÁS",
        ],
    ),
    (
        "LOW FUEL",
        [
            "LOW FUEL",
            "REZERVĂ",
            "RÉSERVE",
            "RISERVA",
            "RESERVA",
            "RESERVA",
            "WENIG TANK",
            "KEVÉS ÜZEMANYAG",
        ],
    ),
    (
        "NO FUEL",
        [
            "NO FUEL",
            "FĂRĂ COMBUSTIBIL",
            "PANNE SÈCHE",
            "SENZA BENZINA",
            "SIN COMBUSTIBLE",
            "SEM COMBUSTÍVEL",
            "TANK LEER",
            "ELFOGYOTT",
        ],
    ),
    (
        "REFUEL",
        [
            "REFUEL",
            "ALIMENTARE",
            "PLEIN",
            "RIFORNIMENTO",
            "REPOSTAJE",
            "ABASTECE",
            "TANKEN",
            "TANKOLÁS",
        ],
    ),
    (
        "SCRAPE",
        [
            "SCRAPE",
            "FRECARE",
            "FROTTEMENT",
            "STRISCIO",
            "ROCE",
            "RASPÃO",
            "STREIFEN",
            "SURLÓDÁS",
        ],
    ),
    (
        "POLICE!",
        [
            "POLICE!",
            "POLIȚIA!",
            "POLICE !",
            "POLIZIA!",
            "POLICÍA!",
            "POLÍCIA!",
            "POLIZEI!",
            "RENDŐRSÉG!",
        ],
    ),
    (
        "BUSTED",
        [
            "BUSTED",
            "DESCALIFICAT",
            "DISQUALIFIÉ",
            "SQUALIFICATO",
            "DESCALIFICADO",
            "DESCLASSIFICADO",
            "DISQUALIFIZIERT",
            "KIZÁRVA",
        ],
    ),
    (
        "TURBO PAD",
        [
            "TURBO PAD",
            "PLATFORMĂ TURBO",
            "PLAQUE TURBO",
            "PEDANA TURBO",
            "PLACA TURBO",
            "PLACA TURBO",
            "TURBOPLATTE",
            "TURBÓ LAP",
        ],
    ),
    (
        "OIL!",
        [
            "OIL!", "ULEI!", "HUILE !", "OLIO!", "ACEITE!", "ÓLEO!", "ÖL!", "OLAJ!",
        ],
    ),
    (
        "CRASH",
        [
            "CRASH",
            "CIOCNIRE",
            "CHOC",
            "SCONTRO",
            "CHOQUE",
            "COLISÃO",
            "CRASH",
            "ÜTKÖZÉS",
        ],
    ),
    (
        "HOLE",
        [
            "POTHOLE",
            "GROAPĂ",
            "NID DE POULE",
            "BUCA",
            "BACHE",
            "BURACO",
            "SCHLAGLOCH",
            "KÁTYÚ",
        ],
    ),
    (
        "WET",
        [
            "WET ROAD",
            "DRUM UD",
            "ROUTE MOUILLÉE",
            "BAGNATO",
            "MOJADO",
            "MOLHADO",
            "NASS",
            "NEDVES",
        ],
    ),
    (
        "MUD",
        [
            "MUD", "NOROI", "BOUE", "FANGO", "BARRO", "LAMA", "SCHLAMM", "SÁR",
        ],
    ),
    (
        "BOOST",
        [
            "BOOST",
            "BOOST",
            "BOOST",
            "BOOST",
            "IMPULSO",
            "IMPULSO",
            "BOOST",
            "LENDÜLET",
        ],
    ),
    (
        "SHIELD",
        [
            "SHIELD", "SCUT", "BOUCLIER", "SCUDO", "ESCUDO", "ESCUDO", "SCHILD", "PAJZS",
        ],
    ),
    (
        "FREEZE",
        [
            "TRAFFIC FREEZE",
            "TRAFIC ÎNGHEȚAT",
            "TRAFIC GELÉ",
            "TRAFFICO FERMO",
            "TRÁFICO LENTO",
            "TRÂNSITO PARADO",
            "VERKEHR STOPP",
            "FORGALOM LASSÚ",
        ],
    ),
    ("X2", ["X2", "X2", "X2", "X2", "X2", "X2", "X2", "X2"]),
    (
        "CRACK",
        [
            "CRACK!",
            "CRĂPĂTURĂ!",
            "FISSURE !",
            "CREPA!",
            "GRIETA!",
            "RACHADURA!",
            "RISS!",
            "REPEDÉS!",
        ],
    ),
    (
        "MAGMA",
        [
            "MAGMA", "MAGMĂ", "MAGMA", "MAGMA", "MAGMA", "MAGMA", "MAGMA", "MAGMA",
        ],
    ),
    (
        "KEEP MOVING",
        [
            "KEEP MOVING",
            "NU TE OPRI",
            "N'ARRÊTEZ PAS",
            "NON FERMARTI",
            "NO PARES",
            "NÃO PARES",
            "WEITERFAHREN",
            "NE ÁLLJ MEG",
        ],
    ),
    (
        "ICE BREAKS",
        [
            "ICE BREAKS",
            "GHEAȚA SE SPARGE",
            "LA GLACE CÈDE",
            "IL GHIACCIO CEDE",
            "EL HIELO CEDE",
            "O GELO CEDE",
            "DAS EIS BRICHT",
            "A JÉG BEROPPAN",
        ],
    ),
];

/// Rank tag ("1ST"/"2ND") for the versus screen.
pub fn rank_tag(rank: usize) -> &'static str {
    tr(if rank == 0 { S::First } else { S::Second })
}

/// Mode names (0-8 game modes, 9 vs CPU, 10 two players), per language.
const MODE_NAMES: [[&str; 8]; 11] = [
    [
        "CAMPAIGN", "CAMPANIE", "CAMPAGNE", "CAMPAGNA", "CAMPAÑA", "CAMPANHA", "KAMPAGNE",
        "KAMPÁNY",
    ],
    [
        "TIME TRIAL",
        "CONTRA CRONOMETRU",
        "CONTRE-LA-MONTRE",
        "CRONOMETRO",
        "CONTRARRELOJ",
        "CONTRA O RELÓGIO",
        "ZEITFAHREN",
        "IDŐFUTAM",
    ],
    [
        "ENDLESS",
        "INFINIT",
        "SANS FIN",
        "INFINITO",
        "INFINITO",
        "INFINITO",
        "ENDLOS",
        "VÉGTELEN",
    ],
    [
        "RUSH HOUR",
        "ORA DE VÂRF",
        "HEURE DE POINTE",
        "ORA DI PUNTA",
        "HORA PUNTA",
        "HORA DE PONTA",
        "STAU",
        "CSÚCSFORGALOM",
    ],
    [
        "ORB HUNT",
        "VÂNĂTOARE DE ORBE",
        "CHASSE AUX ORBES",
        "CACCIA ALLE SFERE",
        "CAZA DE ORBES",
        "CAÇA ÀS ESFERAS",
        "ORBENJAGD",
        "GÖMBVADÁSZAT",
    ],
    [
        "RING SLALOM",
        "SLALOM PRIN INELE",
        "SLALOM D'ANNEAUX",
        "SLALOM A ANELLI",
        "SLALOM DE ANILLOS",
        "SLALOM DE ANÉIS",
        "RINGSLALOM",
        "GYŰRŰSLALOM",
    ],
    [
        "REVERSAL",
        "INVERSARE",
        "INVERSION",
        "INVERSIONE",
        "INVERSIÓN",
        "INVERSÃO",
        "UMKEHR",
        "FORDÍTOTT",
    ],
    [
        "BLACKOUT",
        "ÎNTUNERIC",
        "OBSCURITÉ",
        "BUIO TOTALE",
        "OSCURIDAD",
        "ESCURIDÃO",
        "DUNKELHEIT",
        "SÖTÉT",
    ],
    ["ZEN", "ZEN", "ZEN", "ZEN", "ZEN", "ZEN", "ZEN", "ZEN"],
    [
        "VS CPU",
        "CONTRA CPU",
        "CONTRE CPU",
        "CONTRO CPU",
        "CONTRA CPU",
        "CONTRA CPU",
        "GEGEN CPU",
        "CPU ELLEN",
    ],
    [
        "2 PLAYERS",
        "2 JUCĂTORI",
        "2 JOUEURS",
        "2 GIOCATORI",
        "2 JUGADORES",
        "2 JOGADORES",
        "2 SPIELER",
        "2 JÁTÉKOS",
    ],
];

pub fn mode_name_in(l: Lang, mode: usize) -> &'static str {
    MODE_NAMES[mode.min(10)][l as usize]
}

pub fn mode_name(mode: usize) -> &'static str {
    mode_name_in(lang(), mode)
}
