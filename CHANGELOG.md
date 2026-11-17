# Changelog

## 1.18.0

### Changed
- **Aircraft appear more often**: they now have their own spawn timer (one every 22 to 42 seconds, one at a time, in the sky above cities, jams and open road alike) instead of sharing the 40 to 65 second timer and the single slot of the ground special vehicles. In six simulated runs that accelerate only, aircraft went from about one per run to about four or five per run. The ground special vehicles keep their old rate and no longer wait for an aircraft to leave.
- `examples/air_freq.rs` counts how often each kind appears in simulated runs.

### Not verified
- How often aircraft appear for a real driver (the simulation only accelerates), and whether once every 22 to 42 seconds feels too busy.

## 1.17.0

### Added
- **Linux builds for older systems**: besides `turbocursa-linux-x86_64.tar.gz` (built on Ubuntu 26.04) the release now also has `turbocursa-linux-x86_64-glibc2.35.tar.gz` (built on Ubuntu 22.04; runs on Ubuntu 22.04+, Debian 12+) and `turbocursa-linux-x86_64-glibc2.31.tar.gz` (built in a Debian 11 container; meant for Ubuntu 20.04+, Debian 11+). The one-line installer (`install.sh`) tries the newest first, verifies each download against `SHA256SUMS`, runs `turbocursa --version` and falls back to the next older build if the binary does not start; only then does it build from source. Tested with a mock release; the download page offers the three files with a note in 8 languages.

### Not verified
- The glibc 2.31 build runs in a container job marked experimental (a failure there does not block the release): it was never built before this version, so it may be missing from the release. None of the Linux builds was started on a real older distribution.

## 1.16.0

### Changed
- **CI and release builds now run on Ubuntu 26.04** (`ubuntu-26.04`) instead of `ubuntu-latest` / `ubuntu-22.04`, and the workflows use the Node 24 versions of the GitHub actions (`checkout@v6`, `upload-artifact@v7`, `download-artifact@v8`). Consequence: the Linux binary is now linked against the newer glibc of Ubuntu 26.04 and may not start on older distributions; building from source (`cargo run --release`) still works anywhere.

### Added
- Aerial special vehicles now cast a soft shadow on the road, fainter the higher they fly.
- README lists all 29 special vehicles.

### Not verified
- The first CI run on Ubuntu 26.04 (the project has only been built there by this release), the Linux binary on older distributions, and the sound on real speakers.

## 1.15.0

### Added
- **Nine aerial special vehicles** (kinds 20 to 28) crossing the sky above the road: airliner, banner biplane, hot-air balloon, airship, delivery drone, fighter jets, paraglider, air ambulance and UFO. Each has its own drawing (strobes, burner flame, afterburner, rotor), altitude, cruising speed and engine/air sound. Fast ones overtake from behind, balloon, airship and paraglider drift slowly ahead. They fly over the road and never collide with the car. Police car and helicopter portraits are now kinds 29 and 30 in the tools.
- `Emerg::is_air`, `is_ahead`, `cruise`, `altitude`, `view_dist` describe the new behaviour; the tests cover kinds 0 to 28 (drawn, audible, different from each other, leave the road).
- Presentation page: 29 traffic vehicles plus portraits, names in 8 languages.

### Not verified
- How the new sounds feel on real speakers (only level and difference are measured), and how often each aircraft turns up in a long run. Some portraits (air ambulance, airship, jets) are simple shapes and look plain.

## 1.14.0

### Added
- **Eight more special vehicles** (kinds 12 to 19): garbage truck, city bus, fuel tanker, concrete mixer (the drum spiral turns), mail van, limousine, camper and courier motorcycle. Each has its own drawing and its own engine sound (hydraulic hiss, air brake and stop chime, tanker slosh, drum grinding, mail chime, smooth V8, camper rattle, motorcycle buzz). They overtake from behind like the other traffic and join the random pool; the tests check that each one changes the picture, is audible and sounds different from the others.
- **Presentation page**: the special-vehicles section now shows all 20 traffic vehicles plus portraits of the police car and the traffic helicopter (they were already in the game, but were missing from the page). Names in 8 languages. `examples/special_shots.rs` renders them; `render::draw_portrait` draws the police car and the helicopter for it.

### Not verified
- How the new sounds feel on real speakers (only measured for level and difference), and how often each vehicle turns up in a long run.

## 1.13.0

### Added
- **Finer textures at high native resolution**: when `render_view` draws at twice the base size or more, the road, ground and facade textures come from a second set (1024-pixel tiles, twice the texel density, one more octave of grain), built on first use. Edges and strokes are drawn at the real size, so natively rendered 4K and 8K frames show asphalt cracks, grain and clean diagonals that an enlarged 640x400 picture cannot.
- `examples/res_shots.rs native ...` renders every resolution natively; the gallery on the presentation page uses it. `examples/special_shots.rs` renders portraits of the twelve special vehicles (also on the page, in a new section in 8 languages).
- Presentation page: contrast of the gallery and video-chapter buttons fixed in light and dark mode.

### Not changed
- The real-time game still renders at 640x400 and enlarges to HD to 8K (a native 1728x1080 frame takes about 100 ms on one thread, far from 60 fps). The extra detail therefore appears in natively rendered captures, not in the live window.
- Cars, special vehicles and the HUD are still built from simple shapes, so at high native resolution they look clean but plain.

### Not verified
- Speed on other machines; native frames at 4K and 8K take seconds.

## 1.12.0

### Added
- **Building facades** (`micro::wall`): tileable fractal concrete with grain, panel joints, rain streaks, stains and chips, anchored to the street (distance along the block and height), faded out with distance and fog.
- **Trees and rocks**: every generated tree and dead tree gets leaf/bark-scale mottling, fine speckle, ambient occlusion deep inside the silhouette and a lit rim on the sun side. Rocks get warped strata, a fractal crack network, grain and lichen patches on the lit faces.
- The presentation gallery was re-rendered.

### Changed
- The picture changes again on purpose; the frame hash of `examples/framehash.rs` differs from 1.11.0.
- Cost: about 11.5 to 12.5 ms per frame at 640x400 on the build machine (9 ms in 1.10.2); start-up to the first frame is about 470 ms, because the sprites are generated with the extra passes.

### Not verified
- Speed on other machines and in the browser. Cars, pedestrians, the sky and the HUD were not given new textures; the renderer is still 640x400, so this is more detail on a small grid, not photographic realism.

## 1.11.0

### Added
- **Fractal micro-texture on the road and the ground** (`src/render/micro.rs`). Until now the road and the verge were flat colours per segment. A new pass lays tileable fractal detail over them: asphalt with aggregate grain, pebbles, a branching crack network and repair patches; soil and turf from domain-warped multi-octave noise with fine blades and a slow warm/cool tint; a large-scale mottling layer that survives to the horizon. Everything is generated at start-up from the code, no image assets.
- The texture coordinates are anchored to the track (lateral position in road half-widths, distance in segment units), so the detail rides with the world instead of swimming on the screen. Each texture has a mip chain; a row picks its level from the pixel footprint and the detail fades out before it would shimmer. Fantasy biomes with flat colours (mirror lake, chessboard) and bridge/viaduct/tunnel ground are left untouched; thin ice keeps its own cracks.
- The presentation gallery was re-rendered with the new textures.

### Changed
- The rendered picture is different on purpose: the frame hash of `examples/framehash.rs` no longer matches 1.10.2.
- Cost: about 2.5 ms more per frame on the build machine (about 9 to 11.5 ms for simulation, scene and post-processing at 640x400, measured with `examples/bench.rs`).

### Not verified
- Speed on other machines and in the browser (WASM is single-threaded and slower); weak devices may drop frames.
- The renderer is still 640x400; HD to 8K are sharp upscales, so the new detail is limited by that grid. Real hyper-realism would need rendering at a higher internal resolution, which this release does not do.

## 1.10.2

### Changed
- **Less work per frame**: the night/sunset tint and the fog veil use per-channel lookup tables instead of float maths on every pixel, and the final composite skips the bloom interpolation on rows that have no bloom. Measured with callgrind on 150 frames of a stormy night stage: 22.56 to 20.43 billion instructions (about 10% fewer). A hash of 20 rendered frames over five weather/time/mode combinations (`examples/framehash.rs`) is identical to 1.10.1, so the picture did not change.

### Not verified
- Wall-clock gains on other machines (this one has two noisy cores). The remaining big costs are the final composite (about 26% of instructions), the glow effect (14%) and the sky layers (13%).

## 1.10.1

### Changed
- **Faster frames**: the glow effect (lights, sun, flames) now uses integer maths and skips invisible fringe pixels; the speed-blur and final composite passes run on up to four threads natively (the browser stays single-threaded). Measured on a 2-core machine: about 11.7 ms to about 9.2 ms per frame at 640x400 (noisy; the numbers vary by 10%).
- Release builds are stripped; the web page is about 3% smaller.
- The browser API fuzz test runs in about a third of the time (81 s instead of 220 s), so CI finishes sooner. The new example `examples/startup.rs` prints the start-up time (about 160 ms natively).

### Not verified
- Speed on other hardware and in the browser (WebAssembly runs these passes single-threaded). Glow colours can differ by one level from the float version.

## 1.10.0

### Added
- **Wide landscape layout on phones** (screens 2:1 or wider, e.g. 844x390): the steering, turbo, accelerator and brake sit in the black bars beside the picture instead of over it, so the speed gauge, minimap, gear and fuel stay visible; the gear/photo buttons move to the sky below the progress bar. Narrower landscape screens keep the overlay layout.
- **`?demo` in the browser**: the pilot drives every stage in turn, like `--demo` natively. Enter and Backspace do not leave it, and nothing is recorded (no XP, records or ghosts).

### Not verified
- Real phones: the layout was checked in Chrome's mobile emulation at 844x390 and 667x375 only.

## 1.9.0

### Added
- **Ten more special vehicles**, joining the ambulance and fire engine: snowplough (only in snow, blizzard or glaze, otherwise a street sweeper), tow truck, road-works truck with an arrow board, street sweeper, armoured cash van, school bus, taxi, army truck, ice-cream van and a slow tractor in the right lane ahead. Each has its own picture and sound (diesel rumble, beeps, horns, a jingle, the putt of a tractor). Only the ambulance and fire engine are emergencies (banner, "Move over", good-driver bonus); the others just share the road.
- **2K, 3K, 4K and 8K output**: `--res hd|2k|3k|4k|8k` natively (`--hd` still means Full HD), and in the browser the `HD` button or `F2` cycles SD, HD, 2K, 3K, 4K and 8K (remembered, also `?res=4k`). Like Full HD these are sharp upscales of the 640x400 picture, not native rendering. The upscaler now uses several threads: about 3 ms (2K), 6 ms (4K) and 21 ms (8K) per frame on a 2-core machine, before the window system draws it.

### Not verified
- Frame rates at 4K/8K on real displays and GPUs (browser measured only in software-rendered Chrome: SD 51, HD 35, 2K 25, 3K 21, 4K 17, 8K 7 fps there); sounds of the new vehicles only as generated signal.

## 1.8.0

### Added
- **Stereo sound in the whole game**: music, effects, engine, wind and rain are panned with equal-power gains, the engine is widened, the echo ping-pongs between the ears, and the browser now plays two channels (native output maps the stereo pair to the device channels).
- **Distinct emergency sounds**: police two-tone siren, ambulance yelp, fire-engine wail with an air horn, and a helicopter rotor. Ambulances and fire engines can overtake you, with proximity, panning and Doppler.
- **Full HD mode**: `--hd` (native, borderless 1920x1080) and the `HD` button or `F2` in the browser (remembered, also `?hd=1`). It is a sharp-bilinear upscale of the 640x400 picture to 1728x1080, not native 1080p rendering. The native pass takes about 1.5 ms per frame.

### Not verified
- Sound was only checked as generated signal (tests), never on real speakers; the browser HD path was timed in software-rendered Chrome only.

## 1.7.1

### Added
- **NEW BEST!** banner on the stage-clear screen when the run beats your previous best score of that mode (translated into all 8 languages; native best scores live in `~/.turbocursa_best`, the browser reuses its stored best). Demo runs never show or record it.

## 1.7.0

### Added
- **Pause**: `Esc` pauses a running single-player stage in the native build and in the browser (Esc resumes, Enter restarts the stage, a tap resumes on a phone; the browser toolbar has a pause button). The picture is dimmed and the overlay is translated into all 8 languages. Photo mode hands over to the pause cleanly.
- **Automatic pause** when the window or tab loses focus (native: only after a real focus loss was observed, so platforms that never report focus cannot freeze the game).
- **Calm mode** (`--calm`, `?calm`, and automatically with the browser's reduced-motion setting): no screen shake, no strobing busted text, soft lightning flashes.
- `turbo --help` / `-h`, `--version` / `-V`; an unknown option now stops with a clear message and exit code 2 instead of opening a window.
- Presentation page: direct download links to the latest release for every variant, and two real screen recordings (install step by step, every option) with WebVTT subtitles in 8 languages.
- Tests: `tests/pause.rs`, `browser_pause_freezes_and_restarts` in `tests/web_api.rs`.

### Changed
- `Esc` no longer quits a running stage on the spot (it pauses); outside a stage it still leaves the game, and in the demo it still quits.

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
