#!/usr/bin/env python3
"""Gate transparent registered color layers; picking/overlap tests live in forge-content."""
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageChops, ImageFilter
root = Path(__file__).resolve().parents[1] / 'assets'
base = root / 'map/ashbrook'
r = json.loads((base / 'locations.json').read_text())
size = (r['width'], r['height'])
assert Image.open(base / 'base.png').size == size
assert len(r['locations']) == len({r['id'] for r in r['locations']}) == 8
assert all(r['id'] != 'well' for r in r['locations'])
for region in r['locations']:
    with Image.open(root / region['overlay']) as im:
        assert im.size == size and im.mode == 'RGBA'
        alpha = im.getchannel('A')
        lo, hi = alpha.getextrema()
        assert lo == 0 and 0 < hi <= 90
        mask = Image.new('L', size)
        ImageDraw.Draw(mask).polygon([(x * size[0], y * size[1]) for x, y in region['polygon']], fill=255)
        # Gaussian feathering is intentionally allowed around the polygon.
        mask = mask.filter(ImageFilter.GaussianBlur(8)).point(lambda value: 255 if value else 0)
        assert ImageChops.multiply(alpha, ImageChops.invert(mask)).getbbox() is None
        print(region['id'], 'aligned transparent wash OK')
