**TURBOCURSA 1.7.1**: clean-room pseudo-3D arcade racer in Rust, native (minifb) and web (WASM), 8 languages, MIT.

- **Play in the browser:** download `turbocursa.html` and open it (a single self-contained file, no server needed), or install the web app from the presentation page.
- **Native:** `turbocursa-linux-x86_64.tar.gz`, `turbocursa-windows-x86_64.zip`, `turbocursa-macos-arm64.tar.gz`, `turbocursa-macos-x86_64.tar.gz` (or `cargo run --release --features native,audio --bin turbo`).
- **Source:** `turbocursa-src.zip`. Every file is listed in `SHA256SUMS`; the one-line installers check it before they unpack anything.

What is new: a **NEW BEST!** banner when you beat your best score, a **pause** (`Esc`, also automatic when the window loses focus; Enter restarts, resumes with Esc or a tap), a **calm mode** for reduced motion (`--calm`, `?calm`, or the browser setting), `--help` / `--version` with clear errors for unknown options, and on the presentation page direct download links plus two screen recordings with subtitles in 8 languages. See `CHANGELOG.md`.

Not verified on real hardware: sound output, touch devices, Safari/Firefox, the Windows and macOS binaries (built by CI, never run by the maintainer).
