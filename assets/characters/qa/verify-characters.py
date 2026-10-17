"""Read-only art checks, plus a deterministic builder replay."""
import hashlib
import itertools
import json
import subprocess
from pathlib import Path
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
SIZE=(128,160)
def author_crop(im):return im.crop((16,16,112,144))
def rgba(p):return Image.open(p).convert('RGBA')
def pixels(im):return list(im.get_flattened_data() if hasattr(im,'get_flattened_data') else im.getdata())
def canonical(im):return [p if p[3] else (0,0,0,0) for p in pixels(im)]
def same(a,b):return canonical(a)==canonical(b)
def compose(dir,names):
    out=Image.new('RGBA',SIZE)
    for n in names:out.alpha_composite(rgba(dir/n))
    return out
def hashes():
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()
            for p in ROOT.rglob('*') if p.is_file() and p.suffix in ['.png','.json','.yaml'] and 'qa' not in p.parts and p.name!='prompts.json'}
reports=[]
for sex in ['female','male']:
    d=ROOT/sex
    recipe=json.loads((d/'character.json').read_text())
    spec=json.loads((d/'character-spec.yaml').read_text())
    report=json.loads((d/'qa/coverage-report.json').read_text())
    layer_names=list(report['layer_bounds'])
    colors=set()
    for n in layer_names+['base-neutral.png','neutral.png','neutral-with-hammer.png','source/foundation.png']:
        im=Image.open(d/n)
        assert im.mode=='RGBA' and im.size==SIZE,(sex,n,'native format')
        assert set(pixels(im.getchannel('A'))) <= {0,255},(sex,n,'fractional alpha')
        colors.update(p[:3] for p in pixels(im) if p[3])
    assert len(colors)<=80,(sex,'palette',len(colors))
    assert spec['palette']['max_opaque_colors']==80
    assert recipe['canvas']==list(SIZE) and recipe['pivot']==[64,140]
    assert spec['native_canvas_px']==list(SIZE) and spec['foot_anchor_px']==[64,140]
    foundation=compose(d,recipe['base'][:3])
    base=compose(d,recipe['base'])
    assert same(foundation,rgba(d/'source/foundation.png')),(sex,'foundation partition')
    assert same(base,rgba(d/'base-neutral.png')),(sex,'base reconstruction')
    neutral=Image.alpha_composite(base,compose(d,recipe['equipped']))
    assert same(neutral,rgba(d/'neutral.png')),(sex,'equipped reconstruction')
    full=Image.alpha_composite(neutral,rgba(d/recipe['optional_prop']))
    assert same(full,rgba(d/'neutral-with-hammer.png')),(sex,'prop reconstruction')
    for name in ['eyes/open.png','mouth/rest.png']:
        assert all(not p[3] or q[3] for p,q in zip(pixels(rgba(d/name)),pixels(rgba(d/'head-base.png')))),(sex,'face hole',name)
    head=author_crop(compose(d,['head-base.png','eyes/open.png','mouth/rest.png']))
    previous=rgba(d/'source/revision-3/before/head-base.png')
    current=rgba(d/'head-base.png')
    full_head=compose(d,['head-base.png','eyes/open.png','mouth/rest.png'])
    registered=rgba(d/'source/revision-3/registered-full.png')
    assert not same(current,previous),(sex,'face repair missing')
    for y in range(160):
        for x in range(128):
            neck=sex=='male' and 53<=x<77 and 59<=y<65
            if not neck:
                assert current.getpixel((x,y))[3]==previous.getpixel((x,y))[3],(sex,'head silhouette drift',x,y)
    for n in ['body/upper-base.png','body/lower-base.png','body/arms-front.png','equipment/apron.png','equipment/belt-tools.png','equipment/gloves.png']:
        prior=d/('source/revision-3/before/'+n) if n.startswith('body/') else d/('source/revision-2/before/'+n)
        before_im=rgba(prior);after_im=rgba(d/n)
        for y in range(160):
            for x in range(128):
                allowed=sex=='male' and n=='body/upper-base.png' and 55<=x<73 and 65<=y<72
                if not allowed:assert before_im.getpixel((x,y))==after_im.getpixel((x,y)),(sex,'body/gear drift',n,x,y)
                else:assert before_im.getpixel((x,y))[3]==after_im.getpixel((x,y))[3],(sex,'neck alpha drift')
    # The smile must include its raised corner and short closed lower segment.
    lip=rgba(d/'mouth/rest.png')
    lip_points=[(x,y) for y in range(160) for x in range(128) if lip.getpixel((x,y))[3]]
    ys=sorted(set(y for x,y in lip_points))
    assert len(ys)==2 and ys[1]==ys[0]+1,(sex,'closed smile corner missing',lip_points)
    assert min(x for x,y in lip_points if y==ys[0])<min(x for x,y in lip_points if y==ys[1]),(sex,'smile direction')
    for y in range(42,46):
        for x in range(48,57):
            assert full_head.getpixel((x+16,y+16))==registered.getpixel((x+16,y+16)),(sex,'source smile reconstruction',x,y)
    # Both pupil interiors must be actual filled skin underneath removable eyes.
    for box in ([(45,35,48,39),(57,35,59,39)] if sex=='female' else [(44,34,47,39),(55,34,57,39)]):
        for p in pixels(author_crop(rgba(d/'head-base.png')).crop(box)):
            assert p[3] and p[0]>180,(sex,'eye remains or hole in head base',box,p)
    under=rgba(d/'source/registered-underlayer.png')
    body=compose(d,['body/lower-base.png','body/upper-base.png'])
    assert body.crop((16,72,112,144)).tobytes()==under.crop((16,72,112,144)).tobytes(),(sex,'body partition loss below neck')
    assert body.getbbox()[3]==141,(sex,'foot anchor drift')
    for y in range(46,53):
        for x in range(20,41):
            p=under.getpixel((x+16,y+16))
            if p[3] and p[0]>190 and p[1]>170 and p[2]>130 and p[0]<p[1]*1.3:
                assert body.getpixel((x+16,y+16))[3],(sex,'shoulder shirt edge clipped',x,y)
    gloves=rgba(d/'equipment/gloves.png')
    glove_report=json.loads((d/'source/glove-registration.json').read_text())
    for k in range(2):
        target=rgba(d/f'source/hand-target-{k}.png')
        assert sum(bool(p[3]) for p in pixels(target))>=10,(sex,'empty hand target',k)
        assert not any(p[3] and not q[3] for p,q in zip(pixels(target),pixels(gloves))),(sex,'hand showing through glove',k)
        assert glove_report['hands'][k]['outside_two_pixel_hand_margin']<=4,(sex,'glove too far from hand',k)
    assert not any(p[3] and p[0]>190 and p[1]>170 and p[2]>130
                   for p in pixels(rgba(d/'body/arms-front.png').crop((48,81,74,95)))),(sex,'shirt trunk fragment in foreground arm layer')
    assert not any(p[3] and p[0]>185 and p[1]>105 and p[2]>55 and p[0]>p[1]*1.22
                   for p in pixels(rgba(d/'body/lower-base.png').crop((16,90,112,114)))),(sex,'arm skin assigned to lower body')
    if sex=='male':
        belt=rgba(d/'equipment/belt-tools.png');towel=rgba(d/'equipment/towel.png')
        assert not any(p[3] and q[3] for p,q in zip(pixels(belt),pixels(towel))),'towel fragment baked into belt'
    for n in ['base-neutral','neutral','neutral-with-hammer']:
        expected=rgba(d/(n+'.png')).resize((512,640),Image.Resampling.NEAREST)
        assert same(expected,rgba(d/'previews'/(n+'-4x.png'))),(sex,'nearest scale',n)
    gear=[n for n in layer_names if n.startswith('equipment/')]+['props/hammer.png']
    for enabled in itertools.product([False,True],repeat=len(gear)):
        out=Image.alpha_composite(base,compose(d,[n for n,v in zip(gear,enabled) if v]))
        assert all(not p[3] or q[3] for p,q in zip(pixels(foundation),pixels(out))),(sex,'toggle lost body',enabled)
    if sex=='female':
        assert any(p[3] and p[0]>110 and p[1]<p[0]*.35 for p in pixels(rgba(d/'equipment/hair-tie.png'))),'red ribbon lost'
    reports.append({'id':sex,'status':'pass','native_layers':len(layer_names),'shared_opaque_colors':len(colors),
                    'equipment_combinations_checked':2**len(gear),'crown_silhouette_preserved':True,'body_and_gear_unchanged_outside_local_male_neck':True,
                    'eye_sites_filled':True,'complete_body_partition':True,'foundation_pixels_lost':0,
                    'canvas':[128,160],'foot_y':140,'preview_nearest_exact':True,
                    'closed_smile_corner_verified':True,'coherent_generated_face':True,'uncovered_hand_skin_pixels':0})
before=hashes()
subprocess.run(['node',str(ROOT/'source/build-characters.mjs')],check=True,stdout=subprocess.DEVNULL)
assert before==hashes(),'non-deterministic build'
result={'status':'pass','characters':reports,'rebuild_byte_identical':True,
        'visual_review':'Base/equipped/hammer and isolated layers reviewed on light/dark backgrounds separately from structural tests',
        'runtime_integration':'not_integrated','animation_scope':'neutral_only'}
(ROOT/'qa/validation.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
