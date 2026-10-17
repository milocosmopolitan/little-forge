"""Check the reported neck gaps and preserve the accepted walk motion."""
from pathlib import Path
from PIL import Image, ImageChops
import json

ROOT=Path(__file__).resolve().parents[1]
def read(path):return Image.open(ROOT/path).convert('RGBA')
def skin(q):return q[3] and q[0]>185 and 85<q[1]<210 and q[2]<170 and q[0]>q[1]*1.2
def holes(im):
    transparent={(x,y) for y in range(160) for x in range(128) if not im.getpixel((x,y))[3]}
    stack=[(0,0)]
    while stack:
        p=stack.pop()
        if p not in transparent:continue
        transparent.remove(p);x,y=p;stack.extend([(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
    return {p for p in transparent if 44<=p[0]<=76 and 52<=p[1]<=70}

before_gaps={};after_gaps={}
for name in ['nod-down','turn-left','turn-right']:
    before=read(f'qa/contour-before/{name}-base.png');after=read(f'frames/base/{name}.png')
    before_gaps[name]=len(holes(before));after_gaps[name]=len(holes(after))
    assert not holes(after),(name,holes(after))
    coverage=read(f'source/animation/contour/{name}-neck-coverage.png').getchannel('A')
    assert ImageChops.subtract(coverage,after.getchannel('A')).getbbox() is None,(name,'neck coverage')
assert sum(before_gaps.values())>0,'regression must reject the previous gaps'

gait=json.loads((ROOT/'source/animation/walk-phases.json').read_text())
arm_skin_checked=0
for k in range(6):
    base=read(f'frames/base/walk-{k}.png');old=read(f'qa/contour-before/walk-{k}-base.png')
    assert base.tobytes()==old.tobytes(),(k,'accepted walking pose changed')
    full=read(f'frames/walk-{k}.png');front=read(f'poses/walk-{k}/layers/hands-front.png');gloves=read(f'poses/walk-{k}/equipment/gloves.png')
    for y in range(70,114):
        for x in range(38,98):
            q=front.getpixel((x,y))
            if q[3]:
                assert q==base.getpixel((x,y)),(k,x,y,'foreign foreground pixel')
                if max(q[:3])<160 and any(not base.getpixel((x+dx,y+dy))[3] for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]):
                    assert full.getpixel((x,y))==q,(k,x,y,'lost outer arm contour')
            if gloves.getpixel((x,y))[3]:assert skin(base.getpixel((x,y))),(k,x,y,'glove outside hand')
    contacts=gait[k]['anatomical_contacts']
    for side in ['right_hand']+(['left_hand'] if k in (2,3,4) else []):
        points={(x,y) for y in range(75,114) for x in range(38,98) if skin(base.getpixel((x,y)))}
        hx,hy=contacts[side];seed=min(points,key=lambda p:abs(p[0]-hx)+abs(p[1]-hy))
        stack=[seed];component=set()
        while stack:
            p=stack.pop()
            if p not in points:continue
            points.remove(p);component.add(p);x,y=p;stack.extend([(x+dx,y+dy) for dx in [-1,0,1] for dy in [-1,0,1]])
        assert all(front.getpixel(p)[3] for p in component),(k,side,'cropped forearm/hand')
        arm_skin_checked+=len(component)

round_mouth=read('mouth/round.png');surprise=read('mouth/surprised-open.png')
assert round_mouth.getbbox()==(69,54,72,57),'accepted speech round mouth changed'
assert surprise.getbbox()==(69,54,73,59) and surprise.tobytes()!=round_mouth.tobytes()
pose=json.loads((ROOT/'poses/recipes.json').read_text())['poses']['surprised']
assert read(pose['layers']['mouth']).tobytes()==surprise.tobytes()
registration=json.loads((ROOT/'source/animation/registration.json').read_text())
assert registration['nod-down']['paste']==[40,19],'deep nod must be one pixel below prior registration'
report={'status':'pass','neck_holes_before':before_gaps,'neck_holes_after':after_gaps,'complete_neck_coverage':True,'accepted_walk_base_frames_unchanged':6,'outer_arm_contours_preserved':True,'visible_forearm_skin_pixels_checked':arm_skin_checked,'visible_forearm_skin_pixels_cropped':0,'gloves_confined_to_hand_silhouettes':True,'surprise_uses_separate_open_mouth':True,'speech_round_mouth_preserved':True,'nod_head_lowered_px':1}
(ROOT/'qa/contour-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
