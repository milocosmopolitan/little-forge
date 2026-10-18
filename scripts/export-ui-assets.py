#!/usr/bin/env python3
"""Export independent runtime UI assets from the generated source sheets.

Deterministic: components are found from alpha connectivity (never equal division),
alpha is binarised to remove generator halos, and every export is registered in
assets/ui/runtime/manifest.json with size, alpha bounds, insets and source box.
Run from the repository root; sources are never modified.
"""
import hashlib
import json
from pathlib import Path
from PIL import Image
import sys

sys.path.insert(0, str(Path(__file__).parent))
from ui_components import components  # noqa: E402

SRC = Path('assets/ui/source')
OUT = Path('assets/ui/runtime')
ALPHA_CUT = 96
ICON_CANVAS = 256
# True-pixel contract: each art pixel is a GRID x GRID block of identical RGBA values, so the
# runtime files are the native-resolution art scaled by an integer with no interpolation.
GRID = 4
HEADER_GRID = 2
# Palettes are per asset: a shared median-cut palette is dominated by the dark outlines and
# drops small saturated areas (blue selection, amber fill, potion).
PALETTE = {'controls': 24, 'icons': 32, 'headers': 128}

CONTROL_NAMES = {
    0: ('window', 'window'), 1: ('modal', 'modal'),
    2: ('button-normal', 'button'), 3: ('button-hover', 'button'), 4: ('button-pressed', 'button'),
    5: ('button-disabled', 'button'), 6: ('button-selected', 'button'),
    7: ('tab-normal', 'tab'), 8: ('tab-selected', 'tab'),
    9: ('slot-normal', 'slot'), 10: ('slot-selected', 'slot'), 11: ('slot-locked', 'slot'),
    12: ('progress-track', None), 13: ('inset', 'inset'), 14: ('scrollbar-track', None),
    15: ('scrollbar-thumb', None), 16: ('progress-track-alt', None),
    17: ('progress-fill-amber', None), 18: ('progress-fill-blue', None),
}
# Per-edge insets from the exported edge: left, top, right, bottom (source pixels).
# Multiples of GRID so slice seams fall on art-pixel boundaries.
INSETS = {'window': (52, 88, 56, 40), 'modal': (40, 60, 40, 40), 'button': (20, 20, 20, 20),
          'tab': (24, 24, 24, 24), 'slot': (24, 24, 24, 24), 'inset': (20, 20, 20, 20)}
ICON_NAMES = ['craft', 'materials', 'provisions', 'map', 'commissions', 'research',
              'codex', 'settings', 'close', 'back', 'plus', 'minus',
              'coin', 'lock', 'warning', 'ore', 'dagger', 'timber',
              'leather', 'staples', 'meal', 'water', 'rest', 'queue']
HEADERS = ['carpenter', 'leatherworker', 'farm', 'tavern']
HEADER_ROWS = [0, 226, 446, 666]
# Foreground strips composited above the NPC so a shopkeeper stands behind the counter.
HEADER_FOREGROUND = {'farm': (948, 158, 1250, 200)}


def clean(im):
    im = im.convert('RGBA')
    r, g, b, a = im.split()
    return Image.merge('RGBA', (r, g, b, a.point(lambda v: 255 if v >= ALPHA_CUT else 0)))


def to_native(im, grid):
    """Area-average to the art-pixel grid; alpha stays hard."""
    size = (max(1, round(im.width / grid)), max(1, round(im.height / grid)))
    small = im.resize(size, Image.BOX)
    r, g, b, a = small.split()
    return Image.merge('RGBA', (r, g, b, a.point(lambda v: 255 if v >= 128 else 0)))


def palette_for(images, colors):
    """Median-cut palette from the opaque pixels of `images`."""
    pixels = [p[:3] for im in images for p in im.get_flattened_data() if p[3]]
    mosaic = Image.new('RGB', (len(pixels), 1))
    mosaic.putdata(pixels)
    return mosaic.quantize(colors=colors, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)


def snap(im, palette):
    rgb = im.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE).convert('RGB')
    out = rgb.convert('RGBA')
    out.putalpha(im.getchannel('A'))
    return out


def scaled(im, grid):
    return im.resize((im.width * grid, im.height * grid), Image.NEAREST)


def centred(im, size):
    canvas = Image.new('RGBA', size, (0, 0, 0, 0))
    canvas.paste(im, ((size[0] - im.width) // 2, (size[1] - im.height) // 2), im)
    return canvas


def register(entries, im, path, source, source_box, kind, insets=None, grid=1):
    path.parent.mkdir(parents=True, exist_ok=True)
    im.save(path, optimize=True)
    entry = {
        'path': path.as_posix(), 'kind': kind, 'width': im.width, 'height': im.height,
        'mode': im.mode, 'source': source, 'source_box': list(source_box),
        'alpha_extrema': list(im.getchannel('A').getextrema()) if im.mode == 'RGBA' else None,
        'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'grid': grid, 'native_width': im.width // grid, 'native_height': im.height // grid,
    }
    if insets:
        entry['insets_ltrb'] = list(insets)
    entries.append(entry)


def main():
    entries = []
    sheet = Image.open(SRC / 'controls-v1.png').convert('RGBA')
    boxes = components(SRC / 'controls-v1.png', gap=2)
    assert len(boxes) == len(CONTROL_NAMES), len(boxes)
    native = {i: to_native(clean(sheet.crop(b)), GRID) for i, b in enumerate(boxes)}
    # Variants of one control share a canvas so state swaps never move or resize it.
    groups = {}
    for i, (_, group) in CONTROL_NAMES.items():
        if group:
            groups.setdefault(group, []).append(i)
    for i, (name, group) in CONTROL_NAMES.items():
        im = native[i]
        if group:
            im = centred(im, (max(native[j].width for j in groups[group]), max(native[j].height for j in groups[group])))
        register(entries, scaled(snap(im, palette_for([im], PALETTE['controls'])), GRID), OUT / 'controls' / f'{name}.png', 'source/controls-v1.png', boxes[i],
                 name.split('-')[0], INSETS.get(group) if group else None, GRID)
    icons = Image.open(SRC / 'icons-v1.png').convert('RGBA')
    icon_boxes = sorted(components(SRC / 'icons-v1.png'), key=lambda b: (((b[1] + b[3]) // 2) // 256, b[0]))
    assert len(icon_boxes) == len(ICON_NAMES)
    icon_native = [to_native(clean(icons.crop(b)), GRID) for b in icon_boxes]
    canvas = ICON_CANVAS // GRID
    for name, box, im in zip(ICON_NAMES, icon_boxes, icon_native):
        assert max(im.size) <= canvas, name
        register(entries, scaled(snap(centred(im, (canvas, canvas)), palette_for([im], PALETTE['icons'])), GRID), OUT / 'icons' / f'{name}.png',
                 'source/icons-v1.png', box, 'icon', None, GRID)
    heads = Image.open(SRC / 'merchant-headers-v1.png').convert('RGBA')
    head_native = {}
    for name, y in zip(HEADERS, HEADER_ROWS):
        box = (0, y, 1250, y + 200)
        head_native[name] = (box, to_native(heads.crop(box), HEADER_GRID))
    for name, (box, im) in head_native.items():
        full = scaled(snap(im, palette_for([im], PALETTE['headers'])), HEADER_GRID).convert('RGB')
        register(entries, full, OUT / 'headers' / f'{name}.png', 'source/merchant-headers-v1.png', box, 'header', None, HEADER_GRID)
        if name in HEADER_FOREGROUND:
            fb = HEADER_FOREGROUND[name]
            assert all(v % HEADER_GRID == 0 for v in fb)
            register(entries, full.crop(fb), OUT / 'headers' / f'{name}-counter.png', 'source/merchant-headers-v1.png',
                     (fb[0], box[1] + fb[1], fb[2], box[1] + fb[3]), 'header-foreground', None, HEADER_GRID)
    manifest = {
        'schema': 3, 'epic': 58, 'tasks': [59, 60, 61], 'runtime_ready': False,
        'status': 'true-pixel-pending-visual-approval',
        'alpha_cut': ALPHA_CUT, 'icon_canvas': ICON_CANVAS, 'grid': GRID, 'header_grid': HEADER_GRID,
        'palette_colors': PALETTE,
        'notes': 'Source sheets are area-averaged to the art-pixel grid, palette-limited with hard alpha, '
                 'then scaled by an integer (nearest) so every art pixel is one uniform block. '
                 'Focus ring and separators are drawn by Bevy nodes; no art exists for them.',
        'assets': entries,
    }
    (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'exported {len(entries)} assets')


main()
