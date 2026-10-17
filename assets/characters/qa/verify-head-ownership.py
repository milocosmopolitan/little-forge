"""Read-only checks for the neck/collar fragments identified in the user screenshots."""
import json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return Image.open(p).convert('RGBA')
checked=0
for sex in ['female','male']:
 animation=json.loads((ROOT/sex/'animation.json').read_text())
 for name,pose in animation['poses'].items():
  if not pose.get('head_identity',{}).get('root_pixels_unchanged'):continue
  for equipped in [False,True]:
   head=read(ROOT/sex/'poses'/name/('rendered/head-base.png' if equipped else 'head-base.png'))
   shift=pose['rendered_head_identity' if equipped else 'head_identity']['shift']
   for row,span in animation['head_ownership']['removed_neck_collar_rows'].items():
    for x in range(span[0],span[1]+1):
     assert not head.getpixel((x+shift[0],int(row)+shift[1]))[3],(sex,name,equipped,'neck/collar still in moving head',x,row)
   checked+=1
# These positions were the detached ear and protruding old-jaw line, not garment.
for sex,name,points in [
 ('male','quenching-2',[(68,63),(69,65),(83,77),(84,77)]),
 ('male','quenching-1',[(80,69)]),
 ('female','hammering-2',[(82,68),(83,68)]),
 ('female','quenching-1',[(61,57)])]:
 upper=read(ROOT/sex/'poses'/name/'rendered/body/upper.png')
 for point in points:assert not upper.getpixel(point)[3],(sex,name,'old head fragment remains',point)
# The raised thinking hand is connected to facial skin in the source; it is not head material.
raw=read(ROOT/'female/poses/researching-4/source/equipped-before-head-restoration.png')
upper=read(ROOT/'female/poses/researching-4/rendered/body/upper.png')
for point in [(74,72),(74,73),(75,74),(75,75),(76,76)]:
 assert raw.getpixel(point)[3] and upper.getpixel(point)==raw.getpixel(point),('thinking hand erased',point)
# A single attachment replaces the rejected per-key 1px/2px/3px offsets.
for sex,mount in [('female',[62,66]),('male',[57,64])]:
 animation=json.loads((ROOT/sex/'animation.json').read_text())
 for name,pose in animation['poses'].items():
  if not pose.get('head_identity',{}).get('root_pixels_unchanged'):continue
  for identity in [pose['head_identity'],pose['rendered_head_identity']]:
   cx,cy=identity['source_collar']
   assert identity['root_collar']==mount and identity['shift']==[cx-mount[0],cy-mount[1]],(sex,name,'nonuniform attachment')
   assert identity['anatomical_head_adjustment']==[0,0] and identity['head_translation_adjustment']==[0,0],(sex,name,'ad hoc head correction')
report={'status':'pass','work_head_variants_checked':checked,'moving_head_neck_collar_pixels':0,'reported_old_head_fragment_points':8,'uniform_body_attachment_variants':96,'thinking_hand_source_points_preserved':5,'approved_eyes_and_mouth':'separate exact-pixel assertions in verify-animation.py','visual_acceptance':'not established by these assertions'}
(ROOT/'qa/head-ownership-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
