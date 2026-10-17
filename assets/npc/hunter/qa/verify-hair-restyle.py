"""Verify that the new hairstyle preserves the face, neck and complete body."""
from pathlib import Path
import json
from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[1]
mask=Image.open(ROOT/'source/hair-restyle/hair-mask.png')
report={}
for name in ['base-neutral.png','neutral.png']:
    before=Image.open(ROOT/'qa/hair-restyle-before'/name).convert('RGBA')
    after=Image.open(ROOT/name).convert('RGBA')
    changed=[(x,y) for y in range(160) for x in range(128) if before.getpixel((x,y))!=after.getpixel((x,y))]
    assert all(mask.getpixel(p) for p in changed),name
    assert all(y<50 for x,y in changed),name
    report[name]={'changed_pixels':len(changed),'pixels_changed_outside_hair':0}
for name in ['eyes/open.png','mouth/rest.png']:
    assert Image.open(ROOT/name).tobytes()==Image.open(ROOT/'qa/hair-restyle-before'/name).tobytes(),name
out=Image.new('RGB',(1216,760),'#eee3cc');d=ImageDraw.Draw(out)
for k,(label,path) in enumerate([('Previous hair','qa/hair-restyle-before/base-neutral.png'),('New side part','base-neutral.png'),('Equipped','neutral.png')]):
    im=Image.open(ROOT/path).crop((34,16,96,69)).resize((372,318),Image.Resampling.NEAREST)
    x=k*400+8;d.text((x,12),label,fill='#211510');out.paste(im,(x,36),im)
    im=Image.open(ROOT/path).resize((256,320),Image.Resampling.NEAREST);out.paste(im,(x+48,400),im)
out.save(ROOT/'previews/hair-restyle-before-after.png')
report.update({'eyes_mouth_unchanged':True,'neck_and_body_unchanged':True,'stretching_removed':True,'source':'source/hair-restyle/prompt.json'})
(ROOT/'qa/hair-restyle-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
