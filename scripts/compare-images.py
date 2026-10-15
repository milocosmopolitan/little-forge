#!/usr/bin/env python3
"""Compare a reviewed baseline with a same-renderer candidate; never update goldens implicitly."""
import argparse,json
from pathlib import Path
from PIL import Image,ImageChops
p=argparse.ArgumentParser();p.add_argument('baseline');p.add_argument('candidate');p.add_argument('--channel-tolerance',type=int,default=8);p.add_argument('--max-changed-ratio',type=float,default=0.005);p.add_argument('--diff',default='qa-output/diff.png');a=p.parse_args()
b=Image.open(a.baseline).convert('RGBA');c=Image.open(a.candidate).convert('RGBA')
if b.size!=c.size:raise SystemExit('FAIL: image size changed')
d=ImageChops.difference(b,c);count=sum(max(pixel)>a.channel_tolerance for pixel in d.getdata());ratio=count/(b.width*b.height)
Path(a.diff).parent.mkdir(parents=True,exist_ok=True);d.convert('RGB').save(a.diff)
print(json.dumps({'changed_ratio':ratio,'allowed_ratio':a.max_changed_ratio,'dimensions':b.size}))
raise SystemExit(0 if ratio<=a.max_changed_ratio else 1)
