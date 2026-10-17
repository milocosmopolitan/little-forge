"""Regression checks for all six equipped walk ownership boundaries."""
from pathlib import Path
from PIL import Image
import json
ROOT=Path(__file__).resolve().parents[1]
def read(p):return Image.open(ROOT/p).convert('RGBA')
rows=[]
for k in range(6):
    front=read(f'poses/walk-{k}/layers/hands-front.png')
    old=read(f'qa/walk-occlusion-before/walk-{k}-front.png')
    shoulder=read(f'source/animation/contour/walk-{k}-cloak-shoulder.png')
    back=read(f'source/animation/contour/walk-{k}-cloak-back-coverage.png')
    vest_mask=read(f'source/animation/contour/walk-{k}-vest-coverage.png')
    cape=read(f'poses/walk-{k}/equipment/cloak.png')
    vest=read(f'poses/walk-{k}/equipment/leather-outfit.png')
    removed_shoulder=removed_shirt=back_added=0
    for y in range(160):
        for x in range(128):
            p=(x,y);q=front.getpixel(p);prior=old.getpixel(p)
            if shoulder.getpixel(p)[3]:
                assert not q[3],(k,p,'arm over cape shoulder')
                removed_shoulder+=bool(prior[3])
            if back.getpixel(p)[3]:
                assert cape.getpixel(p)[3],(k,p,'cape back hole')
                back_added+=not bool(shoulder.getpixel(p)[3])
            if vest_mask.getpixel(p)[3]:assert vest.getpixel(p)[3],(k,p,'unfilled neutral wrist cutout')
            if prior[3] and not q[3] and not shoulder.getpixel(p)[3] and prior[0]>150 and prior[1]>130 and prior[2]>100:
                assert vest.getpixel(p)[3],(k,p,'shirt removed without outfit coverage')
                removed_shirt+=1
    assert removed_shoulder>0 and back_added>0,(k,'old shoulder/back defect not exercised')
    assert removed_shirt>0,(k,'torso shirt foreground defect not exercised')
    assert read(f'frames/base/walk-{k}.png').tobytes()==read(f'qa/contour-before/walk-{k}-base.png').tobytes()
    rows.append(dict(key=k,shoulder_pixels_no_longer_over_cape=removed_shoulder,back_coverage_added=back_added,torso_shirt_pixels_removed_from_foreground=removed_shirt))
report=dict(status='pass',all_six_walk_keys_checked=True,bare_walk_motion_unchanged=True,keys=rows)
(ROOT/'qa/walk-occlusion-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
