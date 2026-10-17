"""Verify modular expressions, pose coverage, gait identity and packed pixels."""
import hashlib
import json
import subprocess
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parents[2]
def load(n, native=True):
    image=Image.open(ROOT/n).convert('RGBA')
    if str(n).startswith(('frames/','poses/')):
        assert image.size==(128,160),n
        if native:return image.crop((16,16,112,144))
    return image
def pixels(a):
    return a.get_flattened_data() if hasattr(a,'get_flattened_data') else a.getdata()
def compose(names):
    a=Image.new('RGBA',(96,128))
    for n in names:a=Image.alpha_composite(a,load(n))
    return a
def visible(a):
    return [p if p[3] else (0,0,0,0) for p in pixels(a)]
def covered(a,b):
    return all(not p[3] or q[3] for p,q in zip(pixels(a),pixels(b)))
def same(a,b):
    return visible(a)==visible(b)
recipes=json.loads((ROOT/'expressions/recipes.json').read_text())
poses=json.loads((ROOT/'poses/recipes.json').read_text())['poses']
animation=json.loads((ROOT/'animation.json').read_text())
assert animation['canvas']==[128,160] and animation['foot']==[64,140]
manifest=json.loads((ROOT/'atlas-manifest.json').read_text())
palette={tuple(bytes.fromhex(c[1:])) for c in json.loads((ROOT/'source/palette.json').read_text())}
paths=sorted({p for folder in ['eyes','mouth','expressions','poses','frames'] for p in (ROOT/folder).rglob('*.png')})
union=set()
for p in paths:
    im=Image.open(p)
    expected_size=(128,160) if p.relative_to(ROOT).parts[0] in ['poses','frames'] else (96,128)
    assert im.size==expected_size and im.mode=='RGBA',p
    assert set(pixels(im.getchannel('A'))) <= {0,255},p
    colors={v[:3] for v in pixels(im) if v[3]}
    assert colors<=palette,p
    union.update(colors)
assert len(union)<=80
assert len({load('eyes/'+n+'.png').tobytes() for n in recipes['eyes']})==5
assert len({load('mouth/'+n+'.png').tobytes() for n in recipes['mouth']})==4
assert same(load('frames/neutral.png'),load('neutral.png'))
assert same(load('frames/base/neutral.png'),load('base-neutral.png'))
allowed=Image.alpha_composite(load('source/expression-eye-mask.png'),load('source/expression-mouth-mask.png'))
neutral=load('frames/neutral.png')
for n in recipes['recipes']:
    im=load('expressions/'+n+'.png')
    for prefix in ['','base/']:
        assert same(load(f'frames/{prefix}{n}.png'),load(f'source/outline-before/frames/{prefix}{n}.png')),('approved expression changed',n,prefix)
    assert all(p==q or m[3] for p,q,m in zip(pixels(neutral),pixels(im),pixels(allowed))),n
    assert im.getchannel('A').tobytes()==neutral.getchannel('A').tobytes(),n
for n,recipe in poses.items():
    full=load(f'poses/{n}/composed.png')
    assert same(compose(recipe['layers']),full),n
    basic=[f'poses/{n}/'+p for p in ['body/lower-base.png','body/upper-base.png','head-base.png']]
    foundation=load(f'poses/{n}/complete-underlayer.png')
    assert same(compose(basic),foundation),n
    assert covered(foundation,load(f'poses/{n}/base.png')) and covered(foundation,full),n
    for feature in ['eyes/open.png','mouth/rest.png']:
        assert covered(load(f'poses/{n}/'+feature),load(f'poses/{n}/head-base.png')),(n,feature)
    if n.startswith('walk') or n.startswith('give'):
        assert same(compose(basic[:2]),load(f'poses/{n}/source-body.png')),('partition loss',n)
        assert load(f'poses/{n}/body/lower-base.png').getbbox()[3]==125,('ground',n)
        dx,dy=recipe['body_registration']['upper_translation']
        unchanged_upper=Image.new('RGBA',(96,128))
        unchanged_upper.paste(load(f'poses/{n}/source-upper-untranslated.png'),(dx,dy))
        assert same(unchanged_upper,load(f'poses/{n}/body/upper-base.png')),('upper contour resampled',n)
        # Compare shoulder occupancy to the full generated source, independently
        # of the builder's partition mask. This catches the former neck-width crop.
        source=load(recipe['source'])
        actual=load(f'poses/{n}/body/upper-base.png')
        for y in range(45,55):
            for x in range(20,39):
                sx=int((x+.5)*source.width/96);sy=int((y+.5)*source.height/128)
                if source.getpixel((sx,sy))[3]>=128:
                    assert actual.getpixel((x+dx,y+dy))[3],('shoulder clipped',n,x,y)
    assert all(v is not None and 16<=v[0]<112 and 16<=v[1]<144 for v in recipe['sockets_px'].values()),n
raw_profile=load('source/poses/turn-away-native.png')
for n,near,far in [('nod-down',(39,29,49,37),(52,26,57,37)),
                   ('turn-toward',(38,24,49,37),(52,23,62,37)),
                   ('turn-away',(43,28,51,36),(55,25,59,36))]:
    current=load(f'poses/{n}/head-base.png')
    previous=load(f'source/outline-before/poses/{n}/head-base.png')
    if n=='turn-toward':
        for y in range(128):
            for x in range(96):
                if not (41<=x<54 and 43<=y<48):
                    assert current.getpixel((x,y))==previous.getpixel((x,y)),('unrelated head change',n,x,y)
    for box in [near,far]:
        eye=list(pixels(load(f'poses/{n}/eyes/open.png').crop(box)))
        assert any(p[3] and p[0]>205 and p[1]>170 and p[2]>145 for p in eye),('open eye lost sclera',n,box)
        assert any(p[3] and p[0]<150 and p[1]<110 for p in eye),('open eye lost iris',n,box)
registered_profile=load('source/face-consistency/turn-away-registered.png')
blank_profile=load('poses/turn-away/head-base.png')
assert blank_profile.getchannel('A').tobytes()==registered_profile.getchannel('A').tobytes(),'far-eye silhouette cut'
profile=compose(['poses/turn-away/head-base.png','poses/turn-away/eyes/open.png','poses/turn-away/mouth/rest.png'])
for y in range(128):
    for x in range(96):
        if not (49<=x<52 and 36<=y<38):
            assert profile.getpixel((x,y))==registered_profile.getpixel((x,y)),('angle face changed during separation',x,y)
assert same(load('poses/turn-toward/eyes/open.png'),load('source/outline-before/poses/turn-toward/eyes/open.png')),'toward eye design changed'
registered_nod=load('source/nod-consistency/nod-registered.png')
assert load('poses/nod-down/head-base.png').getchannel('A').tobytes()==registered_nod.getchannel('A').tobytes(),'nod silhouette cut'
nod=compose(['poses/nod-down/head-base.png','poses/nod-down/eyes/open.png','poses/nod-down/mouth/rest.png'])
for y in range(128):
    for x in range(96):
        if not (45<=x<48 and 37<=y<39):
            assert nod.getpixel((x,y))==registered_nod.getpixel((x,y)),('nod face changed during separation',x,y)
for part in ['head-base.png','eyes/open.png','mouth/rest.png']:
    translated=Image.new('RGBA',(96,128))
    translated.paste(load(part),(0,1))
    assert same(translated,load('poses/nod-mid/'+part)),('nod preparation changed face design',part)
x0,y0,x1,y1=blank_profile.getbbox()
assert 50<=x1-x0<=52 and 38<=y1-y0<=40,'away head scale drift'
for n in animation['frames']:
    if n not in ['turn-toward','turn-away','nod-mid','nod-down']:
        for prefix in ['','base/']:
            assert same(load(f'frames/{prefix}{n}.png'),load(f'source/face-consistency/before-frames/{prefix}{n}.png')),('unrelated frame changed',n,prefix)
    if n not in ['nod-mid','nod-down']:
        for prefix in ['','base/']:
            assert same(load(f'frames/{prefix}{n}.png'),load(f'source/nod-consistency/before-frames/{prefix}{n}.png')),('non-nod frame changed',n,prefix)
gait=[]
for k in range(6):
    n=f'walk-{k}'
    leg=load(f'poses/{n}/body/lower-base.png')
    assert leg.getbbox()[3]==125,('ground',n)
    gold=[(x,y) for y in range(98,128) for x in range(96)
          if (lambda c:c[3] and c[0]>180 and c[1]>125 and c[2]<110)(leg.getpixel((x,y)))]
    assert gold,('near boot buckle missing',n)
    gait.append({'key':k,'near_hand':poses[n]['sockets_px']['right_hand'],
                 'near_boot_gold_center':[sum(x for x,y in gold)/len(gold),sum(y for x,y in gold)/len(gold)]})
    assert not poses[n]['held_hide'],n
for i,j in [(0,3),(1,4)]:
    assert (gait[i]['near_hand'][0]-gait[j]['near_hand'][0])*(gait[i]['near_boot_gold_center'][0]-gait[j]['near_boot_gold_center'][0])<0,'same-side arm/leg phase'
for k in [2,5]:
    assert abs(gait[k]['near_hand'][0]-46)<15,'passing arm did not come under shoulder'
assert sum(f.get('event')=='transfer' for f in animation['clips']['give']['frames'])==1
assert animation['clips']['give']['frames'][3]['sprite']=='give-extended-empty'
assert any(pixels(load('poses/give-extended/props/leather-hide.png').getchannel('A')))
for n,c in animation['clips'].items():
    assert all(f['sprite'] in animation['frames'] and f['duration_ms']>0 for f in c['frames']),n
page=load('atlas.png')
for sprite in manifest['sprites']:
    x,y,w,h=sprite['rect'];ox,oy=sprite['trim_offset']
    expected=load(sprite['source'], native=False)
    restored=Image.new('RGBA',tuple(sprite['source_size']))
    restored.paste(page.crop((x,y,x+w,y+h)),(ox,oy))
    assert same(expected,restored),sprite['id']
    for yy in range(-2,h+2):
        for xx in range(-2,w+2):
            p=page.getpixel((x+xx,y+yy))
            if xx in (-2,w+1) or yy in (-2,h+1):assert p[3]==0,('gutter',sprite['id'])
            elif xx in (-1,w) or yy in (-1,h):
                q=page.getpixel((x+max(0,min(w-1,xx)),y+max(0,min(h-1,yy))))
                assert (p if p[3] else (0,0,0,0))==(q if q[3] else (0,0,0,0)),('extrude',sprite['id'])
audit_script=REPO/'.agents/skills/sprite-optimization/scripts/audit_sprites.py'
audit=subprocess.run(['python3',str(audit_script),'--manifest',str(ROOT/'atlas-manifest.json')],capture_output=True,text=True,check=True)
audit=json.loads(audit.stdout)
(ROOT/'qa/sprite-audit.json').write_text(json.dumps(audit,indent=2)+'\n')
assert not audit['errors']
playback=json.loads((ROOT/'qa/playback-report.json').read_text())
assert len(playback['walk'])==8 and all(m['keys_captured']==list(range(6)) for m in playback['walk'])
for mode in playback['walk']:
    indices=[int(t['src'].rsplit('walk-',1)[1].split('.')[0]) for t in mode['transitions']]
    assert set(indices)==set(range(6)),mode
    unique=[]
    for i in indices:
        if not unique or unique[-1]!=i:unique.append(i)
    assert all(j==(i+1)%6 for i,j in zip(unique,unique[1:])),('playback order',mode)
    scale,gear,speed=mode['scale'],mode['gear'],mode['speed']
    for k in range(6):
        src=load(f'frames/{"" if gear else "base/"}walk-{k}.png')
        expected=Image.new('RGBA',(96,128),'#24232a' if gear else '#efe5cd')
        expected.alpha_composite(src)
        expected=expected.convert('RGB').resize((96*scale,128*scale),Image.Resampling.NEAREST)
        capture=Image.open(ROOT/f'qa/playback-captures/walk-{scale}x-{"equipped" if gear else "base"}-{speed}speed-key-{k}.png').convert('RGB')
        assert capture.size==expected.size and capture.tobytes()==expected.tobytes(),('browser rendered pixels',mode,k)
gif_frames_checked=0
gesture_captures_checked=0
for mode in playback['gesture_captures']:
    name=mode['name']
    frames=animation['clips'][name]['frames']
    assert mode['keys_captured']==list(range(len(frames))),('gesture capture missing',name)
    for k,frame in enumerate(frames):
        expected=Image.new('RGBA',(96,128),'#efe5cd')
        expected.alpha_composite(load(f'frames/base/{frame["sprite"]}.png'))
        expected=expected.convert('RGB').resize((384,512),Image.Resampling.NEAREST)
        capture=Image.open(ROOT/f'qa/playback-captures/{name}-base-key-{k}.png').convert('RGB')
        assert capture.size==expected.size and capture.tobytes()==expected.tobytes(),('gesture browser pixels',name,k)
        gesture_captures_checked+=1
for name in ['talk','blink','nod','shake_head','give','walk','talk_blink']:
    for basic in [False,True]:
        for speed in [1,.25] if name=='walk' else [1]:
            suffix=('-base' if basic else '')+('-slow' if speed==.25 else '')
            gif=Image.open(ROOT/f'previews/{name}{suffix}.gif')
            frames=animation['clips'][name]['frames']
            assert gif.n_frames==len(frames),(name,suffix)
            for k,frame in enumerate(frames):
                gif.seek(k)
                expected=Image.new('RGBA',(128,160),'#eee5cd')
                expected.alpha_composite(load(f'frames/{"base/" if basic else ""}{frame["sprite"]}.png', native=False))
                expected=expected.convert('RGB').resize((512,640),Image.Resampling.NEAREST)
                assert gif.convert('RGB').tobytes()==expected.tobytes(),('GIF pixel fidelity',name,suffix,k)
                assert gif.info['duration']==round(frame['duration_ms']/10/speed)*10,('GIF duration',name,k)
                gif_frames_checked+=1
def outputs():
    files=paths+[ROOT/'atlas.png',ROOT/'atlas-manifest.json',ROOT/'animation.json',ROOT/'poses/recipes.json']
    files+=list((ROOT/'previews').glob('*.gif'))+list((ROOT/'previews').glob('*.html'))
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
before=outputs()
subprocess.run(['node',str(ROOT/'source/build-animation.mjs')],check=True,stdout=subprocess.DEVNULL)
assert before==outputs(),'nondeterministic rebuild'
report={'status':'structural_pass','native_files':len(paths),'shared_colors':len(union),
        'eye_states':5,'mouth_states':4,'expression_recipes':8,'pose_recipes':len(poses),
        'clips':len(animation['clips']),'atlas_frames':len(manifest['sprites']),
        'atlas_rgba8_bytes':page.width*page.height*4,'atlas_size':list(page.size),
        'foundation_lost_pixels':0,'expression_outside_feature_changes':0,
        'source_shoulders_preserved':True,'upper_contours_rigid':True,
        'approved_expressions_unchanged':True,'angle_eyes_retain_sclera_and_iris':True,
        'same_side_arm_leg_opposite':True,'gait_measurements':gait,
        'transfer_events':1,'atlas_roundtrip_exact':True,'extrude_gutter_exact':True,
        'archived_native_browser_playback_modes':8,'current_canvas_browser_playback':'not_repeated','png_json_gif_rebuild_identical':True,
        'archived_native_browser_nearest_pixel_exact_captures':48,'gif_pixel_exact_frames':gif_frames_checked,
        'archived_native_gesture_browser_pixel_exact_captures':gesture_captures_checked,
        'runtime_draw_calls':'not_measured','runtime_integration':'not_integrated'}
(ROOT/'qa/animation-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='gait_measurements'},indent=2))
