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
INSETS = {'window': (53, 88, 54, 38), 'modal': (40, 60, 40, 40), 'button': (20, 20, 20, 20),
          'tab': (22, 22, 22, 22), 'slot': (24, 24, 24, 24), 'inset': (18, 18, 18, 18)}
ICON_NAMES = ['craft', 'materials', 'provisions', 'map', 'commissions', 'research',
              'codex', 'settings', 'close', 'back', 'plus', 'minus',
              'coin', 'lock', 'warning', 'ore', 'dagger', 'timber',
              'leather', 'staples', 'meal', 'water', 'rest', 'queue']
HEADERS = ['carpenter', 'leatherworker', 'farm', 'tavern']
HEADER_ROWS = [0, 226, 446, 666]


def clean(im):
    im = im.convert('RGBA')
    r, g, b, a = im.split()
    return Image.merge('RGBA', (r, g, b, a.point(lambda v: 255 if v >= ALPHA_CUT else 0)))


def centred(im, size):
    canvas = Image.new('RGBA', size, (0, 0, 0, 0))
    canvas.paste(im, ((size[0] - im.width) // 2, (size[1] - im.height) // 2), im)
    return canvas


def register(entries, im, path, source, source_box, kind, insets=None):
    path.parent.mkdir(parents=True, exist_ok=True)
    im.save(path, optimize=True)
    entry = {
        'path': path.as_posix(), 'kind': kind, 'width': im.width, 'height': im.height,
        'mode': im.mode, 'source': source, 'source_box': list(source_box),
        'alpha_extrema': list(im.getchannel('A').getextrema()) if im.mode == 'RGBA' else None,
        'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
    }
    if insets:
        entry['insets_ltrb'] = list(insets)
    entries.append(entry)


def main():
    entries = []
    sheet = Image.open(SRC / 'controls-v1.png').convert('RGBA')
    boxes = components(SRC / 'controls-v1.png', gap=2)
    assert len(boxes) == len(CONTROL_NAMES), len(boxes)
    crops = {i: clean(sheet.crop(b)) for i, b in enumerate(boxes)}
    # Variants of one control share a canvas so state swaps never move or resize it.
    groups = {}
    for i, (_, group) in CONTROL_NAMES.items():
        if group:
            groups.setdefault(group, []).append(i)
    for i, (name, group) in CONTROL_NAMES.items():
        im = crops[i]
        if group:
            size = (max(crops[j].width for j in groups[group]), max(crops[j].height for j in groups[group]))
            im = centred(im, size)
        register(entries, im, OUT / 'controls' / f'{name}.png', 'source/controls-v1.png', boxes[i], name.split('-')[0],
                 INSETS.get(group) if group else None)
    icons = Image.open(SRC / 'icons-v1.png').convert('RGBA')
    icon_boxes = sorted(components(SRC / 'icons-v1.png'), key=lambda b: (((b[1] + b[3]) // 2) // 256, b[0]))
    assert len(icon_boxes) == len(ICON_NAMES)
    for name, box in zip(ICON_NAMES, icon_boxes):
        im = clean(icons.crop(box))
        assert max(im.size) <= ICON_CANVAS, name
        register(entries, centred(im, (ICON_CANVAS, ICON_CANVAS)), OUT / 'icons' / f'{name}.png', 'source/icons-v1.png', box, 'icon')
    heads = Image.open(SRC / 'merchant-headers-v1.png').convert('RGB')
    for name, y in zip(HEADERS, HEADER_ROWS):
        box = (0, y, 1250, y + 200)
        register(entries, heads.crop(box), OUT / 'headers' / f'{name}.png', 'source/merchant-headers-v1.png', box, 'header')
    manifest = {
        'schema': 2, 'epic': 58, 'tasks': [59, 60, 61], 'runtime_ready': False,
        'status': 'exported-pending-visual-approval',
        'alpha_cut': ALPHA_CUT, 'icon_canvas': ICON_CANVAS,
        'notes': 'Independent exports from generated source sheets; not yet native-grid pixel art. '
                 'Focus ring and separators are drawn by Bevy nodes; no art exists for them.',
        'assets': entries,
    }
    (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'exported {len(entries)} assets')


main()
