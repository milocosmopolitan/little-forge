"""Read-only contact sheets of authored layers and desk composites."""
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[3];HERE=Path(__file__).resolve().parent
out=Image.new('RGB',(1536,1536),'#eee5cd');draw=ImageDraw.Draw(out)
for sex_row,sex in enumerate(['female','male']):
 for k in range(6):
  p=ROOT/sex/'previews/scenes'/f'researching-{k}.png'
  actor=Image.open(p).convert('RGB').resize((512,384),Image.Resampling.NEAREST)
  x=k%3*512;y=(sex_row*2+k//3)*384
  out.paste(actor,(x,y));draw.text((x+10,y+10),f'{sex} researching-{k}',fill='black')
out.save(ROOT/'qa/research-finish-v12-scenes.png')
proof=Image.new('RGB',(1536,1152),'#87bdc8');draw=ImageDraw.Draw(proof)
for row,(sex,k) in enumerate([('male',2),('female',4),('female',1)]):
 for col,name in enumerate(['composed.png','composed.png','body/arms-front.png','rendered/body/upper.png']):
  folder=HERE/'before'/sex/f'researching-{k}' if col==0 else ROOT/sex/'poses'/f'researching-{k}'
  src=Image.open(folder/name).convert('RGBA').crop((35,58,91,114)).resize((336,336),Image.Resampling.NEAREST)
  x=col*384;y=row*384;proof.paste(src,(x,y+32),src);draw.text((x+4,y+8),f'{sex} {k}: '+('before' if col==0 else 'after' if col==1 else name),fill='black')
proof.save(ROOT/'qa/research-finish-v12-layers.png')
all_keys=Image.new('RGB',(1536,1280),'#87bdc8');draw=ImageDraw.Draw(all_keys)
for row,(sex,mode) in enumerate([('female','base/'),('female',''),('male','base/'),('male','')]):
 for k in range(6):
  actor=Image.open(ROOT/sex/'frames'/mode/f'researching-{k}.png').convert('RGBA').resize((256,320),Image.Resampling.NEAREST)
  all_keys.paste(actor,(k*256,row*320),actor)
  draw.text((k*256+4,row*320+4),f'{sex} '+('base' if mode else 'equipped')+f' {k}',fill='black')
all_keys.save(ROOT/'qa/research-finish-v12-all-24.png')
