"""Read-only close-ups of the independent body, head and final composite."""
from pathlib import Path
from PIL import Image,ImageDraw
R=Path(__file__).resolve().parents[1];Q=R/'qa';S=R/'source/animation/collar-v15'
keys=[*(f'walk-{k}' for k in range(6)),'give-half','give-full','give-release']
for sex in ['female','male']:
 for bg,name in [('#49b9d1','cyan'),('#22232b','dark'),('#eee7ce','light')]:
  o=Image.new('RGB',(9*320,3*304),bg);draw=ImageDraw.Draw(o)
  for k,key in enumerate(keys):
   for row,part in enumerate(['body/upper-base.png','head-base.png','composed.png']):
    a=Image.open(R/sex/'poses'/key/part).convert('RGBA').crop((44,53,84,88)).resize((320,280),Image.Resampling.NEAREST);o.paste(a,(k*320,row*304),a);draw.text((k*320+3,row*304+282),key+' '+['BODY','HEAD','EQUIPPED'][row],fill='#f0b65a' if name=='dark' else '#111111')
  o.save(Q/f'collar-v15-{sex}-{name}.png')
 o=Image.new('RGB',(9*320,2*304),'#49b9d1');draw=ImageDraw.Draw(o)
 for k,key in enumerate(keys):
  for row,p in enumerate([S/'before'/sex/key/'composed.png',R/sex/'poses'/key/'composed.png']):
   a=Image.open(p).convert('RGBA').crop((44,53,84,88)).resize((320,280),Image.Resampling.NEAREST);o.paste(a,(k*320,row*304),a);draw.text((k*320+3,row*304+282),key+(' BEFORE' if row==0 else ' AFTER'),fill='#111111')
 o.save(Q/f'collar-v15-{sex}-before-after.png')
print('Eight collar body/head/composite and before/after sheets written')
