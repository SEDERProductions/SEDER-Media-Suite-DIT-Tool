#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${VERSION:-$(python3 "$ROOT_DIR/scripts/package_common.py" version --root "$ROOT_DIR")}"
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


python3 "$ROOT_DIR/scripts/generate-icons.py" "$ROOT_DIR"

rm -rf "$BUILD_DIR" "$INSTALL_DIR"
mkdir -p "$ARTIFACT_DIR" "$INSTALL_DIR"

CMAKE_EXTRA_ARGS=()
if [[ "$ARCH" == "x86_64" ]]; then
    CMAKE_EXTRA_ARGS+=("-DCMAKE_OSX_ARCHITECTURES=x86_64")
    export SEDER_RUST_TARGET="x86_64-apple-darwin"
fi

cmake -S "$ROOT_DIR/qt" -B "$BUILD_DIR" -G "$GENERATOR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
  -DBUILD_TESTING=OFF \
  ${QT_PREFIX_ARGS[@]+"${QT_PREFIX_ARGS[@]}"} \
  ${CMAKE_EXTRA_ARGS[@]+"${CMAKE_EXTRA_ARGS[@]}"}
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

ARTIFACT_NAME="$(python3 "$ROOT_DIR/scripts/package_common.py" artifact-name --version "$VERSION" --platform "$PLATFORM")"
ARTIFACT="$ARTIFACT_DIR/$ARTIFACT_NAME"
rm -f "$ARTIFACT"
ditto -c -k --keepParent "$APP_BUNDLE" "$ARTIFACT"

python3 "$ROOT_DIR/scripts/package_common.py" checksums --artifact-dir "$ARTIFACT_DIR" >/dev/null
echo "Packaged $ARTIFACT"
