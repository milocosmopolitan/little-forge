"""Read-only validation of the carpenter's fixed-canvas neutral export."""
from pathlib import Path
from PIL import Image
import hashlib
import json

ROOT=Path(__file__).resolve().parent.parent

def load(path):return Image.open(ROOT/path).convert('RGBA')
def pixels(im):return bytes(v for p in im.getdata() for v in (p if p[3] else (0,0,0,0)))
def compose(layers):
    out=Image.new('RGBA',(96,128))
    for layer in layers:out.alpha_composite(layer)
    return out

def verify():
    spec=json.loads((ROOT/'character-spec.yaml').read_text())
    layers=spec['layers'];gear=layers['equipment']
    parts=[layers['lower_base'],layers['upper_base'],layers['head_base'],layers['eyes']['open'],layers['mouth']['rest']]
    files=parts+list(gear.values())+['base-neutral.png','neutral.png']
    colors=set()
    for path in files:
        im=Image.open(ROOT/path)
        assert im.mode=='RGBA' and im.size==(96,128),path
        assert set(im.getchannel('A').getdata())<={0,255},path
        colors.update(p[:3] for p in im.getdata() if p[3])
    assert len(colors)<=32,len(colors)
    base=compose([load(p) for p in parts]);assert pixels(base)==pixels(load('base-neutral.png'))
    full=compose([base]+[load(p) for p in gear.values()]);assert pixels(full)==pixels(load('neutral.png'))
    assert not any(a[3] and not b[3] for a,b in zip(base.getdata(),full.getdata()))
    assert base.getchannel('A').getbbox()[3]==124
    for name in ['neutral','base-neutral']:
        assert load(f'previews/{name}-4x.png').tobytes()==load(f'{name}.png').resize((384,512),Image.Resampling.NEAREST).tobytes()
    manifest=json.loads((ROOT/'atlas-manifest.json').read_text());atlas=load('atlas.png')
    assert tuple(manifest['pages'][0]['size'])==atlas.size
    sprite=next(s for s in manifest['sprites'] if s['id']=='neutral')
    x,y,w,h=sprite['rect']
    source=load(sprite['source'])
    if source.size!=(96,128):
        expected=Image.new('RGBA',source.size);expected.alpha_composite(full,(16,16));full=expected
    assert pixels(atlas.crop((x,y,x+w,y+h)))==pixels(full)
    # Both boundary rows of the untrimmed source are transparent, hence so is extrusion.
    for dx in range(-2,w+2):
        assert atlas.getpixel((x+dx,y-2))[3]==atlas.getpixel((x+dx,y+h+1))[3]==0
        sx=min(max(dx,0),w-1)
        assert atlas.getpixel((x+dx,y-1))==full.getpixel((sx,0))
        assert atlas.getpixel((x+dx,y+h))==full.getpixel((sx,h-1))
    for dy in range(h):
        assert atlas.getpixel((x-2,y+dy))[3]==atlas.getpixel((x+w+1,y+dy))[3]==0
        assert atlas.getpixel((x-1,y+dy))==full.getpixel((0,dy))
        assert atlas.getpixel((x+w,y+dy))==full.getpixel((w-1,dy))
    # Every feature layer must have a complete opaque backing, with no alpha punches.
    head=load('head-base.png')
    for path in [layers['eyes']['open'],layers['mouth']['rest']]:
        assert all(not feature[3] or backing[3] for feature,backing in zip(load(path).getdata(),head.getdata()))
    report={'status':'pass','native_files_checked':len(files),'native_size':[96,128],
        'opaque_union_colors':len(colors),'binary_alpha':True,'layer_recomposition':'pixel-identical',
        'base_loss_pixels':0,'feature_backing_holes':0,'nearest_4x_previews':'exact',
        'atlas_roundtrip':'pixel-identical','extrude_and_gutter':'pass','atlas_size':list(atlas.size),
        'foot_baseline_y':124,'expressions':['neutral'],'pose_count':1,
        'reference_and_candidate_sha256':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in ['source/reference.png','source/generated-underlayer.png']},
        'runtime':'not_integrated_or_profiled','visual_quality':'reviewed separately; not established by these checks'}
    (ROOT/'qa/validation.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
    return report

if __name__=='__main__':verify()
