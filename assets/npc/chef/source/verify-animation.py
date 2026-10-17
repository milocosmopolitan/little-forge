"""Read-only checks for chef expressions, pose composition and atlas export."""
from pathlib import Path
from PIL import Image
import hashlib
import json

ROOT=Path(__file__).resolve().parent.parent

def load(name):
    return Image.open(ROOT/name).convert('RGBA')

def pixels(im):
    return bytes(value for p in im.getdata() for value in (p if p[3] else (0,0,0,0)))

def compose(images):
    im=Image.new('RGBA',(128,160))
    for layer in images:im.alpha_composite(layer)
    return im

def verify():
    animation=json.loads((ROOT/'animation.json').read_text())
    poses=json.loads((ROOT/'poses/recipes.json').read_text())['poses']
    expressions=json.loads((ROOT/'expressions/recipes.json').read_text())
    manifest=json.loads((ROOT/'atlas-manifest.json').read_text())
    locks=json.loads((ROOT/'source/animation/baseline-lock.json').read_text())
    for name,digest in locks.items():
        assert hashlib.sha256(load(name).tobytes()).hexdigest()==digest, f'Foundation modified: {name}'
    native_files=set()
    for name in animation['frames']:
        native_files.add(ROOT/f'frames/{name}.png');native_files.add(ROOT/f'frames/base/{name}.png')
        recipe=poses[name]
        native_files.update(ROOT/path for path in list(recipe['layers'].values())+list(recipe['equipment'].values()))
        layers={key:load(path) for key,path in dict(recipe['layers'],**recipe['equipment']).items()}
        expected=compose([layers[key] for key in recipe['composition_order']])
        base=compose([layers[key] for key in recipe['base_order']])
        assert pixels(expected)==pixels(load(f'frames/{name}.png')),name
        assert pixels(base)==pixels(load(f'frames/base/{name}.png')),name
        assert pixels(base)==pixels(load(f'poses/{name}/base.png')),name
        assert not any(a[3] and not b[3] for a,b in zip(base.getdata(),expected.getdata())),f'Base loss: {name}'
        assert base.getchannel('A').getbbox()[3]==141,f'Foot baseline: {name}'
        assert len(recipe['sockets_px']['screen_right_hand'])==2
        assert all(0<=v<=max(base.size) for point in recipe['sockets_px']['eyes'] for v in point)
    colors=set()
    for path in sorted(native_files):
        im=Image.open(path)
        assert im.mode=='RGBA' and im.size==(128,160),str(path)
        assert set(im.getchannel('A').getdata())<={0,255},str(path)
        colors.update(p[:3] for p in im.getdata() if p[3])
    for section in ['eyes','mouth']:
        parts=[load(path) for path in expressions[section].values()]
        assert len({pixels(im) for im in parts})==len(parts),f'Duplicate {section} states'
        for im in parts:
            assert im.size==(96,128)
            assert set(im.getchannel('A').getdata())<={0,255}
            colors.update(p[:3] for p in im.getdata() if p[3])
    assert len(colors)<=32,len(colors)
    neutral=load('frames/neutral.png')
    eye_mask=load('source/animation/expression-eyes-mask.png')
    mouth_mask=load('source/animation/expression-mouth-mask.png')
    allowed={(x+16,y+16) for y in range(128) for x in range(96)
             if eye_mask.getpixel((x,y))[3] or mouth_mask.getpixel((x,y))[3]}
    for name in expressions['recipes']:
        im=load(f'frames/{name}.png')
        assert im.getchannel('A').tobytes()==neutral.getchannel('A').tobytes(),f'Expression silhouette: {name}'
        assert all(im.getpixel((x,y))==neutral.getpixel((x,y)) for y in range(160) for x in range(128)
                   if (x,y) not in allowed),f'Outside facial features: {name}'
    embedded=Image.new('RGBA',(128,160));embedded.alpha_composite(load('neutral.png'),(16,16))
    assert pixels(embedded)==pixels(neutral)
    for name,clip in animation['clips'].items():
        assert all(k['sprite'] in animation['frames'] and isinstance(k['duration_ms'],int) and k['duration_ms']>0 for k in clip['frames']),name
    assert len(animation['clips']['nod']['frames'])==5
    assert len(animation['clips']['shake_head']['frames'])==5
    # A head-only gesture must never substitute a differently painted shirt or gear.
    standing=poses['neutral']
    for name in ['turn-left','turn-right']:
        for layer in ['upper','lower']:
            assert pixels(load(poses[name]['layers'][layer]))==pixels(load(standing['layers'][layer])),f'Shake clothing changed: {name}/{layer}'
        assert poses[name]['equipment'].keys()==standing['equipment'].keys()
        for gear in standing['equipment']:
            assert pixels(load(poses[name]['equipment'][gear]))==pixels(load(standing['equipment'][gear])),f'Shake equipment changed: {name}/{gear}'
    bridge=load(poses['turn-left']['layers']['neck'])
    upper=load(standing['layers']['upper'])
    assert not any(a[3] and b[3] for a,b in zip(bridge.getdata(),upper.getdata())), 'Neck bridge paints over clothing'

    give=animation['clips']['give']['frames']
    assert len(give)==6 and sum(k.get('event')=='transfer' for k in give)==1
    assert give[3].get('event')=='transfer'
    assert all('spoon' not in poses[k['sprite']]['equipment'] for k in give[3:])
    assert all('spoon' in poses[k['sprite']]['equipment'] for k in give[:3])
    page=load('atlas.png');rects=[]
    assert max(page.size)<=2048 and page.width%16==page.height%16==0
    for sprite in manifest['sprites']:
        x,y,w,h=sprite['rect'];assert (w,h)==(128,160)
        assert x>=2 and y>=2 and x+w+2<=page.width and y+h+2<=page.height
        assert pixels(page.crop((x,y,x+w,y+h)))==pixels(load(sprite['source'])),sprite['id']
        for dx in range(-2,w+2):
            assert page.getpixel((x+dx,y-2))[3]==0
            assert page.getpixel((x+dx,y+h+1))[3]==0
        for dy in range(-2,h+2):
            assert page.getpixel((x-2,y+dy))[3]==0
            assert page.getpixel((x+w+1,y+dy))[3]==0
        for dx in range(-1,w+1):
            sx=min(max(dx,0),w-1)
            assert page.getpixel((x+dx,y-1))==page.getpixel((x+sx,y))
            assert page.getpixel((x+dx,y+h))==page.getpixel((x+sx,y+h-1))
        for dy in range(h):
            assert page.getpixel((x-1,y+dy))==page.getpixel((x,y+dy))
            assert page.getpixel((x+w,y+dy))==page.getpixel((x+w-1,y+dy))
        rect=tuple(sprite['rect'])
        for ox,oy,ow,oh in rects:
            assert rect==(ox,oy,ow,oh) or x+w+2<=ox-2 or ox+ow+2<=x-2 or y+h+2<=oy-2 or oy+oh+2<=y-2
        rects.append(rect)
    report={'status':'pass','native_pose_layers_checked':len(native_files),'expression_count':len(expressions['recipes']),
        'eye_states':len(expressions['eyes']),'mouth_states':len(expressions['mouth']),
        'unique_frames':len(animation['frames']),'clip_count':len(animation['clips']),
        'opaque_union_colors':len(colors),'base_loss_pixels':0,'expression_changes_outside_features':0,
        'shake_clothing_and_gear':'pixel-identical to neutral','neck_bridge_over_clothing_pixels':0,
        'baseline_pixel_locks':len(locks),'transfer_events':1,'atlas_size':list(page.size),
        'atlas_roundtrip':'pixel-identical','extrude_and_gutter':'pass','runtime_performance':'not_measured',
        'visual_review':'See qa/playback-review.json; structural checks do not establish visual quality'}
    (ROOT/'qa').mkdir(exist_ok=True)
    (ROOT/'qa/animation-validation.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
    return report

if __name__=='__main__':verify()
