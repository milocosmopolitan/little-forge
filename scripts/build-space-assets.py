#!/usr/bin/env python3
"""Register the supplied forge-space art as native runtime assets (#90, #93, #94).

Sources (copied byte-for-byte into assets/forge/spaces/source) are generated pixel-art
references supplied by the project owner. This script only reduces, crops and binarises
alpha; it never paints new art. Outputs:

- workshop-backdrop.png / showroom-backdrop.png: the two interiors reduced once with a box
  filter to the native 480x270 scene (about 3.48 source px per art px).
- showroom-window.png / detail-panel.png: nine-slice frames cut from the plain regions of
  the supplied blank panels (corners, edges, flat fill), alpha hard-cut like the UI exports.
- window-glow.png: a tiny banded warm glow for the exterior window (procedural, 24x20).

catalog.json records sizes, insets, SHA-256 and provenance. Re-running is byte-identical.
Run from the repository root.
"""
import hashlib
import json
import shutil
from pathlib import Path
from PIL import Image

SUPPLIED = Path('/Users/minhokang/Work/games/little-forge-assets/blacksmith')
OUT = Path('assets/forge/spaces')
SRC = OUT / 'source'
OUT.mkdir(parents=True, exist_ok=True)
SRC.mkdir(parents=True, exist_ok=True)
SCENE = (480, 270)
ALPHA_CUT = 96
GLOW = (255, 196, 96)

SOURCES = {
    'smithy-interior.png': 'Empty cozy pixel-art smithy interior.png',
    'showroom-interior.png': 'Warm Pixel-Art Showroom Interior.png',
    'showroom-window.png': 'Blank eight-slot showroom inventory window.png',
    'detail-panel.png': 'Blank oak-framed pixel detail panel.png',
    'showroom-draft.png': 'Cozy showroom inventory display.png',
}


def ensure_sources():
    """Copy supplied originals once; afterwards the repository copies are authoritative."""
    for name, supplied in SOURCES.items():
        target = SRC / name
        if not target.exists():
            origin = SUPPLIED / supplied
            if not origin.exists():
                raise SystemExit(f'missing source {target} and no supplied copy at {origin}')
            shutil.copy2(origin, target)
    legacy = Path('assets/forge/workshop/source/smithy-interior.png')
    if legacy.exists() and not (SRC / 'smithy-interior.png').exists():
        shutil.move(legacy, SRC / 'smithy-interior.png')


def interior(name):
    with Image.open(SRC / name) as src:
        return src.convert('RGBA').resize(SCENE, Image.BOX)


def hard_alpha(im):
    im = im.convert('RGBA')
    a = im.getchannel('A').point(lambda v: 255 if v >= ALPHA_CUT else 0)
    im.putalpha(a)
    return im


def nine_slice(name, corner, left_edge, top_edge, right_edge, bottom_edge, center, size):
    """Assemble a clean frame from plain regions of a blank panel.

    corner: side length of the corner tiles; the other arguments are (x, y) of a size x size
    sample for each edge and the fill. Output is 3 * size square with insets = size.
    """
    with Image.open(SRC / name) as src:
        src = hard_alpha(src)
        bbox = src.getchannel('A').getbbox()
        l, t, r, b = bbox
        out = Image.new('RGBA', (3 * size, 3 * size), (0, 0, 0, 0))
        fill = src.crop((center[0], center[1], center[0] + size, center[1] + size))
        for y in range(3):
            for x in range(3):
                out.paste(fill, (x * size, y * size))
        # Edges (plain frame segments) then corners on top.
        out.paste(src.crop((left_edge[0], left_edge[1], left_edge[0] + size, left_edge[1] + size)), (0, size))
        out.paste(src.crop((right_edge[0], right_edge[1], right_edge[0] + size, right_edge[1] + size)), (2 * size, size))
        out.paste(src.crop((top_edge[0], top_edge[1], top_edge[0] + size, top_edge[1] + size)), (size, 0))
        out.paste(src.crop((bottom_edge[0], bottom_edge[1], bottom_edge[0] + size, bottom_edge[1] + size)), (size, 2 * size))
        corners = [
            ((l, t), (0, 0)),
            ((r - size, t), (2 * size, 0)),
            ((l, b - size), (0, 2 * size)),
            ((r - size, b - size), (2 * size, 2 * size)),
        ]
        for (sx, sy), (dx, dy) in corners:
            out.paste(src.crop((sx, sy, sx + size, sy + size)), (dx, dy))
        assert corner <= size
        return out


def window_glow():
    w, h = 24, 20
    im = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    px = im.load()
    for y in range(h):
        for x in range(w):
            dx = (x - 11.5) / 12
            dy = (y - 9.5) / 10
            r = dx * dx + dy * dy
            if r < 1:
                a = int(170 * (1 - r) ** 1.5)
                a = (a // 24) * 24
                if a > 0:
                    px[x, y] = (*GLOW, a)
    return im


def save(name, im, **extra):
    path = OUT / f'{name}.png'
    im.save(path, optimize=False)
    data = path.read_bytes()
    entry = {'path': f'forge/spaces/{name}.png', 'width': im.width, 'height': im.height,
             'mode': im.mode, 'sha256': hashlib.sha256(data).hexdigest()}
    entry.update(extra)
    return entry


ensure_sources()
for stale in ['rack-tier1', 'rack-tier2', 'rack-tier3', 'dagger-item', 'slot-reserved', 'slot-held', 'chimney-puff']:
    p = OUT / f'{stale}.png'
    if p.exists():
        p.unlink()
entries = {}
entries['workshop-backdrop'] = save('workshop-backdrop', interior('smithy-interior.png'), source='source/smithy-interior.png')
entries['showroom-backdrop'] = save('showroom-backdrop', interior('showroom-interior.png'), source='source/showroom-interior.png')
# Eight-slot window 1860x845, alpha bbox (49,9)-(1814,802): tabs sit at y 95-185 in x 134-870 and
# 1360-1730; slots occupy x 355-1505, y 240-715. The sampled regions below are frame-only.
entries['showroom-window'] = save(
    'showroom-window',
    nine_slice('showroom-window.png', corner=80, size=80,
               left_edge=(49, 560), right_edge=(1814 - 80, 560),
               top_edge=(1000, 9), bottom_edge=(200, 802 - 80), center=(200, 560)),
    source='source/showroom-window.png', insets_ltrb=[80, 80, 80, 80], corner_scale=0.5)
# Detail panel 963x1632, alpha bbox (94,19)-(869,1616): item frame y 290-585, separators at
# y ~640/890/1315, buttons y 760-852 and 1345-1455. Samples avoid all of them.
entries['detail-panel'] = save(
    'detail-panel',
    nine_slice('detail-panel.png', corner=110, size=110,
               left_edge=(94, 1000), right_edge=(869 - 110, 1000),
               top_edge=(400, 19), bottom_edge=(400, 1616 - 110), center=(400, 1000)),
    source='source/detail-panel.png', insets_ltrb=[110, 110, 110, 110], corner_scale=0.4)
entries['window-glow'] = save('window-glow', window_glow(), source='procedural banded glow (no image model)')
sources = {name: {'path': f'forge/spaces/source/{name}', 'sha256': hashlib.sha256((SRC / name).read_bytes()).hexdigest(),
                  'supplied_as': supplied} for name, supplied in SOURCES.items()}
catalog = {
    'version': 2,
    'status': 'supplied reference art registered at native scene size; visual approval pending',
    'generator': 'scripts/build-space-assets.py (box reduce, frame crops, hard alpha; no new art painted)',
    'sampler': 'nearest',
    'integer_scale_only': True,
    'alpha_cut': ALPHA_CUT,
    'scene': {'size': list(SCENE), 'floor_rows': [160, 245]},
    'sources': sources,
    'assets': entries,
}
(OUT / 'catalog.json').write_text(json.dumps(catalog, indent=2) + '\n')
print('wrote', len(entries), 'assets to', OUT)
