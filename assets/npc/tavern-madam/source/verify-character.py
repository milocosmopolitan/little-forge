"""Check neutral composition, native pixels, complete grip and packed-frame integrity."""
from pathlib import Path
from PIL import Image
import json
import hashlib

ROOT = Path(__file__).resolve().parents[1]


def load(path):
    return Image.open(ROOT / path).convert('RGBA')


def pixels(image):
    return bytes(v for p in image.getdata() for v in (p if p[3] else (0,0,0,0)))


def compose(paths, size=(96,128)):
    result = Image.new('RGBA', size)
    for path in paths: result.alpha_composite(load(path))
    return result


def verify():
    reg = json.loads((ROOT/'source/registration.json').read_text())
    base_paths = ['body/lower-base.png','body/upper-base.png','head-base.png','eyes/open.png','mouth/rest.png']
    equipment = [f'equipment/{n}.png' for n in reg['equipment_order']]
    base = compose(base_paths); full = compose(base_paths+equipment)
    assert pixels(base)==pixels(load('base-neutral.png'))==pixels(load('source/complete-underlayer-native.png'))
    assert pixels(full)==pixels(load('neutral.png'))
    assert base.getchannel('A').getbbox()[3]==full.getchannel('A').getbbox()[3]==124
    assert all(not a[3] or b[3] for a,b in zip(base.getdata(),full.getdata()))
    face_mask=load('source/face-interior-mask.png').getchannel('A')
    reference=load('source/reference-native.png')
    assert all(base.getpixel((x,y))==reference.getpixel((x,y)) for y in range(128) for x in range(96)
               if (x,y)!=(42,34) and face_mask.getpixel((x,y)) and base.getpixel((x,y))[3] and reference.getpixel((x,y))[3]),'reference face interior changed'
    assert max(base.getpixel((42,34))[:3])<150,'beauty mark lost contrast'
    assert all(base.getpixel((x,45))[3] and max(base.getpixel((x,45))[:3])<60 for x in range(67,71)),'open grip top outline'
    head = load('head-base.png')
    for name in ['eyes/open.png','mouth/rest.png']:
        feature = load(name); assert feature.getbbox()
        assert all(not a[3] or b[3] for a,b in zip(feature.getdata(),head.getdata()))
    # A continuous warm-skin run connects the curled hand to the rolled cuff.
    for y in range(51,64):
        skin = [x for x in range(63,73) if (p:=base.getpixel((x,y)))[3]
                and p[0]>205 and 110<p[1]<205 and p[2]<155]
        assert len(skin)>=2, ('interrupted wrist',y,skin)
    frame = load('frames/neutral.png'); bare_frame = load('frames/base/neutral.png')
    for image,original in [(frame,full),(bare_frame,base)]:
        assert image.size==(128,160)
        assert pixels(image.crop((16,16,112,144)))==pixels(original)
        assert sum(p[3]>0 for p in image.getdata())==sum(p[3]>0 for p in original.getdata())
    recipe = json.loads((ROOT/'poses/recipes.json').read_text())['poses']['neutral']
    layers = {**recipe['layers'],**recipe['equipment']}
    assert pixels(compose([layers[n] for n in recipe['base_order']],(128,160)))==pixels(bare_frame)
    assert pixels(compose([layers[n] for n in recipe['composition_order']],(128,160)))==pixels(frame)
    paths = set(base_paths+equipment+['base-neutral.png','neutral.png','frames/neutral.png','frames/base/neutral.png'])
    paths.update(layers.values()); paths.update(['poses/neutral/base.png','poses/neutral/composed.png'])
    colors=set()
    for path in paths:
        image=Image.open(ROOT/path)
        assert image.mode=='RGBA',path
        assert image.size==((128,160) if path.startswith(('frames/','poses/')) else (96,128)),path
        assert set(image.getchannel('A').getdata())<={0,255},path
        assert image.getbbox(),path
        colors.update(p[:3] for p in image.getdata() if p[3])
    assert len(colors)<=64,len(colors)
    manifest=json.loads((ROOT/'atlas-manifest.json').read_text());sprite=manifest['sprites'][0]
    page=load('atlas.png');x,y,w,h=sprite['rect']
    assert sprite['source_size']==[128,160] and sprite['pivot']==[64,140]
    assert pixels(page.crop((x,y,x+w,y+h)))==pixels(frame)
    for yy in range(-2,h+2):
        for xx in range(-2,w+2):
            p=page.getpixel((x+xx,y+yy))
            if xx in (-2,w+1) or yy in (-2,h+1):assert p[3]==0
            elif xx in (-1,w) or yy in (-1,h):
                expected=frame.getpixel((max(0,min(w-1,xx)),max(0,min(h-1,yy))))
                assert p==expected
    for name,image in [('neutral',frame),('base-neutral',bare_frame)]:
        assert pixels(image.resize((512,640),Image.Resampling.NEAREST))==pixels(load(f'previews/{name}-4x.png'))
    animation=json.loads((ROOT/'animation.json').read_text())
    assert animation['frames'][0]=='neutral'
    report={'status':'pass','native_files_checked':len(paths),'palette_union_colors':len(colors),
            'alpha_values':[0,255],'authoring_canvas':[96,128],'frame_canvas':[128,160],
            'foot_anchor_px':[64,140],'base_loss_pixels':0,'layer_recomposition':'exact',
            'source_to_frame_pixels':'exact offset (16,16)','raised_wrist_skin_join':'continuous',
            'atlas_roundtrip':'exact','extrude_gutter':'pass','nearest_4x':'exact',
            'scope':'approved neutral baseline; animations checked separately','source_hashes':{n:hashlib.sha256((ROOT/'source'/n).read_bytes()).hexdigest()
                for n in ['reference.png','equipped-candidate.png','underlayer-corrected.png','raised-arm-candidate.png']},
            'visual_quality':'reviewed separately','runtime':'not_integrated_or_measured'}
    (ROOT/'qa').mkdir(exist_ok=True)
    (ROOT/'qa/validation.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__=='__main__': verify()
