# SEDER Media Suite DIT Qt App

This is the Qt 6/QML rebuild of the SEDER Media Suite DIT app. The UI is C++/Qt/QML and the heavy folder comparison/report logic stays local in the Rust core exposed through `src/ffi.rs`.

## macOS Homebrew Setup

```sh
brew install qt cmake ninja
cd desktop/seder-dit-tool
cmake -S qt -B qt/build -G Ninja -DCMAKE_PREFIX_PATH="$(brew --prefix qt)"
cmake --build qt/build
./qt/build/seder-media-suite-dit-qt.app/Contents/MacOS/seder-media-suite-dit-qt
```

## Qt Online Installer Setup

```sh
cd desktop/seder-dit-tool
cmake -S qt -B qt/build -G Ninja -DCMAKE_PREFIX_PATH="/path/to/Qt/6.x/macos"
cmake --build qt/build
```

## Tests

Rust core and FFI:

```sh
cd desktop/seder-dit-tool
cargo fmt -- --check
cargo check --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml
```

Qt model/proxy tests, after Qt and CMake are installed:

```sh
cd desktop/seder-dit-tool
cmake -S qt -B qt/build -G Ninja -DCMAKE_PREFIX_PATH="$(brew --prefix qt)" -DBUILD_TESTING=ON
cmake --build qt/build
ctest --test-dir qt/build --output-on-failure
```

## Notes

- The Qt UI calls the Rust core from a `QThread`; scan/checksum work must not run on the QML/UI thread.
- The results table is backed by `QAbstractTableModel` and filtered through `QSortFilterProxyModel` so large result sets stay virtualized.
- MHL export is available only for checksum-backed reports.
- Ignore patterns are comma- or newline-separated and follow the current core behavior: exact filename match or substring match in the path.

## Packaging

The release workflow calls the scripts in `scripts/`:

```sh
VERSION=0.1.0 scripts/package-linux.sh
VERSION=0.1.0 scripts/package-macos.sh
pwsh scripts/package-windows.ps1
```

The scripts write ZIP/AppImage/tarball outputs into `dist/artifacts/`. GitHub Actions then uploads those files to a draft GitHub Release for tags like `v0.1.0`.
