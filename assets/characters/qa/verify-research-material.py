"""Regression checks for missing leather straps and retained source-head seams."""
import hashlib,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];SOURCE=ROOT/'source/animation/research-material-v13'
def im(p):return Image.open(p).convert('RGBA')
def skin(p):return p[3] and p[0]>185 and p[1]>105 and p[2]>55 and p[0]>p[1]*1.22
def leather(p):return p[3] and not skin(p) and p[0]<180 and p[1]<140 and p[2]<110
# Native source columns measured from the four distinct generated front-body poses.
guides={0:[(55,59,66,70,78),(69,73,66,70,78)],1:[(55,59,66,71,79),(70,74,66,71,79)],2:[(55,59,71,75,82),(71,75,71,75,82)],4:[(55,59,64,69,78)]}
root_head=im(ROOT/'female/head-base.png');tones={root_head.getpixel((63,63)),root_head.getpixel((64,63))}
straps=0;rejected_gaps=0
for k,guide in guides.items():
 p=ROOT/'female/poses'/f'researching-{k}';old=SOURCE/'before'/f'researching-{k}'
 upper=im(p/'rendered/body/upper.png');before=im(old/'rendered/body/upper.png');raw=im(p/'source/equipped-before-head-restoration.png');neck=im(p/'rendered/body/neck-skin.png')
 for x0,x1,top,source_y,bottom in guide:
  previous=set()
  for y in range(top,bottom):
   row={x for x in range(x0,x1) if leather(upper.getpixel((x,y)))}
   assert row,(k,'white gap in shoulder strap',y)
   assert not previous or any(abs(x-a)<=1 for x in row for a in previous),(k,'disconnected leather strap',y)
   previous=row
   if not any(leather(before.getpixel((x,y))) for x in range(x0,x1)):rejected_gaps+=1
   for x in range(x0,x1):assert upper.getpixel((x,y))==raw.getpixel((x,max(y,source_y))),(k,'source strap pigment or buckle lost',x,y)
  straps+=1
 # The neck uses the approved identity's two pigments; the thinking hand is separate.
 cx,cy=json.loads((ROOT/'female/animation.json').read_text())['poses'][f'researching-{k}']['neck_assembly']['body_owned_mount']['equipped_point']
 for y in range(cy-3,cy+3):
  for x in range(cx-4,cx+4):
   if k==4 and x>=cx:continue
   q=neck.getpixel((x,y))
   if q[3]:assert q in tones,(k,'old source-face stripe in neck',x,y,q)
 if k==4:
  for xy in [(51,66),(52,66),(53,66),(54,66),(52,67),(53,67)]:assert not upper.getpixel(xy)[3],('old hair seam remains',xy)
assert straps==7 and rejected_gaps>0
for repeated,original in [(3,1),(5,0)]:
 assert (ROOT/f'female/frames/researching-{repeated}.png').read_bytes()==(ROOT/f'female/frames/researching-{original}.png').read_bytes(),('repeat key diverged',repeated)
for k in range(6):
 p=ROOT/'female/poses'/f'researching-{k}';old=SOURCE/'before'/f'researching-{k}'
 for name in ['head-base.png','eyes/open.png','mouth/rest.png','rendered/head-base.png','rendered/eyes/open.png','rendered/mouth/rest.png','base.png','body/lower-base.png','rendered/body/lower.png']:
  assert (p/name).read_bytes()==(old/name).read_bytes(),('unrelated female pixels changed',k,name)
keep=json.loads((SOURCE/'retained-frames.json').read_text())
for relative,digest in keep.items():assert hashlib.sha256((ROOT/relative).read_bytes()).hexdigest()==digest,('unrelated frame changed',relative)
report=dict(status='pass',revision='research-material-v13',unique_material_straps=7,rejected_v12_strap_gap_rows=rejected_gaps,old_hair_seam_pixels=0,female_face_layers_preserved=36,other_frames_preserved=len(keep),source_buckle_and_leather_pigments_preserved=True,visual_acceptance='Not established by pixel assertions')
(ROOT/'qa/research-material-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
