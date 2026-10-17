"""Validate locked neutral, pose layers, expression masks, timing and packed pixels."""
from pathlib import Path
from PIL import Image,ImageDraw
import json,hashlib
ROOT=Path(__file__).resolve().parent.parent

def load(path):return Image.open(ROOT/path).convert('RGBA')
def pixels(im):return bytes(v for p in im.getdata() for v in (p if p[3] else (0,0,0,0)))
def compose(layers):
    im=Image.new('RGBA',(128,160))
    for layer in layers:im.alpha_composite(layer)
    return im

def verify():
    animation=json.loads((ROOT/'animation.json').read_text());poses=json.loads((ROOT/'poses/recipes.json').read_text())['poses']
    expressions=json.loads((ROOT/'expressions/recipes.json').read_text());manifest=json.loads((ROOT/'atlas-manifest.json').read_text())
    locks=json.loads((ROOT/'source/animation/baseline-lock.json').read_text())
    for p,digest in locks.items():assert hashlib.sha256(load(p).tobytes()).hexdigest()==digest,f'Approved neutral changed: {p}'
    native=set();colors=set()
    for n in animation['frames']:
        recipe=poses[n];paths=dict(recipe['layers'],**recipe['equipment']);native.update(paths.values())
        layers={k:load(p) for k,p in paths.items()};base=compose([layers[k] for k in recipe['base_order']]);full=compose([layers[k] for k in recipe['composition_order']])
        assert pixels(base)==pixels(load(f'frames/base/{n}.png'))==pixels(load(f'poses/{n}/base.png')),n
        assert pixels(full)==pixels(load(f'frames/{n}.png'))==pixels(load(f'poses/{n}/composed.png')),n
        assert not any(a[3] and not b[3] for a,b in zip(base.getdata(),full.getdata())),f'Base loss: {n}'
        assert base.getchannel('A').getbbox()[3]==140,f'Foot baseline: {n}'
        assert load(f'frames/{n}.png').resize((512,640),Image.Resampling.NEAREST).tobytes()==load(f'previews/frames/{n}-4x.png').tobytes(),n
        native.update([f'frames/{n}.png',f'frames/base/{n}.png'])
    for p in native:
        im=Image.open(ROOT/p);assert im.mode=='RGBA' and im.size==(128,160),p
        assert set(im.getchannel('A').getdata())<={0,255},p
        colors.update(px[:3] for px in im.getdata() if px[3])
    for section in ['eyes','mouth']:
        ims=[load(p) for p in expressions[section].values()]
        assert len(set(pixels(im) for im in ims))==len(ims),section
        for im in ims:
            assert im.size==(96,128) and set(im.getchannel('A').getdata())<={0,255}
            colors.update(p[:3] for p in im.getdata() if p[3])
    assert len(colors)<=32,len(colors)
    neutral=load('frames/neutral.png');original=Image.new('RGBA',(128,160));original.alpha_composite(load('neutral.png'),(16,16));assert pixels(neutral)==pixels(original)
    masks=[load(f'source/animation/expression-{s}-mask.png') for s in ['eyes','mouth']]
    allowed={(x+16,y+16) for y in range(128) for x in range(96) if any(m.getpixel((x,y))[3] for m in masks)}
    for n in expressions['recipes']:
        im=load(f'frames/{n}.png');assert im.getchannel('A').tobytes()==neutral.getchannel('A').tobytes(),n
        assert all(im.getpixel((x,y))==neutral.getpixel((x,y)) for y in range(160) for x in range(128) if (x,y) not in allowed),n
        for x,y in [(61,35),(60,36),(64,34)]:
            assert im.getpixel((x+16,y+16))==neutral.getpixel((x+16,y+16)),f'Expression removed nose: {n}'
    closed=load('frames/base/blink.png')
    for y in range(28,35):
        for x in range(47,54):
            r,g,b,a=closed.getpixel((x+16,y+16))
            assert not (a and r>=230 and g>=200 and b>=170),'Eye white survived a closed lid'
    collar=[(55,59),(56,59),(57,60),(58,61),(59,62),(60,63),(61,64),
            (69,62),(70,63),(71,64),(72,65)]
    base_neutral=load('frames/base/neutral.png')
    for n in ['nod-mid','nod-down','turn-left','turn-right']:
        visible=load(f'frames/base/{n}.png')
        foreground=load(poses[n]['layers']['head'])
        if n in ['turn-left','turn-right']:
            assert foreground.getchannel('A').getbbox()[3]==61,f'Turned jaw lifted away from neck: {n}'
        for point in collar:
            # A turned chin can occlude the top of the fixed collar. Exposed cloth stays exact.
            if not foreground.getpixel(point)[3]:
                assert visible.getpixel(point)==base_neutral.getpixel(point),f'Moving exposed collar pixel: {n}/{point}'
        for y in range(59,64):
            skin=sum((p:=visible.getpixel((x,y)))[3]>0 and p[0]-p[1]>=60 and p[1]-p[2]>=45 for x in range(60,70))
            assert skin>=2,f'Dark stump or gap interrupts the neck: {n}/{y}'
        for part in ['upper','lower']:assert pixels(load(poses[n]['layers'][part]))==pixels(load(poses['neutral']['layers'][part])),f'Head gesture changed clothes: {n}'
        assert poses[n]['equipment'].keys()==poses['neutral']['equipment'].keys()
        for part in poses['neutral']['equipment']:
            assert pixels(load(poses[n]['equipment'][part]))==pixels(load(poses['neutral']['equipment'][part])),f'Head gesture changed gear: {n}/{part}'
    for name,clip in animation['clips'].items():
        assert all(k['sprite'] in animation['frames'] and isinstance(k['duration_ms'],int) and k['duration_ms']>0 for k in clip['frames']),name
    assert len(animation['clips']['nod']['frames'])==len(animation['clips']['shake_head']['frames'])==5
    give=animation['clips']['give']['frames'];assert len(give)==6
    assert sum(k.get('event')=='transfer' for k in give)==1 and give[3].get('event')=='transfer'
    assert all('mallet' not in poses[k['sprite']]['equipment'] for k in give[3:])
    assert all('mallet' in poses[k['sprite']]['equipment'] for k in give[:3])
    assert pixels(load('frames/base/give-full.png'))==pixels(load('frames/base/give-empty.png'))
    assert pixels(load('frames/base/give-mid.png'))==pixels(load('frames/base/give-retract.png'))
    hand_mask=Image.new('L',(128,160))
    ImageDraw.Draw(hand_mask).polygon([(91,91),(93,91),(94,92),(95,92),(96,93),(96,96),
        (95,97),(95,99),(93,100),(91,99),(90,97),(89,95)],fill=255)
    upper=load(poses['neutral']['layers']['upper']);original_tool=load(poses['neutral']['equipment']['mallet'])
    for name,dx,dy in [('give-mid',7,-5),('give-retract',7,-5),('give-full',17,-10),('give-empty',17,-10)]:
        im=load(f'frames/base/{name}.png')
        for y in range(91,101):
            for x in range(89,97):
                p=upper.getpixel((x,y))
                if x>=92 and hand_mask.getpixel((x,y)) and p[3]:
                    assert im.getpixel((x+dx,y+dy))==p,f'Approved gripping hand changed: {name}'
        if name in ['give-mid','give-full']:
            expected=Image.new('RGBA',(128,160));expected.alpha_composite(original_tool,(dx,dy))
            tool=load(poses[name]['equipment']['mallet'])
            assert pixels(tool)==pixels(expected),f'Tool angle or gripping gap changed: {name}'
            assert sum(p[3]>0 for p in tool.getdata())==sum(p[3]>0 for p in original_tool.getdata()),f'Tool clipped: {name}'
    # Supporting anatomy stays fixed while the opposite arm or head moves.
    for n in animation['frames']:
        for prefix,reference in [('frames/base',load('base-neutral.png')),('frames',load('neutral.png'))]:
            im=load(f'{prefix}/{n}.png')
            for y in range(66,83):
                for x in range(24,38):
                    actual=im.getpixel((x+16,y+16));expected=reference.getpixel((x,y))
                    assert actual[3]==expected[3] and (not expected[3] or actual==expected),f'Supporting hand changed: {n}'
        im=load(f'frames/{n}.png')
        for x,y in [(29,66),(29,67),(29,68),(30,68),(31,68)]:
            assert im.getpixel((x+16,y+16))[3]==255,f'Gap between board and forearm: {n}'
    page=load('atlas.png');assert max(page.size)<=2048 and page.width%16==page.height%16==0
    rects=[]
    for sprite in manifest['sprites']:
        x,y,w,h=sprite['rect'];assert (w,h)==(128,160)
        assert pixels(page.crop((x,y,x+w,y+h)))==pixels(load(sprite['source']))
        for dx in range(-2,w+2):
            assert page.getpixel((x+dx,y-2))[3]==page.getpixel((x+dx,y+h+1))[3]==0
        for dy in range(-2,h+2):
            assert page.getpixel((x-2,y+dy))[3]==page.getpixel((x+w+1,y+dy))[3]==0
        for dx in range(-1,w+1):
            sx=min(max(dx,0),w-1)
            assert page.getpixel((x+dx,y-1))==page.getpixel((x+sx,y))
            assert page.getpixel((x+dx,y+h))==page.getpixel((x+sx,y+h-1))
        for dy in range(h):
            assert page.getpixel((x-1,y+dy))==page.getpixel((x,y+dy))
            assert page.getpixel((x+w,y+dy))==page.getpixel((x+w-1,y+dy))
        for ox,oy,ow,oh in rects:assert x+w+2<=ox-2 or ox+ow+2<=x-2 or y+h+2<=oy-2 or oy+oh+2<=y-2
        rects.append((x,y,w,h))
    result={'status':'pass','baseline_pixel_locks':len(locks),'native_layers_and_frames_checked':len(native),
        'eye_states':len(expressions['eyes']),'mouth_states':len(expressions['mouth']),'expression_count':len(expressions['recipes']),
        'unique_frames':len(animation['frames']),'clip_count':len(animation['clips']),'opaque_union_colors':len(colors),
        'base_loss_pixels':0,'expression_changes_outside_features':0,'head_gesture_clothing_and_gear':'source layers pixel-identical; exposed collar preserved',
        'transfer_events':1,'atlas_size':list(page.size),'atlas_roundtrip':'pixel-identical','extrude_and_gutter':'pass',
        'nearest_4x_previews':'exact','visual_review':'recorded separately','runtime_performance':'not_measured'}
    result['review_regressions']=['nose retained in all expressions','no open-eye white in closed lid',
        'exposed collar pixels fixed','skin joins across neck rows','supporting forearm contour continuous',
        'turned jaws remain at neutral chin height','approved gripping fingers and upright tool retained']
    (ROOT/'qa/animation-validation.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2));return result

if __name__=='__main__':verify()
