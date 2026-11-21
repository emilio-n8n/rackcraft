#!/usr/bin/env bash
# Build a RackCraft AppImage from an existing Linux binary.
#
# Usage:
#   APPIMAGETOOL=/path/to/appimagetool ./tools/build_appimage.sh \
#       target/release/rackcraft dist/rackcraft-x86_64.AppImage 0.1.1
#
# The binary should be built on an old-glibc system for broad compatibility
# (GitHub Actions uses Ubuntu 22.04 for this reason).
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BIN=${1:?usage: build_appimage.sh <binary> <output.AppImage> [version]}
OUT=${2:?usage: build_appimage.sh <binary> <output.AppImage> [version]}
VERSION=${3:-${VERSION:-0.1.1}}
APPIMAGETOOL=${APPIMAGETOOL:-$(command -v appimagetool || true)}
ARCH=${ARCH:-x86_64}

if [[ ! -x "$BIN" ]]; then
    echo "error: binary not found or not executable: $BIN" >&2
    exit 2
fi
if [[ -z "$APPIMAGETOOL" || ! -x "$APPIMAGETOOL" ]]; then
    echo "error: appimagetool not found; set APPIMAGETOOL=/path/to/appimagetool" >&2
    exit 2
fi

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
APPDIR="$WORK/RackCraft.AppDir"

install -Dm755 "$BIN" "$APPDIR/usr/bin/rackcraft"
install -Dm644 "$ROOT_DIR/packaging/rackcraft.desktop" \
    "$APPDIR/usr/share/applications/rackcraft.desktop"
install -Dm644 "$ROOT_DIR/assets/rackcraft.png" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps/rackcraft.png"
install -Dm755 "$ROOT_DIR/packaging/AppRun" "$APPDIR/AppRun"

# AppImage metadata lives at the AppDir root.
cp "$ROOT_DIR/packaging/rackcraft.desktop" "$APPDIR/"
cp "$ROOT_DIR/assets/rackcraft.png" "$APPDIR/"
ln -sf rackcraft.png "$APPDIR/.DirIcon"

mkdir -p "$(dirname "$OUT")"
ARCH="$ARCH" VERSION="$VERSION" APPIMAGE_EXTRACT_AND_RUN=1 \
    "$APPIMAGETOOL" --no-appstream "$APPDIR" "$OUT"
chmod +x "$OUT"
echo "built $OUT ($(du -h "$OUT" | cut -f1))"
