#!/usr/bin/env python3
"""Pack authored native frames for the fixed-cell Bevy catalog without resampling."""
import hashlib
import io
import json
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'assets/npc/farmer'


def export(check=False):
    animation = json.loads((FOLDER / 'animation.json').read_text())
    poses = json.loads((FOLDER / 'poses/recipes.json').read_text())['poses']
    width, height = animation['canvas']
    # Neutral is first for consumers that initially display atlas index zero.
    names = ['neutral'] + sorted(set(animation['frames']) - {'neutral'})
    columns = 4
    rows = (len(names) + columns - 1) // columns
    atlas = Image.new('RGBA', (width * columns, height * rows))
    rects = {}
    for index, name in enumerate(names):
        frame = Image.open(FOLDER / animation['frames'][name]['equipped']).convert('RGBA')
        if frame.size != (width, height):
            raise ValueError(f'Unexpected native canvas: {name}')
        x, y = index % columns * width, index // columns * height
        atlas.paste(frame, (x, y))
        rects[name] = [x, y, width, height]

    offset = animation.get('base_offset_px', [0, 0])
    def socket(point):
        return [a + b for a, b in zip(point, offset)]

    catalog = dict(id='farmer', version=1, faces=None,
                   image='npc/farmer/runtime-atlas.png', size=list(atlas.size),
                   cell=[width, height], foot=animation['foot'], head=socket([53, 37]),
                   frames=[], clips={})
    for name, clip in animation['clips'].items():
        indices = []
        for step in clip['frames']:
            sprite = step['sprite']
            sockets = poses.get(sprite, {}).get('sockets_px', {})
            indices.append(len(catalog['frames']))
            catalog['frames'].append(dict(
                rect=rects[sprite], duration=step['duration_ms'] / 1000,
                hand=sockets.get('left_hand', socket([70, 85])),
                transfer=step.get('event') == 'transfer'))
        catalog['clips'][name] = dict(frames=indices, looping=clip['loop'])

    buffer = io.BytesIO()
    atlas.save(buffer, format='PNG')
    outputs = {
        FOLDER / 'runtime-atlas.png': buffer.getvalue(),
        FOLDER / 'character.json': (json.dumps(catalog, indent=2) + '\n').encode(),
    }
    manifest_path = ROOT / 'assets/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    # Retire only the legacy farmer inventory; retain every other asset entry.
    manifest['assets'] = [a for a in manifest['assets'] if not a['id'].startswith('farmer_')]
    for name, size, grid, anchor in [
        ('runtime-atlas', atlas.size, (columns, rows), [0.5, animation['foot'][1] / height]),
        ('carrot', Image.open(FOLDER / 'carrot.png').size, (1, 1), [0.5, 0.5]),
    ]:
        asset_path = FOLDER / f'{name}.png'
        data = outputs.get(asset_path, asset_path.read_bytes() if asset_path.exists() else b'')
        manifest['assets'].append(dict(
            id='farmer_' + name.replace('-', '_'), path=f'npc/farmer/{name}.png',
            width=size[0], height=size[1], columns=grid[0], rows=grid[1], alpha=True,
            anchor=anchor, sha256=hashlib.sha256(data).hexdigest()))
    outputs[manifest_path] = (json.dumps(manifest, indent=2) + '\n').encode()
    for target, data in outputs.items():
        if check:
            if not target.exists() or target.read_bytes() != data:
                raise ValueError(f'Stale runtime export: {target.relative_to(ROOT)}')
        else:
            target.write_bytes(data)
    print(f'Farmer runtime {"verified" if check else "exported"}: {len(names)} native sprites, {len(catalog["clips"])} clips')


if __name__ == '__main__':
    export(check='--check' in sys.argv)
