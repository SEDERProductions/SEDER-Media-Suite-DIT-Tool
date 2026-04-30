#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${VERSION:-}"
if [[ -z "$VERSION" ]]; then
  VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -1)"
fi
VERSION="${VERSION#v}"

BUILD_DIR="${BUILD_DIR:-$ROOT_DIR/qt/build-release-linux}"
APPDIR="${APPDIR:-$ROOT_DIR/dist/linux/AppDir}"
ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/dist/artifacts}"
GENERATOR="${CMAKE_GENERATOR:-Ninja}"
QT_PREFIX_ARGS=()
if [[ -n "${CMAKE_PREFIX_PATH:-}" ]]; then
  QT_PREFIX_ARGS+=("-DCMAKE_PREFIX_PATH=${CMAKE_PREFIX_PATH}")
fi

rm -rf "$BUILD_DIR" "$APPDIR"
mkdir -p "$ARTIFACT_DIR" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/icons/hicolor/scalable/apps"

cmake -S "$ROOT_DIR/qt" -B "$BUILD_DIR" -G "$GENERATOR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$APPDIR/usr" \
  -DBUILD_TESTING=OFF \
  "${QT_PREFIX_ARGS[@]}"
cmake --build "$BUILD_DIR" --config Release
cmake --install "$BUILD_DIR" --config Release

cp "$ROOT_DIR/assets/icon.svg" "$APPDIR/usr/share/icons/hicolor/scalable/apps/seder-dit-tool.svg"
cat > "$APPDIR/usr/share/applications/seder-dit-tool.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=SEDER DIT Tool
Comment=Local DIT folder verification
Exec=seder-media-suite-dit-qt
Icon=seder-dit-tool
Categories=AudioVideo;Utility;
Terminal=false
DESKTOP

TARBALL="$ARTIFACT_DIR/seder-dit-tool-v${VERSION}-linux-x64.tar.gz"
tar -C "$APPDIR/usr" -czf "$TARBALL" .

LINUXDEPLOYQT_BIN="${LINUXDEPLOYQT:-}"
if [[ -n "$LINUXDEPLOYQT_BIN" && -x "$LINUXDEPLOYQT_BIN" ]]; then
  export VERSION
  export APPIMAGE_EXTRACT_AND_RUN="${APPIMAGE_EXTRACT_AND_RUN:-1}"
  (
    cd "$ROOT_DIR/dist/linux"
    "$LINUXDEPLOYQT_BIN" "$APPDIR/usr/share/applications/seder-dit-tool.desktop" \
      -appimage \
      -unsupported-allow-new-glibc || true
  )
  appimage="$(find "$ROOT_DIR/dist/linux" -maxdepth 1 -name '*.AppImage' -print -quit)"
  if [[ -n "$appimage" ]]; then
    mv "$appimage" "$ARTIFACT_DIR/seder-dit-tool-v${VERSION}-linux-x64.AppImage"
  else
    echo "linuxdeployqt ran but did not produce an AppImage" >&2
    exit 1
  fi
fi

echo "Packaged Linux artifacts in $ARTIFACT_DIR"
