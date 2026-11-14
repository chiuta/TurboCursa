**TURBOCURSA 1.9.0**: clean-room pseudo-3D arcade racer in Rust, native (minifb) and web (WASM), 8 languages, MIT.

- **Play in the browser:** download `turbocursa.html` and open it (a single self-contained file, no server needed), or install the web app from the presentation page.
- **Native:** `turbocursa-linux-x86_64.tar.gz`, `turbocursa-windows-x86_64.zip`, `turbocursa-macos-arm64.tar.gz`, `turbocursa-macos-x86_64.tar.gz` (or `cargo run --release --features native,audio --bin turbo`).
- **Source:** `turbocursa-src.zip`. Every file is listed in `SHA256SUMS`; the one-line installers check it before they unpack anything.

What is new: **ten more special vehicles** on the road (snowplough, tow truck, road-works truck, street sweeper, cash van, school bus, taxi, army truck, ice-cream van with a jingle, and a slow tractor), each with its own look and sound, and **2K, 3K, 4K and 8K output** next to Full HD (`--res hd|2k|3k|4k|8k` natively; the `HD` button or `F2` cycles SD, HD, 2K, 3K, 4K, 8K in the browser). These are sharp upscales of the 640×400 picture. See `CHANGELOG.md`.

Not verified on real hardware: sound output, touch devices, Safari/Firefox, the Windows and macOS binaries (built by CI, never run by the maintainer).
