"""Far cape fills the exposed gaps without changing either walking arm."""
from pathlib import Path
from PIL import Image
import json
ROOT=Path(__file__).resolve().parents[1]
def read(p):return Image.open(ROOT/p).convert('RGBA')
rows=[]
for k in range(6):
    before=read(f'qa/far-cape-before/walk-{k}.png');after=read(f'frames/walk-{k}.png')
    body=read(f'source/animation/contour/walk-{k}-body.png')
    mask=read(f'source/animation/contour/walk-{k}-far-cape-coverage.png')
    shape=read(f'source/animation/contour/walk-{k}-far-cape-shape.png')
    cape=read(f'poses/walk-{k}/equipment/cloak.png');changed=0;coverage=0
    for y in range(160):
        for x in range(128):
            p=(x,y)
            if shape.getpixel(p)[3] and not body.getpixel(p)[3]:
                assert mask.getpixel(p)[3] and cape.getpixel(p)[3],(k,p,'far cape gap')
                coverage+=1
            if mask.getpixel(p)[3]:assert not body.getpixel(p)[3],(k,p,'cape over body/arm')
            if before.getpixel(p)!=after.getpixel(p):
                assert mask.getpixel(p)[3] and not body.getpixel(p)[3],(k,p,'change outside exposed rear cape')
                changed+=1
    assert changed>0 and coverage>0
    assert read(f'frames/base/walk-{k}.png').tobytes()==read(f'qa/contour-before/walk-{k}-base.png').tobytes()
    rows.append(dict(key=k,visible_far_cape_pixels=coverage,changed_pixels=changed,arm_pixels_changed=0))
report=dict(status='pass',keys=rows,scope='six walk frames; exact preservation outside exposed rear cape')
(ROOT/'qa/far-cape-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
