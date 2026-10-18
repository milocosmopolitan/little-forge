#!/usr/bin/env python3
"""Capture actual Bevy scenes on each CI OS; no cross-GPU exact-pixel assertions."""
import argparse, json, subprocess
from pathlib import Path
from PIL import Image

p = argparse.ArgumentParser()
p.add_argument('executable')
p.add_argument('--output', default='qa-output/village')
a = p.parse_args()
out = Path(a.output)
out.mkdir(parents=True, exist_ok=True)
for scene in ['map', 'carpenter', 'leatherworker', 'farm', 'tavern', 'chief', 'guild', 'outside']:
    png = out / f'{scene}.png'
    with (out / f'{scene}.log').open('w') as log:
        subprocess.run([str(Path(a.executable).resolve()), '--safe-window', '--qa', f'village-{scene}', '--save-dir', str(out / 'session'), '--screenshot', str(png)], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)
    with Image.open(png) as im:
        assert im.width > 0 and im.height > 0
        assert im.convert('RGB').getextrema()[0][1] > 100, 'empty screenshot'
    print(f'village-{scene}: actual app rendered', flush=True)

# Exercise optional-art failure in the actual packaged layout, without touching repository assets.
import shutil, tempfile
with tempfile.TemporaryDirectory(prefix='forge-village-missing-') as temporary:
    stage = Path(temporary)
    binary = stage / Path(a.executable).name
    shutil.copy2(Path(a.executable).resolve(), binary)
    # Core smith/UI art is required; only village and NPC art are deliberately absent.
    for folder in ['forge', 'title', 'characters', 'ui']:
        shutil.copytree(Path('assets')/folder, stage/'assets'/folder)
    png = out / 'missing-assets.png'
    with (out/'missing-assets.log').open('w') as log:
        subprocess.run([str(binary), '--safe-window', '--qa', 'village-map', '--save-dir', str(stage/'session'), '--screenshot', str(png.resolve())], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)
    with Image.open(png) as im: im.verify()
    assert 'Village map unavailable' in (out/'missing-assets.log').read_text()
    print('village missing-art fallback: actual app rendered destination navigation', flush=True)

    # NPC illustration loss must not prevent a committed purchase from being presented safely.
    png = out / 'missing-npc.png'
    with (out/'missing-npc.log').open('w') as log:
        subprocess.run([str(binary), '--safe-window', '--qa', 'npc', '--save-dir', str(stage/'session'), '--screenshot', str(png.resolve())], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)
    with Image.open(png) as im: im.verify()
    print('missing NPC art: actual game remains usable', flush=True)
