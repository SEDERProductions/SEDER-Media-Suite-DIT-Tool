# SEDER DIT Tool

[![CI](https://github.com/sederproductions/seder-dit-tool/actions/workflows/ci.yml/badge.svg)](https://github.com/sederproductions/seder-dit-tool/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/sederproductions/seder-dit-tool?label=latest%20release)](https://github.com/sederproductions/seder-dit-tool/releases/latest)
[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Local Processing](https://img.shields.io/badge/processing-local--only-1f7a4d)](#privacy)

Local-first DIT folder verification for source and destination offloads. The app uses a Qt 6/QML interface with a compiled Rust core for recursive scans, checksums, and TXT/CSV/MHL reports.

## Download

**[Download the latest release](https://github.com/sederproductions/seder-dit-tool/releases/latest)**

| Platform | Asset |
| --- | --- |
| macOS Apple Silicon | `seder-dit-tool-v0.1.0-macos-arm64-unsigned.zip` |
| macOS Intel | `seder-dit-tool-v0.1.0-macos-x64-unsigned.zip` |
| Windows x64 | `seder-dit-tool-v0.1.0-windows-x64.zip` |
| Linux x64 | `seder-dit-tool-v0.1.0-linux-x64.AppImage` |
| Linux fallback | `seder-dit-tool-v0.1.0-linux-x64.tar.gz` |

Download `SHA256SUMS.txt` from the same release and verify the file before launching.

### Unsigned Builds

The first public binaries are unsigned. macOS and Windows may show extra launch warnings. This is expected for `v0.1.0`; code signing and notarization are intentionally not required for the first open-source release.

## Features

- Compare source and destination folders with path/size, modified-time, or checksum modes.
- Ignore hidden/system files and custom ignore patterns.
- Run scan and checksum work off the UI thread.
- Inspect large result sets through a virtualized Qt table model.
- Export TXT, CSV, and checksum-backed MHL reports.
- Keep all processing local.

## Privacy

The app does not require an account and does not send media paths, filenames, checksums, or reports to a cloud service. All comparison work runs on the local machine.

## Build From Source

Requirements:

- Rust stable toolchain
- Qt 6.5 or newer
- CMake 3.21 or newer
- Ninja, or another CMake-supported generator

macOS Homebrew example:

```sh
brew install qt cmake ninja
git clone https://github.com/sederproductions/seder-dit-tool.git
cd seder-dit-tool
cmake -S qt -B qt/build -G Ninja -DCMAKE_PREFIX_PATH="$(brew --prefix qt)" -DCMAKE_BUILD_TYPE=Release
cmake --build qt/build --config Release
```

Developer checks:

```sh
cargo fmt -- --check
cargo check --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml

cmake -S qt -B qt/build -G Ninja -DCMAKE_PREFIX_PATH="$(brew --prefix qt)" -DBUILD_TESTING=ON
cmake --build qt/build
ctest --test-dir qt/build --output-on-failure
```

More Qt-specific build notes are in [qt/README.md](qt/README.md).

## Release Process

Public releases are built by GitHub Actions on standard GitHub-hosted runners. Push a version tag to create a draft release with platform assets:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release workflow uploads ZIP/AppImage/tarball assets and `SHA256SUMS.txt` to GitHub Releases.

## License

Code is released under [GPL-3.0-only](LICENSE).

The SEDER, SEDER Media Suite, and Seder Productions names, marks, and visual identity are not granted under the GPL license. See [TRADEMARKS.md](TRADEMARKS.md) before redistributing modified builds using SEDER branding.

Third-party dependency notes are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
