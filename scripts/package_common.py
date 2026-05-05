#!/usr/bin/env python3
"""Shared helpers for packaging scripts."""

from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

ARTIFACT_STEM = "seder-dit-tool"
PLATFORM_EXTENSIONS = {
    "macos-arm64": ".zip",
    "macos-x64": ".zip",
    "windows-x64": ".zip",
    "linux-x64": ".tar.gz",
    "linux-x64-appimage": ".AppImage",
}


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def detect_version(root: Path) -> str:
    cargo_toml = (root / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"(.+)"', cargo_toml, flags=re.MULTILINE)
    if not match:
        raise SystemExit("Unable to find version in Cargo.toml")
    return match.group(1).lstrip("v")


def artifact_name(version: str, platform: str) -> str:
    ext = PLATFORM_EXTENSIONS[platform]
    return f"{ARTIFACT_STEM}-v{version}-{platform}{ext}"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_checksum_file(artifact_dir: Path) -> Path:
    artifacts = sorted(p for p in artifact_dir.iterdir() if p.is_file())
    output = artifact_dir / "SHA256SUMS.txt"
    with output.open("w", encoding="utf-8") as f:
        for artifact in artifacts:
            if artifact.name == output.name:
                continue
            f.write(f"{sha256(artifact)}  {artifact.name}\n")
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description="Shared package helper")
    sub = parser.add_subparsers(dest="command", required=True)

    version_parser = sub.add_parser("version")
    version_parser.add_argument("--root", default=str(repo_root()))

    name_parser = sub.add_parser("artifact-name")
    name_parser.add_argument("--version", required=True)
    name_parser.add_argument("--platform", choices=sorted(PLATFORM_EXTENSIONS), required=True)

    sum_parser = sub.add_parser("checksums")
    sum_parser.add_argument("--artifact-dir", required=True)

    args = parser.parse_args()
    if args.command == "version":
        print(detect_version(Path(args.root)))
    elif args.command == "artifact-name":
        print(artifact_name(args.version.lstrip("v"), args.platform))
    elif args.command == "checksums":
        print(write_checksum_file(Path(args.artifact_dir)))


if __name__ == "__main__":
    main()
