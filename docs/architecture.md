# Architecture

This document describes how the SEDER Media Suite DIT Tool is wired
together. It is the place to look before making architectural changes,
adding a new platform, or introducing a new module.

## Top-level layout

```
SEDER-Media-Suite-DIT-Tool/
├── Cargo.toml             # Rust crate manifest (static lib + rlib)
├── src/                   # Rust core
│   ├── lib.rs             # re-exports
│   ├── ffi.rs             # C ABI surface used by Qt
│   ├── report.rs          # TXT / CSV / MHL / ALE / JSON formatters
│   └── offload/
│       ├── mod.rs         # shared data structs
│       ├── engine.rs      # scan + copy + verify pipeline
│       ├── hash.rs        # checksum algorithms (trait + impls)
│       ├── ffprobe.rs     # ffprobe discovery + JSON parsing
│       ├── thumbnail.rs   # ffmpeg-based thumbnails, content-addressed
│       ├── proxy.rs       # ffmpeg-based proxy presets
│       ├── media.rs       # MediaKind classifier, format breakdown
│       ├── template.rs    # destination path template expander
│       ├── volume.rs      # same-volume + LTFS detection
│       └── checkpoint.rs  # crash-recovery JSON state file
├── tests/integration.rs   # end-to-end Rust offload tests
└── qt/
    ├── CMakeLists.txt     # Qt build, links libseder_dit_tool.a
    ├── cpp/               # Qt 6 application
    │   ├── main.cpp
    │   ├── AppController.{h,cpp}
    │   ├── DestinationItem.{h,cpp}
    │   ├── DestinationListModel.{h,cpp}
    │   ├── DitOffloadWorker.{h,cpp}
    │   ├── SettingsStore.{h,cpp}
    │   ├── ThemeController.{h,cpp}
    │   └── seder_ffi.h       # C ABI mirror of src/ffi.rs
    ├── qml/               # QtQuick UI
    └── tests/             # QtTest C++ unit tests
```

## Process model

The app is a single OS process. Inside it there are three logical
worlds:

1. **Qt main thread** — owns the QML engine, the `QApplication`, all
   user-visible state via `AppController`, the persistent settings via
   `SettingsStore`, and the native menu bar (`Qt.labs.platform`). It
   never blocks on I/O: all long-running work moves off it.
2. **Qt worker thread** — one `QThread` per active offload. The worker
   is `DitOffloadWorker`, which calls into the Rust FFI. Progress
   events come back through a C callback that the worker re-emits as
   Qt signals on its own thread; AppController bridges those to the UI
   via queued connections.
3. **Rust core** — the offload engine spins up additional std::thread
   writers (one per destination) so source-read I/O and destination
   writes overlap. ffmpeg / ffprobe invocations are out-of-process
   children.

## Rust ⇄ C++ contract

The boundary is a small C ABI in `src/ffi.rs`, mirrored in
`qt/cpp/seder_ffi.h`. The shape is:

- `SederOffloadRequest` (input struct) carries all knobs: source,
  destinations, ignore patterns, verify flag, checksum algorithm,
  extract_metadata flag, a cancel token (1-byte atomic owned by Qt),
  and project metadata.
- `seder_offload_start` runs synchronously inside the calling thread
  (which is the Qt worker thread) and emits `SederOffloadProgress`
  events via the supplied callback.
- On success it returns an `OffloadReportHandle *` that owns the
  serialized report exports (TXT, CSV, MHL, metadata JSON, ALE) as
  CStrings. Qt fetches each via a `seder_report_export_*` accessor —
  the strings are borrowed, lifetime is the handle.
- The handle is freed with `seder_report_free`. Any heap-allocated
  string Rust hands out (template expansion, thumbnail path, proxy
  output path, checkpoint JSON, expanded template) must be freed by
  Qt with `seder_string_free`.

There is no shared mutable state across the boundary; every call
copies in and out.

## Threading & cancellation

- Cancellation is cooperative: Qt sets a byte in a single-byte cancel
  token. The Rust engine reads it in tight loops (per-chunk during
  copy, per-file during verify). A cancellation cleanly tears down
  writer threads via the channels they listen on.
- Destination writer threads communicate with the source reader via
  bounded `crossbeam_channel` (CHANNEL_BOUND = 16 1-MiB chunks) so
  writers can back-pressure the reader without unbounded buffering.
- Hash computation happens on the source-reader thread (during scan)
  and again on the verify pass (during read-back). The trait
  `hash::FileHasher` lets us swap algorithms without touching the
  pipeline.

## Reliability

- All I/O sites (`File::open`, `File::create`, `read`, `write_all`,
  `sync_data`) are wrapped in `retry_io` which retries up to 3 times
  with exponential backoff (50 / 100 / 200 ms) on transient
  `io::ErrorKind`s (Interrupted, WouldBlock, TimedOut, ConnectionReset,
  ConnectionAborted, BrokenPipe). Permission denied / not found /
  invalid input fail fast.
- The checkpoint writer (`offload::checkpoint::save`) uses atomic
  `.tmp` → rename so partial writes never corrupt the on-disk state.
- ffmpeg / ffprobe subprocess failures clean up partial output files
  before returning Err so the next invocation re-runs from scratch.

## Settings persistence

Everything user-visible that should survive across launches goes
through `SettingsStore` (Qt-side, `QSettings`-backed). That includes
window geometry, recent source / destination paths (max 10 each), last
project / shoot / card / camera metadata, all default offload toggles,
the default checksum algorithm, the destination subfolder template,
and the "extract metadata" flag.

Theme preference lives in `ThemeController` (legacy reasons — would
fit equally in `SettingsStore`).

## External dependencies

| Capability                | Binary required | Behavior when missing                |
|---------------------------|-----------------|--------------------------------------|
| Hashing                   | none            | always works                         |
| Reports (TXT/CSV/MHL/ALE) | none            | always works                         |
| Clip metadata sidecar     | `ffprobe`       | toggle disabled in Preferences       |
| Thumbnails                | `ffmpeg`        | extract returns Err                  |
| Proxy transcoding         | `ffmpeg`        | transcode returns Err                |

Discovery walks `$PATH` plus OS-specific fallback dirs
(`/opt/homebrew/bin`, `/usr/local/bin`, `/opt/local/bin` on macOS;
`/usr/local/bin`, `/usr/bin`, `/snap/bin` on Linux;
`C:\Program Files\ffmpeg\bin`, `C:\ffmpeg\bin` on Windows).

## Build flow

```
cargo build --release           # produces target/release/libseder_dit_tool.a
cmake -S qt -B qt/build         # CMakeLists.txt invokes cargo as a custom target
cmake --build qt/build          # compiles Qt code, links the static rust lib
```

The CMake `seder_dit_rust_build` custom target wraps `cargo build` so
each Qt CMake config (Debug / Release / RelWithDebInfo / MinSizeRel)
points at the corresponding Cargo profile. `RUST_SOURCES` enumerates
the .rs files so CMake invalidates the build when any of them change.

## Release

GitHub Actions auto-bumps the patch version on every push to `main`
(`.github/workflows/release.yml`) and publishes unsigned cross-platform
binaries plus `SHA256SUMS.txt`. No signing is configured: macOS users
right-click → Open the first time, Windows users SmartScreen → Run
Anyway, Linux users `chmod +x`.
