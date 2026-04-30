# Third-Party Notices

SEDER DIT Tool uses open-source dependencies.

## Runtime And UI

- Qt 6 is used for the C++/QML desktop application. See the Qt open-source licensing documentation for the terms that apply to the Qt components used in your build.

## Rust Dependencies

- `anyhow`
- `blake3`
- `twox-hash`
- `walkdir`
- `tempfile` for tests

Run `cargo metadata --manifest-path Cargo.toml` to inspect the exact dependency graph for a checked-out revision.
