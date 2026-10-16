#!/usr/bin/env python3
"""Capture and assert the actual Bevy lifecycle; safe surface for reproducible CI pixels."""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('executable');p.add_argument('--output',default='qa-output/lifestyle');a=p.parse_args()
out=Path(a.output);out.mkdir(parents=True,exist_ok=True)
for scene in ['life-loop','life-ambient']:
    png=out/f'{scene}.png'
    with (out/f'{scene}.log').open('w') as log:
        subprocess.run([str(Path(a.executable).resolve()),'--safe-window','--qa',scene,'--save-dir',str(out/scene),'--screenshot',str(png)],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=240)
    assert png.is_file(), f'missing {png}'
    report=json.loads(Path(f'{png}.json').read_text());s=report['state'];events=report['events']
    assert s['sold']==2 and s['command_id']==3 and s['life']['pantry']['slots'][0]['count']==3
    for action in ['Sleeping','Eating','Drinking','Working']:
        assert any(e['kind']=={'ActionChanged':action} for e in events),action
    assert any(e['kind']=='Ate' for e in events)
    assert any(e['kind']=='Drank' for e in events)
    print(f'{scene}: complete life cycle, two sales, explicit farmer purchase, screenshots and trace verified',flush=True)
