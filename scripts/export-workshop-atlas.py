#!/usr/bin/env python3
"""Workshop props redrawn for 1:1 characters from the supplied large atlas, plus 1:1 smith pages.

v1 reduced the same supplied art for 3/4 characters. v2 keeps characters at their canonical
128x160 and re-reduces each prop from the large source at 4/3 of its v1 size, so props gain
pixel density instead of being upscaled.
"""
import argparse
import collections
import json
import math
import tempfile
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/forge/atlas-v2'
SOURCE = OUT / 'source'
# Owner layout (reference sketch): the furnace stands against the back wall (floor y=146);
# the anvil sits in front of its left half and the quench tub in front of its right half
# (floors y=170). Short walks keep each work stage visible.
POSITIONS = {'furnace': [520, -30], 'anvil': [464, 106],
             'quenching-tub': [592, 106], 'workbench': [768, 64], 'counter': [96, 64],
             'display-shelf': [99, 20], 'coal-bin': [440, 77],
             'coal-bin-half': [440, 77], 'coal-bin-empty': [440, 77]}  # bin floor y=138, by the back wall
IDS = {'furnace': 'forge', 'counter': 'display'}  # shelf keeps its own id: display-shelf
CANVAS, PIVOT = (128, 160), (64, 140)


def save_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def reduce(crop, w, h):
    """One native pixel per source block: sample its inner 60% so neighbouring art pixels and
    soft generator edges do not bleed in, keep it opaque at >=50% coverage, and take the mean of
    the dominant colour cluster. Alpha stays binary."""
    out = Image.new('RGBA', (w, h))
    W, H = crop.size
    px = crop.load()
    for y in range(h):
        for x in range(w):
            x0, x1, y0, y1 = x * W / w, (x + 1) * W / w, y * H / h, (y + 1) * H / h
            ix0 = int(x0 + (x1 - x0) * 0.2)
            ix1 = max(ix0 + 1, int(x1 - (x1 - x0) * 0.2))
            iy0 = int(y0 + (y1 - y0) * 0.2)
            iy1 = max(iy0 + 1, int(y1 - (y1 - y0) * 0.2))
            block = [px[i, j] for j in range(iy0, iy1) for i in range(ix0, ix1)]
            opaque = [c for c in block if c[3] > 127]
            if len(opaque) * 2 < len(block):
                continue
            key = collections.Counter((c[0] // 8, c[1] // 8, c[2] // 8) for c in opaque).most_common(1)[0][0]
            cluster = [c for c in opaque if (c[0] // 8, c[1] // 8, c[2] // 8) == key]
            out.putpixel((x, y), tuple(sum(c[k] for c in cluster) // len(cluster) for k in range(3)) + (255,))
    return out


def cut_black(im, outline=(14, 9, 7, 255)):
    """For sources generated on an opaque black background: dark pixels connected to the border
    are background. The source outline is also black, so a fresh 1px outline ring is drawn."""
    w, h = im.size
    px = im.load()
    dark = lambda x, y: max(px[x, y][:3]) <= 24
    seen = set()
    stack = [(x, y) for x in range(w) for y in (0, h - 1)] + [(x, y) for y in range(h) for x in (0, w - 1)]
    while stack:
        x, y = stack.pop()
        if (x, y) in seen or not (0 <= x < w and 0 <= y < h) or not dark(x, y):
            continue
        seen.add((x, y))
        stack += [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
    for x, y in seen:
        px[x, y] = (0, 0, 0, 0)
    ring = [(x, y) for x, y in seen
            if any(0 <= x + dx < w and 0 <= y + dy < h and (x + dx, y + dy) not in seen
                   for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]
    for x, y in ring:
        px[x, y] = outline
    return im


def props(out, spec):
    rows = []
    for name, s in spec['sprites'].items():
        large = Image.open(SOURCE / s.get('source', 'generated-atlas-large.png')).convert('RGBA')
        art = reduce(large.crop(tuple(s['source_rect'])), *s['size'])
        if s.get('black_background'):
            art = cut_black(art)
        full = Image.new('RGBA', tuple(s['cell']))
        full.alpha_composite(art, tuple(s['pad']))
        # Horizontal split: rows from split_y down draw in front of a working smith.
        front = Image.new('RGBA', full.size)
        front.alpha_composite(full.crop((0, s['split_y'], full.width, full.height)), (0, s['split_y']))
        back = full.copy()
        back.paste((0, 0, 0, 0), (0, s['split_y'], full.width, full.height))
        assert Image.alpha_composite(back, front).tobytes() == full.tobytes(), name
        layers = []
        # Padding protects the external outline at tile boundaries.
        for im in (full, back, front):
            padded = Image.new('RGBA', (im.width + 2, im.height + 2))
            padded.alpha_composite(im, (1, 1))
            layers.append(padded)
        full, back, front = layers
        alpha = full.getchannel('A')
        ring = ImageChops.subtract(alpha.filter(ImageFilter.MaxFilter(3)), alpha)
        front_ring = ImageChops.multiply(ring, front.getchannel('A').filter(ImageFilter.MaxFilter(3)))
        ident = IDS.get(name, name)
        full.save(out / f'{ident}.png')
        for kind, im, mask in [('back', back, ImageChops.subtract(ring, front_ring)),
                               ('front', front, front_ring)]:
            im.save(out / f'{ident}-{kind}.png')
            outline = Image.new('RGBA', full.size, 'white')
            outline.putalpha(mask)
            outline.save(out / f'{ident}-{kind}-outline.png')
        if s.get('variant_of'):
            continue  # alternate state art of a catalogued fixture; same origin and size
        modes = {k: [v[0] + 1, v[1] + 1] for k, v in s.get('mode_contacts', {}).items()}
        origin = [v - 1 for v in POSITIONS[name]]
        rows.append(dict(id=ident, source=f"source/{s.get('source', 'generated-atlas-large.png')}#{name}",
                         grid_origin=POSITIONS[name], origin=origin, size=list(full.size),
                         floor=s['floor'] + 1, depth=POSITIONS[name][1] + s['floor'],
                         contact=[v + 1 for v in s['contact']], mode_contacts=modes,
                         hit_mask=[''.join('1' if alpha.getpixel((x, y)) else '0' for x in range(full.width))
                                   for y in range(full.height)]))
    return rows


def characters(out):
    for sex in ('female', 'male'):
        folder = ROOT / 'assets/characters' / sex
        manifest = json.loads((folder / 'atlas-manifest.json').read_text())
        page = Image.open(folder / 'atlas.png').convert('RGBA')
        frames = []
        for frame in manifest['sprites']:
            assert frame['source_size'] == list(CANVAS)
            x, y, w, h = frame['rect']
            im = Image.new('RGBA', CANVAS)
            im.alpha_composite(page.crop((x, y, x + w, y + h)), tuple(frame['trim_offset']))
            frames.append((frame['id'], im))
        for variant in ('', 'base/'):
            for i in range(6):
                path = folder / (f'frames/scene-quenching-{i}.png' if not variant
                                 else f'frames/scene-base/quenching-{i}.png')
                frames.append((f'{variant}quenching/scene-{i}', Image.open(path).convert('RGBA')))
        # Untrimmed canvases: every frame shares one pivot, no per-frame trim arithmetic at runtime.
        atlas = Image.new('RGBA', (CANVAS[0] * 8, CANVAS[1] * math.ceil(len(frames) / 8)))
        for i, (_, im) in enumerate(frames):
            assert im.size == CANVAS
            atlas.alpha_composite(im, ((i % 8) * CANVAS[0], (i // 8) * CANVAS[1]))
        atlas.save(out / f'{sex}.png')
        save_json(out / f'{sex}.json', dict(canvas=list(CANVAS), pivot=list(PIVOT), columns=8,
                                            rows=math.ceil(len(frames) / 8), ids=[x[0] for x in frames]))
        arms_dir = out / 'arms'
        arms_dir.mkdir(exist_ok=True)
        for variant in ('base', 'equipped'):
            for i in range(6):
                pose = folder / f'poses/researching-{i}'
                im = Image.open(pose / 'body' / ('arms-front-base.png' if variant == 'base'
                                                  else 'arms-front.png')).convert('RGBA')
                if variant == 'equipped':
                    im.alpha_composite(Image.open(pose / 'equipment/gloves.png').convert('RGBA'))
                im.save(arms_dir / f'{sex}-{variant}-researching-{i}.png')


def build(out):
    out.mkdir(parents=True, exist_ok=True)
    spec = json.loads((SOURCE / 'workshop-atlas.json').read_text())
    rows = props(out, spec)
    save_json(out / 'catalog.json', rows)
    characters(out)
    registrations = []
    for sex in ('female', 'male'):
        animation = json.loads((ROOT / f'assets/characters/{sex}/animation.json').read_text())
        for mode, reg in animation['workshop']['stations'].items():
            ident = 'workbench' if mode in ('researching', 'salvaging') else reg['station_id']
            prop = next(row for row in rows if row['id'] == ident)
            contact = prop['mode_contacts'].get(mode, prop['contact'])
            foot = [prop['origin'][i] + contact[i] - (reg['contact_points']['character'][i] - PIVOT[i])
                    for i in range(2)]
            registrations.append(dict(character=sex, mode=mode, fixture=ident, foot=foot,
                                      canonical_contact=reg['contact_points']['character']))
    save_json(out / 'layout.json', dict(
        format='little-forge-layout-v1', grid=32, viewport=[960, 180], expanded=[960, 540],
        background='forge/interior/background.png', character_scale=[1, 1],
        fixtures=rows, registrations=registrations,
        sampling='Canonical 128x160 frames at 1:1; props re-reduced from the large source at 4/3 of v1',
        depth='Floor depth for travel; authored work-mode foreground exceptions; outline follows its part'))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            build(root)
            for path in root.rglob('*'):
                if path.is_file():
                    assert path.read_bytes() == (OUT / path.relative_to(root)).read_bytes(), str(path)
        print('Atlas v2 props, binary masks, layer reconstruction and 1:1 smith pages match')
    else:
        build(OUT)
