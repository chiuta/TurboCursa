**TURBOCURSA 1.19.0**: clean-room pseudo-3D arcade racer in Rust, native (minifb) and web (WASM), 8 languages, MIT.

- **Play in the browser:** download `turbocursa.html` and open it (a single self-contained file, no server needed), or install the web app from the presentation page.
- **Native:** `turbocursa-linux-x86_64.tar.gz` (plus `-glibc2.35` and `-glibc2.31` variants for older Linux), `turbocursa-windows-x86_64.zip`, `turbocursa-macos-arm64.tar.gz`, `turbocursa-macos-x86_64.tar.gz` (or `cargo run --release --features native,audio --bin turbo`).
- **Source:** `turbocursa-src.zip`. Every file is listed in `SHA256SUMS`; the one-line installers check it before they unpack anything.

What is new: twelve curated showcase stages (`turbo --showcase N`), CONTRIBUTING and a third-party licence list. See `CHANGELOG.md`.

Not verified on real hardware: sound output, touch devices, Safari/Firefox, the Windows and macOS binaries (built by CI, never run by the maintainer).
