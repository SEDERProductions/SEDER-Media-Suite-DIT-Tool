# Changelog

All notable changes to this project will be documented here. The project
uses automatic patch-version bumps on every push to `main` via the
`Release` workflow, so individual `0.0.x` tags are not enumerated below;
this file tracks meaningful milestones.

## Unreleased — towards v1.0.0

A large rework of the DIT feature surface. The core engine that scans,
copies, verifies, and reports is unchanged in spirit but extended in
every direction.

### Reliability
- Transient I/O errors (Interrupted, WouldBlock, TimedOut,
  ConnectionReset, ConnectionAborted, BrokenPipe) now retry up to three
  times with exponential backoff (50 / 100 / 200 ms) at every read,
  write, open, create, and sync site in the engine, with unit tests
  for the retry policy itself.
- New crash-recovery checkpoint format (`offload::checkpoint`) with
  atomic save (.tmp + rename) so partial writes never corrupt the
  on-disk state. FFI exposes save / load / clear; the UI hookup for
  "resume previous offload?" is scaffolded.

### Hashing
- Pluggable checksum algorithms via a `FileHasher` trait: BLAKE3 (the
  unchanged default), MD5, SHA-1, XXH3-64, XXH3-128. Picked per
  offload from the Preferences dialog and persisted via SettingsStore.
- MHL `<hashmethod>` element now reflects the actual algorithm used
  rather than hardcoding blake3.

### Reports
- MHL output extended with `<creatorinfo>` (tool name + version,
  creation date, project, shoot date, card, camera) and `<ignored>`
  blocks (each path the scan filtered out, XML-escaped).
- New ALE (Avid Log Exchange) sidecar export with Heading / Column /
  Data sections, FPS derived from clip metadata when present.
- New JSON metadata sidecar emitted when `extract_metadata` is on:
  per-file hash + algo + media kind + the ffprobe payload.
- TXT report grows a per-format breakdown (R3D / ARRI / BRAW / MXF /
  MOV / MP4 / MPEG-TS / Audio / Subtitle / Sidecar / Other) and a
  one-line skipped-file count.

### Media
- New `offload::media` classifier — extension-based, conservative,
  case-insensitive. Drives the format breakdown above.
- New `offload::ffprobe` discovery + JSON parser. ffprobe is a soft
  runtime dependency: missing binary downgrades to a no-op.
- New `offload::thumbnail` content-addressed cache: ffmpeg pulls a
  poster frame per clip into `<cache_dir>/<algo>-<hash>.jpg`.
- New `offload::proxy` with three presets (ProRes Proxy 1080p,
  H.264 720p, DNxHR LB). Outputs to `<root>/<preset>/<stem>.<ext>`,
  idempotent on re-run.
- New `offload::volume::is_ltfs_volume` — parses /proc/mounts on
  Linux, `mount` on macOS, PowerShell `Get-Volume` on Windows.

### UI / UX
- New `SettingsStore` (QSettings-backed) persists window geometry,
  recent source / destination paths (up to 10 each), last metadata,
  all default offload toggles, default checksum algorithm,
  destination subfolder template, and the extract-metadata flag.
- Native menu bar via `Qt.labs.platform` with File / Edit / Help
  menus and a coherent set of shortcuts: Ctrl+O (open source),
  Ctrl+D (add destination), Ctrl+E (export TXT), Ctrl+, (preferences),
  Ctrl+Q (quit), Esc (cancel running offload).
- About dialog and full Preferences dialog (theme, default toggles,
  default ignore patterns, default checksum algorithm, destination
  template with live preview, extract-metadata with availability
  hint).
- Drag-and-drop folders onto the source picker and the destinations
  area; Recent ▾ menu on the source picker.
- Destination path templates: `{project}/{shoot_date}/{card}/{camera}`
  with case-insensitive tokens, filesystem-hostile char sanitization
  per component, and a live preview in Preferences.
- File menu exports TXT / CSV / MHL / Metadata JSON / ALE.

### Documentation
- New `docs/architecture.md` describing process model, FFI contract,
  threading and cancellation, reliability, settings, external deps,
  and build flow.
- New `CONTRIBUTING.md` covering the dev loop, branching, commit
  style, test expectations, FFI conventions, and release process.

## v0.0.x series — initial public surface

- Qt 6/QML DIT verification interface with light/dark theme,
  per-destination state machine, and virtualized result table.
- Rust core with BLAKE3 checksums, multi-destination offload,
  configurable ignore patterns, hidden/system file filtering, and
  cancellation.
- TXT, CSV, and ASC MHL v2.0 report exports backed by source
  checksums.
- C FFI bridge between the Qt frontend and the Rust engine, with
  threaded copy/verify work kept off the UI thread.
- GitHub Actions CI (fmt, clippy, tests on macOS/Linux/Windows) and
  Release workflow that auto-bumps the patch version, builds unsigned
  cross-platform binaries, and publishes `SHA256SUMS.txt`.
