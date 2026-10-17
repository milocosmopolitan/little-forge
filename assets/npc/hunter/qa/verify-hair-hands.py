"""Check crown restoration and the two requested one-pixel hand fills."""
from pathlib import Path
import json
from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[1]
mask=Image.open(ROOT/'source/hair-hand-repair/hair-mask.png')
holes=[(45,91),(61,92)]
report={}
for name in ['base-neutral.png','neutral.png']:
    before=Image.open(ROOT/'qa/hair-hand-before'/name).convert('RGBA')
    checkpoint=ROOT/'qa/hair-restyle-before'/name
    after=Image.open(checkpoint if checkpoint.exists() else ROOT/name).convert('RGBA')
    changed=[(x,y) for y in range(160) for x in range(128) if before.getpixel((x,y))!=after.getpixel((x,y))]
    assert all(mask.getpixel(p) or p in holes for p in changed),name
    report[name]={'changed_pixels':len(changed),'changes_outside_hair_and_two_hand_pixels':0}
    if name=='base-neutral.png':
        assert all(not before.getpixel(p)[3] and after.getpixel(p)[3]==255 for p in holes)
        report['crown_top_before_after']=[before.getbbox()[1],after.getbbox()[1]]
        assert after.getbbox()[1]<before.getbbox()[1]
for name in ['eyes/open.png','mouth/rest.png']:
    assert Image.open(ROOT/name).tobytes()==Image.open(ROOT/'qa/hair-hand-before'/name).tobytes(),name
report['neck_and_face_outside_crown']='pixel-identical to approved repair checkpoint'
report['hand_pixels_filled']=[list(p) for p in holes]
out=Image.new('RGB',(1088,400),'#eee3cc');d=ImageDraw.Draw(out)
for k,(label,path,box) in enumerate([
    ('Before / crown','qa/hair-hand-before/base-neutral.png',(36,18,93,51)),
    ('After / crown','base-neutral.png',(36,18,93,51)),
    ('Before / hands','qa/hair-hand-before/base-neutral.png',(39,83,64,99)),
    ('After / hands','base-neutral.png',(39,83,64,99))]):
    im=Image.open(ROOT/path).crop(box);scale=4 if k<2 else 8
    im=im.resize((im.width*scale,im.height*scale),Image.Resampling.NEAREST)
    x=(k%2)*544+8;y=(k//2)*200+34
    d.text((x,y-22),label,fill='#211510');out.paste(im,(x,y),im)
out.save(ROOT/'previews/hair-hands-before-after.png')
(ROOT/'qa/hair-hands-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
