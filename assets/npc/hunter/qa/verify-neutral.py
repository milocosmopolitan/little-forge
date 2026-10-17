"""Validate authored layers, every equipment toggle and atlas reconstruction."""
from pathlib import Path
import hashlib
import importlib.util
import itertools
import json
import subprocess
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
def read(path):return Image.open(ROOT/path).convert('RGBA')
def composite(paths):
    result=Image.new('RGBA',(128,160))
    for path in paths:result.alpha_composite(read(path))
    return result
def same(a,b):return a.tobytes()==b.tobytes()

recipe=json.loads((ROOT/'poses/recipes.json').read_text())['poses']['neutral']
paths=recipe['layers']|recipe['equipment'];gear=list(recipe['equipment'])
base=read('base-neutral.png');full=read('neutral.png')
assert same(composite([paths[n] for n in recipe['base_order']]),base)
assert same(composite([paths[n] for n in recipe['composition_order']]),full)
all_colors=set()
for path in set(paths.values())|{'base-neutral.png','neutral.png'}:
    image=read(path);assert image.size==(128,160),path
    assert set(image.getchannel('A').getdata())<={0,255},path
    all_colors.update(p[:3] for p in image.getdata() if p[3])
assert len(all_colors)<=64,len(all_colors)
combinations=[]
for flags in itertools.product([False,True],repeat=len(gear)):
    enabled={n for n,on in zip(gear,flags) if on}
    result=composite([paths[n] for n in recipe['composition_order'] if n not in gear or n in enabled])
    missing=sum(a and not b for a,b in zip(base.getchannel('A').getdata(),result.getchannel('A').getdata()))
    assert missing==0,(enabled,missing)
    if not enabled:assert same(result,base),'bare-hand foreground changed equipment-off base'
    combinations.append({'enabled':sorted(enabled),'underlayer_pixels_lost':missing})
assert base.getbbox()[3]==140
for name in ['neutral','base-neutral']:
    assert same(read(name+'.png').resize((512,640),Image.Resampling.NEAREST),read('previews/'+name+'-4x.png'))
atlas=read('atlas.png');assert same(atlas.crop((2,2,130,162)),full)
for y in range(-1,161):
    for x in range(-1,129):
        if x in (-1,128) or y in (-1,160):
            assert atlas.getpixel((x+2,y+2))==full.getpixel((max(0,min(127,x)),max(0,min(159,y))))
manifest=json.loads((ROOT/'atlas-manifest.json').read_text());assert manifest['sprites'][0]['pivot']==[64,140]
assert 'neutral' in json.loads((ROOT/'animation.json').read_text())['frames']
before={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in ROOT.rglob('*') if p.is_file() and 'qa' not in p.relative_to(ROOT).parts}
subprocess.run(['python3',str(ROOT/'source/build-character.py')],check=True,capture_output=True)
after={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in ROOT.rglob('*') if p.is_file() and 'qa' not in p.relative_to(ROOT).parts}
assert before==after,'rebuild changed authored outputs'
report={'status':'pass','canvas':[128,160],'foot_anchor_px':[64,140],'opaque_colors':len(all_colors),'binary_alpha':True,
        'exact_base_layer_roundtrip':True,'all_equipment_combinations':len(combinations),'underlayer_pixels_lost':0,
        'exact_nearest_previews':True,'atlas_roundtrip_and_extrusion':True,'deterministic_rebuild':True,
        'scope':'neutral only; no game runtime integration','combinations':combinations}
(ROOT/'qa/validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='combinations'},indent=2))
