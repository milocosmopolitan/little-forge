"""Inspect the body, head and composite separately against the rejected v10 output."""
import json
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[3];SOURCE=Path(__file__).resolve().parent
cases=[('female','forging-0'),('female','forging-4'),('female','forging-5'),('female','researching-0'),('female','researching-2'),('male','researching-0')]
def tile(file,box=(35,38,103,94),scale=4):
 im=Image.open(file).convert('RGBA').crop(box);b=Image.new('RGBA',im.size,'#79b7ca');b.alpha_composite(im);return b.convert('RGB').resize((im.width*scale,im.height*scale),Image.Resampling.NEAREST)
out=Image.new('RGB',(6*276,4*248),'#22242a');draw=ImageDraw.Draw(out)
for col,(sex,id)in enumerate(cases):
 p=ROOT/sex/'poses'/id
 files=[SOURCE/'before'/sex/'equipped'/(id+'.png'),p/'rendered/body/upper.png',p/'rendered/head-base.png',ROOT/sex/'frames'/(id+'.png')]
 for row,file in enumerate(files):
  x=col*276;y=row*248;out.paste(tile(file),(x,y+22));draw.text((x+2,y+3),sex+' '+id+' '+['v10 rejected','body only','head only','composite'][row],fill='white')
out.save(ROOT/'qa/joint-rebuild-v11-layers.png')
for sex in ['female','male']:
 a=json.loads((ROOT/sex/'animation.json').read_text());ids=[id for id,p in a['poses'].items()if p.get('head_identity',{}).get('root_pixels_unchanged')]
 out=Image.new('RGB',(6*229,8*228),'#22242a');draw=ImageDraw.Draw(out)
 for v,variant in enumerate(['base','equipped']):
  for i,id in enumerate(ids):
   file=ROOT/sex/a['poses'][id]['composed' if variant=='equipped' else 'base'];x=i%6*229;y=(v*4+i//6)*228
   out.paste(tile(file,(35,35,110,103),3),(x,y+22));draw.text((x+2,y+2),variant+' '+id,fill='white')
 out.save(ROOT/('qa/joint-rebuild-v11-'+sex+'-48.png'))

for sex in ['female','male']:
 a=json.loads((ROOT/sex/'animation.json').read_text());ids=[id for id,p in a['poses'].items()if p.get('head_identity',{}).get('root_pixels_unchanged')]
 out=Image.new('RGB',(6*229,8*192),'#22242a');draw=ImageDraw.Draw(out)
 for v,equipped in enumerate([False,True]):
  for i,id in enumerate(ids):
   file=ROOT/sex/'poses'/id/('rendered/body/upper.png' if equipped else 'body/upper-base.png');x=i%6*229;y=(v*4+i//6)*192
   out.paste(tile(file,(35,45,110,100),3),(x,y+22));draw.text((x+2,y+2),('equipped ' if equipped else 'base ')+id,fill='white')
 out.save(ROOT/('qa/joint-rebuild-v11-'+sex+'-body-only.png'))
