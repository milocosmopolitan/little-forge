"""Contact sheets from existing authoring images, without mutating any asset."""
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[3];HERE=Path(__file__).resolve().parent
# Same native coordinates in all cells, before/after on both diagnostic backgrounds.
out=Image.new('RGB',(1536,1280));d=ImageDraw.Draw(out)
for row,(before,bg) in enumerate([(True,'#232329'),(False,'#232329'),(True,'#80bcc8'),(False,'#80bcc8')]):
 for col,k in enumerate([0,1,2,4]):
  x=col*384;y=row*320;d.rectangle((x,y,x+383,y+319),fill=bg)
  folder=HERE/'before'/f'researching-{k}' if before else ROOT/'female/poses'/f'researching-{k}'
  im=Image.open(folder/'composed.png').convert('RGBA').crop((40,52,86,87)).resize((368,280),Image.Resampling.NEAREST)
  out.paste(im,(x,y+32),im);d.text((x+4,y+8),f'key {k} '+('before' if before else 'after'),fill='white' if bg=='#232329' else 'black')
out.save(ROOT/'qa/research-material-v13-before-after.png')
all_keys=Image.new('RGB',(1536,1280),'#80bcc8');d=ImageDraw.Draw(all_keys)
for row,(base,bg) in enumerate([(False,'#232329'),(False,'#eee5cd'),(True,'#80bcc8'),(True,'#232329')]):
 for k in range(6):
  x=k*256;y=row*320;d.rectangle((x,y,x+255,y+319),fill=bg)
  p=ROOT/'female/frames'/('base' if base else '')/f'researching-{k}.png'
  im=Image.open(p).convert('RGBA').resize((256,320),Image.Resampling.NEAREST);all_keys.paste(im,(x,y),im);d.text((x+3,y+3),f'{k} '+('base' if base else 'equipped'),fill='white' if bg=='#232329' else 'black')
all_keys.save(ROOT/'qa/research-material-v13-all-keys.png')
