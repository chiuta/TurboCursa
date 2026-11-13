#!/bin/sh
# TURBOCURSA installer for Linux and macOS.
#   curl -fsSL https://turbocursa.chiuta.ro/install.sh | sh
# Downloads the latest release for your system into ~/.local/bin and, on Linux, adds a menu launcher.
# If no release is available it builds from source with cargo.
set -eu
REPO="chiuta/TurboCursa"
BIN="${HOME}/.local/bin"
case "$(uname -s)" in
  Linux)  OS=linux ;;
  Darwin) OS=macos ;;
  *) echo "Unsupported system: $(uname -s). On Windows use install.ps1."; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) ARCH=x86_64 ;;
  arm64|aarch64) ARCH=arm64 ;;
  *) ARCH=unknown ;;
esac
NAME="turbocursa-${OS}-${ARCH}.tar.gz"
BASE="https://github.com/${REPO}/releases/latest/download"
mkdir -p "$BIN"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
  else echo ""; fi
}
# download, then check the archive against the release's SHA256SUMS before anything is unpacked
fetch_verified() {
  curl -fsSL "${BASE}/${NAME}" -o "$TMP/pkg.tar.gz" 2>/dev/null || return 1
  curl -fsSL "${BASE}/SHA256SUMS" -o "$TMP/SHA256SUMS" 2>/dev/null || { echo "Cannot fetch SHA256SUMS; refusing to install an unverified download."; return 1; }
  WANT="$(grep " ${NAME}\$" "$TMP/SHA256SUMS" | cut -d' ' -f1)"
  HAVE="$(sha256_of "$TMP/pkg.tar.gz")"
  if [ -z "$WANT" ] || [ -z "$HAVE" ] || [ "$WANT" != "$HAVE" ]; then
    echo "Checksum mismatch (or no sha256 tool); refusing to install."; return 1
  fi
}
if fetch_verified; then
  # unpack only the one file we expect
  tar -xzf "$TMP/pkg.tar.gz" -C "$TMP" turbocursa 2>/dev/null || tar -xzf "$TMP/pkg.tar.gz" -C "$TMP" ./turbocursa
  install -m 755 "$TMP/turbocursa" "$BIN/turbocursa"
  echo "Installed: $BIN/turbocursa (checksum verified)"
elif command -v cargo >/dev/null 2>&1; then
  echo "No release binary for ${OS}-${ARCH}; building from source (a few minutes)..."
  cargo install --git "https://github.com/${REPO}" --features native,audio --bin turbo --root "${HOME}/.local" --force
  mv "$BIN/turbo" "$BIN/turbocursa"
else
  echo "No release binary for ${OS}-${ARCH} and cargo is not installed."
  echo "Install Rust from https://rustup.rs and run this script again."; exit 1
fi
if [ "$OS" = linux ]; then
  APPS="${HOME}/.local/share/applications"; mkdir -p "$APPS"
  cat > "$APPS/turbocursa.desktop" <<D
[Desktop Entry]
Type=Application
Name=TURBOCURSA
Comment=Pseudo-3D arcade racer
Exec="$BIN/turbocursa"
Terminal=false
Categories=Game;
D
  echo "Menu launcher added: TURBOCURSA"
fi
case ":$PATH:" in *":$BIN:"*) ;; *) echo "Add $BIN to your PATH to run: turbocursa";; esac
echo "Run it with: turbocursa   (options: --daily, --weekly, --champ CODE, --code CODE)"
