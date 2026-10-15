#!/usr/bin/env python3
"""Validate committed original PNGs, atlas geometry, hashes and meaningful transparency."""
import hashlib,json,struct
from PIL import Image
from pathlib import Path
root=Path(__file__).resolve().parents[1]/'assets'
m=json.loads((root/'manifest.json').read_text())
for a in m['assets']:
    data=(root/a['path']).read_bytes()
    assert data[:8]==b'\x89PNG\r\n\x1a\n', a['id']
    w,h,depth,kind=struct.unpack('>IIBB',data[16:26])
    assert (w,h)==(a['width'],a['height']),a['id']
    assert w%a['columns']==h%a['rows']==0,a['id']
    assert kind in (4,6) if a['alpha'] else True,a['id']
    assert hashlib.sha256(data).hexdigest()==a['sha256'],a['id']
    if a['alpha']:
        with Image.open(root/a['path']) as im:
            lo,hi=im.getchannel('A').getextrema()
            assert lo==0 and hi>0,(a['id'],'missing transparent or visible pixels')
    print(a['id'],w,h,'OK')
