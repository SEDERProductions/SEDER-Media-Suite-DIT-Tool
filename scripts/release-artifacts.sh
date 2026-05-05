#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_FILE="$ROOT_DIR/Cargo.toml"

PACKAGE_NAME="$(sed -n 's/^name = "\(.*\)"/\1/p' "$CARGO_FILE" | head -1)"
PACKAGE_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$CARGO_FILE" | head -1)"

if [[ -z "$PACKAGE_NAME" || -z "$PACKAGE_VERSION" ]]; then
  echo "Unable to derive package metadata from Cargo.toml" >&2
  exit 1
fi

BASE_NAME="${PACKAGE_NAME}"

version_input="${VERSION:-$PACKAGE_VERSION}"
version_tag="${version_input#v}"

artifact_name() {
  local platform="$1"
  case "$platform" in
    linux-x64) echo "${BASE_NAME}-v${version_tag}-linux-x64.tar.gz" ;;
    windows-x64) echo "${BASE_NAME}-v${version_tag}-windows-x64.zip" ;;
    macos-arm64) echo "${BASE_NAME}-v${version_tag}-macos-arm64.zip" ;;
    macos-x64) echo "${BASE_NAME}-v${version_tag}-macos-x64.zip" ;;
    *)
      echo "Unknown platform: $platform" >&2
      exit 1
      ;;
  esac
}

usage() {
  cat <<USAGE
Usage:
  scripts/release-artifacts.sh [--base-name|--version|--pattern|--list|--filename PLATFORM]
USAGE
}

case "${1:-}" in
  --base-name)
    echo "$BASE_NAME"
    ;;
  --version)
    echo "$version_tag"
    ;;
  --pattern)
    echo "${BASE_NAME}-v<version>-<platform>.<ext>"
    ;;
  --list)
    for platform in macos-arm64 macos-x64 windows-x64 linux-x64; do
      printf '%s\n' "$(artifact_name "$platform")"
    done
    ;;
  --filename)
    [[ -n "${2:-}" ]] || { usage; exit 1; }
    artifact_name "$2"
    ;;
  *)
    usage
    exit 1
    ;;
esac
