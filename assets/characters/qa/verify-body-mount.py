"""Check complete body-owned shoulders and one attachment, without hiding gaps with a head."""
import hashlib,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];SOURCE=ROOT/'source/animation/joint-rebuild-v11'
def image(p):return Image.open(p).convert('RGBA')
results=[];rejected_binding_failures=0
for sex,mount in [('female',[62,66]),('male',[57,64])]:
 root=ROOT/sex;animation=json.loads((root/'animation.json').read_text());before=json.loads((SOURCE/(sex+'-before-animation.json')).read_text())
 for id,pose in animation['poses'].items():
  if not pose.get('head_identity',{}).get('root_pixels_unchanged'):continue
  assert pose['neck_assembly']['body_owned_mount']['revision']=='joint-rebuild-v11'
  for equipped in [False,True]:
   prefix='rendered/' if equipped else '';folder=root/'poses'/id
   upper=image(folder/('rendered/body/upper.png' if equipped else 'body/upper-base.png'))
   coverage=image(folder/(prefix+'body/required-coverage.png'))
   identity=pose['rendered_head_identity' if equipped else 'head_identity'];cx,cy=identity['source_collar'];expected=[cx-mount[0],cy-mount[1]]
   assert identity['shift']==expected,(sex,id,equipped,'head not on body attachment')
   assert identity['anatomical_head_adjustment']==[0,0] and identity['head_translation_adjustment']==[0,0],(sex,id,'ad hoc correction')
   baseline=before['poses'][id]['rendered_head_identity' if equipped else 'head_identity']
   rejected_binding_failures+=baseline['shift']!=expected
   points=0
   for y in range(160):
    for x in range(128):
     if coverage.getpixel((x,y))[3]:
      assert upper.getpixel((x,y))[3]==255,(sex,id,equipped,'body coverage missing',x,y)
      points+=1
   assert points>0
   # The short neck is body-owned even where the head later hides it.
   for y in range(cy-3,cy+3):
    for x in range(cx-4,cx+4):assert upper.getpixel((x,y))[3]==255,(sex,id,equipped,'neck mount missing',x,y)
   # Isolated pixels around the neck are old source-head fragments, not shoulders.
   seen=set()
   for y in range(160):
    for x in range(128):
     if (x,y) in seen or not upper.getpixel((x,y))[3]:continue
     group=[(x,y)];seen.add((x,y))
     for xx,yy in group:
      for dx in [-1,0,1]:
       for dy in [-1,0,1]:
        point=(xx+dx,yy+dy)
        if 0<=point[0]<128 and 0<=point[1]<160 and point not in seen and upper.getpixel(point)[3]:seen.add(point);group.append(point)
     unwanted=len(group)<=40 and all(yy<cy+10 and cx-24<=xx<cx+28 and not coverage.getpixel((xx,yy))[3] for xx,yy in group)
     assert not unwanted,(sex,id,equipped,'floating head fragment in body',group)
   source_cloth=image(folder/(prefix+'body/source-connection.png'))
   raw=image(folder/('source/equipped-before-head-restoration.png' if equipped else 'source/body-before-head-restoration.png'))
   for y in range(160):
    for x in range(128):
     if source_cloth.getpixel((x,y))[3]:assert upper.getpixel((x,y))==raw.getpixel((x,y)),(sex,id,equipped,'source garment altered',x,y)
   results.append(dict(sex=sex,pose=id,equipped=equipped,required_body_pixels=points,missing_required_body_pixels=0))
retained=json.loads((SOURCE/'retained-frames.json').read_text())
for relative,digest in retained.items():assert hashlib.sha256((ROOT/relative).read_bytes()).hexdigest()==digest,('unrelated frame changed',relative)
assert len(results)==96 and rejected_binding_failures>0
report=dict(status='pass',revision='joint-rebuild-v11',variants_checked=96,missing_body_coverage_pixels=0,nonuniform_head_bindings=0,rejected_v10_binding_failures=rejected_binding_failures,unrelated_frames_preserved=len(retained),coverage_scope='Body-owned neck and shoulder plate, checked without the head; source garment colors checked separately',visual_acceptance='Not established by pixel assertions',variants=results)
(ROOT/'qa/body-mount-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='variants'}))
