"""Deterministic chef expressions and three pose clips from retained native sources."""
from pathlib import Path
from PIL import Image, ImageDraw
import argparse
import base64
import hashlib
import io
import json
import math
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
SIZE = (128, 160)
OFFSET = (16, 16)
ANCHOR = (64, 140)
EQUIPMENT = ['apron', 'straps', 'belt-tools', 'towel', 'scarf', 'spoon']
RECIPES = {
    'neutral': ['open', 'rest'], 'talk-small': ['open', 'small-open'],
    'talk-wide': ['open', 'wide-open'], 'talk-round': ['open', 'round-talk'],
    'blink': ['closed', 'rest'], 'happy': ['happy-closed', 'laugh'],
    'surprised': ['wide', 'round'], 'relaxed': ['half', 'rest'],
}

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

def build():
    baseline = ROOT/'source/animation/baseline'
    locks = json.loads((ROOT/'source/animation/baseline-lock.json').read_text())
    for name,digest in locks.items():
        assert hashlib.sha256(read(name).tobytes()).hexdigest() == digest, f'Base changed: {name}'
    images = [Image.open(baseline/name).convert('RGBA') for name in locks]
    colors = colors_in(images)
    assert len(colors) <= 32
    palette = Image.new('P',(1,1))
    palette.putpalette([v for c in colors for v in c] + [0]*(768-len(colors)*3))
    def c(value):
        target = tuple(bytes.fromhex(value.lstrip('#')))
        return min(colors,key=lambda px:sum((a-b)**2 for a,b in zip(px,target)))+(255,)
    dark, skin, shadow, white = c('#211510'), c('#e9a15b'), c('#c4783f'), c('#ffffff')
    eyes_open, mouth_rest = read('eyes/open.png'), read('mouth/rest.png')
    eye_states = {'open':eyes_open}
    shared = eyes_open.copy()
    for y in range(21,29):
        for x in range(43,62):
            shared.putpixel((x,y),(0,0,0,0))
    for name in ['half','closed','happy-closed','wide']:
        im = shared.copy()
        d = ImageDraw.Draw(im)
        if name == 'half':
            d.polygon([(46,23),(50,23),(50,26),(46,26)],fill=white)
            d.rectangle((49,24,50,26),fill=dark)
            d.line([(45,23),(48,22),(51,23)],fill=dark,width=1)
            d.line([(46,27),(49,27)],fill=shadow,width=1)
            d.rectangle((58,23,60,24),fill=white)
            d.line([(60,23),(60,25)],fill=dark,width=1)
            d.line([(57,22),(60,22),(61,23)],fill=dark,width=1)
        elif name == 'closed':
            d.line([(45,24),(46,25),(49,25),(50,24)],fill=dark,width=1)
            d.line([(57,23),(58,24),(60,24),(61,23)],fill=dark,width=1)
            d.line([(46,26),(49,26)],fill=shadow,width=1)
        elif name == 'happy-closed':
            d.line([(45,25),(47,24),(49,24),(51,25)],fill=dark,width=1)
            d.line([(57,24),(58,23),(60,23),(61,24)],fill=dark,width=1)
            d.line([(46,27),(49,27)],fill=shadow,width=1)
            d.line([(58,26),(60,26)],fill=shadow,width=1)
        else:
            # Open the reference lids without changing its pupil width or rightward gaze.
            im = eyes_open.copy(); d = ImageDraw.Draw(im)
            d.line([(46,23),(46,22),(47,21),(50,21)],fill=dark)
            d.rectangle((47,22,48,24),fill=white)
            d.line([(49,22),(49,25)],fill=dark)
            d.line([(50,22),(50,25)],fill=dark)
            d.line([(58,22),(58,21),(59,20),(61,20)],fill=dark)
            d.line([(59,21),(59,23)],fill=white)
            d.line([(60,21),(60,24)],fill=dark)
        eye_states[name] = save(im,f'eyes/{name}.png')
    mouth_states = {'rest':mouth_rest}
    for name in ['small-open','wide-open','round','round-talk','smile','laugh']:
        im = blank((96,128)); d = ImageDraw.Draw(im)
        if name == 'small-open':
            d.polygon([(54,35),(57,35),(58,36),(57,38),(55,38),(54,37)],fill=dark)
            d.line([(55,38),(56,38)],fill=c('#96332b'),width=1)
        elif name == 'wide-open':
            d.polygon([(53,35),(58,35),(59,36),(58,38),(54,38)],fill=dark)
            d.line([(54,35),(57,35)],fill=c('#fff0d5'),width=1)
            d.line([(55,38),(57,38)],fill=c('#bb4536'),width=1)
        elif name in ['round','round-talk']:
            d.polygon([(55,34),(57,34),(58,35),(58,37),(57,38),(55,38),(54,37),(54,35)],fill=dark)
            d.line([(55,35),(55,36)],fill=c('#6b2520'),width=1)
        elif name == 'smile':
            d.line([(53,35),(54,36),(57,36),(59,34)],fill=dark,width=1)
            d.line([(55,36),(57,36)],fill=c('#fff0d5'),width=1)
        else:
            # An open laugh beneath the moustache, with teeth and a curved lower lip.
            d.polygon([(52,34),(60,34),(61,35),(60,38),(58,40),(55,40),(53,38)],fill=dark)
            d.line([(53,35),(59,35)],fill=c('#fff0d5'))
            d.line([(54,36),(58,36)],fill=white)
            d.line([(55,39),(58,39)],fill=c('#bb4536'))
        if name in ['small-open','wide-open','round-talk']:
            im=translate(im,0,-2)
        elif name == 'round':
            im=translate(im,0,-1)
        elif name == 'laugh':
            im=translate(im,-1,-1)
        mouth_states[name] = save(im,f'mouth/{name}.png')

    lower = embed(read('body/lower-base.png'))
    upper = embed(read('body/upper-base.png'))
    head = embed(read('head-base.png'))
    gear = {name:embed(read(f'equipment/{name}.png')) for name in EQUIPMENT}
    base_layers = {'lower':lower,'upper':upper,'head':head,
                   'eyes':embed(eyes_open),'mouth':embed(mouth_rest)}
    frames, base_frames, pose_recipes = {}, {}, {}

    def export_pose(name, parts, equipment, sockets, notes=None, head_front=False):
        directory = f'poses/{name}'
        paths = {}
        for key,im in parts.items():
            path = f'{directory}/layers/{key}.png';save(im,path);paths[key]=path
        gear_paths = {}
        for key,im in equipment.items():
            path=f'{directory}/equipment/{key}.png';save(im,path);gear_paths[key]=path
        base_order = [key for key in ['lower','upper','arm','neck','head','eyes','mouth'] if key in parts]
        base = compose([parts[key] for key in base_order])
        order = base_order + list(equipment)
        if head_front:
            order = [key for key in base_order if key not in ['head','eyes','mouth']] + list(equipment) + ['head','eyes','mouth']
        all_layers = dict(parts,**equipment)
        full = compose([all_layers[key] for key in order])
        assert not (opaque(base)-opaque(full)), name
        save(base,f'{directory}/base.png');save(full,f'{directory}/composed.png')
        save(base,f'frames/base/{name}.png');save(full,f'frames/{name}.png')
        frames[name]=full;base_frames[name]=base
        pose_recipes[name]={'layers':paths,'equipment':gear_paths,
            'base_order':base_order,'composition_order':order,'sockets_px':sockets,
            'source_size':list(SIZE),'foot_anchor_px':list(ANCHOR),'notes':notes or []}
        return base,full

    sockets = {'neck':[65,62],'waist':[65,100],'eyes':[[63,40],[76,39]],
               'mouth':[71,52],'screen_left_hand':[47,90],'screen_right_hand':[102,74]}
    expression_images=[]
    expression_masks = {'eyes':blank((96,128)),'mouth':blank((96,128))}
    for im in eye_states.values(): expression_masks['eyes'].alpha_composite(im)
    for im in mouth_states.values(): expression_masks['mouth'].alpha_composite(im)
    for key,im in expression_masks.items():
        mask_im=blank(im.size);mask_im.putalpha(im.getchannel('A'))
        save(mask_im,f'source/animation/expression-{key}-mask.png')
    for name,(eye_id,mouth_id) in RECIPES.items():
        parts=dict(base_layers,eyes=embed(eye_states[eye_id]),mouth=embed(mouth_states[mouth_id]))
        mouth_dy=-2 if name.startswith('talk-') else -1 if name=='surprised' else 0
        base,full=export_pose(name,parts,gear,dict(sockets,mouth=[70 if name=='happy' else 71,52+mouth_dy]))
        save(full.crop((16,16,112,144)),f'expressions/{name}.png')
        save(base.crop((16,16,112,144)),f'expressions/base/{name}.png')
        expression_images.append((name,full))
    assert rgba_pixels(frames['neutral'])==rgba_pixels(embed(read('neutral.png')))
    assert rgba_pixels(base_frames['neutral'])==rgba_pixels(embed(read('base-neutral.png')))
    contact(expression_images,'previews/expressions-4x.png',columns=4,scale=4,crop=(40,20,86,64))
    contact(expression_images,'previews/expressions-dark-4x.png',columns=4,scale=4,crop=(40,20,86,64),dark=True)
    combinations=[]
    for eye_id,eyes in eye_states.items():
        for mouth_id,mouth in mouth_states.items():
            im=compose([lower,upper,head,embed(eyes),embed(mouth)]+list(gear.values()))
            combinations.append((eye_id+'/'+mouth_id,im))
    contact(combinations,'previews/eye-mouth-combinations.png',columns=5,scale=3,crop=(40,20,86,64))
    write_json('expressions/recipes.json',{'eyes':{k:f'eyes/{k}.png' for k in eye_states},
        'mouth':{k:f'mouth/{k}.png' for k in mouth_states},
        'recipes':{k:{'eyes':v[0],'mouth':v[1]} for k,v in RECIPES.items()},
        'eyebrow_owner':'eyes','expression_mask':'source/animation/expression-eyes-mask.png',
        'mouth_mask':'source/animation/expression-mouth-mask.png',
        'priority':'surprised locks blink; talk mouth and idle eyes otherwise independent'})

    # Normalize only isolated head candidates; placement is a fixed per-pose contract.
    head_sources={}
    head_registration={'nod-down':((40,39),(43,24)),
                       'turn-left':((38,39),(44,22)),'turn-right':((38,40),(45,22))}
    for name,(native_size,xy) in head_registration.items():
        source=read(f'source/animation/candidates/{name}.png')
        bbox=source.getchannel('A').point(lambda a:255 if a>=128 else 0).getbbox()
        native=source.crop(bbox).resize(native_size,Image.Resampling.NEAREST)
        native=normalize_candidate(native,palette)
        layer=blank();layer.alpha_composite(native,xy)
        head_sources[name]=layer
        save(layer,f'source/animation/native/{name}.png')
    contact([(k,v) for k,v in head_sources.items()], 'previews/head-pose-candidates.png',
            columns=3,scale=8,crop=(40,20,86,66))

    # The retained upper layer already has a complete clean collar beneath the beard.
    # Repainting its overlap as skin created the false neck in moving poses.
    neck_upper=upper.copy()
    # A short neck bridge fills only the gap uncovered by the turned jaw.
    # Clothing pixels remain owned by the unchanged upper layer.
    turn_left_neck=blank()
    ImageDraw.Draw(turn_left_neck).polygon([(70,54),(74,53),(78,58),(77,61),(66,61)],fill=shadow)
    for y in range(160):
        for x in range(128):
            if upper.getpixel((x,y))[3]:
                turn_left_neck.putpixel((x,y),(0,0,0,0))

    def split_head(name,complete):
        eye_boxes={
            'nod-down':[(59,39,71,45),(59,45,70,51),(74,39,82,44),(74,44,82,50)],
            'turn-left':[(46,31,55,37),(47,37,55,44),(59,32,69,38),(60,38,69,44)],
            'turn-right':[(61,31,74,39),(62,39,73,46),(78,31,83,37),(78,37,83,44)],
        }
        mouth_boxes={'nod-down':(68,55,73,58),'turn-left':(54,50,60,53),'turn-right':(70,51,75,54)}
        h=complete.copy(); e=blank(); m=blank()
        for box in eye_boxes[name]:
            for y in range(box[1],box[3]):
                for x in range(box[0],box[2]):
                    px=complete.getpixel((x,y))
                    interior=all(complete.getpixel((x+dx,y+dy))[3] for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)])
                    if px[3] and interior and (max(px[:3])<=150 or min(px[:3])>=225):
                        e.putpixel((x,y),px);h.putpixel((x,y),skin)
        box=mouth_boxes[name]
        for y in range(box[1],box[3]):
            for x in range(box[0],box[2]):
                px=complete.getpixel((x,y))
                if px[3] and max(px[:3])<150:
                    m.putpixel((x,y),px);h.putpixel((x,y),skin)
        assert rgba_pixels(compose([h,e,m]))==rgba_pixels(complete),name
        assert opaque(h)==opaque(complete),name
        return h,e,m

    mid=dict(base_layers,upper=neck_upper,head=translate(head,0,1),
             eyes=translate(base_layers['eyes'],0,1),mouth=translate(base_layers['mouth'],0,1))
    export_pose('nod-mid',mid,gear,dict(sockets,neck=[65,63],eyes=[[63,41],[76,40]],mouth=[71,53]),
                ['Integer transition key; complete head and facial parts move together'],head_front=True)
    for name,complete in head_sources.items():
        h,e,m=split_head(name,complete)
        part=dict(base_layers,upper=neck_upper,head=h,eyes=e,mouth=m)
        if name=='turn-left':part['neck']=turn_left_neck
        eye_centers={'nod-down':[[65,47],[78,46]],'turn-left':[[51,40],[64,41]],'turn-right':[[68,42],[80,40]]}
        mouths={'nod-down':[70,56],'turn-left':[57,51],'turn-right':[72,52]}
        export_pose(name,part,gear,dict(sockets,eyes=eye_centers[name],mouth=mouths[name]),
                    ['Generated head registered to fixed neck; old head is replaced, not overlaid',
                     'Upper clothing and equipment reused unchanged; neck bridge fills transparent jaw gap only'],head_front=True)

    # A complete torso underlay replaces the old raised arm before a new arm is added.
    give_torso=upper.copy()
    ImageDraw.Draw(give_torso).rectangle((86,58,127,89),fill=(0,0,0,0))
    torso_draw=ImageDraw.Draw(give_torso)
    torso_draw.polygon([(83,58),(85,59),(87,62),(88,67),(89,74),(89,80),
                        (88,85),(88,89),(85,89)],fill=c('#fff0d5'))
    torso_draw.line([(83,58),(85,59),(87,62),(88,67),(89,74),(89,80),
                     (88,85),(88,89)],fill=dark)
    torso_draw.line([(86,66),(87,74),(87,82),(86,86)],fill=c('#d9bb96'))

    arm_source=read('source/animation/candidates/give-arms.png')
    arm_contracts={
        'give-mid': {'root':[520,400], 'polygon':[(510,390),(549,406),(582,421),(596,449),
            (627,443),(655,414),(679,405),(692,412),(693,430),(712,428),(733,434),
            (728,453),(712,476),(685,490),(643,497),(614,512),(590,523),(566,519),
            (545,505),(528,480),(515,462)],'palm':[107,74]},
        'give-full': {'root':[1240,380], 'polygon':[(1230,376),(1260,376),(1288,381),(1305,409),
            (1394,410),(1420,385),(1454,368),(1475,375),(1475,395),(1466,406),
            (1505,409),(1538,403),(1548,417),(1540,437),(1519,453),(1482,461),
            (1439,458),(1419,469),(1370,478),(1321,477),(1296,495),(1268,500),
            (1248,481),(1234,449)],'palm':[117,72]},
    }
    arm_layers={}
    for name,contract in arm_contracts.items():
        rx,ry=contract['root']
        im=arm_source.transform(SIZE,Image.Transform.AFFINE,
            (8,0,rx-86*8,0,8,ry-63*8),Image.Resampling.NEAREST)
        im=normalize_candidate(im,palette)
        # Cloth-red nearest matches are not skin shadows on these bare arms.
        for y in range(160):
            for x in range(128):
                px=im.getpixel((x,y))
                if px[3] and px[0] < 210 and px[0] > px[1]*1.7 and px[2] > px[1]*0.6:
                    im.putpixel((x,y),c('#c4783f'))
        points=[(round(86+(x-rx)/8),round(63+(y-ry)/8)) for x,y in contract['polygon']]
        im=cut(im,points)
        # Repair only the sampled outer rim of these two native arm candidates.
        rim={
            'give-mid':[(85,62),(86,63),(87,63),(88,64),(90,64),(92,65),(93,66),
                (94,67),(95,68),(96,69),(98,69),(99,68),(101,67),(102,66),
                (103,65),(105,65),(106,64),(107,64)],
            'give-full':[(85,62),(88,62),(89,63),(92,63),(93,65),(94,67),
                (105,67),(106,66),(107,65),(108,64),(110,64),(111,63),
                (112,63),(113,62),(115,62)]}
        rim_draw=ImageDraw.Draw(im)
        rim_draw.line(rim[name],fill=dark,width=1)
        if name=='give-full':
            rim_draw.line([(116,67),(122,67),(123,66)],fill=dark,width=1)
        arm_layers[name]=im
        save(im,f'source/animation/native/{name}-arm.png')
    contact(list(arm_layers.items()),'previews/give-arm-candidates.png',columns=2,scale=6,crop=(80,56,128,90))
    horizontal_spoon=read('source/spoon-complete.png')
    horizontal_spoon=horizontal_spoon.crop(horizontal_spoon.getbbox()).transpose(Image.Transpose.ROTATE_270)
    for name in ['give-mid','give-full','give-empty','give-retract']:
        source_id='give-mid' if name in ['give-mid','give-retract'] else 'give-full'
        contract=arm_contracts[source_id];palm=contract['palm']
        parts=dict(base_layers,upper=give_torso,arm=arm_layers[source_id])
        equipment={k:v for k,v in gear.items() if k!='spoon'}
        if name in ['give-mid','give-full']:
            prop=blank()
            prop.alpha_composite(horizontal_spoon,(palm[0]-horizontal_spoon.width+9,palm[1]-horizontal_spoon.height+1))
            # Fingers on the palm rim obscure the lower bowl edge, not the entire arm.
            for y in range(palm[1]-1,palm[1]+3):
                for x in range(palm[0]-6,min(palm[0]+10,128)):
                    if arm_layers[source_id].getpixel((x,y))[3]:
                        prop.putpixel((x,y),(0,0,0,0))
            equipment['spoon']=prop
        export_pose(name,parts,equipment,dict(sockets,screen_right_hand=palm),
            ['Single torso underlay; old bent arm removed',
             'Generated sleeve/forearm/open palm registered on fixed 8:1 grid',
             'Spoon quarter-turn uses exact integer transpose; no resampled rotation',
             'Spoon absent after transfer event' if name in ['give-empty','give-retract'] else 'Spoon rests on open palm'])
    export_pose('neutral-empty',base_layers,{k:v for k,v in gear.items() if k!='spoon'},sockets,
                ['Return pose after the spoon has been transferred'])

    def key(sprite,duration,event=None):
        item={'sprite':sprite,'duration_ms':duration}
        if event:item['event']=event
        return item
    clips={
        'idle':{'loop':True,'frames':[key('neutral',1200),key('relaxed',40),key('blink',80),key('relaxed',40),key('neutral',1400)]},
        'blink':{'loop':False,'frames':[key('neutral',120),key('relaxed',40),key('blink',80),key('relaxed',40),key('neutral',120)]},
        'talk':{'loop':True,'frames':[key('neutral',100),key('talk-small',100),key('talk-wide',120),key('talk-small',90),key('talk-round',100),key('neutral',120)]},
        'happy':{'loop':False,'frames':[key('neutral',160),key('happy',700),key('neutral',160)]},
        'surprised':{'loop':False,'frames':[key('neutral',160),key('surprised',600),key('neutral',180)]},
        'nod':{'loop':False,'frames':[key('neutral',120),key('nod-mid',100),key('nod-down',140),key('nod-mid',100),key('neutral',120)]},
        'shake_head':{'loop':False,'frames':[key('neutral',100),key('turn-left',140),key('neutral',100),key('turn-right',140),key('neutral',100)]},
        'give':{'loop':False,'frames':[key('neutral',120),key('give-mid',120),key('give-full',180),key('give-empty',120,'transfer'),key('give-retract',120),key('neutral-empty',120)]},
    }
    write_json('poses/recipes.json',{'canvas':list(SIZE),'offset_from_base':list(OFFSET),'poses':pose_recipes,
        'coordinate_sides':'Screen-left hand is anatomical right, screen-right hand is anatomical left in the reference 3/4 view',
        'reconstructed_occluded_regions':['Short neck bridge in transparent gap under left-turned chin','Right torso side under removed bent arm'],
        'scope':['neutral','nod','shake_head','give'],'walk':'not_generated'})
    write_json('animation.json',{'version':1,'id':'chef','canvas':list(SIZE),'foot_anchor_px':list(ANCHOR),
        'duration_unit':'ms','sampler':'nearest','root_motion':False,
        'frames':{name:{'source':f'frames/{name}.png','base':f'frames/base/{name}.png',
                        'pose_recipe':f'poses/recipes.json#{name}'} for name in sorted(frames)},
        'clips':clips,'channels':{'eyes':'independent blink, fixed QA idle timeline','mouth':'independent talk',
                                  'priority':'surprised locks automatic blink'}})

    # Fixed-cell page, one extruded pixel and one transparent gutter on every side.
    cell_w,cell_h=SIZE[0]+4,SIZE[1]+4
    ids=sorted(frames);columns=8;rows=math.ceil(len(ids)/columns)
    atlas=blank((math.ceil(columns*cell_w/16)*16,math.ceil(rows*cell_h/16)*16));sprites=[];dedup={}
    for i,name in enumerate(ids):
        im=frames[name];digest=hashlib.sha256(rgba_pixels(im)).hexdigest()
        if digest in dedup:
            rect=dedup[digest]
        else:
            x,y=i%columns*cell_w+2,i//columns*cell_h+2
            rect=[x,y,*SIZE];atlas.alpha_composite(im,(x,y));dedup[digest]=rect
            # Extrusion occupies only the immediately adjacent pixel; outer gutter stays clear.
            atlas.paste(im.crop((0,0,1,160)),(x-1,y));atlas.paste(im.crop((127,0,128,160)),(x+128,y))
            atlas.paste(im.crop((0,0,128,1)),(x,y-1));atlas.paste(im.crop((0,159,128,160)),(x,y+160))
            for dx,dy,sx,sy in [(-1,-1,0,0),(128,-1,127,0),(-1,160,0,159),(128,160,127,159)]:
                atlas.putpixel((x+dx,y+dy),im.getpixel((sx,sy)))
        sprites.append({'id':name,'page':'chef-0','rect':rect,'source_size':list(SIZE),
            'trim_offset':[0,0],'pivot':list(ANCHOR),'source':f'frames/{name}.png','rotated':False,
            'sha256_pixels':digest})
    save(atlas,'atlas.png')
    write_json('atlas-manifest.json',{'version':1,'sampler':'nearest','mipmaps':False,'extrude_px':1,'gutter_px':1,
        'trimmed':False,'runtime_status':'authoring_export_not_integrated',
        'pages':[{'id':'chef-0','path':'atlas.png','size':list(atlas.size)}],'sprites':sprites,'clips':clips})

    contact([(name,frames[name]) for name in ['neutral','nod-mid','nod-down','turn-left','turn-right',
        'give-mid','give-full','give-empty','give-retract','neutral-empty']],
        'previews/poses-3x.png',columns=5,scale=3)
    contact([(name,base_frames[name]) for name in ['neutral','nod-mid','nod-down','turn-left','turn-right',
        'give-mid','give-full','give-empty','give-retract','neutral-empty']],
        'previews/poses-base-3x.png',columns=5,scale=3)
    for name,im in frames.items():
        save(im.resize((512,640),Image.Resampling.NEAREST),f'previews/frames/{name}-4x.png')
    make_previews(frames,base_frames,clips,colors)
    revision_before=ROOT/'source/animation/revision-before/frames'
    if revision_before.exists():
        for name,ids,crop in [
            ('faces',['talk-small','talk-wide','talk-round','happy','surprised'],(42,29,84,61)),
            ('joints',['nod-mid','nod-down','turn-left','turn-right','give-full'],(41,48,127,85))]:
            pairs=[]
            for sprite in ids:
                pairs.extend([(sprite+' BEFORE',read(str((revision_before/'base'/f'{sprite}.png').relative_to(ROOT)))),
                              (sprite+' AFTER',base_frames[sprite])])
            contact(pairs,f'previews/revision-{name}-4x.png',columns=2,scale=4,crop=crop)

    happy_before=ROOT/'source/animation/happy-revision-before/frames/happy.png'
    if happy_before.exists():
        contact([('BEFORE',read(str(happy_before.relative_to(ROOT)))),('AFTER',frames['happy'])],
                'previews/happy-correction-8x.png',columns=2,scale=8,crop=(42,28,85,62))
    write_json('source/animation/registration.json',{
        'canvas':list(SIZE),'base_offset':list(OFFSET),'head_registration':head_registration,
        'arm_contracts':arm_contracts,'arm_native_scale':0.125,'retained_baseline':str(baseline.relative_to(ROOT)),
        'candidate_tool':'built-in image_gen','prompts':'source/animation/generation-prompts.json'})
    spec=json.loads((ROOT/'character-spec.yaml').read_text())
    spec['palette']['rgba_hex']=['#'+''.join(f'{v:02x}' for v in rgb)+'ff' for rgb in colors]
    spec['scope']='8 expressions; nod, shake_head and give pose clips; no walk or runtime integration'
    spec['pose_canvas']={'grid':'B','canvas_cells':[8,10],'native_canvas_px':list(SIZE),
                         'base_offset_px':list(OFFSET),'foot_anchor_px':list(ANCHOR)}
    spec['layers']['eyes']={key:f'eyes/{key}.png' for key in eye_states}
    spec['layers']['mouth']={key:f'mouth/{key}.png' for key in mouth_states}
    spec['expression_recipes']={key:{'eyes':value[0],'mouth':value[1]} for key,value in RECIPES.items()}
    spec['expression_channels']={'eyes':'Independent blink: open, half, closed, half, open',
        'mouth':'Independent talk: rest, small-open, wide-open, small-open, round-talk, rest',
        'priority':'surprised temporarily locks automatic blink'}
    spec['pose_overrides']={'recipes':'poses/recipes.json','registration':'source/animation/registration.json'}
    spec['animation']={'manifest':'animation.json','frame_order':sorted(frames),'clips':list(clips),
        'duration_unit':'ms','transfer':{'clip':'give','key_index_zero_based':3,'event':'transfer','count':1}}
    spec['runtime_export'].update({'status':'authoring_export_not_integrated','mode':'baked',
        'manifest':'atlas-manifest.json','atlas':'atlas.png','frame_size':list(SIZE),
        'trim_supported_by_consumer':False,'extrude_px':1,'gutter_px':1})
    spec['production_plan']={'status':'generated','eyes':list(eye_states),'mouth':list(mouth_states),
        'expressions':list(RECIPES),'pose_clips':['nod','shake_head','give'],'walk':'not_generated'}
    spec['generation']['status']='Expressions and three pose clips generated and natively registered'
    spec['generation']['animation_builder']='source/build-animation.py'
    spec['generation']['animation_prompts']='source/animation/generation-prompts.json'
    spec['verification']['animation']='qa/animation-validation.json'
    spec['verification']['playback']='qa/playback-review.json'
    spec['verification']['static_atlas_audit']='qa/sprite-audit.json'
    write_json('character-spec.yaml',spec)
    return locals()

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
<title>Chef 표정·포즈 검수</title><style>
*{box-sizing:border-box}body{margin:0;background:#18191d;color:#eee3cc;font:16px system-ui,sans-serif}main{max-width:1180px;margin:auto;padding:28px}
h1{font-size:24px;margin:0 0 8px}p{color:#aaa}button,select{font:inherit;background:#35363e;color:#fff;border:1px solid #5a5b65;border-radius:7px;padding:8px 14px}button{cursor:pointer}nav{display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin:22px 0}label{display:flex;gap:8px;align-items:center}.stage{display:grid;grid-template-columns:1fr 1fr;gap:16px}figure{margin:0;background:#eee3cc;border-radius:12px;text-align:center;overflow:hidden}figcaption{padding:12px;background:#282930;color:#fff}canvas{width:512px;height:640px;max-width:100%;image-rendering:pixelated;object-fit:contain}#status{font-variant-numeric:tabular-nums;min-height:26px;margin:16px 0}.grid{display:flex;gap:8px;flex-wrap:wrap;margin-top:18px}.hint{font-size:14px}.dark figure{background:#302b32}@media(max-width:800px){.stage{grid-template-columns:1fr}canvas{width:256px;height:320px}}
</style><main><h1>Chef 표정·포즈</h1><p>같은 프레임의 장비 없는 바탕과 착용본을 나란히 확인합니다.</p>
<nav><label>동작 <select id="clip"></select></label><label>속도 <select id="speed"><option value="1">1× 정상</option><option value="0.25">0.25× 저속</option></select></label><label>배율 <select id="scale"><option value="4">4× 픽셀</option><option value="1">1× 원본</option></select></label><button id="play">일시정지</button><button id="next">다음 키</button><button id="background">어두운 배경</button></nav>
<div class="stage"><figure><figcaption>장비 없는 언더레이어</figcaption><canvas id="base" width="128" height="160"></canvas></figure><figure><figcaption>장비 착용</figcaption><canvas id="equipped" width="128" height="160"></canvas></figure></div>
<div id="status" aria-live="off"></div><div class="grid" id="expressions"></div><p class="hint">재생 미리보기는 비반복 동작도 끝에서 400ms 후 처음부터 다시 보여줍니다. 건네기 4번째 키의 transfer 이벤트 뒤에는 스푼이 없습니다. 게임 런타임에는 연결하지 않았습니다.</p>
<script>const DATA=__DATA__;const names={idle:'대기·깜빡임',blink:'깜빡임',talk:'말하기',happy:'기쁨',surprised:'놀람',nod:'끄덕임',shake_head:'고개젓기',give:'건네기'};
const clip=document.querySelector('#clip'),speed=document.querySelector('#speed');for(const key of Object.keys(DATA.clips)){const o=document.createElement('option');o.value=key;o.textContent=names[key]||key;clip.append(o)}clip.value='nod';let playing=true,index=0,elapsed=0,last=performance.now(),manual=null;
const images={};for(const [id,states]of Object.entries(DATA.frames)){images[id]={};for(const[k,v]of Object.entries(states)){const im=new Image();im.src=v;images[id][k]=im}}
function draw(){const seq=DATA.clips[clip.value].frames,key=manual?{sprite:manual}:seq[index];for(const kind of ['base','equipped']){const ctx=document.querySelector('#'+kind).getContext('2d');ctx.imageSmoothingEnabled=false;ctx.clearRect(0,0,128,160);const im=images[key.sprite][kind];if(im.complete)ctx.drawImage(im,0,0)}document.querySelector('#status').textContent=manual?'표정: '+manual:((names[clip.value]||clip.value)+' · 키 '+(index+1)+'/'+seq.length+' · '+key.sprite+' · '+key.duration_ms+'ms'+(key.event?' · '+key.event:''))}
function tick(now){const delta=now-last;last=now;if(playing&&!manual){elapsed+=delta*Number(speed.value);const seq=DATA.clips[clip.value].frames;const duration=seq[index].duration_ms+(index===seq.length-1&&!DATA.clips[clip.value].loop?400:0);if(elapsed>=duration){elapsed%=duration;index=(index+1)%seq.length}}draw();requestAnimationFrame(tick)}
document.querySelector('#scale').onchange=()=>{const scale=Number(document.querySelector('#scale').value);for(const kind of ['base','equipped']){const canvas=document.querySelector('#'+kind);canvas.style.width=128*scale+'px';canvas.style.height=160*scale+'px'}};clip.onchange=()=>{index=0;elapsed=0;manual=null;draw()};document.querySelector('#play').onclick=()=>{playing=!playing;document.querySelector('#play').textContent=playing?'일시정지':'재생'};document.querySelector('#next').onclick=()=>{playing=false;document.querySelector('#play').textContent='재생';manual=null;index=(index+1)%DATA.clips[clip.value].frames.length;elapsed=0;draw()};document.querySelector('#background').onclick=()=>{const on=document.body.classList.toggle('dark');document.querySelector('#background').textContent=on?'밝은 배경':'어두운 배경'};
for(const id of ['neutral','talk-small','talk-wide','talk-round','blink','happy','surprised','relaxed']){const b=document.createElement('button');b.textContent=id;b.onclick=()=>{manual=id;draw()};document.querySelector('#expressions').append(b)}requestAnimationFrame(tick);
</script></main></html>'''
    (ROOT/'previews/animation.html').write_text(html.replace('__DATA__',json.dumps(payload,separators=(',',':'))))

if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--base-ready',action='store_true');parser.add_argument('--prepare',action='store_true')
    args=parser.parse_args()
    if not args.base_ready:
        subprocess.run([sys.executable,str(ROOT/'source/build-character.py')],
                       env=dict(os.environ,CHEF_BASE_ONLY='1'),check=True,capture_output=True)
    state=build()
    if not args.prepare:
        subprocess.run([sys.executable,str(ROOT/'source/verify-animation.py')],check=True)
    print('Built chef: 8 expressions, 3 pose clips, 17 frames and deterministic atlas')
