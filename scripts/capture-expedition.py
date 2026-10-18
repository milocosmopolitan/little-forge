"""Actual desktop snapshots at deterministic combat event boundaries, not painted mockups."""
import argparse,json,subprocess
from pathlib import Path
from PIL import Image
p=argparse.ArgumentParser();p.add_argument('executable');p.add_argument('--output',default='qa-output/expedition/game');a=p.parse_args()
out=Path(a.output);out.mkdir(parents=True,exist_ok=True);exe=str(Path(a.executable).resolve())
for scene in ['entry','movement','bow','release','dagger','wolf','boar','retreat','victory']:
 png=out/f'{scene}.png'
 with (out/f'{scene}.log').open('w') as log:
  subprocess.run([exe,'--safe-window','--qa',f'expedition-{scene}','--save-dir',str(out/f'session-{scene}'),'--screenshot',str(png)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
 im=Image.open(png);assert im.width>=640 and im.height>=360
 assert max(hi for lo,hi in im.convert("RGB").getextrema())>100, "empty combat screenshot"
 assert 'panicked' not in (out/f'{scene}.log').read_text();print(scene,'captured',flush=True)
(out/'summary.json').write_text(json.dumps({'scenes':['entry','movement','bow','release','dagger','wolf','boar','retreat','victory'],'evidence':'actual Bevy desktop app, frozen authoritative event boundaries, animation frames advance; not an interactive-input acceptance run'},indent=2))
