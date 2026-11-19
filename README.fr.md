[English](README.md) · [Română](README.ro.md) · **Français** · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)

# **TURBOCURSA**

Version : 1.20.1 · Dernière mise à jour : 19 novembre 2026

## De quoi s'agit-il

TURBOCURSA est un jeu de course arcade pseudo-3D écrit en Rust, en « clean-room » : moteur, graphismes, musique et nom originaux, sans code, ressources ni marques d'un jeu existant. Tout ce que vous voyez et entendez est généré à partir d'une graine : arbres, rochers, nuages, nébuleuses, aurores et bannières de Mandelbrot aux points de contrôle, ainsi qu'une bande-son et un bruit de moteur procéduraux. La bibliothèque de base n'a aucune dépendance à l'exécution.

Dix-neuf paysages générés : forêt, désert, crépuscule, arbres-mondes colossaux aux cimes cachées dans les nuages, mesas rouges stratifiées, îles flottantes avec cascades et planète à anneaux, un lac miroir sous deux lunes, une toundra aux aurores et à la neige, un volcan aux veines de lave, panache de fumée fractal et éclairs récursifs, un abysse ensoleillé de coraux et de varech, un cosmos avec nébuleuse de Julia et route arc-en-ciel, une citadelle néon de pyramides de Sierpinski et de blocs de Menger, et une jungle de mycélium lumineuse. Chaque étape enchaîne un paysage classique et trois plus récents, dans un ordre dicté par la graine. Six autres mondes, possibles et impossibles : un monde inversé dont une seconde chaîne de montagnes pend du ciel, un désert-prisme sous des bandes d'aurores spectrales, un monde d'horlogerie aux engrenages géants, une mer de nuages, un trou noir à anneau d'accrétion déformé par lentille gravitationnelle et un canyon fractal aux strates rayées.

## Utilisation

- Natif : `cargo run --release --features native --bin turbo` (ajoutez `--versus` pour 2 joueurs, `--daily` pour l'étape du jour, `--seed N` pour une étape fixe, `--lang xx` pour la langue).
- Web : `./build_web.sh` produit un seul fichier hors ligne `dist/turbocursa.html` (commandes tactiles, sélecteur de langue, meilleur score). Paramètres : `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Commandes : flèches ou WASD, Espace = turbo, Entrée = rejouer, M = muet. Joueur 2 : WASD + Maj gauche.
- Interface et documentation : English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. Le texte du jeu utilise une police pixel ; les accents sont dessinés comme une marque au-dessus de la lettre.
- Modes : `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (course contre un rival informatique), `--versus` (2 joueurs) ; web : `?mode=NOM`. · `--career` (divisions, championnats et coupes, avec sauvegarde)
- Démo : `--demo` laisse un pilote IA parcourir tous les modes à tous les niveaux de difficulté, étape après étape (la voiture de démo est protégée et ne consomme pas de carburant ; rien n'est enregistré)
- Pause: `Échap` met une étape en cours en pause (Échap reprend, Entrée recommence) ; le jeu se met aussi en pause quand la fenêtre perd le focus. `--calm` (ou le réglage de mouvement réduit du système, dans le navigateur) supprime les secousses d'écran et adoucit les éclairs ; `--help` et `--version` listent les options.
- Son et image : le son est stéréo partout ; les sirènes de police, d'ambulance et de pompiers et le rotor d'hélicoptère sonnent différemment. `--hd` (natif) ou le bouton `HD` / `F2` (navigateur) active une image Full HD nette : un agrandissement du rendu 640×400 en 1920×1080, pas un rendu 1080p natif.
- Véhicules spéciaux : outre l'ambulance et les pompiers, la route compte une déneigeuse, une dépanneuse, un camion de chantier, une balayeuse, un fourgon blindé, un bus scolaire, un taxi, un camion militaire, un camion de glaces et un tracteur lent, chacun avec son aspect et son son. `--res hd|2k|3k|4k|8k` (natif) ou le bouton `HD` / `F2` (navigateur) choisit un agrandissement net de l'image jusqu'à 8K.
- Les téléphones en paysage (2:1 ou plus large) ont les commandes dans les bandes latérales, sans couvrir la vitesse, la mini-carte et le rapport ; `?demo` dans le navigateur laisse le pilote conduire seul toutes les étapes.

## Modes de jeu

- **Campagne** — Battez le chronomètre à travers quatre paysages générés ; les points de contrôle ajoutent du temps.
- **Contre-La-Montre** — Sans trafic, le chrono compte vers le haut : finissez le plus vite possible.
- **Sans Fin** — Des tours sans fin : chaque tour est une étape inédite générée, +25 s.
- **Heure De Pointe** — Trafic triplé ; chaque dépassement rapporte 0,8 seconde.
- **Chasse Aux Orbes** — Suivez des traînées fractales d'orbes ; chacune ajoute du temps et du combo.
- **Slalom D'Anneaux** — Traversez les anneaux : chaque passage monte le multiplicateur, un raté coûte une seconde.
- **Inversion** — La direction s'inverse six secondes sur quatorze.
- **Obscurité** — Obscurité : seul le cône des phares est visible, le paysage n'est qu'une rumeur.
- **Zen** — Sans chrono, sans trafic, sans accident : roulez simplement dans des paysages sans fin.
- **Contre CPU** — Affrontez un rival piloté par l'ordinateur sur la même étape, écran partagé.
- **2 Joueurs** — Deux humains, une étape, écran partagé : le premier à l'arrivée gagne.

## Carrière

Lancez avec `--career` (web : `?mode=career`). La progression est sauvegardée automatiquement (fichier `~/.turbocursa_career` en natif, `localStorage` dans le navigateur).
- **Divisions** — Bronze, Argent, Or, Platine, Diamant. Chaque saison est un championnat de six journées à 12 pilotes : les trois premiers montent, les trois derniers descendent ; le champion du Diamant gagne une invitation à la Coupe du monde.
- **Coupes à élimination** — Votre temps est comparé à des rivaux simulés, d'autant plus forts que la coupe et la division sont élevées ; les plus lents sont éliminés à chaque tour (16 → 8 → 4 → finale en coupe nationale ; jusqu'à 32 → 24 → 16 → 8 → 4 → finale en Coupe du monde).
- **Qualifications multiples** — Deux chances de *repêchage* par coupe : si vous êtes éliminé, les meilleurs éliminés courent à nouveau et les deux premiers reviennent.
- **L'échelle** — Atteignez la finale de la **coupe nationale** pour vous qualifier pour la **coupe continentale** ; sa finale ouvre la **coupe intercontinentale** ; sa finale, la **Coupe du monde**.
- **Chaque pays** — 196 pays sur six continents ; vous pouvez aussi disputer la coupe nationale de n'importe quel autre pays (coupes à l'étranger) et gagner un trophée pour chacun dans la salle des trophées.
- **Classements** — un classement mondial de 300 pilotes plus vous, un classement des nations, le tableau du championnat et vos statistiques.

## Voitures par pays

Chaque pays a sa propre voiture de course : une livrée peinte aux couleurs nationales (approximatives, art original) avec plaque d'immatriculation, et un profil de conduite (vitesse max, accélération, adhérence, 2 à 4 charges turbo). Le profil est un budget à somme nulle : aucun pays n'a une voiture strictement meilleure. Choix avec `--car RO` (ou `--car random`), `?car=RO` sur le web, ou parcourez-les dans le Garage de la carrière ; en carrière, la voiture de votre pays est utilisée automatiquement.

## Difficulté, véhicules, obstacles et bonus

- **5 niveaux de difficulté** — Facile, Normal, Difficile, Expert, Fou : ils modulent le chrono de départ et les bonus de points de contrôle (×2,0 … ×1,6), la densité et la vitesse du trafic, le nombre d'obstacles et de bonus sur la route et le coût d'un choc. En carrière, le niveau renforce ou affaiblit aussi les rivaux simulés.
- **19 classes de véhicules** pour votre voiture : voiture de sport, berline, citadine, camion, semi-remorque (TIR), cyclomoteur, moto, trike, tuk-tuk, quad, buggy, monoplace, voiture de rallye, pick-up, fourgon, autobus, tracteur, limousine et monster truck. Chacune garde la livrée et le profil du pays et ajoute son compromis : vitesse de pointe, accélération, adhérence, largeur, résistance aux chocs et chrono. Les deux-roues sont étroits, agiles et fragiles ; bus, tracteur et monster truck sont larges et robustes ; la monoplace est la plus rapide et la plus fragile.
- **Trafic varié** : voitures, citadines, camions, semi-remorques, bus, tracteurs, motos et trikes ; plus le niveau est haut, plus il y a de poids lourds.
- **Sur la route** : obstacles (cônes, flaques d'huile, barrières, nids-de-poule, flaques d'eau, boue) et bonus (+4 s, turbo supplémentaire, accélération, bouclier, pièces, trafic gelé, score ×2). Les effets actifs s'affichent en barres colorées sous le chrono.
- **Écran de sélection** au début de chaque partie : ← → véhicule, ↑ ↓ difficulté, Entrée pour partir, Retour arrière pour y revenir. `--class` / `--level` / `--nopick` (natif) ou `?class=…&level=…` / les deux sélecteurs (web) le sautent ; en carrière, utilisez les lignes *Difficulté* / *Véhicule* du hub.

## Monde, systèmes et outils

- **Météo et sols** — 11 météos (dégagé, pluie, orage, brouillard, neige, blizzard, tempête de sable, grêle, verglas, canicule, pluie de cendres) et 6 sols (asphalte, lac gelé, salar, coulée de lave, dunes, marais) changent l'adhérence, la visibilité et l'aspect ; laissez-les sur Auto ou choisissez-les (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Jour et nuit** — aube, jour, crépuscule ou nuit (`--time`, `?time=`), avec phares, étoiles et grillons.
- **La vie au bord de la route** — ponts, tunnels, viaducs et cols de montagne ; passages à niveau avec trains ; animaux ; rampes et plaques de turbo ; police qui poursuit les chauffards (être pris, c'est être disqualifié).
- **Carburant et dégâts** — le carburant s'épuise et les stations le remplissent ; les chocs abîment la carrosserie et le moteur (vitesse maximale réduite, accélération plus faible, direction plus floue) et les stations les réparent.
- **Boîte, pneus et réglages** — boîte automatique ou manuelle (E / Q passent les vitesses, T bascule) ; pneus route, pluie, neige et tout-terrain et quatre réglages (équilibré, sprint, appui, économie) : `--tyre`, `--setup`.
- **Rang 0–999 et jeu responsable** — le rang monte avec l'expérience et adapte la difficulté (`--rank N` pour le forcer) ; les jours d'affilée sont comptés, un rappel propose une pause après une longue session et un message indique quand c'est assez pour aujourd'hui.
- **Badges et collections** — 25 badges et une collection de classes, météos, sols, moments de la journée et pays parcourus.
- **Défi du jour** — `--daily` / `?daily` : la même étape, le même véhicule et les mêmes conditions pour tous, avec un court code de résultat à comparer.
- **Fantôme** — votre meilleure course sur une étape est rejouée en fantôme cyan avec l'écart de temps (enregistré dans `~/.turbocursa_ghost_*` ou `localStorage` ; surtout utile avec une graine fixe ou l'étape du jour).
- **Rallye** — `--rally` / `?mode=rally` : cinq étapes de l'aube à la nuit, météo et sol changeants ; dégâts, carburant et usure des pneus se reportent ; les minutes au parc d'assistance s'ajoutent à votre temps ; le classement final est calculé face à 15 équipages simulés.
- **Mode photo** — P caméra libre (les flèches la déplacent), H affiche ou masque l'affichage tête haute, C enregistre un PNG (`turbocursa_photo_N.png` ; dans le navigateur, il est téléchargé).
- **Commandes** — clavier, boutons tactiles, manette dans le navigateur (sticks ou croix directionnelle, RT/A gaz, LT/B frein, X turbo, Start entrée, Select retour, LB/RB vitesses, Y manuel) et direction par inclinaison sur téléphone (bouton ⟲).
- **Accessibilité** — filtres pour le daltonisme (`--colorblind protan|deutan|tritan|contrast`), aides qui tiennent les virages, recentrent la voiture et adoucissent les murs (`--assist 1..3`), mode lent (`--slow 70` = 70 % de la vitesse maximale) et grande interface (`--large`).
- **Son** — moteur procédural avec rapports, vent, pluie, orage, grêle, sirènes, klaxon de train, glace, bip de carburant bas, moteur endommagé et cris d'animaux.
- **Concepteur d'étapes et codes** — `--design` (web : `?mode=design`) : choisissez la graine, le mode, le véhicule, le niveau, la météo, le sol, l'heure, les quatre paysages, la sinuosité, le relief et la longueur ; un graphique montre le profil de la route. Chaque étape a un court code vérifié `TCS1-…` : roulez-le avec `--code CODE` ou `?code=CODE` ; dans la version web, appuyez sur L pour l'afficher et copier un lien.
- **Villes et code de la route** — chaque étape traverse des villes avec boulevards et rues : façades, trottoirs, passages piétons, feux (vert, orange, rouge), piétons et limite de vitesse. Griller un feu rouge, dépasser la limite ou ne pas céder le passage à un piéton appelle la police (en Easy, une amende coûte du temps) ; renverser un piéton coûte du temps et des dégâts.
- **La conduite responsable paie** — s'arrêter au rouge, céder le passage, respecter la limite et traverser une ville sans infraction rapportent des points, du temps et un multiplicateur jusqu'à x3 (« BON CONDUCTEUR », « VILLE PROPRE ») ; toute infraction remet la série à zéro. Laisser dépasser une ambulance ou un camion de pompiers sirène en marche rapporte aussi, la gêner est une infraction.
- **Bouchons, déviations et RadarMap** — certaines étapes ont un bouchon : une file immobile avec un couloir sinueux où l'on peut se faufiler lentement (la percuter est un accident). Une bretelle à droite avant le bouchon mène à une rue libre ; le RadarMap du HUD (carte cap en haut) montre la route devant, les villes, le bouchon en rouge, la déviation en vert, les feux et le temps de traversée par la route principale ou par la sortie.
- **Véhicules d'urgence et hélicoptères** — ambulances et camions de pompiers dépassent gyrophares allumés ; une scène d'accident (un hélicoptère qui se pose sur la route à mesure que vous approchez, une ambulance et un camion de pompiers) bloque la voie de droite et laisse celle de gauche libre ; un hélicoptère de surveillance du trafic survole villes et bouchons.
- **Missions, terrain d'essai, parkings** — prends un colis ou un passager et dépose-le dans la baie avant la fin du temps ; le parcours comporte un terrain d'essai (pente, anneaux de slalom, rampe) et des parkings annoncés par des portiques, chacun avec un bonus.
- **Véhicules de service à débloquer** — voiture de police (10 villes sans faute), ambulance (8 livraisons), camion de pompiers (3 défis du jour et 4 bonus de terrain d'essai ou de parking), hélicoptère (les trois autres plus 7 défis). Ils peuvent enfreindre le code, la circulation s'écarte, et l'hélicoptère survole le trafic et se pose à faible vitesse.
- **Événements et extras** — championnats de 7, 10 ou 15 étapes et tour de huit étapes d'un pays (`--champ`, `--tour`, code `TCC1-…`) ; défi hebdomadaire avec un modificateur tournant (`--weekly`) ; import et export de fantômes (`--ghost`, `--ghost-export`) ; peloton visible de 8 à 16 rivaux avec grille, grille inversée et tour de qualification (`--pack`, `--inverted`, `--qualify`) ; écran partagé à 3–4 joueurs avec complément CPU (`--players`) ; bifurcations (rapide à gauche, panoramique à droite) avec l'hélicoptère qui montre le chemin ; poursuite avec la voiture de police ; chaîne de dépassements de 200 à 2000 ; garage où les pièces achètent des améliorations (`--garage`, `--buy`) ; styles de musique, mph, dispositions de touches et codes secrets de peinture (`--music`, `--mph`, `--keys`, `--secret`) ; sirènes pour la voiture de police, l'ambulance et le camion de pompiers. Tout se règle sur la page **EXTRAS** de l'écran de départ (Retour arrière sur l'écran du véhicule). Un champ de code (⌨ dans le navigateur) accepte un championnat, une étape, un fantôme, un mot de peinture, DAILY ou WEEKLY.
- **Véhicules spéciaux dans la circulation** — Vingt-neuf véhicules spéciaux se partagent la route et le ciel : neuf volent (avion de ligne, biplan, montgolfière, dirigeable, drone, avions de chasse, parapente, hélicoptère du SAMU, OVNI), plus la voiture de police qui vous poursuit et l'hélicoptère de la circulation au-dessus des villes et des bouchons. Sirènes, feux et sons diffèrent pour chacun ; les aéronefs ne vous percutent jamais.
- **Douze étapes choisies** — Des étapes au tracé fixe, avec météo et heure du jour : listez-les avec --showcase, lancez-en une avec --showcase N, ou ouvrez la version web avec ?showcase=N.

## Comment il se situe, en toute honnêteté

Nous avons comparé TURBOCURSA, en novembre 2026, aux jeux de course pseudo-3D open source et commerciaux que nous avons trouvés. Les faits viennent de pages publiques ; les verdicts sont les nôtres et, comme nous avons écrit le code, lisez-les avec cette réserve.

- Où il gagne : licence MIT et aucune ressource d'un autre jeu (Cannonball exige les ROM d'origine et son cœur libretro est non commercial ; javascript-racer est un petit tutoriel), un seul code pour le bureau et le navigateur, 8 langues, un binaire d'environ 2,5 Mo et environ 200 tests automatiques.
- Côté modes, il est à égalité avec les jeux rétro commerciaux : carrière, championnats, tours par pays, défis quotidiens et hebdomadaires, rallye, écran partagé à 3 ou 4, fantômes, éditeur d'étapes et douze étapes choisies.
- Où il perd : l'image et les graphismes. L'image de base fait 640×400 agrandie et les véhicules sont des formes simples, loin du fini de Slipstream ou de l'aspect 4K de Horizon Chase.
- Où il perd : les preuves. Un seul auteur et une première version en novembre 2026, sans testeurs extérieurs, alors que les jeux commerciaux comptent des milliers d'avis de joueurs et des années de mise au point.
- Où il perd : les étapes et la musique. La plupart des étapes sont générées à partir d'une graine, alors que les titres commerciaux livrent des dizaines de circuits dessinés à la main et des bandes-son composées ; la nôtre est synthétisée et n'a jamais été écoutée sur de vraies enceintes.
- Aucun lien avec un jeu ou une marque commerciale ; aucun code, aucune ressource et aucune marque n'ont été copiés.

L'audit de la version 1.20.0 a lancé des tests à entrées hostiles sur chaque étape et chaque classe de véhicule et trouvé deux vrais défauts : une vue de moins de 16×10 pixels faisait planter le rendu, et une position de sirène NaN pouvait corrompre le mixage stéréo. Les deux sont corrigés et couverts par des tests.

Contribuer : voir CONTRIBUTING.md (règles clean-room ; rapports de playtest et de test sur vieux Linux bienvenus). Licences des dépendances : THIRD_PARTY_LICENSES.md.

## Limites

- La version WebAssembly et la sortie audio native (`--features audio`, cpal) n'ont pas pu être compilées ni testées dans l'environnement de développement ; la logique qu'elles encapsulent est couverte par des tests. Le son synthétisé a été vérifié numériquement, pas à l'oreille.
- Les pistes sont générées : la difficulté varie selon la graine (un bot à 70 % d'accélération termine 60 étapes sur 60, à 55 % aussi 60 sur 60, mais avec 0,6 s de marge seulement).
- Ce n'est la copie d'aucun jeu commercial et il n'est affilié à aucun.
- Vitesse : sur la machine de développement, une version native release prend environ 13 ms par image (99e centile : 23 ms) ; les machines plus lentes peuvent passer sous 60 fps ; la version navigateur n'a pas été mesurée.
- Pays : 196 sont inclus (les 193 membres de l'ONU plus le Vatican, la Palestine et Taïwan) ; les couleurs des drapeaux sur les voitures sont approximatives et les noms de pays dans les menus sont en anglais seulement (le sélecteur web utilise les noms traduits par le navigateur).
- Carrière : les rivaux sont simulés à partir du temps de référence du pilote automatique. La courbe de difficulté a été vérifiée avec des pilotes synthétiques (le rythme de référence gagne la coupe nationale en Bronze et perd la Coupe du monde en Diamant), pas avec de vrais joueurs, et la page web n'a été testée qu'avec un moteur simulé. 196 pays sont inclus, pas tous les pays du monde.
- Les niveaux de difficulté, les 19 classes de véhicules, les obstacles et les bonus n'ont été équilibrés qu'avec des pilotes simulés et des tests automatiques, pas avec de vrais joueurs ; les compromis sont plausibles, pas calibrés. Les sprites sont de simples dessins en pixels et la bannière de point de contrôle affiche toujours « +30 » à tous les niveaux.
- La version web n'a tourné avec le vrai moteur WebAssembly que dans Chromium sans écran (pas sur de vrais téléphones), le son natif n'a jamais été écouté sur un vrai appareil, et la manette native (Linux, `/dev/input/js0`) n'a été testée que par son décodeur, pas avec une vraie manette ; les autres systèmes n'ont pas encore de manette native.
- L'équilibrage du carburant, de la police, des animaux, des dégâts et des rivaux du rallye n'a été vérifié qu'avec des pilotes simulés, pas avec de vrais joueurs.
- Les règles de ville, de bouchon, de déviation et de véhicules d'urgence n'ont été équilibrées et testées qu'avec des conducteurs simulés, pas avec des joueurs réels.
- L'équilibre des missions, du terrain d'essai et des parkings et les seuils de déblocage n'ont été réglés qu'avec des pilotes simulés, pas avec de vrais joueurs .
- Les nouveaux événements et extras n'ont été testés qu'avec des pilotes simulés et des contrôles automatiques : le son (styles de musique, sirènes) n'a pas été écouté, rien n'a été essayé sur une vraie manette ou un téléphone, l'équilibrage n'a pas été joué par de vrais joueurs, et les classements en ligne manquent car ils exigent un serveur.
- Les douze étapes choisies diffèrent exprès, mais personne d'autre que l'auteur ne les a jouées. La comparaison avec d'autres jeux s'appuie sur des pages web publiques lues en novembre 2026 et sur nos propres tests : nous n'avons pas joué aux titres commerciaux.

## Licence

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).
