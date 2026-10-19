#!/usr/bin/env python3
"""Deterministic station exports; never changes canonical character artwork."""
import argparse
import hashlib
import json
import tempfile
from pathlib import Path
from PIL import Image, ImageChops, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/forge/interior'
SPECS = [
    ('forge', 'ChatGPT Image Oct 19, 2026, 12_20_01 PM.png', 148, [385, 8], [45, 103], None),
    ('anvil', 'ChatGPT Image Oct 19, 2026, 12_20_05 PM.png', 64, [515, 100], [26, 4], 16),
    ('quenching-tub', 'ChatGPT Image Oct 19, 2026, 12_20_22 PM.png', 58, [635, 103], [36, 18], 30),
    ('workbench', 'Pixel-Art Craftsman’s Workbench.png', 78, [770, 74], [80, 28], None),
    ('display', 'ChatGPT Image Oct 19, 2026, 12_20_29 PM.png', 74, [154, 84], [85, 25], None),
]

def build(source):
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / 'source').mkdir(exist_ok=True)
    rows = []
    for name, filename, height, origin, contact, split in SPECS:
        raw = (source / filename).read_bytes()
        (OUT / 'source' / filename).write_bytes(raw)
        image = Image.open(source / filename).convert('RGBA')
        original_bounds = image.getchannel('A').getbbox()
        alpha = image.getchannel('A').point(lambda a: 255 if a >= 224 else 0)
        bounds = alpha.getbbox()
        image.putalpha(alpha)
        cropped = image.crop(bounds)
        width = round(cropped.width * height / cropped.height)
        native = cropped.resize((width, height), Image.Resampling.NEAREST)
        # Padding includes the full outer one-pixel outline at image edges.
        full = Image.new('RGBA', (width + 2, height + 2))
        full.alpha_composite(native, (1, 1))
        mask = full.getchannel('A')
        ring = ImageChops.subtract(mask.filter(ImageFilter.MaxFilter(3)), mask)
        back = full.copy()
        front = Image.new('RGBA', full.size)
        if split is not None:
            front.paste(full.crop((0, split + 1, full.width, full.height)), (0, split + 1))
            back.paste((0, 0, 0, 0), (0, split + 1, full.width, full.height))
        assert Image.alpha_composite(back, front).tobytes() == full.tobytes()
        full.save(OUT / f'{name}.png')
        front_ring = ImageChops.multiply(ring, front.getchannel('A').filter(ImageFilter.MaxFilter(3)))
        for kind, layer, outline in [('back', back, ImageChops.subtract(ring, front_ring)), ('front', front, front_ring)]:
            layer.save(OUT / f'{name}-{kind}.png')
            border = Image.new('RGBA', full.size, (255, 255, 255, 0))
            border.putalpha(outline)
            border.save(OUT / f'{name}-{kind}-outline.png')
        rows.append(dict(id=name, source=filename, sha256=hashlib.sha256(raw).hexdigest(),
                         source_size=list(image.size), alpha_bounds=original_bounds, opaque_bounds=bounds,
                         size=list(full.size), origin=[origin[0]-1, origin[1]-1],
                         contact=[contact[0]+1, contact[1]+1], split=split,
                         hit_mask=[''.join('1' if mask.getpixel((x,y)) else '0' for x in range(full.width)) for y in range(full.height)]))
    (OUT / 'catalog.json').write_text(json.dumps(rows, indent=2) + '\n')
    (OUT / 'arms').mkdir(exist_ok=True)
    for sex in ['female', 'male']:
        for frame in range(6):
            pose = ROOT / f'assets/characters/{sex}/poses/researching-{frame}'
            for variant in ['base', 'equipped']:
                name = 'arms-front-base.png' if variant == 'base' else 'arms-front.png'
                arms = Image.open(pose / 'body' / name).convert('RGBA')
                if variant == 'equipped':
                    arms.alpha_composite(Image.open(pose / 'equipment/gloves.png').convert('RGBA'))
                arms.save(OUT / f'arms/{sex}-{variant}-researching-{frame}.png')
    # Preserve the unmodified background pending the owner's 960x180 replacement.
    bg = 'ChatGPT Image Oct 19, 2026, 12_19_56 PM.png'
    (OUT / 'source' / bg).write_bytes((source / bg).read_bytes())
    wide = ROOT / 'assets/forge/interior/source/background-wide-v2.png'
    raw = wide.read_bytes()
    (OUT / 'source/background-wide-v2.png').write_bytes(raw)
    # Preserve the entire edited room. Only the white presentation margins are removed.
    crop = [0, 247, 2048, 638]
    room = Image.open(wide).convert('RGBA').crop(crop)
    width = round(room.width * 180 / room.height)
    native = room.resize((width, 180), Image.Resampling.NEAREST)
    canvas = Image.new('RGBA', (960, 180))
    left = (960-width)//2
    canvas.alpha_composite(native, (left, 0))
    canvas.save(OUT / 'background.png')
    (OUT / 'background-export.json').write_text(json.dumps(dict(source_sha256=hashlib.sha256(raw).hexdigest(), crop=crop,
        native_size=[width,180], canvas=[960,180], padding_left=left, padding_right=960-width-left,
        status='Owner-requested source-preserving edit; runtime visual approval pending'), indent=2)+'\n')
    for row in rows:
        print(row['id'], row['source_size'], row['opaque_bounds'], row['size'], row['origin'], row['contact'])

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('source', type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        expected = OUT
        with tempfile.TemporaryDirectory() as directory:
            OUT = Path(directory)
            build(args.source)
            for path in OUT.rglob('*'):
                if path.is_file():
                    relative = path.relative_to(OUT)
                    assert (expected / relative).read_bytes() == path.read_bytes(), relative
        print('All source, layer, outline, mask and arm exports reproduce exactly.')
    else:
        build(args.source)
