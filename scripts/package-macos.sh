#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${VERSION:-}"
if [[ -z "$VERSION" ]]; then
  VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -1)"
fi
VERSION="${VERSION#v}"

ARCH="${ARCH:-$(uname -m)}"
case "$ARCH" in
  arm64) PLATFORM="macos-arm64" ;;
  x86_64|amd64) PLATFORM="macos-x64" ;;
  *) PLATFORM="macos-${ARCH}" ;;
esac

BUILD_DIR="${BUILD_DIR:-$ROOT_DIR/qt/build-release-${PLATFORM}}"
INSTALL_DIR="${INSTALL_DIR:-$ROOT_DIR/dist/${PLATFORM}}"
ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/dist/artifacts}"
GENERATOR="${CMAKE_GENERATOR:-Ninja}"
QT_PREFIX_ARGS=()
if [[ -n "${CMAKE_PREFIX_PATH:-}" ]]; then
  QT_PREFIX_ARGS+=("-DCMAKE_PREFIX_PATH=${CMAKE_PREFIX_PATH}")
fi

rm -rf "$BUILD_DIR" "$INSTALL_DIR"
mkdir -p "$ARTIFACT_DIR" "$INSTALL_DIR"

cmake -S "$ROOT_DIR/qt" -B "$BUILD_DIR" -G "$GENERATOR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
  -DBUILD_TESTING=OFF \
  ${QT_PREFIX_ARGS[@]+"${QT_PREFIX_ARGS[@]}"}
cmake --build "$BUILD_DIR" --config Release
cmake --install "$BUILD_DIR" --config Release

APP_BUNDLE="$(find "$INSTALL_DIR" -maxdepth 1 -name '*.app' -type d -print -quit)"
if [[ -z "$APP_BUNDLE" ]]; then
  echo "No .app bundle found in $INSTALL_DIR" >&2
  exit 1
fi

codesign --force --deep --options runtime \
  --identifier com.sederproductions.media-suite.dit-qt \
  --sign - \
  --timestamp=none \
  "$APP_BUNDLE"
codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE"

ARTIFACT="$ARTIFACT_DIR/seder-dit-tool-v${VERSION}-${PLATFORM}.zip"
rm -f "$ARTIFACT"
ditto -c -k --keepParent "$APP_BUNDLE" "$ARTIFACT"

echo "Packaged $ARTIFACT"
