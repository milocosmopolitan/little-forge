#!/usr/bin/env python3
"""Capture the forge spaces in the actual Bevy app (#89, #91, #93, #94, #96).

Every scenario uses an isolated save directory and the fixed QA clock; no user save is touched.
Workshop captures include the heating / hammering / quenching stages of one real 5-second job.
No cross-GPU exact-pixel assertion: the images are review candidates for the retained artifacts.
"""
import argparse
import json
import subprocess
from pathlib import Path
from PIL import Image

p = argparse.ArgumentParser()
p.add_argument('executable')
p.add_argument('--output', default='qa-output/forge')
a = p.parse_args()
exe = str(Path(a.executable).resolve())
out = Path(a.output)
out.mkdir(parents=True, exist_ok=True)


def run(scene, name=None, env=None):
    name = name or scene
    png = out / f'{name}.png'
    with (out / f'{name}.log').open('w') as log:
        subprocess.run([exe, '--safe-window', '--qa', scene, '--save-dir', str(out / f'session-{name}'),
                        '--screenshot', str(png)], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=240, env=env)
    with Image.open(png) as im:
        assert im.width > 0 and im.height > 0
        assert im.convert('RGB').getextrema()[0][1] > 60, f'{name}: empty screenshot'
    log_text = (out / f'{name}.log').read_text()
    assert 'panicked' not in log_text, f'{name}: panic in log'
    print(f'{scene}: actual app rendered', flush=True)


for scene in ['forge-exterior', 'forge-interior', 'forge-workshop', 'forge-showroom', 'forge-full', 'forge-select',
              'forge-workshop-female', 'forge-showroom-female', 'forge-exterior-female']:
    run(scene)
for stage in ['045', '100', '140']:
    for who in ['forge-workshop', 'forge-workshop-female']:
        assert (out / f'{who}-{stage}.png').is_file(), f'missing stage capture {who}-{stage}'
summary = {
    'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
    'scenes': sorted(p.name for p in out.glob('*.png')),
    'clock': 'QA fixed 1/30 s per frame; production uses monotonic elapsed time',
    'note': 'Visual candidates. Native macOS/Windows transparency, DPI and input are separate gates.',
}
(out / 'summary.json').write_text(json.dumps(summary, indent=2))
print('forge spaces captured in', out)
