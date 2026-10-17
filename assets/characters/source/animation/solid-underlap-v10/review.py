"""Render native pixels on high-contrast backgrounds for seam inspection."""
import json
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[3];SOURCE=Path(__file__).resolve().parent
animation=json.loads((SOURCE/'before-animation.json').read_text())
ids=[k for k,p in animation['poses'].items() if p.get('head_identity',{}).get('root_pixels_unchanged')]
def tile(path,box=(35,42,100,88),scale=4):
 im=Image.open(path).convert('RGBA').crop(box);bg=Image.new('RGBA',im.size,'#79b7ca');bg.alpha_composite(im);return bg.convert('RGB').resize((im.width*scale,im.height*scale),Image.Resampling.NEAREST)
canvas=Image.new('RGB',(6*268,8*208),'#202126');draw=ImageDraw.Draw(canvas)
for row,variant in enumerate(['base','equipped']):
 for i,id in enumerate(ids):
  x=i%6*268;y=(row*4+i//6)*208
  pose=animation['poses'][id];canvas.paste(tile(ROOT/'female'/pose['composed' if variant=='equipped' else 'base']),(x,y+22));draw.text((x+4,y+4),variant+' '+id,fill='white')
canvas.save(ROOT/'qa/solid-underlap-v10-all-48.png')
selected=['quenching-2','forging-0','forging-1','forging-2','forging-4','researching-2']
canvas=Image.new('RGB',(6*268,2*208),'#202126');draw=ImageDraw.Draw(canvas)
for i,id in enumerate(selected):
 for row in range(2):
  file=SOURCE/'before/equipped'/(id+'.png') if row==0 else ROOT/'female'/animation['poses'][id]['composed']
  canvas.paste(tile(file),(i*268,row*208+22));draw.text((i*268+4,row*208+4),('before ' if row==0 else 'after ')+id,fill='white')
canvas.save(ROOT/'qa/solid-underlap-v10-before-after.png')
canvas=Image.new('RGB',(6*268,4*240),'#202126');draw=ImageDraw.Draw(canvas)
for i,id in enumerate(ids):
 x=i%6*268;y=i//6*240
 file=ROOT/'female/poses'/id/'rendered/body/upper.png'
 canvas.paste(tile(file,box=(35,42,100,96)),(x,y+22));draw.text((x+4,y+4),id+' body only',fill='white')
canvas.save(ROOT/'qa/solid-underlap-v10-body-layers.png')
