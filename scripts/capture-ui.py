#!/usr/bin/env python3
"""Capture exact shared Bevy widgets, isolated from gameplay and user saves."""
import argparse
import os
import subprocess
import tempfile
from pathlib import Path
from PIL import Image

p = argparse.ArgumentParser()
p.add_argument('executable')
p.add_argument('--output', default='qa-output/ui-preview')
a = p.parse_args()
exe = str(Path(a.executable).resolve())
out = Path(a.output)
out.mkdir(parents=True, exist_ok=True)
scenes = ['farm', 'chef', 'carpenter', 'leatherworker', 'empty', 'full',
          'insufficient', 'sold-out', 'long-label', 'controls', 'confirmation']
def capture(name, args, env=None):
    png = out / (name + '.png')
    with (out / (name + '.log')).open('w') as log:
        subprocess.run([exe, '--capture', str(png), *args], check=True,
                       timeout=180, stdout=log, stderr=subprocess.STDOUT, env=env)
    with Image.open(png) as im:
        im.verify()
    print(name, flush=True)
for scene in scenes:
    capture(scene, ['--story', scene])
capture('small', ['--story', 'long-label', '--width', '640', '--height', '480'])
capture('dpi-200', ['--story', 'farm'], {**os.environ, 'WINIT_X11_SCALE_FACTOR': '2'})
with tempfile.TemporaryDirectory(prefix='forge-ui-missing-') as folder:
    # Keep the licensed font to capture a readable fallback without art.
    import shutil
    target = Path(folder) / 'ui/fonts'
    target.mkdir(parents=True)
    shutil.copy2('assets/ui/fonts/NotoSansKR.ttf', target / 'NotoSansKR.ttf')
    capture('missing-art', ['--story', 'farm', '--assets', folder])
