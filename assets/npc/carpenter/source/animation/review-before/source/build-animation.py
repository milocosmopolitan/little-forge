"""Carpenter expressions and registered pose animations; approved neutral stays locked."""
from pathlib import Path
from PIL import Image, ImageDraw
import argparse, base64, hashlib, io, json, math, os, subprocess, sys
ROOT=Path(__file__).resolve().parent.parent
SIZE=(128,160)
OFFSET=(16,16)
ANCHOR=(64,140)
EQUIPMENT=['apron-tools','straps','belt-pouches','towel','boards','mallet','gloves']
RECIPES={'neutral':['open','rest'],'talk-small':['open','small-open'],
         'talk-wide':['open','wide-open'],'talk-round':['open','round'],
         'blink':['closed','rest'],'happy':['happy-closed','rest'],
         'surprised':['wide','round'],'relaxed':['half','rest']}


def read(name):
    return Image.open(ROOT / name).convert('RGBA')

def blank(size=SIZE):
    return Image.new('RGBA', size)

def save(im, name):
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    im.save(path)
    return im

def write_json(name, value):
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')

def compose(images, size=SIZE):
    im = blank(size)
    for layer in images:
        im.alpha_composite(layer)
    return im

def embed(im, xy=OFFSET):
    out = blank()
    out.alpha_composite(im, xy)
    return out

def translate(im, dx, dy):
    out = blank(im.size)
    out.alpha_composite(im, (dx, dy))
    return out

def polygon_mask(points, size=SIZE):
    mask = Image.new('L', size)
    ImageDraw.Draw(mask).polygon(points, fill=255)
    return mask

def cut(im, points):
    out = blank(im.size)
    out.paste(im, (0, 0), polygon_mask(points, im.size))
    return out

def opaque(im):
    return {i for i, px in enumerate(im.getdata()) if px[3]}

def rgba_pixels(im):
    return bytes(v for px in im.getdata() for v in (px if px[3] else (0,0,0,0)))

def colors_in(images):
    return sorted({px[:3] for im in images for px in im.getdata() if px[3]})

def normalize_candidate(im, palette):
    alpha = im.getchannel('A').point(lambda a: 255 if a >= 128 else 0)
    out = im.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE).convert('RGBA')
    out.putalpha(alpha)
    return out

def contact(items, name, columns=4, scale=4, crop=None, dark=False):
    cw, ch = ((crop[2]-crop[0], crop[3]-crop[1]) if crop else items[0][1].size)
    tile_w, tile_h = cw*scale+16, ch*scale+32
    rows = math.ceil(len(items)/columns)
    bg, fg = ('#302b32', '#eee3cc') if dark else ('#eee3cc', '#211510')
    out = Image.new('RGB', (columns*tile_w, rows*tile_h), bg)
    d = ImageDraw.Draw(out)
    for i,(label,im) in enumerate(items):
        if crop:
            im = im.crop(crop)
        im = im.resize((cw*scale,ch*scale),Image.Resampling.NEAREST)
        x,y = i%columns*tile_w, i//columns*tile_h
        d.text((x+8,y+8),label,fill=fg)
        out.paste(im,(x+8,y+26),im)
    save(out,name)

def make_previews(frames,base_frames,clips,colors):
    for name,clip in clips.items():
        for speed in [1,4]:
            images=[]
            for key in clip['frames']:
                out=Image.new('RGB',(SIZE[0]*8+48,SIZE[1]*4+40),'#eee3cc')
                d=ImageDraw.Draw(out)
                d.text((12,10),name+' / empty equipment',fill='#211510')
                d.text((SIZE[0]*4+32,10),'equipped / '+key['sprite'],fill='#211510')
                for i,bank in enumerate([base_frames,frames]):
                    im=bank[key['sprite']].resize((SIZE[0]*4,SIZE[1]*4),Image.Resampling.NEAREST)
                    out.paste(im,(12+i*(SIZE[0]*4+24),32),im)
                images.append(out)
            images[0].save(ROOT/f'previews/{name}{"-slow" if speed==4 else ""}.gif',
                save_all=True,append_images=images[1:],duration=[k['duration_ms']*speed for k in clip['frames']],
                loop=0,disposal=2,optimize=False)

    def data(im):
        buf=io.BytesIO();im.save(buf,format='PNG')
        return 'data:image/png;base64,'+base64.b64encode(buf.getvalue()).decode()
    payload={'clips':clips,'frames':{k:{'base':data(base_frames[k]),'equipped':data(v)} for k,v in frames.items()}}
    html='''<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Carpenter 표정·포즈 검수</title><style>
*{box-sizing:border-box}body{margin:0;background:#18191d;color:#eee3cc;font:16px system-ui,sans-serif}main{max-width:1180px;margin:auto;padding:28px}
h1{font-size:24px;margin:0 0 8px}p{color:#aaa}button,select{font:inherit;background:#35363e;color:#fff;border:1px solid #5a5b65;border-radius:7px;padding:8px 14px}button{cursor:pointer}nav{display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin:22px 0}label{display:flex;gap:8px;align-items:center}.stage{display:grid;grid-template-columns:1fr 1fr;gap:16px}figure{margin:0;background:#eee3cc;border-radius:12px;text-align:center;overflow:hidden}figcaption{padding:12px;background:#282930;color:#fff}canvas{width:512px;height:640px;max-width:100%;image-rendering:pixelated;object-fit:contain}#status{font-variant-numeric:tabular-nums;min-height:26px;margin:16px 0}.grid{display:flex;gap:8px;flex-wrap:wrap;margin-top:18px}.hint{font-size:14px}.dark figure{background:#302b32}@media(max-width:800px){.stage{grid-template-columns:1fr}canvas{width:256px;height:320px}}
</style><main><h1>Carpenter 표정·포즈</h1><p>같은 프레임의 장비 없는 바탕과 착용본을 나란히 확인합니다.</p>
<nav><label>동작 <select id="clip"></select></label><label>속도 <select id="speed"><option value="1">1× 정상</option><option value="0.25">0.25× 저속</option></select></label><label>배율 <select id="scale"><option value="4">4× 픽셀</option><option value="1">1× 원본</option></select></label><button id="play">일시정지</button><button id="next">다음 키</button><button id="background">어두운 배경</button></nav>
<div class="stage"><figure><figcaption>장비 없는 언더레이어</figcaption><canvas id="base" width="128" height="160"></canvas></figure><figure><figcaption>장비 착용</figcaption><canvas id="equipped" width="128" height="160"></canvas></figure></div>
<div id="status" aria-live="off"></div><div class="grid" id="expressions"></div><p class="hint">재생 미리보기는 비반복 동작도 끝에서 400ms 후 처음부터 다시 보여줍니다. 건네기 4번째 키의 transfer 이벤트 뒤에는 망치가 없습니다. 게임 런타임에는 연결하지 않았습니다.</p>
<script>const DATA=__DATA__;const names={idle:'대기·깜빡임',blink:'깜빡임',talk:'말하기',happy:'기쁨',surprised:'놀람',nod:'끄덕임',shake_head:'고개젓기',give:'건네기'};
const clip=document.querySelector('#clip'),speed=document.querySelector('#speed');for(const key of Object.keys(DATA.clips)){const o=document.createElement('option');o.value=key;o.textContent=names[key]||key;clip.append(o)}clip.value='nod';let playing=true,index=0,elapsed=0,last=performance.now(),manual=null;
const images={};for(const [id,states]of Object.entries(DATA.frames)){images[id]={};for(const[k,v]of Object.entries(states)){const im=new Image();im.src=v;images[id][k]=im}}
function draw(){const seq=DATA.clips[clip.value].frames,key=manual?{sprite:manual}:seq[index];for(const kind of ['base','equipped']){const ctx=document.querySelector('#'+kind).getContext('2d');ctx.imageSmoothingEnabled=false;ctx.clearRect(0,0,128,160);const im=images[key.sprite][kind];if(im.complete)ctx.drawImage(im,0,0)}document.querySelector('#status').textContent=manual?'표정: '+manual:((names[clip.value]||clip.value)+' · 키 '+(index+1)+'/'+seq.length+' · '+key.sprite+' · '+key.duration_ms+'ms'+(key.event?' · '+key.event:''))}
function tick(now){const delta=now-last;last=now;if(playing&&!manual){elapsed+=delta*Number(speed.value);const seq=DATA.clips[clip.value].frames;const duration=seq[index].duration_ms+(index===seq.length-1&&!DATA.clips[clip.value].loop?400:0);if(elapsed>=duration){elapsed%=duration;index=(index+1)%seq.length}}draw();requestAnimationFrame(tick)}
document.querySelector('#scale').onchange=()=>{const scale=Number(document.querySelector('#scale').value);for(const kind of ['base','equipped']){const canvas=document.querySelector('#'+kind);canvas.style.width=128*scale+'px';canvas.style.height=160*scale+'px'}};clip.onchange=()=>{index=0;elapsed=0;manual=null;draw()};document.querySelector('#play').onclick=()=>{playing=!playing;document.querySelector('#play').textContent=playing?'일시정지':'재생'};document.querySelector('#next').onclick=()=>{playing=false;document.querySelector('#play').textContent='재생';manual=null;index=(index+1)%DATA.clips[clip.value].frames.length;elapsed=0;draw()};document.querySelector('#background').onclick=()=>{const on=document.body.classList.toggle('dark');document.querySelector('#background').textContent=on?'밝은 배경':'어두운 배경'};
for(const id of ['neutral','talk-small','talk-wide','talk-round','blink','happy','surprised','relaxed']){const b=document.createElement('button');b.textContent=id;b.onclick=()=>{manual=id;draw()};document.querySelector('#expressions').append(b)}requestAnimationFrame(tick);
</script></main></html>'''
    (ROOT/'previews/animation.html').write_text(html.replace('__DATA__',json.dumps(payload,separators=(',',':'))))

def expression_parts(c):
    original=read('eyes/open.png');head=read('head-base.png')
    shared=original.copy()
    for y in range(26,35):
        for x in range(45,66):
            interior=all(0<=x+dx<96 and 0<=y+dy<128 and head.getpixel((x+dx,y+dy))[3]
                         for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)])
            if interior:shared.putpixel((x,y),(0,0,0,0))
    dark,white,shadow=c('#281d18'),c('#fffaf2'),c('#b36739')
    eyes={'open':original}
    for state in ['half','closed','happy-closed','wide']:
        im=shared.copy();d=ImageDraw.Draw(im)
        if state=='half':
            d.polygon([(48,30),(53,30),(53,33),(49,33),(47,31)],fill=white)
            d.rectangle((51,30,53,32),fill=dark)
            d.line([(47,30),(49,29),(53,29),(54,30)],fill=dark)
            d.polygon([(61,30),(63,30),(63,33),(61,32)],fill=white)
            d.line([(63,30),(63,33)],fill=dark)
            d.line([(60,30),(62,29),(63,30)],fill=dark)
        elif state=='closed':
            d.line([(47,31),(49,32),(52,32),(54,30)],fill=dark)
            d.line([(60,31),(61,32),(63,31)],fill=dark)
            d.line([(49,33),(52,33)],fill=shadow)
        elif state=='happy-closed':
            # Low smiling arcs, distinct from a pointed caret or resting blink.
            d.line([(47,31),(49,30),(52,30),(54,31)],fill=dark)
            d.line([(60,31),(61,30),(62,30),(63,31)],fill=dark)
            d.line([(49,33),(52,33)],fill=shadow)
        else:
            # Preserve the approved eye width and rightward pupils, opening only the lids.
            im=original.copy();d=ImageDraw.Draw(im)
            d.polygon([(48,28),(50,27),(53,27),(54,29),(54,32),(52,34),(49,34),(47,32),(47,30)],fill=white,outline=dark)
            d.rectangle((51,28,53,32),fill=dark)
            d.point((51,28),fill=white)
            d.polygon([(61,28),(63,28),(64,30),(64,33),(62,34),(60,32),(60,30)],fill=white,outline=dark)
            d.line([(63,29),(63,32)],fill=dark)
        for y in range(21,36):
            for x in range(44,67):
                if not head.getpixel((x,y))[3]:im.putpixel((x,y),(0,0,0,0))
        eyes[state]=save(im,f'eyes/{state}.png')
    mouths={'rest':read('mouth/rest.png')}
    for state in ['small-open','wide-open','round']:
        im=blank((96,128));d=ImageDraw.Draw(im)
        if state=='small-open':
            d.polygon([(54,36),(58,36),(59,37),(58,39),(55,39),(54,38)],fill=dark)
            d.line([(55,36),(57,36)],fill=c('#f1d1af'))
        elif state=='wide-open':
            d.polygon([(53,35),(59,35),(60,36),(59,38),(57,40),(55,39),(53,37)],fill=dark)
            d.line([(54,36),(58,36)],fill=white)
            d.line([(55,39),(57,39)],fill=shadow)
        else:
            d.polygon([(56,35),(58,35),(59,36),(59,38),(58,39),(56,39),(55,38),(55,36)],fill=dark)
            d.line([(56,36),(56,37)],fill=c('#754027'))
        mouths[state]=save(im,f'mouth/{state}.png')
    return eyes,mouths

def build():
    locks=json.loads((ROOT/'source/animation/baseline-lock.json').read_text())
    for path,digest in locks.items():
        assert hashlib.sha256(read(path).tobytes()).hexdigest()==digest,f'Approved neutral changed: {path}'
    palette=Image.open(ROOT/'source/palette-locked.png').convert('P')
    entries=palette.getpalette();colors=[tuple(entries[i:i+3]) for i in range(0,96,3)]
    def c(value):
        target=tuple(bytes.fromhex(value.lstrip('#')))
        return min(colors,key=lambda p:sum((a-b)**2 for a,b in zip(p,target)))+(255,)
    eyes,mouths=expression_parts(c)
    base_parts={'lower':embed(read('body/lower-base.png')),'upper':embed(read('body/upper-base.png')),
                'head':embed(read('head-base.png')),'eyes':embed(eyes['open']),'mouth':embed(mouths['rest'])}
    gear={n:embed(read(f'equipment/{n}.png')) for n in EQUIPMENT}
    frames,bases,recipes={},{},{}
    sockets={'neck':[66,62],'waist':[66,98],'eyes':[[67,46],[78,47]],'mouth':[72,54],
             'screen_left_hand':[48,94],'screen_right_hand':[93,96]}
    def export(name,parts,equipment,points=None,notes=None):
        base_order=[k for k in ['lower','upper','arm','neck','head','eyes','mouth'] if k in parts]
        order=base_order+list(equipment)
        base=compose([parts[k] for k in base_order]);all_parts=dict(parts,**equipment)
        full=compose([all_parts[k] for k in order]);assert not opaque(base)-opaque(full)
        layer_paths={};equipment_paths={}
        for key,im in parts.items():
            path=f'poses/{name}/layers/{key}.png';save(im,path);layer_paths[key]=path
        for key,im in equipment.items():
            path=f'poses/{name}/equipment/{key}.png';save(im,path);equipment_paths[key]=path
        for path,im in [(f'poses/{name}/base.png',base),(f'poses/{name}/composed.png',full),
                        (f'frames/{name}.png',full),(f'frames/base/{name}.png',base)]:save(im,path)
        frames[name]=full;bases[name]=base
        recipes[name]={'layers':layer_paths,'equipment':equipment_paths,'base_order':base_order,
            'composition_order':order,'sockets_px':points or sockets,'source_size':list(SIZE),
            'foot_anchor_px':list(ANCHOR),'notes':notes or []}
        return base,full
    for name,(eye,mouth) in RECIPES.items():
        parts=dict(base_parts,eyes=embed(eyes[eye]),mouth=embed(mouths[mouth]))
        base,full=export(name,parts,gear)
        save(full.crop((16,16,112,144)),f'expressions/{name}.png')
        save(base.crop((16,16,112,144)),f'expressions/base/{name}.png')
    assert rgba_pixels(frames['neutral'])==rgba_pixels(embed(read('neutral.png')))
    for section,states in [('eyes',eyes),('mouth',mouths)]:
        combined=blank((96,128))
        for im in states.values():combined.alpha_composite(im)
        mask_im=blank((96,128));mask_im.putalpha(combined.getchannel('A'))
        save(mask_im,f'source/animation/expression-{section}-mask.png')
    contact([(n,frames[n]) for n in RECIPES],'previews/expressions-4x.png',columns=4,scale=4,crop=(42,18,87,63))
    contact([(n,frames[n]) for n in RECIPES],'previews/expressions-dark-4x.png',columns=4,scale=4,crop=(42,18,87,63),dark=True)
    combinations=[]
    for eye,eye_im in eyes.items():
        for mouth,mouth_im in mouths.items():
            im=compose([base_parts['lower'],base_parts['upper'],base_parts['head'],embed(eye_im),embed(mouth_im)]+list(gear.values()))
            combinations.append((eye+'/'+mouth,im))
    contact(combinations,'previews/eye-mouth-combinations.png',columns=4,scale=3,crop=(42,18,87,63))
    write_json('expressions/recipes.json',{'eyes':{n:f'eyes/{n}.png' for n in eyes},
        'mouth':{n:f'mouth/{n}.png' for n in mouths},'recipes':{n:{'eyes':e,'mouth':m} for n,(e,m) in RECIPES.items()},
        'eyebrow_owner':'eyes','blink_priority':'surprised locks automatic blink'})
    return locals()


def package(state):
    frames,bases,recipes=state['frames'],state['bases'],state['recipes']
    clips={}
    def key(sprite,ms,event=None):
        result={'sprite':sprite,'duration_ms':ms}
        if event:result['event']=event
        return result
    clips['idle']={'loop':True,'frames':[key('neutral',1400),key('relaxed',40),key('blink',80),key('relaxed',40),key('neutral',1800)]}
    clips['blink']={'loop':False,'frames':[key('neutral',120),key('relaxed',40),key('blink',80),key('relaxed',40),key('neutral',120)]}
    clips['talk']={'loop':True,'frames':[key('neutral',100),key('talk-small',100),key('talk-wide',120),key('talk-small',90),key('talk-round',100),key('neutral',120)]}
    clips['happy']={'loop':False,'frames':[key('neutral',160),key('happy',700),key('neutral',160)]}
    clips['surprised']={'loop':False,'frames':[key('neutral',160),key('surprised',600),key('neutral',180)]}
    clips['nod']={'loop':False,'frames':[key('neutral',120),key('nod-mid',100),key('nod-down',140),key('nod-mid',100),key('neutral',120)]}
    clips['shake_head']={'loop':False,'frames':[key('neutral',100),key('turn-left',140),key('neutral',100),key('turn-right',140),key('neutral',100)]}
    clips['give']={'loop':False,'frames':[key('neutral',120),key('give-mid',120),key('give-full',180),key('give-empty',120,'transfer'),key('give-retract',120),key('neutral-empty',120)]}
    write_json('animation.json',{'version':1,'id':'carpenter','canvas':list(SIZE),'foot_anchor_px':list(ANCHOR),
        'frames':sorted(frames),'clips':clips,'runtime_status':'authoring_export_not_integrated'})
    write_json('poses/recipes.json',{'canvas':list(SIZE),'offset_from_base':list(OFFSET),'poses':recipes,
        'scope':['neutral','nod','shake_head','give'],'walk':'not_generated','head_only_clothing':'fixed original upper/lower and equipment',
        'sides':'screen-right arm is anatomical left; screen-left supporting hand and boards remain fixed'})
    columns=8;cell=(132,164);rows=math.ceil(len(frames)/columns)
    page_size=(math.ceil(columns*cell[0]/16)*16,math.ceil(rows*cell[1]/16)*16)
    atlas=Image.new('RGBA',page_size);sprites=[]
    for i,name in enumerate(sorted(frames)):
        im=frames[name];x=i%columns*cell[0]+2;y=i//columns*cell[1]+2
        atlas.paste(im,(x,y))
        atlas.paste(im.crop((0,0,1,160)),(x-1,y));atlas.paste(im.crop((127,0,128,160)),(x+128,y))
        atlas.paste(im.crop((0,0,128,1)),(x,y-1));atlas.paste(im.crop((0,159,128,160)),(x,y+160))
        for dx,dy,sx,sy in [(-1,-1,0,0),(128,-1,127,0),(-1,160,0,159),(128,160,127,159)]:atlas.putpixel((x+dx,y+dy),im.getpixel((sx,sy)))
        sprites.append({'id':name,'page':'carpenter-0','rect':[x,y,128,160],'source_size':list(SIZE),
            'trim_offset':[0,0],'pivot':list(ANCHOR),'source':f'frames/{name}.png','rotated':False,
            'sha256_pixels':hashlib.sha256(rgba_pixels(im)).hexdigest()})
    save(atlas,'atlas.png')
    write_json('atlas-manifest.json',{'version':1,'sampler':'nearest','mipmaps':False,'extrude_px':1,'gutter_px':1,
        'trimmed':False,'runtime_status':'authoring_export_not_integrated','pages':[{'id':'carpenter-0','path':'atlas.png','size':list(page_size)}],
        'sprites':sprites,'clips':clips})
    names=['neutral','nod-mid','nod-down','turn-left','turn-right','give-mid','give-full','give-empty','give-retract','neutral-empty']
    for suffix,bank in [('',frames),('-base',bases)]:
        contact([(n,bank[n]) for n in names],f'previews/poses{suffix}-3x.png',columns=5,scale=3)
        contact([(n,bank[n]) for n in names],f'previews/poses{suffix}-dark-3x.png',columns=5,scale=3,dark=True)
    for name,im in frames.items():save(im.resize((512,640),Image.Resampling.NEAREST),f'previews/frames/{name}-4x.png')
    make_previews(frames,bases,clips,state['colors'])
    spec=json.loads((ROOT/'character-spec.yaml').read_text())
    spec['scope']='8 expressions; nod, shake_head and mallet-give pose animations'
    spec['pose_canvas']={'grid':'B','canvas_cells':[8,10],'native_canvas_px':list(SIZE),'base_offset_px':list(OFFSET),'foot_anchor_px':list(ANCHOR)}
    spec['layers']['eyes']={n:f'eyes/{n}.png' for n in state['eyes']}
    spec['layers']['mouth']={n:f'mouth/{n}.png' for n in state['mouths']}
    spec['expression_recipes']={n:{'eyes':e,'mouth':m} for n,(e,m) in RECIPES.items()}
    spec['expression_channels']={'eyes':'open/half/closed/half/open; automatic blink locked during surprise','mouth':'rest/small-open/wide-open/small-open/round/rest'}
    spec['animation']={'manifest':'animation.json','clips':list(clips),'duration_unit':'ms',
        'transfer':{'clip':'give','key_index_zero_based':3,'event':'transfer','count':1},'walk':'not_generated'}
    spec['pose_overrides']={'recipes':'poses/recipes.json','registration':'source/animation/registration.json'}
    spec['runtime_export'].update({'status':'authoring_export_not_integrated','manifest':'atlas-manifest.json','mode':'baked','frame_size':list(SIZE)})
    spec['production_plan']={'status':'generated','expressions':list(RECIPES),'pose_clips':['nod','shake_head','give'],'walk':'not_generated'}
    spec['generation']['animation_prompts']='source/animation/generation-prompts.json'
    spec['generation']['animation_builder']='source/build-animation.py'
    spec['verification']={'animation':'qa/animation-validation.json','playback':'qa/playback-review.json','static_atlas_audit':'qa/sprite-audit.json'}
    write_json('character-spec.yaml',spec)
    index=ROOT/'previews/index.html'
    if index.exists():
        html=index.read_text();html=html.replace('<h1>Carpenter 기본 자세</h1>','<h1>Carpenter 기본 자세</h1><p><a href="animation.html" style="color:#e5b78b">표정·포즈 애니메이션 보기</a></p>');index.write_text(html)
    return clips

def add_poses(state):
    c,palette=state['c'],state['palette'];base_parts,gear=state['base_parts'],state['gear']
    export,sockets=state['export'],state['sockets']
    registrations={'nod-down':{'size':[41,41],'xy':[44,21]},'turn-left':{'size':[41,41],'xy':[44,20]},'turn-right':{'size':[41,41],'xy':[44,20]}}
    head_parts={}
    eye_boxes={
        'nod-down':[(64,42,72,45),(62,46,72,52),(74,43,79,46),(73,47,79,53)],
        'turn-left':[(54,39,62,42),(57,42,65,49),(50,41,55,44),(50,44,55,49)],
        'turn-right':[(66,38,73,41),(63,41,73,49),(76,42,80,48)]}
    mouth_boxes={'nod-down':(66,53,74,57),'turn-left':(53,50,61,54),'turn-right':(69,52,75,55)}
    for name,contract in registrations.items():
        source=read(f'source/animation/candidates/{name}.png');bbox=source.getchannel('A').point(lambda a:255 if a>=128 else 0).getbbox()
        im=normalize_candidate(source.crop(bbox).resize(tuple(contract['size']),Image.Resampling.NEAREST),palette)
        complete=blank();complete.alpha_composite(im,tuple(contract['xy']))
        h=complete.copy();e=blank();m=blank()
        for box in eye_boxes[name]:
            for y in range(box[1],box[3]):
                for x in range(box[0],box[2]):
                    px=complete.getpixel((x,y))
                    interior=all(complete.getpixel((x+dx,y+dy))[3] for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)])
                    if interior and px[3] and (max(px[:3])<160 or min(px[:3])>220):
                        e.putpixel((x,y),px);h.putpixel((x,y),c('#fcb573'))
        box=mouth_boxes[name]
        for y in range(box[1],box[3]):
            for x in range(box[0],box[2]):
                px=complete.getpixel((x,y))
                if px[3] and 100<=px[0]<=200 and px[1]<130 and px[0]>px[1]*1.3 and px[0]>px[2]*1.6:
                    m.putpixel((x,y),px);h.putpixel((x,y),c('#fcb573'))
        assert rgba_pixels(compose([h,e,m]))==rgba_pixels(complete),name
        assert opaque(h)==opaque(complete),name
        save(complete,f'source/animation/native/{name}.png')
        head_parts[name]=(h,e,m)
    mid=dict(base_parts,head=translate(base_parts['head'],0,1),eyes=translate(base_parts['eyes'],0,1),mouth=translate(base_parts['mouth'],0,1))
    export('nod-mid',mid,gear,dict(sockets,mouth=[72,55],eyes=[[67,47],[78,48]]),
        ['Complete original head/eyes/mouth move together by one integer pixel; clothing fixed'])
    centers={'nod-down':([[67,49],[76,50]],[70,55]),'turn-left':([[61,46],[52,46]],[57,52]),'turn-right':([[68,45],[77,45]],[72,53])}
    for name,(h,e,m) in head_parts.items():
        eye_pos,mouth_pos=centers[name]
        export(name,dict(base_parts,head=h,eyes=e,mouth=m),gear,dict(sockets,eyes=eye_pos,mouth=mouth_pos),
            ['Genuine alternate head perspective; fixed native head scale and neck registration',
             'Original upper/lower clothing and all gear reused pixel-identically'])
    contact([(n,compose(p)) for n,p in head_parts.items()],'previews/head-pose-candidates.png',columns=3,scale=8,crop=(40,16,89,65))

    # Remove the old arm at its shoulder; retain the approved torso and its fitted hem.
    upper=base_parts['upper'].copy();ud=ImageDraw.Draw(upper)
    ud.rectangle((80,74,127,104),fill=(0,0,0,0))
    ud.line([(79,84),(79,96),(78,98)],fill=c('#281d18'))
    contracts=json.loads((ROOT/'source/animation/arm-contracts-draft.json').read_text())
    contracts['give-mid'].update({'palm':[104,84],'wrist':[99,85]})
    contracts['give-full'].update({'palm':[117,81],'wrist':[110,82]})
    arms={};gloves={}
    for name,contract in contracts.items():
        source=read(f'source/animation/candidates/{name}.png');rx,ry=contract['root'];scale=contract['scale']
        im=normalize_candidate(source.transform(SIZE,Image.Transform.AFFINE,
            (scale,0,rx-80*scale,0,scale,ry-74*scale),Image.Resampling.NEAREST),palette)
        points=[(round(80+(x-rx)/scale),round(74+(y-ry)/scale)) for x,y in contract['polygon']]
        arm=cut(im,points)
        # Restore only the isolated arm's sampled outer rim, leaving the shoulder seam open.
        alpha=arm.getchannel('A')
        for y in range(68,104):
            for x in range(82,127):
                if alpha.getpixel((x,y)) and any(not alpha.getpixel((x+dx,y+dy)) for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]):
                    arm.putpixel((x,y),c('#281d18'))
        assert arm.getchannel('A').getbbox()[2]<128,name
        arms[name]=arm;save(arm,f'source/animation/native/{name}-arm.png')
        glove=gear['gloves'].copy();gd=ImageDraw.Draw(glove);gd.rectangle((84,86,104,104),fill=(0,0,0,0))
        wx,wy=contract['wrist'];new=blank();nd=ImageDraw.Draw(new)
        glove_points=[(wx-2,wy-2),(wx+1,wy-3),(wx+4,wy-2),(wx+6,wy),(wx+5,wy+2),(wx+1,wy+3),(wx-2,wy+2)]
        nd.polygon(glove_points,fill=c('#5e3e2d'),outline=c('#281d18'))
        nd.line([(wx,wy-1),(wx+3,wy)],fill=c('#8b5939'))
        for y in range(160):
            for x in range(128):
                if not arm.getpixel((x,y))[3]:new.putpixel((x,y),(0,0,0,0))
        glove.alpha_composite(new);gloves[name]=glove
    contact(list(arms.items()),'previews/give-arm-candidates.png',columns=2,scale=6,crop=(76,68,128,103))
    mallet=read('source/mallet-complete.png');mallet=mallet.crop(mallet.getbbox()).transpose(Image.Transpose.ROTATE_90)
    for name in ['give-mid','give-full','give-empty','give-retract']:
        source='give-mid' if name in ['give-mid','give-retract'] else 'give-full'
        palm=contracts[source]['palm'];arm=arms[source]
        equipment={k:v for k,v in gear.items() if k!='mallet'};equipment['gloves']=gloves[source]
        if name in ['give-mid','give-full']:
            prop=blank();prop.alpha_composite(mallet,(palm[0]-27,palm[1]-17))
            for y in range(palm[1]-1,palm[1]+2):
                for x in range(palm[0]-3,min(palm[0]+5,128)):
                    if arm.getpixel((x,y))[3]:prop.putpixel((x,y),(0,0,0,0))
            # Keep the same rear-tool/front-glove order as the neutral grip.
            equipment={k:v for k,v in equipment.items() if k!='gloves'}
            equipment['mallet']=prop;equipment['gloves']=gloves[source]
        export(name,dict(base_parts,upper=upper,arm=arm),equipment,dict(sockets,screen_right_hand=palm),
            ['Replaced far arm and complete open palm; supporting hand and boards fixed',
             'Fingerless glove follows wrist; mallet uses exact quarter-turn transpose',
             'Mallet removed from transfer key onward' if name in ['give-empty','give-retract'] else 'Mallet shaft rests on open palm; fingers occlude only contact band'])
    export('neutral-empty',base_parts,{k:v for k,v in gear.items() if k!='mallet'},sockets,
        ['Approved neutral without transferred mallet; complete bare hand is retained'])
    write_json('source/animation/registration.json',{'canvas':list(SIZE),'base_offset':list(OFFSET),
        'head_registration':registrations,'head_eye_boxes':eye_boxes,'head_mouth_boxes':mouth_boxes,
        'arm_contracts':contracts,'arm_target_root':[80,74],'palette':'source/palette-locked.png'})

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--base-ready',action='store_true');parser.add_argument('--prepare',action='store_true');args=parser.parse_args()
    if not args.base_ready:
        subprocess.run([sys.executable,str(ROOT/'source/build-character.py')],env=dict(os.environ,CARPENTER_BASE_ONLY='1'),capture_output=True,check=True)
    state=build();add_poses(state);package(state)
    if not args.prepare:subprocess.run([sys.executable,str(ROOT/'source/verify-animation.py')],check=True)
    print('Built carpenter: eight expressions, three pose animations, 17 distinct frames')
