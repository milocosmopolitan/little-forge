"""Verify hunter authoring frames, source identity, atlas and equipment combinations."""
from pathlib import Path
from PIL import Image, ImageChops
import hashlib, itertools, json, subprocess, sys

ROOT=Path(__file__).resolve().parents[1]
def read(p):return Image.open(ROOT/p).convert('RGBA')
def same(a,b):return a.tobytes()==b.tobytes()
def compose(items):
    out=Image.new('RGBA',(128,160))
    for im in items:out.alpha_composite(im)
    return out
def move(im,dx,dy):
    out=Image.new('RGBA',(128,160));out.paste(im,(dx,dy));return out
def components(im):
    points={(x,y) for y in range(160) for x in range(128) if im.getpixel((x,y))[3]};sizes=[]
    while points:
        stack=[points.pop()];n=0
        while stack:
            x,y=stack.pop();n+=1
            for dx,dy in itertools.product([-1,0,1],repeat=2):
                p=x+dx,y+dy
                if p in points:points.remove(p);stack.append(p)
        sizes.append(n)
    return sorted(sizes,reverse=True)

baseline=json.loads((ROOT/'source/animation/baseline-hashes.json').read_text())
for path,digest in baseline.items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,path
animation=json.loads((ROOT/'animation.json').read_text());recipes=json.loads((ROOT/'poses/recipes.json').read_text())['poses']
all_colors=set();toggle_count=0;alpha_components={};layer_count=0
for name in animation['frames']:
    r=recipes[name];layers={n:read(p) for n,p in (r['layers']|r['equipment']).items()}
    base=read(f'frames/base/{name}.png');full=read(f'frames/{name}.png')
    assert same(compose([layers[n] for n in r['base_order']]),base),name
    assert same(compose([layers[n] for n in r['composition_order']]),full),name
    for im in layers.values():
        layer_count+=1;assert im.size==(128,160)
        assert set(im.getchannel('A').getdata())<={0,255}
        all_colors.update(p[:3] for p in im.getdata() if p[3])
    alpha_components[name]=components(base)
    assert all(n<=12 for n in alpha_components[name][1:]),(name,'disconnected body/head/hand',alpha_components[name])
    equipment=list(r['equipment'])
    for flags in itertools.product([False,True],repeat=len(equipment)):
        enabled={n for n,on in zip(equipment,flags) if on}
        im=compose([layers[n] for n in r['composition_order'] if n not in equipment or n in enabled])
        assert ImageChops.subtract(base.getchannel('A'),im.getchannel('A')).getbbox() is None,name
        toggle_count+=1
    if name.startswith('walk-'):assert base.getbbox()[3]==140,(name,base.getbbox())
assert len(all_colors)<=64,len(all_colors)
assert same(read('frames/base/neutral.png'),read('base-neutral.png'))
assert same(read('frames/neutral.png'),read('neutral.png'))
expressions=json.loads((ROOT/'expressions/recipes.json').read_text())
mask=Image.new('L',(128,160))
override_paths=[path for channels in expressions.get('recipe_overrides',{}).values() for path in channels.values()]
for path in list(expressions['eyes'].values())+list(expressions['mouth'].values())+override_paths:mask=ImageChops.lighter(mask,read(path).getchannel('A'))
neutral=read('base-neutral.png')
for name in expressions['recipes']:
    im=read(f'frames/base/{name}.png')
    for y in range(160):
        for x in range(128):
            if im.getpixel((x,y))!=neutral.getpixel((x,y)):assert mask.getpixel((x,y)),(name,x,y)
motion=read('source/animation/motion-head.png')
for name in ['nod-mid']+[f'walk-{i}' for i in range(6)]:
    r=recipes[name];h=compose([read(r['layers'][n]) for n in ['head','eyes','mouth']]);dx,dy=r['head_translation']
    assert same(h,move(motion,dx,dy)),(name,'head identity')
atlas_manifest=json.loads((ROOT/'atlas-manifest.json').read_text());atlas=read('atlas.png')
for sprite in atlas_manifest['sprites']:
    x,y,w,h=sprite['rect'];source=read(sprite['source'])
    assert sprite['source_size']==[128,160] and sprite['pivot']==[64,140]
    assert same(atlas.crop((x,y,x+w,y+h)),source),sprite['id']
    for xx in range(-1,129):
        for yy in [-1,160]:assert atlas.getpixel((x+xx,y+yy))==source.getpixel((max(0,min(127,xx)),max(0,min(159,yy))))
    for yy in range(160):
        for xx in [-1,128]:assert atlas.getpixel((x+xx,y+yy))==source.getpixel((max(0,min(127,xx)),yy))
for name,clip in animation['clips'].items():
    assert len(clip['frames'])==len(clip['durations_ms']) and all(ms>0 for ms in clip['durations_ms'])
    assert all(n in animation['frames'] for n in clip['frames'])
    with Image.open(ROOT/f'previews/{name}.gif') as gif:assert gif.n_frames==len(clip['frames']),(name,gif.n_frames)
assert len(animation['clips']['walk']['frames'])==6 and animation['clips']['walk']['loop']
assert animation['clips']['give']['events']==[{'key':3,'name':'transfer','hand':'left','item':None,'fires':'once_on_entry'}]
before={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in ROOT.rglob('*') if p.is_file() and 'qa' not in p.relative_to(ROOT).parts}
subprocess.run([sys.executable,str(ROOT/'source/build-animation.py')],check=True,capture_output=True)
after={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in ROOT.rglob('*') if p.is_file() and 'qa' not in p.relative_to(ROOT).parts}
assert before==after,'non-deterministic animation rebuild'
report={'status':'pass','frames':len(animation['frames']),'clips':len(animation['clips']),'native_layers_checked':layer_count,
        'locked_neutral_files':len(baseline),'opaque_colors':len(all_colors),'equipment_combinations':toggle_count,
        'underlayer_pixels_lost':0,'expression_changes_outside_features':0,'walk_and_nod_preparation_face_identity':True,
        'atlas_roundtrip_and_extrusion':True,'transfer_events':1,'walk_keys':6,'deterministic_rebuild':True,
        'alpha_components':alpha_components,'scope':'authoring validation; not runtime integration'}
(ROOT/'qa/animation-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='alpha_components'},indent=2))
