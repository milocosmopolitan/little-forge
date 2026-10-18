#!/usr/bin/env python3
"""Capture isolated player-smith stories: both characters, all 17 clips, equipped and base,
mirrored facing, light and dark backgrounds, stations on and off. No simulation or save IO.

The preview uses the same `forge_npc::player` component and station floor plan as the game;
these images are motion/composition review candidates, not game-state evidence.
"""
import subprocess
import sys
from pathlib import Path
from PIL import Image

exe = Path(sys.argv[1] if len(sys.argv) > 1 else 'target/debug/npc-preview').resolve()
out = Path(sys.argv[2] if len(sys.argv) > 2 else 'qa-output/player')
out.mkdir(parents=True, exist_ok=True)
CLIPS = ['idle', 'blink', 'talk', 'talk_blink', 'happy', 'surprised', 'relaxed', 'nod', 'shake_head',
         'give', 'walk', 'front_blink', 'hammering', 'quenching', 'forging', 'researching', 'salvaging']
SAMPLE = {'blink': 0.85, 'front_blink': 0.93, 'give': 0.4, 'nod': 0.3, 'shake_head': 0.38,
          'hammering': 0.3, 'quenching': 0.5, 'forging': 0.9, 'researching': 0.6, 'salvaging': 0.7}


def capture(name, args):
    png = out / f'{name}.png'
    with (out / f'{name}.log').open('w') as log:
        subprocess.run([str(exe), *args, '--capture', str(png)], stdout=log, stderr=subprocess.STDOUT,
                       check=True, timeout=120)
    with Image.open(png) as im:
        im.verify()
    assert 'panicked' not in (out / f'{name}.log').read_text(), name


for character in ['female', 'male']:
    for clip in CLIPS:
        capture(f'{character}-{clip}', ['--character', character, '--story', clip, '--at', str(SAMPLE.get(clip, 0.3))])
    # Work clips: 6-frame sequences, base variant, mirrored and light background checks.
    for clip in ['hammering', 'quenching', 'forging', 'researching', 'salvaging']:
        capture(f'{character}-{clip}-seq', ['--character', character, '--story', clip, '--sequence'])
        capture(f'{character}-{clip}-base', ['--character', character, '--story', clip, '--base', '--at', str(SAMPLE[clip])])
    capture(f'{character}-idle-left-light', ['--character', character, '--story', 'idle', '--left', '--light'])
    capture(f'{character}-hammering-no-stations', ['--character', character, '--story', 'hammering', '--no-stations', '--at', '0.3'])
    capture(f'{character}-walk-to-anvil', ['--character', character, '--story', 'walk-to-anvil'])
print('Captured player stories in', out)
