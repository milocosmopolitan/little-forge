"""Native walk/give ownership, hand contours, planted feet and preservation checks."""
import hashlib,json
from pathlib import Path
from PIL import Image
R=Path(__file__).resolve().parents[1];S=R/'source/animation/mobile-poses-v14'
def im(p):return Image.open(p).convert('RGBA')
def data(a):return a.tobytes()
def compose(folder,names):
 out=Image.new('RGBA',(128,160))
 for name in names:out.alpha_composite(im(folder/name))
 return out
checks=[];old_missing=0
for sex in ['female','male']:
 d=R/sex;a=json.loads((d/'animation.json').read_text());root_head=im(d/'head-base.png')
 for row,(lo,hi) in a['head_ownership']['removed_neck_collar_rows'].items():
  for x in range(lo,hi+1):root_head.putpixel((x,int(row)),(0,0,0,0))
 for key in [*(f'walk-{k}' for k in range(6)),'give-half','give-full','give-release']:
  p=a['poses'][key];b=d/'poses'/key;assert p['layer_finish']=='collar-v15'
  upper=im(b/'body/upper-base.png');lower=im(b/'body/lower-base.png');required=im(b/'body/required-coverage.png');gloves=im(b/'equipment/gloves.png');base=im(d/p['base']);equipped=im(d/p['composed'])
  old=im(S/'before'/sex/key/'body/upper-base.png')
  for y in range(160):
   for x in range(128):
    if required.getpixel((x,y))[3]:
     assert upper.getpixel((x,y))[3],(sex,key,'missing shoulder',x,y)
     old_missing+=not old.getpixel((x,y))[3]
  for prefix,identity in [('',p['motion_head_identity']),('rendered/',p['motion_equipped_head_identity'])]:
   dx,dy=identity['shift']
   for name,root in [('head-base.png',root_head),('eyes/open.png',im(d/'eyes/open.png')),('mouth/rest.png',im(d/'mouth/rest.png'))]:
    src=root.copy()
    if prefix and name=='head-base.png':src.alpha_composite(im(d/'equipment'/('hair-tie.png' if sex=='female' else 'headband.png')))
    expected=Image.new('RGBA',(128,160));expected.paste(src,(dx,dy))
    assert data(expected)==data(im(b/(prefix+name))),(sex,key,'approved face or hair changed',prefix,name)
  head=im(b/'head-base.png');assert any(head.getpixel((x,y))[3] and upper.getpixel((xx,yy))[3] for y in range(50,85) for x in range(30,105) for xx,yy in [(x,y),(x-1,y),(x+1,y),(x,y+1)]),(sex,key,'detached head')
  apron=im(b/'equipment/apron.png');body=compose(b,['body/lower-base.png','body/upper-base.png'])
  assert all(not apron.getpixel((x,y))[3] or body.getpixel((x,y))[3] for y in range(p['sockets']['waist'][1]) for x in range(128)),(sex,key,'floating apron edge')
  hands=p['sockets']['hands'];assert len(hands)==2 and {h['side'] for h in hands}=={'anatomical_left','anatomical_right'},(sex,key,'missing arm')
  hand_pixels={tuple(xy) for h in hands for xy in h['hand_pixels']}
  actual={(x,y) for y in range(160) for x in range(128) if gloves.getpixel((x,y))[3]}
  assert actual==hand_pixels and all(h['glove_pixels']>=10 for h in hands),(sex,key,'glove contour lost')
  for xy in hand_pixels:
   assert upper.getpixel(xy)[3] or lower.getpixel(xy)[3],(sex,key,'glove outside source hand')
  assert base.getbbox()[3]==141,(sex,key,'ground baseline')
  if key.startswith('give'):
   assert data(base.crop((0,117,128,160)))==data(im(d/'frames/base/neutral.png').crop((0,117,128,160))),(sex,key,'planted feet changed')
  checks.append(dict(sex=sex,pose=key,hand_pixels=len(hand_pixels),neck=p['sockets']['neck'],waist=p['sockets']['waist']))
 assert len({tuple(a['poses'][f'walk-{k}']['sockets']['waist'][:1]) for k in range(6)})==1,(sex,'pelvis horizontal drift')
 give=a['clips']['give'];assert give['events']==[{'frame':3,'event':'transfer'}] and give['frames'][0]['sprite']==give['frames'][-1]['sprite']=='neutral'
keep=json.loads((S/'retained-frames.json').read_text())
for name,digest in keep.items():assert hashlib.sha256((R/name).read_bytes()).hexdigest()==digest,('unrelated frame changed',name)
assert len(checks)==18 and old_missing>0
report=dict(status='pass',revision='collar-v15',unique_poses_checked=len(checks),variants_checked=36,unrelated_frames_preserved=len(keep),old_missing_coverage_pixels_detected=old_missing,approved_identity_preserved=True,source_hand_contours_preserved=True,give_feet_planted=True,checks=checks)
(R/'qa/mobile-poses-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='checks'}))
