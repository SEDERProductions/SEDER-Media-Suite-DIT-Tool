#!/usr/bin/env python3
"""Generate platform icon sets from assets/icon.svg.

Outputs into <app>/assets/:
  icon-<size>.png   (16, 32, 48, 64, 128, 256, 512, 1024)
  icon.icns         (macOS bundle icon)
  icon.ico          (Windows .exe icon)

Rasterizer preference: magick > rsvg-convert > sips. macOS .icns built with
iconutil (mac-only). Run on macOS to populate .icns; other CI runners only
need the PNGs and .ico.
"""

from __future__ import annotations

import io
import shutil
import struct
import subprocess
import sys
from pathlib import Path

SIZES = [16, 32, 48, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 32, 48, 64, 128, 256]

ICONSET_ENTRIES = [
    (16, "icon_16x16.png"),
    (32, "icon_16x16@2x.png"),
    (32, "icon_32x32.png"),
    (64, "icon_32x32@2x.png"),
    (128, "icon_128x128.png"),
    (256, "icon_128x128@2x.png"),
    (256, "icon_256x256.png"),
    (512, "icon_256x256@2x.png"),
    (512, "icon_512x512.png"),
    (1024, "icon_512x512@2x.png"),
]


def have(cmd: str) -> bool:
    return shutil.which(cmd) is not None


def rasterize(svg: Path, out_png: Path, size: int) -> None:
    out_png.parent.mkdir(parents=True, exist_ok=True)
    if have("magick"):
        subprocess.run(
            ["magick", "-background", "none", "-density", "384",
             "-resize", f"{size}x{size}", str(svg), str(out_png)],
            check=True,
        )
        return
    if have("rsvg-convert"):
        subprocess.run(
            ["rsvg-convert", "-w", str(size), "-h", str(size),
             "-o", str(out_png), str(svg)],
            check=True,
        )
        return
    if have("sips"):
        subprocess.run(
            ["sips", "-s", "format", "png", "-Z", str(size),
             str(svg), "--out", str(out_png)],
            check=True, stdout=subprocess.DEVNULL,
        )
        return
    raise RuntimeError(
        "No SVG rasterizer found. Install ImageMagick (`magick`), "
        "librsvg (`rsvg-convert`), or run on macOS (`sips`)."
    )


def build_icns(assets_dir: Path) -> bool:
    if not have("iconutil"):
        return False
    iconset = assets_dir / "icon.iconset"
    if iconset.exists():
        shutil.rmtree(iconset)
    iconset.mkdir()
    try:
        for size, name in ICONSET_ENTRIES:
            src = assets_dir / f"icon-{size}.png"
            shutil.copyfile(src, iconset / name)
        subprocess.run(
            ["iconutil", "-c", "icns", str(iconset),
             "-o", str(assets_dir / "icon.icns")],
            check=True,
        )
        return True
    finally:
        shutil.rmtree(iconset, ignore_errors=True)


def build_ico(assets_dir: Path) -> None:
    """Pack PNGs into a Windows .ico file with PNG-encoded entries."""
    entries = []
    for size in ICO_SIZES:
        png_bytes = (assets_dir / f"icon-{size}.png").read_bytes()
        entries.append((size, png_bytes))

    out = io.BytesIO()
    out.write(struct.pack("<HHH", 0, 1, len(entries)))
    offset = 6 + 16 * len(entries)
    for size, data in entries:
        width = 0 if size >= 256 else size
        height = 0 if size >= 256 else size
        out.write(struct.pack(
            "<BBBBHHII",
            width, height, 0, 0, 1, 32, len(data), offset,
        ))
        offset += len(data)
    for _, data in entries:
        out.write(data)
    (assets_dir / "icon.ico").write_bytes(out.getvalue())


def main(app_root: Path) -> None:
    svg = app_root / "assets" / "icon.svg"
    if not svg.exists():
        sys.exit(f"missing {svg}")
    assets = app_root / "assets"
    for size in SIZES:
        rasterize(svg, assets / f"icon-{size}.png", size)
    icns_ok = build_icns(assets)
    build_ico(assets)
    print(f"OK {app_root.name}: PNGs + .ico" + (" + .icns" if icns_ok else " (no iconutil; .icns skipped)"))


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit("usage: generate_icons.py <app_root> [<app_root> ...]")
    for arg in sys.argv[1:]:
        main(Path(arg).resolve())
