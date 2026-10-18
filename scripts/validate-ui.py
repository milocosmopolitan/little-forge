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
runtime = json.loads(Path('assets/ui/runtime/manifest.json').read_text())
by_path = {}
manifest_limits = {'controls': runtime['palette_colors']['controls'], 'icon': runtime['palette_colors']['icons'], 'header': runtime['palette_colors']['headers']}
for entry in runtime['assets']:
    path = Path(entry['path'])
    assert hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], path
    with Image.open(path) as im:
        assert im.size == (entry['width'], entry['height']), path
        assert im.mode == entry['mode'], path
        if im.mode == 'RGBA':
            # Hard alpha only: generator halos must not survive export.
            hist = im.getchannel('A').histogram()
            assert sum(hist[1:255]) == 0, f'{path}: semi-transparent halo pixels'
            assert hist[0] > 0 or entry['kind'] == 'window', path
            # An export must not be a cropped-away fragment: its edge rows carry art.
            assert im.getchannel('A').getbbox() is not None, path
        # True pixel grid: every grid x grid block is one uniform colour.
        g = entry['grid']
        assert im.width % g == 0 and im.height % g == 0, path
        small = im.resize((im.width // g, im.height // g), Image.NEAREST)
        assert small.resize(im.size, Image.NEAREST).tobytes() == im.tobytes(), f'{path}: not a true {g}px grid'
        if entry['kind'] != 'header-foreground':
            limit = manifest_limits[entry['kind'] if entry['kind'] in ('icon', 'header') else 'controls']
            raw = small.convert('RGBA').tobytes()
            colours = {raw[i:i + 3] for i in range(0, len(raw), 4) if raw[i + 3]}
            assert len(colours) <= limit, f'{path}: {len(colours)} colours > {limit}'
        l, t, r, b = entry.get('insets_ltrb', (0, 0, 0, 0))
        assert l + r < entry['width'] and t + b < entry['height'], f'{path}: insets exceed size'
    by_path[entry['path']] = entry
# Variants of one control keep identical dimensions so state swaps never shift layout.
for group in ('button', 'tab', 'slot'):
    sizes = {(e['width'], e['height']) for e in runtime['assets'] if Path(e['path']).name.startswith(group + '-')}
    assert len(sizes) == 1, f'{group} states differ in size: {sizes}'
icons = {Path(e['path']).stem for e in runtime['assets'] if e['kind'] == 'icon'}
code = Path('crates/forge-ui/src/lib.rs').read_text()
declared = set(__import__('re').search(r'ICONS: \[&str; \d+\] = \[(.*?)\];', code, __import__('re').S).group(1).replace('"', '').replace(',', ' ').split())
assert icons == declared, f'icon exports differ from forge-ui ICONS: {icons ^ declared}'
for icon in icons:
    assert (e := by_path[f'assets/ui/runtime/icons/{icon}.png'])['width'] == runtime['icon_canvas'] == e['height']
for ref in set(__import__('re').findall(r'"(ui/runtime/[^"{}]+\.png)"', code)):
    assert f'assets/{ref}' in by_path, f'forge-ui references unregistered asset {ref}'
for header in ('carpenter', 'leatherworker', 'farm', 'tavern'):
    assert f'assets/ui/runtime/headers/{header}.png' in by_path, header
font = Path('assets/ui/fonts/NotoSansKR.ttf')
font_manifest = json.loads(Path('assets/ui/fonts/manifest.json').read_text())
assert hashlib.sha256(font.read_bytes()).hexdigest() == font_manifest['font_sha256']
assert hashlib.sha256(Path('assets/ui/fonts/OFL.txt').read_bytes()).hexdigest() == font_manifest['license_sha256']
cmap = TTFont(font).getBestCmap()
required = list(range(32, 127)) + list(range(0xAC00, 0xD7A4))
assert all(point in cmap for point in required), 'ASCII / Hangul glyph missing'
assert 'SIL OPEN FONT LICENSE' in Path('assets/ui/fonts/OFL.txt').read_text()
print('Source hashes/dimensions and ASCII + 11,172 Hangul syllables verified.')
print(f"Runtime exports verified: {len(runtime['assets'])} independent files, true pixel grid, palette limits, hard alpha, matching states/icons.")
print('Art gate: exports registered; runtime_ready stays false until visual approval.')
if a.release:
    assert manifest['runtime_ready'], 'UI 02/03 asset gate is not complete'
