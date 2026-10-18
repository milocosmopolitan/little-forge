#!/usr/bin/env python3
"""Render runtime exports on light and dark checkerboards for seam/halo review."""
import sys
from pathlib import Path
from PIL import Image

out = Path(sys.argv[1] if len(sys.argv) > 1 else 'docs/ui/evidence/runtime-assets-review.png')
files = sorted(Path('assets/ui/runtime/controls').glob('*.png'))
icons = sorted(Path('assets/ui/runtime/icons').glob('*.png'))


def checker(size, a, b, n=16):
    im = Image.new('RGB', size, a)
    px = im.load()
    for y in range(size[1]):
        for x in range(size[0]):
            if (x // n + y // n) % 2:
                px[x, y] = b
    return im


def sheet(bg, items, cell, cols):
    rows = (len(items) + cols - 1) // cols
    im = checker((cols * cell, rows * cell), *bg)
    for i, f in enumerate(items):
        t = Image.open(f).convert('RGBA')
        t.thumbnail((int(cell * .9), int(cell * .9)))
        im.paste(t, ((i % cols) * cell + (cell - t.width) // 2, (i // cols) * cell + (cell - t.height) // 2), t)
    return im


parts = [sheet(bg, items, cell, cols) for bg in (((235, 235, 235), (200, 200, 200)), ((20, 20, 20), (50, 50, 50)))
         for items, cell, cols in ((files, 300, 5), (icons, 128, 12))]
w = max(p.width for p in parts)
res = Image.new('RGB', (w, sum(p.height for p in parts)))
y = 0
for p in parts:
    res.paste(p, (0, y))
    y += p.height
out.parent.mkdir(parents=True, exist_ok=True)
res.save(out)
print(out, res.size)
