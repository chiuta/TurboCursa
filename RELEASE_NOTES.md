**TURBOCURSA 1.8.0**: clean-room pseudo-3D arcade racer in Rust, native (minifb) and web (WASM), 8 languages, MIT.

- **Play in the browser:** download `turbocursa.html` and open it (a single self-contained file, no server needed), or install the web app from the presentation page.
- **Native:** `turbocursa-linux-x86_64.tar.gz`, `turbocursa-windows-x86_64.zip`, `turbocursa-macos-arm64.tar.gz`, `turbocursa-macos-x86_64.tar.gz` (or `cargo run --release --features native,audio --bin turbo`).
- **Source:** `turbocursa-src.zip`. Every file is listed in `SHA256SUMS`; the one-line installers check it before they unpack anything.

What is new: **stereo sound everywhere** (engine, wind, rain, music and effects are panned and widened; the browser plays two channels), distinct **sirens** (police two-tone, ambulance yelp, fire-engine wail with air horn) with an ambulance or fire engine that overtakes you, a helicopter rotor, and a **Full HD mode** (`--hd` natively, the `HD` button or `F2` in the browser): a sharp 1920×1080 upscale of the 640×400 picture. See `CHANGELOG.md`.

Not verified on real hardware: sound output, touch devices, Safari/Firefox, the Windows and macOS binaries (built by CI, never run by the maintainer).
