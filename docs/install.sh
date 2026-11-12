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
URL="https://github.com/${REPO}/releases/latest/download/${NAME}"
mkdir -p "$BIN"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
if curl -fsSL "$URL" -o "$TMP/pkg.tar.gz" 2>/dev/null; then
  tar -xzf "$TMP/pkg.tar.gz" -C "$TMP"
  install -m 755 "$TMP/turbocursa" "$BIN/turbocursa"
  echo "Installed: $BIN/turbocursa"
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
Exec=$BIN/turbocursa
Terminal=false
Categories=Game;
D
  echo "Menu launcher added: TURBOCURSA"
fi
case ":$PATH:" in *":$BIN:"*) ;; *) echo "Add $BIN to your PATH to run: turbocursa";; esac
echo "Run it with: turbocursa   (options: --daily, --weekly, --champ CODE, --code CODE)"
