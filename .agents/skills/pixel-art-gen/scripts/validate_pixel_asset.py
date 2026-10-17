#!/usr/bin/env python3
"""Read-only structural checks for native pixel art and an optional preview."""
import argparse
from pathlib import Path

from PIL import Image, ImageChops


def positive(value):
    result = int(value)
    if result < 1:
        raise argparse.ArgumentTypeError("must be a positive integer")
    return result


def visible_rgba(im):
    # Invisible RGB can vary between encoders without changing the artwork.
    result = im.convert("RGBA")
    result.putdata([p if p[3] else (0, 0, 0, 0) for p in result.getdata()])
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("native", type=Path)
    parser.add_argument("--grid", choices=["A", "B", "C"], required=True)
    parser.add_argument("--cells", nargs=2, type=positive, required=True, metavar=("COLS", "ROWS"))
    parser.add_argument("--max-colors", type=positive, default=32)
    parser.add_argument("--preview", type=Path)
    parser.add_argument("--scale", type=positive)
    args = parser.parse_args()
    if bool(args.preview) != bool(args.scale):
        parser.error("--preview and --scale must be provided together")

    errors = []
    unit = {"A": 8, "B": 16, "C": 32}[args.grid]
    expected = tuple(unit * count for count in args.cells)
    with Image.open(args.native) as source:
        if source.format != "PNG" or source.mode != "RGBA":
            errors.append("native must be an RGBA PNG")
        im = source.convert("RGBA")
    if im.size != expected:
        errors.append(f"native size {im.size}, expected {expected}")
    pixels = list(im.getdata())
    if not {p[3] for p in pixels}.issubset({0, 255}):
        errors.append("partial alpha found; expected only 0 and 255")
    colors = {p[:3] for p in pixels if p[3] > 0}
    if not colors:
        errors.append("asset is entirely transparent")
    if len(colors) > args.max_colors:
        errors.append(f"{len(colors)} visible colors exceeds {args.max_colors}")
    if args.preview:
        with Image.open(args.preview) as source:
            preview = visible_rgba(source)
        enlarged = visible_rgba(im).resize(
            (im.width * args.scale, im.height * args.scale), Image.Resampling.NEAREST
        )
        if preview.size != enlarged.size:
            errors.append(f"preview size {preview.size}, expected {enlarged.size}")
        elif any(band.getbbox() for band in ImageChops.difference(preview, enlarged).split()):
            errors.append("preview differs from exact nearest-neighbor enlargement")
    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1
    print(f"PASS: {args.native}, Grid {args.grid}, {im.width}x{im.height}, {len(colors)} colors, binary alpha")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
