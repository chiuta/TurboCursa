# Changelog

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
