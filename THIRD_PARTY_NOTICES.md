# Third-Party Notices

SEDER DIT Tool uses open-source dependencies.

## Runtime And UI

- Qt 6 is used for the C++/QML desktop application. See the Qt open-source licensing documentation for the terms that apply to the Qt components used in your build.

## FFmpeg (bundled)

Release builds bundle the `ffmpeg` and `ffprobe` binaries from the [FFmpeg project](https://ffmpeg.org/) to power thumbnail generation and clip-metadata extraction. The bundled binaries are **GPL** static builds, which is compatible with this project's GPL-3.0-only license.

- FFmpeg is licensed under the GPL/LGPL; see <https://ffmpeg.org/legal.html>.
- Corresponding FFmpeg source is available from <https://ffmpeg.org/download.html> and from the upstream build provider used for the platform artifact.
- The exact build source URLs are recorded in `scripts/fetch-ffmpeg.sh` (and the Windows packaging script) and can be pinned/overridden per release via the `SEDER_FFMPEG_*` environment variables.
- If a packaged build does not include FFmpeg, the app falls back to any `ffmpeg`/`ffprobe` found on the host `PATH`; thumbnail and metadata features are simply disabled when none is available.

## Rust Dependencies

- `anyhow`
- `blake3`
- `md-5`
- `sha1`
- `xxhash-rust`
- `crossbeam-channel`
- `globset`
- `walkdir`
- `serde` / `serde_json`
- `tempfile` (tests only)

Run `cargo metadata --manifest-path Cargo.toml` to inspect the exact dependency graph for a checked-out revision.
