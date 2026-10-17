"""Read final layers and the preceding native review sheet; create nearest proof images."""
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[3]
CASES=['hammering-2','hammering-3','hammering-4','quenching-0','quenching-1','quenching-2','quenching-5']
ACTIONS=['hammering','quenching','forging','researching','salvaging']
def panel(im,label):
 bg=Image.new('RGBA',(128,160),'#eee5cd');bg.alpha_composite(im)
 view=bg.crop((30,43,102,91)).resize((288,192),Image.Resampling.NEAREST).convert('RGB')
 out=Image.new('RGB',(296,216),'#eee5cd');out.paste(view,(4,20));ImageDraw.Draw(out).text((4,4),label,fill='black');return out
before_after=Image.new('RGB',(1184,216*len(CASES)),'#eee5cd')
layers=Image.new('RGB',(1184,216*len(CASES)*2),'#eee5cd')
for row,key in enumerate(CASES):
 action,index=key.rsplit('-',1);index=int(index)
 for si,sex in enumerate(['female','male']):
  directory=ROOT/sex;pose=directory/'poses'/key
  sheet=Image.open(ROOT/'source/animation/layer-ownership'/f'{sex}-before.png').convert('RGBA')
  x=index*132+2;y=ACTIONS.index(action)*164+2
  old=sheet.crop((x,y,x+128,y+160));new=Image.open(directory/'frames'/f'{key}.png').convert('RGBA')
  for col,(im,label) in enumerate([(old,f'{sex} {key} BEFORE'),(new,f'{sex} {key} AFTER')]):before_after.paste(panel(im,label),((si*2+col)*296,row*216))
  for col,(file,label) in enumerate([('rendered/head-base.png','head only'),('rendered/body/upper.png','upper only'),('rendered/body/neck-skin.png','body neck only'),('composed.png','composed')]):
   im=Image.open(pose/file).convert('RGBA');layers.paste(panel(im,f'{sex} {key} {label}'),(col*296,(row*2+si)*216))
before_after.save(ROOT/'qa/head-ownership-before-after.png');layers.save(ROOT/'qa/head-ownership-layers.png')

for suffix,bg in [('dark','#22252b'),('light','#eee5cd')]:
 out=Image.new('RGB',(1608,1664),bg);draw=ImageDraw.Draw(out)
 for si,sex in enumerate(['female','male']):
  for ai,action in enumerate(ACTIONS[:4]):
   for index in range(6):
    im=Image.open(ROOT/sex/'frames'/f'{action}-{index}.png').convert('RGBA').crop((35,42,100,88))
    tile=Image.new('RGBA',im.size,bg);tile.alpha_composite(im);tile=tile.resize((260,184),Image.Resampling.NEAREST).convert('RGB')
    x=index*268;y=(si*4+ai)*208;out.paste(tile,(x+4,y+20));draw.text((x+4,y+3),f'{sex} {action} {index}',fill='white' if suffix=='dark' else 'black')
 out.save(ROOT/'qa'/f'neck-all-48-{suffix}.png')
