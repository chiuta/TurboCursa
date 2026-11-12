**English** · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)

# **TURBOCURSA**

Version: 1.5 · Last updated: 12 November 2026

## What it is

TURBOCURSA is a clean-room pseudo-3D arcade racer written in Rust: original engine, art, music and name, with no code, assets or trademarks from any existing game. Everything you see and hear is generated from a seed: fractal trees, rocks, clouds, nebulae, aurora and Mandelbrot checkpoint banners, plus a procedural soundtrack and engine sound. The core library has no runtime dependencies.

Nineteen generated landscapes: forest, desert, dusk, colossal world-trees whose crowns hide in the clouds, red stratified mesas, floating islands with waterfalls and a ringed planet, a mirror lake under twin moons, an aurora tundra with falling snow, a volcano with lava veins, a fractal smoke plume and recursive lightning, a sunlit abyss with coral and kelp, a cosmos with a Julia-set nebula and a rainbow road, a neon citadel of Sierpinski pyramids and Menger blocks, and a glowing mycelium jungle. Each stage chains one classic and three newer landscapes in a seeded order. Six more worlds, possible and impossible: an inverted world with a second mountain range hanging from the sky, a prism desert under spectral aurora bands, a clockwork world of giant turning gears, a sea of clouds, a black hole with a lensed accretion ring, and a fractal canyon of banded strata.

## Use

- Native: `cargo run --release --features native --bin turbo` (add `--versus` for 2 players, `--daily` for the daily stage, `--seed N` for a fixed stage, `--lang xx` for the language).
- Web: `./build_web.sh` produces one offline file `dist/turbocursa.html` (touch controls, language selector, personal best). Parameters: `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Controls: arrows or WASD, Space = turbo, Enter = retry, M = mute. Player 2: WASD + Left Shift.
- Interface and documentation: English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. In-game text uses a pixel font; accents are drawn as a mark above the letter.
- Modes: `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (race a computer rival), `--versus` (2 players); web: `?mode=NAME`. Career: `--career` (divisions, leagues and cups, saved)

## Game modes

- **Campaign** — Beat the clock through four generated landscapes; checkpoints add time.
- **Time Trial** — No traffic, the clock counts up: finish as fast as you can.
- **Endless** — Laps forever: every lap is a brand-new generated stage, +25 s each.
- **Rush Hour** — Triple traffic; every overtake buys you 0.8 seconds.
- **Orb Hunt** — Follow flowing fractal trails of orbs; each one adds time and combo.
- **Ring Slalom** — Thread the rings: each pass raises the combo multiplier, a miss costs a second.
- **Reversal** — Steering flips to its mirror image for six seconds out of every fourteen.
- **Blackout** — Darkness: only the headlight cone is visible, the landscape is a rumour.
- **Zen** — No clock, no traffic, no crashes: just drive through endless landscapes.
- **Vs CPU** — Race a computer-driven rival on the same stage, split screen.
- **2 Players** — Two humans, one stage, split screen: first to the finish wins.

## Career

Run it with `--career` (web: `?mode=career`). Progress is saved automatically (a file `~/.turbocursa_career` natively, `localStorage` in the browser).
- **Divisions** — Bronze, Silver, Gold, Platinum, Diamond. Each season is a six-matchday league of 12 drivers: the top three move up, the bottom three move down; the Diamond champion earns a World Cup wild card.
- **Cups with elimination rounds** — Your lap time is compared with simulated rivals whose strength grows with the cup and the division; the slowest are knocked out each round (16 → 8 → 4 → final in a national cup; up to 32 → 24 → 16 → 8 → 4 → final in the World Cup).
- **Multiple qualifications** — Two *repechage* chances per cup: if you are knocked out, the best of the eliminated race again and the first two return.
- **Ladder** — Reach the final of your **national cup** to qualify for your **continental cup**; reach its final for the **intercontinental cup**; reach that final for the **World Cup**.
- **Every country** — 196 countries on six continents; you can also enter the national cup of any other country (guest cups) and collect a trophy for each in the trophy cabinet.
- **Rankings** — a world ranking of 300 drivers plus you, a nations ranking, the league table and your statistics.

## Country cars

Every country has its own race car: a livery painted from its national colours (approximate, original art) with a number plate, and a handling profile (top speed, acceleration, grip, 2-4 turbo charges). The profile is a zero-sum budget, so no country has a strictly better car. Choose it with `--car RO` (or `--car random`), `?car=RO` on the web, or browse all of them in the career Garage; in the career your own country's car is used automatically.

## Difficulty, vehicles, obstacles and bonuses

- **5 difficulty levels** — Easy, Normal, Hard, Expert, Insane: they scale the starting clock and checkpoint bonuses (×1.25 … ×0.68), the amount and speed of traffic, how many obstacles and bonuses lie on the road, and how much speed a crash costs. In career mode the level also makes the simulated rivals stronger or weaker.
- **19 vehicle classes** for your car: sports car, sedan, city microcar, truck, semi-trailer (TIR), moped, motorcycle, trike, tuk-tuk, quad, buggy, formula car, rally car, pickup, van, bus, tractor, limousine and monster truck. Each keeps the country's livery and handling profile and adds its own trade-off: top speed, acceleration, grip, width, crash resilience and clock. Two-wheelers are narrow, nimble and fragile; buses, tractors and monster trucks are wide and tough; the formula car is the fastest and the most brittle.
- **Mixed traffic**: cars, microcars, trucks, semi-trailers, buses, tractors, motorbikes and trikes; the higher the level, the more heavy vehicles.
- **On the road**: obstacles (cones, oil slicks, barriers, potholes, puddles, mud) and bonuses (+4 s, extra turbo charge, speed boost, shield, coins, traffic freeze, ×2 score). Active effects show as coloured bars under the clock.
- **Selection screen** at the start of every game: ← → vehicle, ↑ ↓ difficulty, Enter to start, Backspace to come back to it. `--class` / `--level` / `--nopick` (native) or `?class=…&level=…` / the two selectors (web) skip it; in career mode use the *Difficulty* / *Vehicle* lines of the hub.

## World, systems and tools

- **Weather and surfaces** — 11 weathers (clear, rain, storm, fog, snow, blizzard, sandstorm, hail, black ice, heatwave, ash fall) and 6 surfaces (asphalt, frozen lake, salt flat, lava field, dunes, bog) change grip, visibility and look; leave them on Auto or pick them (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Day and night** — dawn, day, dusk or night (`--time`, `?time=`), with headlights, stars and crickets.
- **Life by the road** — bridges, tunnels, viaducts and mountain passes; level crossings with trains; roaming animals; ramps and boost pads; police who chase speeders (being caught means disqualification).
- **Fuel and damage** — fuel runs down and stations refill it; crashes damage the body and the engine (less top speed, slower acceleration, looser steering) and stations repair them.
- **Gearbox, tyres and setup** — automatic or manual gearbox (E / Q shift, T switches); road, rain, snow and off-road tyres and four setups (balanced, sprint, downforce, economy): `--tyre`, `--setup`.
- **Rank 0–999 and responsible play** — your rank rises with experience and scales the difficulty (`--rank N` to override); a day streak is counted, a reminder suggests a break after a long session and a notice says when it is enough for today.
- **Badges and collections** — 25 badges and a collection of classes, weathers, surfaces, times of day and countries driven.
- **Daily challenge** — `--daily` / `?daily`: the same stage, vehicle and conditions for everybody, with a short result code to compare.
- **Ghost** — your best run on a stage is replayed as a cyan ghost with a time gap (saved in `~/.turbocursa_ghost_*` or `localStorage`; best on a fixed seed or the daily stage).
- **Rally** — `--rally` / `?mode=rally`: five stages from dawn to night with changing weather and surface; damage, fuel and tyre wear carry over; service-park minutes count against you; the final position is worked out against 15 simulated crews.
- **Photo mode** — P free camera (arrows move it), H shows or hides the HUD, C saves a PNG (`turbocursa_photo_N.png`; in the browser it downloads).
- **Controls** — keyboard, touch buttons, gamepad in the browser (sticks or d-pad steer, RT/A gas, LT/B brake, X turbo, Start enter, Select back, LB/RB gears, Y manual) and tilt steering on phones (the ⟲ button).
- **Accessibility** — colour-blind filters (`--colorblind protan|deutan|tritan|contrast`), assists that hold bends, re-centre the car and soften walls (`--assist 1..3`), a slow mode (`--slow 70` = 70% top speed) and a large interface (`--large`).
- **Sound** — procedural engine with gears, wind, rain, storm, hail, sirens, train horns, ice, a fuel warning beep, a damaged engine and animal calls.
- **Stage designer and codes** — `--design` (web: `?mode=design`): pick the seed, mode, vehicle, level, weather, surface, time, the four landscapes, how twisty and hilly the road is and how long; a chart previews the road. Every stage has a short typo-checked code `TCS1-…`: drive it with `--code CODE` or `?code=CODE`; in the web build press L to show it and copy a link.
- **Cities and traffic rules** — every stage crosses cities with boulevards and streets: facades, pavements, crosswalks, traffic lights (green, yellow, red), pedestrians and a speed limit. Running a red light, speeding or not giving way to a pedestrian calls the police (on Easy a fine costs time instead); hitting a pedestrian costs time and damage.
- **Responsible driving pays** — stopping at a red light, giving way, keeping to the limit and crossing a city without an offence earn score, time and a multiplier up to x3 ("GOOD DRIVER", "CLEAN CITY"); any offence resets the streak. Letting an ambulance or fire engine overtake with its siren on pays too, blocking it is an offence.
- **Jams, detours and the RadarMap** — some stages have a traffic jam: a standing queue with a winding gap you can creep through (driving into it is a crash). A slip road to the right before the jam leads onto a free street; the heading-up RadarMap in the HUD shows the road ahead, cities, the jam in red, the detour in green, the traffic lights and the time to cross the jam on the main road versus the exit.
- **Emergency vehicles and helicopters** — ambulances and fire engines overtake with flashing lights; an accident scene (a helicopter that lands on the road as you approach, an ambulance and a fire engine) blocks the right lane and leaves the left open; a traffic helicopter hovers over cities and jams.
- **Missions, proving ground, parking** — pick up a parcel or a passenger and deliver it to the bay before the clock runs out; the route has a proving ground (slope, slalom rings, a ramp) and parking lots announced by gantries, each paying a bonus.
- **Unlockable service vehicles** — police car (10 clean cities), ambulance (8 deliveries), fire engine (3 daily challenges and 4 proving-ground or parking bonuses), helicopter (the other three plus 7 daily challenges). They may break traffic rules, traffic gives way, and the helicopter flies over traffic and lands when slow.
- **Events and extras** — championships of 7, 10 or 15 stages and the eight-stage tour of a country (`--champ`, `--tour`, code `TCC1-…`); a weekly challenge with a rotating modifier (`--weekly`); ghost import and export (`--ghost`, `--ghost-export`); a visible pack of 8–16 rivals with a grid, inverted grid and qualifying lap (`--pack`, `--inverted`, `--qualify`); 3–4 player split screen with CPU fill (`--players`); route forks (quick on the left, scenic on the right) with the helicopter pointing the way; a pursuit with the police car; an overtake chain worth 200 up to 2000; a garage where coins buy upgrades (`--garage`, `--buy`); music styles, mph, key layouts and secret paint codes (`--music`, `--mph`, `--keys`, `--secret`); sirens for the police car, ambulance and fire engine. All of this can be set on the **EXTRAS** page of the start screen (Backspace on the vehicle screen). A code field there (⌨ in the browser) takes a championship, stage, ghost, paint word, DAILY or WEEKLY.

## Limits

- The WebAssembly build and the native audio output (`--features audio`, cpal) could not be built or tested in the authoring environment; the logic they wrap is covered by tests. The synthesised sound has been checked numerically, not by ear.
- Tracks are generated: difficulty varies by seed (a bot at 70% throttle finishes 60 of 60 stages, at 55% also 60 of 60, but with only 0.6 s to spare).
- Not a copy of any commercial game and not affiliated with one.
- Speed: on the authoring machine a native release build takes about 13 ms per frame (99th percentile 23 ms), so slower machines may dip below 60 fps; the browser build has not been measured.
- Countries: 196 are included (the 193 UN members plus the Vatican, Palestine and Taiwan); flag colours on the cars are approximate and country names in the menus are in English only (the web car selector uses the browser's own translated names).
- Career: rivals are simulated from the autopilot's reference time. The difficulty curve was checked with synthetic drivers (par pace wins the bronze national cup, loses the diamond World Cup), not with real players, and the browser career page was only tested against a stubbed engine.
- The difficulty levels, the 19 vehicle classes, the obstacles and the bonuses were balanced only with simulated drivers and automated tests, not with real players; the trade-offs are plausible, not tuned. The vehicle sprites are simple pixel drawings, and the checkpoint banner still reads "+30" at every level.
- The browser build was run with the real WebAssembly engine only in headless Chromium here (not on real phones), the native audio output was never heard on a real device, and the native gamepad support (Linux `/dev/input/js0`) was only tested through its decoder, not with a physical pad; other systems have no native gamepad yet.
- Balance of fuel, police, animals, damage and the rally rivals was checked only with simulated drivers, not with real players.
- City, jam, detour and emergency-vehicle rules were balanced and tested only with simulated drivers, not with real players.
- Missions, proving-ground and parking balance and the unlock thresholds were tuned with simulated drivers only, not with real players.
- The new events and extras were tested with simulated drivers and automated checks only: the sound (music styles, sirens) was not listened to, nothing was tried on a physical gamepad or phone, the balance has not been played by real people, and online leaderboards are not included because they need a server.

## License

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).

## Graphics
640x400 software renderer (no GPU, no dependencies): segment-projected road, parallax sky with sun/moon,
stars, clouds and ridges; atmospheric haze; additive glow lighting (lamps, LED tail-lights, turbo flames);
particles (dust, sparks, flames); post pipeline in `src/fx.rs`: radial speed blur, bloom, chromatic
aberration, vignette, ACES tonemapping, per-biome grading, film grain.
`cargo run --release --example bench` prints frame cost.

## Fractal generation (`src/fractal.rs`)
All of it is derived from the stage seed at start-up (no assets):
- periodic value noise, fBm and ridged multifractals;
- recursive branching trees (broadleaf, conifer fronds with self-similar twigs, bare trees), pre-rendered
  to mip-mapped sprites; fBm-perturbed lit rocks;
- Mandelbrot ribbons (smooth colouring) as checkpoint banners;
- domain-warped fBm nebula, fBm clouds, fBm aurora curtains, ridged mountain ranges with snow caps;
- the road itself: curves, hills and ground tone are fBm samples, so each retry is a new track.
`DUMP_GATE=0 cargo run --release --bin dump -- frames 1` renders the first checkpoint banner.

## Modes, sound, seeds
- **Versus (2 players, split screen):** `cargo run --release --features native --bin turbo -- --versus`
  P1 arrows + Space (turbo), P2 WASD + Left Shift. Same stage for both; first to the finish wins. Web: `?versus`.
- **Daily stage:** `-- --daily` (or web `?daily`): same seed for everyone on a UTC day. `--seed N` / `?seed=N` for a fixed one.
- **Sound:** fully procedural (`src/audio.rs`): geared engine model, wind, off-road rumble, turbo hiss, crash /
  checkpoint / finish effects, generative music (per-biome scale, tempo follows speed). In the browser it starts on
  the first key press or tap (M = mute). Native playback: `--features audio` (cpal; needs ALSA dev files on Linux).
  `cargo run --release --bin dump -- frames 1` also writes `frames/audio.wav` from the bot run.
- Assets (fractal trees, sky textures, Mandelbrot banners) are generated once, in parallel, and cached for retries.
