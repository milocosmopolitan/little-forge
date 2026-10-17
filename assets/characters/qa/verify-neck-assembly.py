"""Read-only regression checks for cut shoulders and a second jaw line in the neck."""
import json
from pathlib import Path
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
def image(path): return Image.open(path).convert('RGBA')
def fabric(p):
 return p[3] and p[0]>170 and p[1]>150 and p[2]>125 and p[0]-p[1]<32 and p[1]-p[2]<38

def shirt_points(raw,collar_y):
 candidates={(x,y) for y in range(46,112) for x in range(128) if fabric(raw.getpixel((x,y)))}
 points=set()
 while candidates:
  group=[candidates.pop()]
  for x,y in group:
   for dx in [-1,0,1]:
    for dy in [-1,0,1]:
     point=(x+dx,y+dy)
     if point in candidates:candidates.remove(point);group.append(point)
  if len(group)>=4 and any(y>=collar_y-3 for x,y in group):points.update(group)
 return points

report=[];relocated=0
for sex in ['female','male']:
 directory=ROOT/sex
 animation=json.loads((directory/'animation.json').read_text())
 for name,pose in animation['poses'].items():
  if not pose.get('head_identity',{}).get('root_pixels_unchanged'): continue
  for equipped in [False,True]:
   folder=directory/'poses'/name
   raw=image(folder/('source/equipped-before-head-restoration.png' if equipped else 'source/body-before-head-restoration.png'))
   upper=image(folder/('rendered/body/upper.png' if equipped else 'body/upper-base.png'))
   lower=image(folder/('rendered/body/lower.png' if equipped else 'body/lower-base.png'))
   head=image(folder/('rendered/head-base.png' if equipped else 'head-base.png'))
   bridge=image(folder/('rendered/body/neck-skin.png' if equipped else 'body/neck-skin.png'))
   final=image(directory/pose['composed' if equipped else 'base'])
   removed_path=folder/('rendered/body/removed-research-prop.png' if equipped else 'body/removed-research-prop.png')
   removed=image(removed_path) if removed_path.exists() else Image.new('RGBA',(128,160))
   material=Image.new('RGBA',(128,160))
   if sex=='female' and name.startswith('researching-') and equipped and pose.get('layer_finish')=='research-material-v13':
    material=Image.alpha_composite(image(folder/'rendered/body/restored-straps.png'),image(folder/'rendered/body/restored-collar.png'))
   cloth_pixels=0;neck_pixels=0;shirt=shirt_points(raw,pose['rendered_head_identity' if equipped else 'head_identity']['source_collar'][1])
   for y in range(160):
    for x in range(128):
     source=raw.getpixel((x,y))
     if removed.getpixel((x,y))[3]:
      assert sex=='male' and name.startswith('researching-') and equipped and 38<=x<54 and 93<=y<110
      assert removed.getpixel((x,y))==source and not upper.getpixel((x,y))[3] and not lower.getpixel((x,y))[3]
      continue
     if (x,y) in shirt:
      if material.getpixel((x,y))[3]:
       assert 55<=x<75 and 63<=y<82 and upper.getpixel((x,y))[3]==255,(sex,name,'invalid material relocation',x,y)
       relocated+=1
       continue
      assert upper.getpixel((x,y))==source or lower.getpixel((x,y))==source,(sex,name,equipped,'source shirt cut',x,y)
      cloth_pixels+=1
     n=bridge.getpixel((x,y))
     if n[3] and not head.getpixel((x,y))[3]:
      assert final.getpixel((x,y))==n,(sex,name,equipped,'old contour overwrites neck skin',x,y)
      neck_pixels+=1
   # A bent head may naturally cover the neck; do not require extra exposed skin.
   report.append({'sex':sex,'pose':name,'equipped':equipped,'source_fabric_pixels_preserved':cloth_pixels,'exposed_neck_skin_pixels':neck_pixels})
output={'status':'pass','variants_checked':len(report),'source_fabric_loss':0,'intentional_material_relocations':relocated,'neck_skin_overwritten_by_old_contour':0,'variants':report,'visual_acceptance':'separate from these pixel checks'}
(ROOT/'qa/neck-assembly-validation.json').write_text(json.dumps(output,indent=2)+'\n')
print(json.dumps({k:v for k,v in output.items() if k!='variants'}))
