"""Build nearest review proofs from current assets and the rejected v8 snapshots."""
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[3]
HERE=Path(__file__).resolve().parent
KEYS=['quenching-2','quenching-4','forging-0','forging-4','researching-0','researching-1','researching-2','researching-4']
def panel(im,label):
 bg=Image.new('RGBA',(128,160),'#eee5cd');bg.alpha_composite(im)
 crop=bg.crop((30,43,102,91)).resize((288,192),Image.Resampling.NEAREST).convert('RGB')
 out=Image.new('RGB',(296,216),'#eee5cd');out.paste(crop,(4,20));ImageDraw.Draw(out).text((4,4),label,fill='black');return out
out=Image.new('RGB',(1184,len(KEYS)*216),'#eee5cd')
for row,key in enumerate(KEYS):
 for si,sex in enumerate(['female','male']):
  old=Image.open(HERE/'before'/sex/key/'frame.png').convert('RGBA');new=Image.open(ROOT/sex/'frames'/f'{key}.png').convert('RGBA')
  for ci,(im,label) in enumerate([(old,f'{sex} {key} BEFORE'),(new,f'{sex} {key} AFTER')]):out.paste(panel(im,label),((si*2+ci)*296,row*216))
out.save(ROOT/'qa/neck-cloth-v9-before-after.png')
out=Image.new('RGB',(1040,816),'#eee5cd');draw=ImageDraw.Draw(out)
for si,sex in enumerate(['female','male']):
 for ci,(file,label) in enumerate([(HERE/'before'/sex/'quenching-2/scene.png',f'{sex} BEFORE'),(ROOT/sex/'previews/scenes/quenching-2.png',f'{sex} AFTER')]):
  im=Image.open(file).convert('RGB').resize((512,384),Image.Resampling.NEAREST);x=ci*520;y=si*408;out.paste(im,(x+4,y+20));draw.text((x+4,y+4),label,fill='black')
out.save(ROOT/'qa/quenching-water-before-after.png')
