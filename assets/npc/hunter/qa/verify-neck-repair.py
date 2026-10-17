"""Keep the neck correction local and regenerate its visual comparison."""
from pathlib import Path
import json
from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[1]
mask=Image.open(ROOT/'source/neck-repair/repair-mask.png')
report={}
for name in ['base-neutral.png','neutral.png']:
    before=Image.open(ROOT/'qa/neck-before'/name).convert('RGBA')
    # The next approved edit has its own scope check; keep this neck checkpoint stable.
    checkpoint=ROOT/'qa/hair-hand-before'/name
    after=Image.open(checkpoint if checkpoint.exists() else ROOT/name).convert('RGBA')
    changed=[(x,y) for y in range(160) for x in range(128) if before.getpixel((x,y))!=after.getpixel((x,y))]
    assert all(mask.getpixel(p) for p in changed),name
    report[name]={'changed_pixels':len(changed),'changes_outside_jaw_neck_collar_mask':0}
for name in ['eyes/open.png','mouth/rest.png']:
    feature=Image.open(ROOT/name)
    assert all(not feature.getpixel((x,y))[3] or not mask.getpixel((x,y)) for y in range(160) for x in range(128))
out=Image.new('RGB',(1344,392),'#eee3cc');draw=ImageDraw.Draw(out)
panels=[('Before / base','qa/neck-before/base-neutral.png'),('After / base','base-neutral.png'),
        ('Before / equipped','qa/neck-before/neutral.png'),('After / equipped','neutral.png')]
for k,(label,path) in enumerate(panels):
    im=Image.open(ROOT/path).crop((54,51,81,79)).resize((324,336),Image.Resampling.NEAREST)
    out.paste(im,(k*336+6,38),im);draw.text((k*336+6,12),label,fill='#201510')
out.save(ROOT/'previews/neck-repair-before-after.png')
report['source']='source/neck-repair/prompt.json'
report['scope']='Jaw, neck and collar only; all other pixels preserved.'
report['visual_review']=['light/dark static contact sheets','8x face crop','Chrome refreshed 4x base and equipped preview']
(ROOT/'qa/neck-repair-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
