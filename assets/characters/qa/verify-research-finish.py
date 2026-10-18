from retained_scope import current_retained
"""Guard the reported research defects, approved faces and unrelated action frames."""
import hashlib,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'source/animation/research-finish-v12'
def im(p):return Image.open(p).convert('RGBA')
def skin(p):return p[3] and p[0]>185 and p[1]>105 and p[2]>55 and p[0]>p[1]*1.22
checks=[]
for sex in ['female','male']:
 for k in range(6):
  folder=ROOT/sex/'poses'/f'researching-{k}';before=SOURCE/'before'/sex/f'researching-{k}'
  for name in ['head-base.png','eyes/open.png','mouth/rest.png','rendered/head-base.png','rendered/eyes/open.png','rendered/mouth/rest.png']:
   assert (folder/name).read_bytes()==(before/name).read_bytes(),(sex,k,'approved face changed',name)
  for equipped in [False,True]:
   front=im(folder/('body/arms-front.png' if equipped else 'body/arms-front-base.png'))
   if sex=='male':
    # Reported towel hung through the desktop at native y96..107.
    assert not any(front.getpixel((x,y))[3] for y in range(96,115) for x in range(38,54)),(sex,k,'towel still in front pass')
    if equipped:
     old_front=im(before/'body/arms-front.png')
     assert any(old_front.getpixel((x,y))[3] for y in range(96,110) for x in range(38,54)),('fixture misses reported defect',k)
     removed=im(folder/'rendered/body/removed-research-prop.png');final=im(folder/'composed.png')
     assert any(removed.getpixel((x,y))[3] for y in range(96,108) for x in range(38,54))
     for y in range(93,110):
      for x in range(38,54):
       if removed.getpixel((x,y))[3]:assert not final.getpixel((x,y))[3],('hanging cloth still present',k,x,y)
   if sex=='female':
    # Brightness of the old shoulder plate did not establish correct garment materials.
    # Strap continuity and collar ownership are checked by verify-research-material.py.
    if k==4:
     donor=im(ROOT/sex/'poses/researching-0/source'/('equipped-before-head-restoration.png' if equipped else 'body-before-head-restoration.png'))
     upper=im(folder/('rendered/body/upper.png' if equipped else 'body/upper-base.png'))
     # Distal finger landmarks must survive both authoring and the desktop pass.
     for xy in [(60,88),(61,89),(60,90),(61,91)]:
      assert donor.getpixel(xy)[3] and upper.getpixel(xy)==donor.getpixel(xy) and front.getpixel(xy)==donor.getpixel(xy),('lost resting finger',equipped,xy)
   checks.append([sex,k,equipped])
retained=json.loads((SOURCE/'retained-frames.json').read_text())
retained,superseded=current_retained(ROOT,retained)
for relative,digest in retained.items():assert hashlib.sha256((ROOT/relative).read_bytes()).hexdigest()==digest,('unrelated frame changed',relative)
report=dict(status='pass',revision='research-finish-v12',research_variants=len(checks),approved_face_layers_preserved=72,non_research_frames_preserved=len(retained),authorized_mobile_frames_superseded=len(superseded),male_towel_in_front_pass=0,resting_finger_landmarks_preserved=True,visual_acceptance='Requires visual review; these checks cover the reported pixel regressions')
(ROOT/'qa/research-finish-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
