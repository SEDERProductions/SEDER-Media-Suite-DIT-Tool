#!/usr/bin/env bash
# fetch-ffmpeg.sh — Download GPL static ffmpeg/ffprobe for the current platform.
#
# Usage: source this file, then call fetch_ffmpeg <output_dir>
#
# Env vars:
#   SEDER_FFMPEG_DIR   – if set, copies pre-downloaded binaries from this path
#                        instead of hitting the network (CI/offline use).
#   SEDER_REQUIRE_FFMPEG=1 – abort if download/copy fails (default: warn and skip).
set -euo pipefail

fetch_ffmpeg() {
    local out_dir="${1:?output directory required}"
    mkdir -p "$out_dir"

    # If a vendor directory is provided, use it directly.
    if [[ -n "${SEDER_FFMPEG_DIR:-}" ]]; then
        echo "[fetch-ffmpeg] Copying from SEDER_FFMPEG_DIR=$SEDER_FFMPEG_DIR"
        for bin in ffmpeg ffprobe; do
            for candidate in "$SEDER_FFMPEG_DIR/$bin" "$SEDER_FFMPEG_DIR/${bin}.exe"; do
                if [[ -f "$candidate" ]]; then
                    cp "$candidate" "$out_dir/"
                    chmod +x "$out_dir/$(basename "$candidate")"
                fi
            done
        done
        return 0
    fi

    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    if [[ "$os" == "Darwin" ]]; then
        if [[ "$arch" == "arm64" ]]; then
            local url="https://evermeet.cx/ffmpeg/getrelease/zip"
            local url_probe="https://evermeet.cx/ffprobe/getrelease/zip"
        else
            local url="https://evermeet.cx/ffmpeg/getrelease/zip"
            local url_probe="https://evermeet.cx/ffprobe/getrelease/zip"
        fi
        echo "[fetch-ffmpeg] Downloading macOS ffmpeg/ffprobe (GPL static)..."
        _download_and_unzip "$url" ffmpeg "$out_dir" || _ffmpeg_fail
        _download_and_unzip "$url_probe" ffprobe "$out_dir" || _ffmpeg_fail

    elif [[ "$os" == "Linux" ]]; then
        # Use John Van Sickle's GPL static builds
        local base_url="https://johnvansickle.com/ffmpeg/releases"
        if [[ "$arch" == "x86_64" ]]; then
            local tarball="ffmpeg-release-amd64-static.tar.xz"
        elif [[ "$arch" == "aarch64" ]]; then
            local tarball="ffmpeg-release-arm64-static.tar.xz"
        else
            echo "[fetch-ffmpeg] Unsupported arch: $arch" >&2
            _ffmpeg_fail
            return
        fi
        echo "[fetch-ffmpeg] Downloading Linux ffmpeg/ffprobe (GPL static)..."
        local tmp
        tmp="$(mktemp -d)"
        trap "rm -rf $tmp" EXIT
        curl -fsSL "$base_url/$tarball" -o "$tmp/ffmpeg.tar.xz"
        tar -xf "$tmp/ffmpeg.tar.xz" -C "$tmp" --strip-components=1
        cp "$tmp/ffmpeg" "$tmp/ffprobe" "$out_dir/"
        chmod +x "$out_dir/ffmpeg" "$out_dir/ffprobe"

    else
        echo "[fetch-ffmpeg] Unsupported OS: $os (Windows handled by package-windows.ps1)" >&2
        return 0
    fi

    echo "[fetch-ffmpeg] ffmpeg/ffprobe placed in $out_dir"
}

_download_and_unzip() {
    local url="$1" name="$2" out_dir="$3"
    local tmp
    tmp="$(mktemp -d)"
    curl -fsSL "$url" -o "$tmp/$name.zip"
    unzip -q "$tmp/$name.zip" -d "$tmp"
    local bin
    bin="$(find "$tmp" -maxdepth 2 -name "$name" -type f | head -1)"
    if [[ -z "$bin" ]]; then
        rm -rf "$tmp"
        return 1
    fi
    cp "$bin" "$out_dir/$name"
    chmod +x "$out_dir/$name"
    rm -rf "$tmp"
}

_ffmpeg_fail() {
    if [[ "${SEDER_REQUIRE_FFMPEG:-0}" == "1" ]]; then
        echo "[fetch-ffmpeg] FATAL: Could not obtain ffmpeg binaries." >&2
        exit 1
    else
        echo "[fetch-ffmpeg] WARNING: Could not obtain ffmpeg binaries; thumbnails will be unavailable." >&2
    fi
}
