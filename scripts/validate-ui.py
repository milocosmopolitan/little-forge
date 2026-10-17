#!/usr/bin/env python3
"""Verify candidate provenance and bundled font; --release enforces the art gate.

Never modifies images or equates structural checks with visual acceptance.
"""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image
from fontTools.ttLib import TTFont

p = argparse.ArgumentParser()
p.add_argument('--release', action='store_true')
a = p.parse_args()
manifest = json.loads(Path('assets/ui/source-manifest.json').read_text())
for entry in manifest['assets']:
    path = Path(entry['path'])
    assert hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], path
    with Image.open(path) as im:
        assert im.size == (entry['width'], entry['height']), path
        assert im.mode == entry['mode'], path
        if entry['alpha_extrema'] is not None:
            assert list(im.getextrema()[3]) == entry['alpha_extrema'], path
font = Path('assets/ui/fonts/NotoSansKR.ttf')
font_manifest = json.loads(Path('assets/ui/fonts/manifest.json').read_text())
assert hashlib.sha256(font.read_bytes()).hexdigest() == font_manifest['font_sha256']
assert hashlib.sha256(Path('assets/ui/fonts/OFL.txt').read_bytes()).hexdigest() == font_manifest['license_sha256']
cmap = TTFont(font).getBestCmap()
required = list(range(32, 127)) + list(range(0xAC00, 0xD7A4))
assert all(point in cmap for point in required), 'ASCII / Hangul glyph missing'
assert 'SIL OPEN FONT LICENSE' in Path('assets/ui/fonts/OFL.txt').read_text()
print('Source hashes/dimensions and ASCII + 11,172 Hangul syllables verified.')
print('Art gate: CANDIDATES ONLY. Independent native exports / seam review remain.')
if a.release:
    assert manifest['runtime_ready'], 'UI 02/03 asset gate is not complete'
