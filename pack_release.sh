#!/usr/bin/env bash
# Source archive without build output and the debug captures (frames/): ./pack_release.sh ../turbocursa.zip
set -euo pipefail
out="${1:-../turbocursa.zip}"
cd "$(dirname "$0")/.."
rm -f "$out"
zip -qr "$out" "$(basename "$OLDPWD")" -x '*/target/*' '*/.git/*' '*/frames/*'
ls -la "$out"
