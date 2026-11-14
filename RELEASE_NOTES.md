**TURBOCURSA 1.12.0**: clean-room pseudo-3D arcade racer in Rust, native (minifb) and web (WASM), 8 languages, MIT.

- **Play in the browser:** download `turbocursa.html` and open it (a single self-contained file, no server needed), or install the web app from the presentation page.
- **Native:** `turbocursa-linux-x86_64.tar.gz`, `turbocursa-windows-x86_64.zip`, `turbocursa-macos-arm64.tar.gz`, `turbocursa-macos-x86_64.tar.gz` (or `cargo run --release --features native,audio --bin turbo`).
- **Source:** `turbocursa-src.zip`. Every file is listed in `SHA256SUMS`; the one-line installers check it before they unpack anything.

What is new: fractal detail on more of the picture: building facades (concrete grain, panel joints, rain streaks, stains, chips), trees and rocks (ambient occlusion, sun-side rim light, leaf and bark mottling, rock strata, cracks and lichen), on top of the road and ground texture from 1.11.0. About 3 ms per frame more than 1.10.2 in total. See `CHANGELOG.md`.

Not verified on real hardware: sound output, touch devices, Safari/Firefox, the Windows and macOS binaries (built by CI, never run by the maintainer).
