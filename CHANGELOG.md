# Changelog

## 1.6.0

### Added
- **Demo walkthrough** (`turbo --demo`, native): an AI pilot (`src/ace.rs`, a receding-horizon planner that plays candidate plans on a clone of the real simulation) drives all 40 stages (8 finite modes x 5 difficulty levels, Endless as one lap) one after the other. Headless check: `cargo run --release --example demo_report`. The demo runs with an assist (`Game::demo`): the car is shielded, uses no fuel and ignores jam queues, the clock stops at 2 s and the police cannot bust it; nothing is saved (no XP, coins, records or ghosts) and Enter/Backspace are ignored. In the headless run all 40 stages finish (the hardest in 70-140 s).
- `Game::drive(lane, yield_traffic, ace)`: the autopilot with its lane choice exposed; `Game` and `Track` are `Clone`.
- `SHA256SUMS` verification in `install.sh` and `install.ps1`: a download that does not match is refused (the installers then fall back to building from source).
- Tests: `tests/demo_pilot.rs` (playlist coverage, determinism, assist, banner; one slow test with `--ignored`).

### Changed
- **Easier difficulty levels**: the starting clock and checkpoint bonuses are now x2.0 / x1.9 / x1.8 / x1.7 / x1.6 (Easy .. Insane; they were x1.25 .. x0.68), and the police close in more slowly (pressure and contact penalty scale with the level). Scores and times from 1.5 are not comparable with 1.6.
- The checkpoint banner shows the real bonus in seconds (it said "+30" at every level), in all 8 languages.
- The autopilot steers the right way during Mirror mode's inverted phases (it used to fight the inversion).
- Native saves are written atomically (temporary file + rename) and read with a 2 MB cap.
- Service worker: one cache entry per path (query strings no longer fill the cache), range requests and non-200 answers are not cached, bounded cache size, working offline fallback.
- Release workflow: the tag is validated and passed through the environment instead of being pasted into shell text; `.desktop` launcher quotes its path; `check.sh` now runs `cargo fmt --check` and clippy with `-D warnings`.
- Clippy and rustfmt clean across all targets.

## 1.5.0

### Added
- Endless mode with a 2000-lap chain cap, coins earned per stage and per rally, a second EXTRAS options page (music, units, keys, pack size, inverted pack, players, code field, garage).
- Codes: championship (`TCC1-`), stage (`TCS1-`), ghost (`TG1`/`TG2`, compact format), paint word, `DAILY`, `WEEKLY`; typed or pasted in the native code field and the web ⌨ button.
- Garage with upgrades and stock mode for fair daily/weekly/shared stages.
- Settings and garage persist in the browser and sync across tabs.
- Fugitive marker on the radar, fork signs visible to everyone.

### Phone and install (web)
- Touch controls rebuilt: big labelled GAS and BRAKE pedals, wide steering buttons and a turbo bar in portrait; side pedals over the picture in landscape; sliding a thumb between buttons works.
- Menu, rotate-to-landscape (orientation lock, with a forced-rotation fallback for iOS) and fullscreen buttons.
- Installable as an app (manifest, service worker, offline) with a one-click install button where the browser supports it.
- Release workflow builds native binaries for Linux, Windows and macOS, the web build and the source zip; `install.sh` and `install.ps1` install and add a launcher.

### Changed
- `game.rs` and `render.rs` split into modules; pack calibration reworked (rivals run at skill 0.50–0.68).
- Web page: no per-frame 1 MB copy, key state released on blur or tab switch, audio suspended while the tab is hidden, Ctrl/Cmd/Alt shortcuts no longer trigger share or photo, keyboard focus outlines.
- Solo hints now list gears, manual mode and photo mode in all eight languages.

### Fixed
- Split-screen player count no longer hijacks career, championship and rally launches.
- Web: a crash in the game loop now shows a message with a reload link instead of freezing silently.
- Web: a poisoned lock can no longer take down every later call.
- Settings not applied on reload; garage row highlighting; overlapping EXTRAS hint; code button hidden on desktop.

### Tests
- New: endless, extras page, ghost v2, adversarial 2, launch priority, codes, parser fuzzing (hostile text, truncations, mutations).
- `scripts`: `check.sh` runs tests, web tests, clippy, native build and a wasm check.
