"""Regression checks for the reported hunter ownership and registration failures."""
from pathlib import Path
from PIL import Image, ImageChops
import json

ROOT=Path(__file__).resolve().parents[1]
def read(path):return Image.open(ROOT/path).convert('RGBA')
def skin(q):return q[3] and q[0]>185 and 85<q[1]<210 and q[2]<170 and q[0]>q[1]*1.2
def compose(images):
    out=Image.new('RGBA',(128,160))
    for im in images:out.alpha_composite(im)
    return out

recipes=json.loads((ROOT/'poses/recipes.json').read_text())['poses']
channels=json.loads((ROOT/'expressions/recipes.json').read_text())
coverage=read('source/animation/repair/eye-coverage.png').getchannel('A')
neutral=read('base-neutral.png')
for state,path in channels['eyes'].items():
    assert ImageChops.subtract(read(path).getchannel('A'),coverage).getbbox() is None,state
for path in (ROOT/'previews/combinations').glob('*.png'):
    im=Image.open(path).convert('RGBA')
    assert im.getchannel('A').tobytes()==neutral.getchannel('A').tobytes(),path
    for y in range(45,54):
        for x in range(77,81):assert im.getpixel((x,y))==neutral.getpixel((x,y)),(path,x,y,'fixed hair')
    for p in [(73,52),(73,53)]:assert im.getpixel(p)==neutral.getpixel(p),(path,'nose',p)

# This assertion catches the rejected blink's actual changed hair pixels.
before=Image.open(ROOT/'qa/animation-repair-before/faces-8.png').convert('RGB')
def old_face_pixel(col,row,x,y):
    return before.getpixel((col*232+8+(x-57)*8+4,row*220+28+(y-39)*8+4))
assert any(old_face_pixel(0,1,x,y)!=old_face_pixel(0,0,x,y) for y in range(46,54) for x in range(77,81))

fixed=read('source/animation/fixed-body.png')
head_metrics={}
for name in ['nod-down','turn-left','turn-right']:
    r=recipes[name]
    head=compose([read(r['layers'][p]) for p in ['head','eyes','mouth']])
    source=read(f'source/animation/repair/{name}-motion-head.png')
    assert head.tobytes()==source.tobytes(),(name,'whole authored jaw retained')
    body=compose([read(r['layers'][p]) for p in ['lower','upper']])
    for y in range(60,112):
        for x in range(38,91):
            q=fixed.getpixel((x,y))
            if q[3] and not skin(q):assert body.getpixel((x,y))==q,(name,x,y,'fixed collar/shoulder')
    l,t,right,bottom=head.getbbox();assert 40<=right-l<=47,(name,'head width')
    face=[(x,y) for y in range(40,63) for x in range(59,81) if skin(head.getpixel((x,y)))]
    head_metrics[name]={'head_bbox':list(head.getbbox()),'face_skin_pixels':len(face)}
    if name.startswith('turn-'):assert len(face)>=170,(name,'shrunken face')
for x in range(66,73):assert read('poses/nod-down/layers/head.png').getpixel((x,60))[3],('jaw',x)

foreground_pixels=0
for name,r in recipes.items():
    if not(name.startswith('give-') or name.startswith('walk-')):continue
    base=read(f'frames/base/{name}.png');front=read(r['layers']['hands-front'])
    for y in range(160):
        for x in range(128):
            q=front.getpixel((x,y))
            if q[3]:
                foreground_pixels+=1
                assert base.getpixel((x,y))==q,(name,x,y,'stale arm/hand')

gait=json.loads((ROOT/'source/animation/walk-phases.json').read_text())
hands=[p['anatomical_contacts']['right_hand'][0] for p in gait]
assert hands[0]>hands[1]>hands[2]>hands[3]
assert hands[3]<hands[4]<hands[5]<hands[0]
for k in [0,1,3,4]:
    p=gait[k]['anatomical_contacts'];assert (p['right_hand'][0]-65)*(p['right_boot'][0]-65)<0,(k,'same-side phase')
prompts=json.loads((ROOT/'source/animation/repair/prompts.json').read_text())
assert len(prompts)==10 and all(p.get('prompt') for p in prompts.values())
report={'status':'pass','expression_combinations':20,'feature_pixels_outside_eye_coverage':0,'neutral_alpha_and_hair_preserved':True,'nose_preserved':True,'rejected_blink_detected':True,'whole_generated_jaw_retained':True,'head_metrics':head_metrics,'stale_foreground_pixels':0,'foreground_pixels_checked':foreground_pixels,'near_hand_progresses_through_both_half_cycles':True,'contact_and_support_same_side_opposition':True,'scope':'structural regression evidence; visual playback reviewed separately'}
(ROOT/'qa/repair-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
