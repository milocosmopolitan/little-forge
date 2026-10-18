"""Read-only frame and ownership sheets for walk/give authoring review."""
from pathlib import Path
from PIL import Image,ImageDraw
R=Path(__file__).resolve().parents[1];S=R/'source/animation/mobile-poses-v14';Q=R/'qa'
keys=[*(f'walk-{k}' for k in range(6)),'give-half','give-full','give-release']
for sex in ['female','male']:
 for variant in ['base','composed']:
  sheet=Image.new('RGB',(9*256,2*348),'#49b9d1');draw=ImageDraw.Draw(sheet)
  for k,key in enumerate(keys):
   for row,p in enumerate([S/'before'/sex/key/(variant+'.png'),R/sex/'poses'/key/(variant+'.png')]):
    a=Image.open(p).convert('RGBA').resize((256,320),Image.Resampling.NEAREST);sheet.paste(a,(k*256,row*348),a);draw.text((k*256+4,row*348+323),('BEFORE ' if row==0 else 'AFTER ')+sex+' '+key,fill='#151515')
  sheet.save(Q/f'mobile-poses-v14-{sex}-{variant}-before-after.png')
 # Dark/light all-key sheets keep transparent holes visible on both backgrounds.
 for background,name in [('#22232b','dark'),('#eee7ce','light')]:
  sheet=Image.new('RGB',(9*256,2*348),background);draw=ImageDraw.Draw(sheet)
  for row,variant in enumerate(['base','composed']):
   for k,key in enumerate(keys):
    a=Image.open(R/sex/'poses'/key/(variant+'.png')).convert('RGBA').resize((256,320),Image.Resampling.NEAREST);sheet.paste(a,(k*256,row*348),a);draw.text((k*256+4,row*348+323),variant+' '+key,fill='#f0b65a' if name=='dark' else '#252525')
  sheet.save(Q/f'mobile-poses-v14-{sex}-{name}.png')
print('Eight native before/after and dark/light review sheets written')
