**TURBOCURSA 1.6.0**: clean-room pseudo-3D arcade racer in Rust, native (minifb) and web (WASM), 8 languages, MIT.

- **Play in the browser:** download `turbocursa.html` and open it (a single self-contained file, no server needed), or install the web app from the presentation page.
- **Native:** `turbocursa-linux-x86_64.tar.gz`, `turbocursa-windows-x86_64.zip`, `turbocursa-macos-arm64.tar.gz`, `turbocursa-macos-x86_64.tar.gz` (or `cargo run --release --features native,audio --bin turbo`).
- **Source:** `turbocursa-src.zip`. Every file is listed in `SHA256SUMS`; the one-line installers check it before they unpack anything.

What is new: a **demo walkthrough** (`--demo`: an AI pilot plays every mode on every difficulty, nothing is saved), **easier difficulty levels** (longer clocks, slower police), the checkpoint banner now shows the real bonus seconds, atomic saves, a hardened release workflow and service worker. See `CHANGELOG.md`.

Balance note: scores and times from 1.5 are not comparable with 1.6 (longer clocks leave more time on the counter at the finish).

Not verified on real hardware: sound output, touch devices, Safari/Firefox, the Windows and macOS binaries (built by CI, never run by the maintainer).
