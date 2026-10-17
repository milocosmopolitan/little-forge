"""Required body coverage must hold even when the head hides the missing pixels."""
import hashlib,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'source/animation/solid-underlap-v10'
def image(p):return Image.open(p).convert('RGBA')
def enclosed_body_holes(frame,raw,shift):
 pixels=frame.load();seen=set();bad=[];allowed={(44,57),(44,58),(45,59),(45,60)}
 for y in range(160):
  for x in range(128):
   if (x,y) in seen or pixels[x,y][3]:continue
   group=[(x,y)];seen.add((x,y));outside=False
   for xx,yy in group:
    outside|=xx in (0,127) or yy in (0,159)
    for point in [(xx-1,yy),(xx+1,yy),(xx,yy-1),(xx,yy+1)]:
     a,b=point
     if 0<=a<128 and 0<=b<160 and point not in seen and not pixels[a,b][3]:seen.add(point);group.append(point)
   if outside:continue
   if all(40<=a<110 and 45<=b<100 and raw.getpixel((a,b))[3] for a,b in group):
    if not all((a-shift[0],b-shift[1]) in allowed for a,b in group):bad.extend(group)
 return bad
animation=json.loads((ROOT/'female/animation.json').read_text());baseline=json.loads((SOURCE/'before-animation.json').read_text());results=[]
for id,pose in baseline['poses'].items():
 if not pose.get('head_identity',{}).get('root_pixels_unchanged'):continue
 for variant in ['base','equipped']:
  equipped=variant=='equipped';folder=ROOT/'female/poses'/id
  raw=image(folder/('source/equipped-before-head-restoration.png' if equipped else 'source/body-before-head-restoration.png'))
  upper=image(folder/('rendered/body/upper.png' if equipped else 'body/upper-base.png'))
  before_upper=image(SOURCE/'before-upper'/variant/(id+'.png'))
  contract=image(SOURCE/'contracts'/variant/(id+'.png'))
  before=image(SOURCE/'before'/variant/(id+'.png'));final=image(ROOT/'female'/pose['composed' if equipped else 'base'])
  identity=pose['rendered_head_identity' if equipped else 'head_identity'];shift=identity['shift']
  assert identity==animation['poses'][id]['rendered_head_identity' if equipped else 'head_identity'],(id,variant,'head transform changed')
  missing_before=0;added=0
  for y in range(160):
   for x in range(128):
    old=before_upper.getpixel((x,y));new=upper.getpixel((x,y))
    if old[3]:assert old==new,(id,variant,'existing body repainted',x,y)
    if not old[3] and new[3]:added+=1
    if contract.getpixel((x,y))[3]:
     missing_before+=not old[3]
     assert new[3]==255,(id,variant,'missing required body',x,y)
  holes_before=enclosed_body_holes(before,raw,shift);holes_after=enclosed_body_holes(final,raw,shift)
  assert not holes_after,(id,variant,'unexpected body holes',holes_after)
  results.append(dict(pose=id,variant=variant,missing_body_before=missing_before,added_body_pixels=added,unexpected_holes_before=len(holes_before),unexpected_holes_after=0))
# The two genuine gaps between the arms remain transparent.
for variant in ['base','equipped']:
 frame=image(ROOT/'female'/baseline['poses']['quenching-2']['composed' if variant=='equipped' else 'base'])
 original=image(SOURCE/'before'/variant/'quenching-2.png')
 for point in [(78,91),(78,92)]:assert frame.getpixel(point)==original.getpixel(point),('arm region changed',variant,point)
 for point in [(76,76),(81,80),(73,75),(74,75),(70,78),(71,78)]:assert frame.getpixel(point)[3]==255,('screenshot gap remains',variant,point)
retained=json.loads((SOURCE/'retained-frames.json').read_text())
for relative,digest in retained.items():assert hashlib.sha256((ROOT/relative).read_bytes()).hexdigest()==digest,('unrelated frame changed',relative)
assert len(results)==48
assert sum(r['unexpected_holes_before'] for r in results)>0,'Rejected baseline must fail the missing-pixel check'
report=dict(status='pass',variants_checked=48,missing_required_body_before=sum(r['missing_body_before'] for r in results),unexpected_body_hole_pixels_before=sum(r['unexpected_holes_before'] for r in results),unexpected_body_hole_pixels_after=0,unrelated_frames_preserved=len(retained),intentional_back_hair_cutouts='root coordinates (44,57), (44,58), (45,59), (45,60)',results=results)
(ROOT/'qa/solid-underlap-v10-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='results'}))
