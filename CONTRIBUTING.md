# Contributing

Thanks for taking the time. This document covers the day-to-day flow
for changes to the SEDER Media Suite DIT Tool. For the big picture of
how the code is organized, see [docs/architecture.md](docs/architecture.md).

## Prerequisites

- Rust stable toolchain
- Qt 6.4 or newer (Core, Gui, Quick, QuickControls2, Widgets, Test)
- CMake 3.21 or newer
- Ninja (or another CMake-supported generator)
- Optional at runtime, used by the metadata/thumbnail/proxy features:
  `ffmpeg` and `ffprobe`

## Local development loop

```sh
# Rust core
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --manifest-path Cargo.toml

# Qt
cmake -S qt -B qt/build -G Ninja -DBUILD_TESTING=ON
cmake --build qt/build
ctest --test-dir qt/build --output-on-failure
```

All four of those must be green before you push.

## Branching

- `main` is the only long-lived branch. CI auto-bumps the patch
  version and publishes a release on every push to it.
- Feature work goes on a feature branch and lands via PR.
- The PR is the unit of review; please keep commits focused.

## Commit style

Look at `git log` for the local convention — single-sentence subject
in active voice ("Add ffprobe metadata sidecar"), followed by a short
body explaining the *why*. We avoid commit message footers like
`Signed-off-by` and the like.

## Tests

- Rust: add unit tests inside the module under `#[cfg(test)]`, or
  bigger flows to `tests/integration.rs`.
- Qt: add to `qt/tests/dit_model_tests.cpp` (the existing harness).
- Anything that touches the FFI should grow a test on the Rust side
  that exercises the parsing / serialization logic, even if a real
  end-to-end FFI test isn't practical.

## Code conventions

- Rust: rustfmt + `clippy::all` clean (CI runs `-D warnings`). Prefer
  `anyhow::Result` at the boundary, `thiserror` only if a downstream
  needs typed errors.
- C++: 4-space indent, headers under `qt/cpp/`, `pragma once`, follow
  the surrounding `Q_PROPERTY` patterns.
- QML: lowerCamelCase ids, properties top, signals next, layouts
  last. Reach for the styled wrappers (`QuietButton`,
  `StyledCheckBox`, `StyledComboBox`) before introducing new
  primitives.
- No emojis in code or commit messages unless they're a deliberate
  part of the UI surface (e.g. the existing state glyphs in
  `Main.qml`'s `destinationStateLabel`).

## Adding an FFI export

1. Add the function to `src/ffi.rs` with `#[no_mangle] pub
   unsafe extern "C"`, wrap the body in `catch_unwind`, and return
   sensible defaults on panic (0 / null / empty string).
2. Mirror the prototype in `qt/cpp/seder_ffi.h`.
3. Add a test in `src/ffi.rs`'s `tests` module that calls the
   underlying Rust function with realistic inputs.
4. Call it from the Qt side in `AppController.cpp` or
   `DitOffloadWorker.cpp`.
5. Any C string Rust returns from `into_raw()` must be freed by Qt
   via `seder_string_free`. Borrowed pointers (`CString::as_ptr` off
   a field on `OffloadReportHandle`) must not.

## Adding a new module under `src/offload/`

1. Declare it in `src/offload/mod.rs`.
2. Add it to the `RUST_SOURCES` list in `qt/CMakeLists.txt` so CMake
   re-runs cargo when it changes.
3. Re-export anything the FFI needs at the top of `src/offload/mod.rs`.

## Releasing

Don't tag manually. Push to `main` and `.github/workflows/release.yml`
will bump the patch version, build all platform binaries, and publish
the release plus `SHA256SUMS.txt`. Builds are unsigned; that is a
deliberate project choice documented in the README.

If a release needs a higher-than-patch version bump (minor or major),
tag manually with a `vX.Y.0` tag and push the tag — the workflow
picks that up.
