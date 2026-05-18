# Changelog

All notable changes to this project will be documented here. The project
uses automatic patch-version bumps on every push to `main` via the
`Release` workflow, so individual `0.0.x` tags are not enumerated below;
this file tracks meaningful milestones.

## Unreleased

- Removed unused `offload::progress` and `offload::verify` stub modules
  (progress types live in `offload::mod`, verification lives in
  `offload::engine::verify_file`).
- README signing section reworded: builds are unsigned and verified via
  `SHA256SUMS.txt`; no ad-hoc signing claim.

## v0.0.x series — initial public surface

- Qt 6/QML DIT verification interface with light/dark theme,
  per-destination state machine, and virtualized result table.
- Rust core with BLAKE3 checksums, multi-destination offload, configurable
  ignore patterns, hidden/system file filtering, and cancellation.
- TXT, CSV, and ASC MHL v2.0 report exports backed by source checksums.
- C FFI bridge between the Qt frontend and the Rust engine, with
  threaded copy/verify work kept off the UI thread.
- GitHub Actions CI (fmt, clippy, tests on macOS/Linux/Windows) and
  Release workflow that auto-bumps the patch version, builds unsigned
  cross-platform binaries, and publishes `SHA256SUMS.txt`.
