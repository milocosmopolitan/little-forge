"""Check real source neck/collar ownership, independently of the rendered head."""
import hashlib,json
from pathlib import Path
from PIL import Image
R=Path(__file__).resolve().parents[1];S=R/'source/animation/collar-v15'
def im(p):return Image.open(p).convert('RGBA')
results=[];rejected_pixels=0
for sex in ['female','male']:
 a=json.loads((R/sex/'animation.json').read_text())
 for key in [*(f'walk-{k}' for k in range(6)),'give-half','give-full','give-release']:
  p=a['poses'][key];assert p['layer_finish']=='collar-v15'
  b=R/sex/'poses'/key;raw=im(b/'source/complete-pose.png');body=im(b/'body/upper-base.png');neck=im(b/'body/neck-skin.png');required=im(b/'body/required-coverage.png');old=im(S/'before'/sex/key/'body/upper-base.png')
  joint=p['motion_assembly']['source_collar'];shift=p['motion_head_identity']['shift'];top=joint['jaw_exit']
  assert shift[1]==top-joint['root_exit'][1],(sex,key,'head offset does not follow actual jaw exit')
  assert p['motion_head_identity']['anatomical_head_adjustment']==[0,0],(sex,key,'ad hoc head adjustment')
  count=0;changed=0
  for y in range(160):
   for x in range(128):
    if required.getpixel((x,y))[3]:assert body.getpixel((x,y))==raw.getpixel((x,y)),(sex,key,'source collar damaged',x,y)
    if neck.getpixel((x,y))[3]:
     assert neck.getpixel((x,y))==raw.getpixel((x,y))==body.getpixel((x,y)),(sex,key,'fabricated or missing neck pixel',x,y)
     count+=1;changed+=old.getpixel((x,y))!=raw.getpixel((x,y))
  # Every authored neck row must retain its native pigments and contour.
  for y,span in joint['neck_rows'].items():
   assert any(body.getpixel((x,int(y)))[3] for x in range(span[0],span[1]+1)),(sex,key,'broken neck row',y)
  assert count>0
  rejected_pixels+=changed;results.append(dict(sex=sex,pose=key,source_neck_pixels=count,v14_source_mismatches=changed))
keep=json.loads((S/'retained-frames.json').read_text())
for name,digest in keep.items():assert hashlib.sha256((R/name).read_bytes()).hexdigest()==digest,('unrelated frame changed',name)
assert len(results)==18 and rejected_pixels>0
report=dict(status='pass',revision='collar-v15',poses_checked=18,source_neck_and_garment_pixels_preserved=True,fabricated_neck_pixels=0,v14_source_neck_mismatch_pixels=rejected_pixels,unrelated_frame_pngs_preserved=len(keep),poses=results)
(R/'qa/source-collar-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='poses'}))
