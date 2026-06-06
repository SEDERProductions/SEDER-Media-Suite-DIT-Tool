# Third-Party Notices

SEDER DIT Tool uses open-source dependencies.

## Runtime And UI

- **Qt 6** is used for the C++/QML desktop application under the GNU Lesser General Public License v3 (LGPL-3.0) or the Qt Commercial License. See [qt.io/licensing](https://www.qt.io/licensing/) for the terms that apply to the Qt components used in your build.

## Rust Dependencies

- `anyhow` — MIT OR Apache-2.0
- `blake3` — CC0-1.0 / Apache-2.0
- `twox-hash` / `xxhash-rust` — MIT
- `walkdir` — MIT OR Apache-2.0
- `globset` — MIT OR Apache-2.0
- `serde` / `serde_json` — MIT OR Apache-2.0
- `crossbeam-channel` — MIT OR Apache-2.0
- `tempfile` (tests only) — MIT OR Apache-2.0

Run `cargo metadata --manifest-path Cargo.toml` to inspect the exact dependency graph for a checked-out revision.

## FFmpeg (optional / bundled in release packages)

When distributed as a release package, SEDER Media Suite DIT optionally bundles
**FFmpeg** and **FFprobe** static binaries to enable video thumbnail extraction.

FFmpeg is licensed under the **GNU General Public License version 2 or later
(GPL-2.0-or-later)**. SEDER Media Suite DIT is licensed under GPL-3.0-only,
which is compatible with GPLv2+ for distribution purposes.

Because GPLv2+ source code is being distributed with this application, you may
obtain the corresponding FFmpeg source code by:

1. Downloading it from [ffmpeg.org/download.html](https://ffmpeg.org/download.html), or
2. Sending a written request to: Seder Productions, c/o tahaseder@protonmail.com

The FFmpeg source offer remains valid for three years from the date of distribution.

FFmpeg is Copyright © the FFmpeg developers and contributors. A full list of
contributors is available in the FFmpeg source tree at `CREDITS`.
