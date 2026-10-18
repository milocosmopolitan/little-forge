#!/usr/bin/env python3
"""Verify the registered forge-space art: sources untouched, exports byte-exact, hard alpha,
scene-sized interiors and sane nine-slice insets. Structural only; never a visual approval.
"""
import hashlib
import json
from pathlib import Path
from PIL import Image

root = Path('assets/forge/spaces')
catalog = json.loads((root / 'catalog.json').read_text())
assert catalog['version'] == 2 and catalog['sampler'] == 'nearest' and catalog['integer_scale_only']
for name, src in catalog['sources'].items():
    path = Path('assets') / src['path']
    assert hashlib.sha256(path.read_bytes()).hexdigest() == src['sha256'], f'source changed: {name}'
scene = tuple(catalog['scene']['size'])
for name, entry in catalog['assets'].items():
    path = Path('assets') / entry['path']
    assert hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], f'export changed: {name}'
    with Image.open(path) as im:
        assert im.size == (entry['width'], entry['height']), name
        assert im.mode == 'RGBA', name
        if name.endswith('-backdrop'):
            assert im.size == scene, f'{name} is not the native scene size'
        if 'insets_ltrb' in entry:
            l, t, r, b = entry['insets_ltrb']
            assert l + r < im.width and t + b < im.height, f'{name}: insets exceed size'
            hist = im.getchannel('A').histogram()
            assert sum(hist[1:255]) == 0, f'{name}: semi-transparent halo pixels survived the alpha cut'
            # The centre tile must be opaque fill so stretched panels never show the scene through.
            assert im.getpixel((im.width // 2, im.height // 2))[3] == 255, name
    print(name, im.size, 'OK')
# Station art the game composes with these scenes.
stations = json.loads(Path('assets/characters/stations/catalog.json').read_text())
for sid, s in stations['stations'].items():
    for key in ('path', 'back', 'front', 'water', 'fire'):
        with Image.open(Path('assets/characters') / s[key]) as im:
            assert im.size == tuple(s['size']), (sid, key)
print('forge spaces OK')
