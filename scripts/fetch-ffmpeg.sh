#!/usr/bin/env bash
# Fetch static ffmpeg + ffprobe binaries and stage them into a destination
# directory so they ship next to the app and the Rust core's bundle-aware
# discovery (src/offload/ffprobe.rs) finds them before any host install.
#
# Usage: fetch-ffmpeg.sh <platform> <dest_dir>
#   platform: linux-x64 | macos-arm64 | macos-x64
#   dest_dir: directory to place `ffmpeg` and `ffprobe` into
#
# Sourcing strategy (in order):
#   1. $SEDER_FFMPEG_DIR — a directory you pre-populated with ffmpeg/ffprobe.
#      This is the reproducible, offline-friendly path; prefer it for releases.
#   2. A documented static-build URL per platform (overridable via env), used
#      as a convenience. Builds are GPL, which matches this project's licence.
#
# Fail-soft: if neither source yields the binaries this prints a warning and
# exits 0, so a transient network issue never kills the whole package build.
# Set SEDER_REQUIRE_FFMPEG=1 to make a missing binary a hard error instead.
set -euo pipefail

PLATFORM="${1:?usage: fetch-ffmpeg.sh <platform> <dest_dir>}"
DEST_DIR="${2:?usage: fetch-ffmpeg.sh <platform> <dest_dir>}"
mkdir -p "$DEST_DIR"

# Default download URLs (override any of these in the environment). These point
# at well-known GPL static builds. They are intentionally overridable so a
# release can pin an exact, checksum-verified artifact.
LINUX_X64_URL="${SEDER_FFMPEG_LINUX_X64_URL:-https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz}"
MACOS_FFMPEG_URL="${SEDER_FFMPEG_MACOS_FFMPEG_URL:-https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip}"
MACOS_FFPROBE_URL="${SEDER_FFMPEG_MACOS_FFPROBE_URL:-https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip}"

warn() { echo "warning: $*" >&2; }

fail_or_warn() {
  if [[ "${SEDER_REQUIRE_FFMPEG:-0}" == "1" ]]; then
    echo "error: $*" >&2
    exit 1
  fi
  warn "$* — packaging will continue without a bundled ffmpeg (the app still works if the host has one)."
  exit 0
}

have_both() {
  [[ -x "$DEST_DIR/ffmpeg" && -x "$DEST_DIR/ffprobe" ]]
}

# 1. Vendored directory wins.
if [[ -n "${SEDER_FFMPEG_DIR:-}" ]]; then
  for bin in ffmpeg ffprobe; do
    if [[ -f "$SEDER_FFMPEG_DIR/$bin" ]]; then
      cp "$SEDER_FFMPEG_DIR/$bin" "$DEST_DIR/$bin"
      chmod +x "$DEST_DIR/$bin"
    fi
  done
  if have_both; then
    echo "Bundled ffmpeg/ffprobe from SEDER_FFMPEG_DIR=$SEDER_FFMPEG_DIR"
    exit 0
  fi
  warn "SEDER_FFMPEG_DIR set but did not contain both ffmpeg and ffprobe; falling back to download."
fi

command -v curl >/dev/null 2>&1 || fail_or_warn "curl not available to download ffmpeg"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

case "$PLATFORM" in
  linux-x64)
    echo "Downloading static ffmpeg for $PLATFORM…"
    curl -fsSL "$LINUX_X64_URL" -o "$TMP/ffmpeg.tar.xz" || fail_or_warn "download failed: $LINUX_X64_URL"
    tar -xJf "$TMP/ffmpeg.tar.xz" -C "$TMP" || fail_or_warn "could not extract ffmpeg archive"
    # The tarball expands to a versioned dir containing both binaries.
    found_ffmpeg="$(find "$TMP" -type f -name ffmpeg -print -quit || true)"
    found_ffprobe="$(find "$TMP" -type f -name ffprobe -print -quit || true)"
    [[ -n "$found_ffmpeg" && -n "$found_ffprobe" ]] || fail_or_warn "ffmpeg/ffprobe not found in archive"
    cp "$found_ffmpeg" "$DEST_DIR/ffmpeg"
    cp "$found_ffprobe" "$DEST_DIR/ffprobe"
    ;;
  macos-arm64|macos-x64)
    echo "Downloading static ffmpeg for $PLATFORM…"
    curl -fsSL "$MACOS_FFMPEG_URL" -o "$TMP/ffmpeg.zip" || fail_or_warn "download failed: $MACOS_FFMPEG_URL"
    curl -fsSL "$MACOS_FFPROBE_URL" -o "$TMP/ffprobe.zip" || fail_or_warn "download failed: $MACOS_FFPROBE_URL"
    unzip -o -q "$TMP/ffmpeg.zip" -d "$TMP/ffmpeg-extract" || fail_or_warn "could not unzip ffmpeg"
    unzip -o -q "$TMP/ffprobe.zip" -d "$TMP/ffprobe-extract" || fail_or_warn "could not unzip ffprobe"
    found_ffmpeg="$(find "$TMP/ffmpeg-extract" -type f -name ffmpeg -print -quit || true)"
    found_ffprobe="$(find "$TMP/ffprobe-extract" -type f -name ffprobe -print -quit || true)"
    [[ -n "$found_ffmpeg" && -n "$found_ffprobe" ]] || fail_or_warn "ffmpeg/ffprobe not found in archives"
    cp "$found_ffmpeg" "$DEST_DIR/ffmpeg"
    cp "$found_ffprobe" "$DEST_DIR/ffprobe"
    ;;
  *)
    fail_or_warn "unknown platform: $PLATFORM (expected linux-x64, macos-arm64, or macos-x64)"
    ;;
esac

chmod +x "$DEST_DIR/ffmpeg" "$DEST_DIR/ffprobe"
have_both || fail_or_warn "ffmpeg/ffprobe missing after fetch"
echo "Bundled ffmpeg/ffprobe into $DEST_DIR"
