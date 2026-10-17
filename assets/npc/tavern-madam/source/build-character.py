"""Build the single neutral tavern-madam pose from retained generated sources."""
from pathlib import Path
from PIL import Image, ImageDraw
import json
import base64
import io
import yaml

ROOT = Path(__file__).resolve().parents[1]
SIZE = (96, 128)


def load(path):
    return Image.open(ROOT / path).convert('RGBA')


def save(image, path):
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    image.save(target)
    return image


def write(path, value):
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def blank(size=SIZE):
    return Image.new('RGBA', size)


def sample(image, transform):
    sx, sy = image.width / 1024, image.height / 1536
    a, b, c, d, e, f = transform
    result = image.transform(SIZE, Image.Transform.AFFINE,
                             (a*sx, b*sx, c*sx, d*sy, e*sy, f*sy),
                             Image.Resampling.NEAREST)
    result.putalpha(result.getchannel('A').point(lambda a: 255 if a >= 128 else 0))
    return result


def compose(layers, size=SIZE):
    result = blank(size)
    for layer in layers:
        result.alpha_composite(layer)
    return result


def embed(image):
    result = blank((128, 160))
    result.paste(image, (16, 16))
    return result


def build():
    reg = json.loads((ROOT / 'source/registration.json').read_text())
    reference = sample(load('source/equipped-candidate.png'), reg['reference_transform'])
    foundation = sample(load('source/underlayer-corrected.png'), reg['underlayer_transform'])
    # One shared palette for face, cloth, gold, beer, leather and all restoration pixels.
    arm = load('source/raised-arm-candidate.png').resize((32,28), Image.Resampling.NEAREST)
    arm.putalpha(arm.getchannel('A').point(lambda a:255 if a>=128 else 0))
    foundation.paste(arm.crop((7,1,23,25)), (63,43))
    colors = [p[:3] for image in [reference, foundation] for p in image.getdata() if p[3]]
    strip = Image.new('RGB', (len(colors), 1)); strip.putdata(colors)
    palette = strip.quantize(colors=61, method=Image.Quantize.MEDIANCUT)
    entries = palette.getpalette()
    # Preserve small green-bottle clusters that a body-weighted palette can overlook.
    for i, rgb in enumerate([(43,64,32),(62,88,44),(84,111,56)],61):
        entries[i*3:i*3+3] = rgb
    entries[192:] = entries[:3]*192
    palette.putpalette(entries)
    palette.save(ROOT / 'source/palette.png')

    def native(image):
        result = image.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE).convert('RGBA')
        result.putalpha(image.getchannel('A'))
        for y in range(128):
            for x in range(96):
                if not result.getpixel((x, y))[3]: result.putpixel((x, y), (0, 0, 0, 0))
        return result

    reference, foundation = native(reference), native(foundation)
    # Close the top of the empty grip with one native outline row.
    outline=min((tuple(palette.getpalette()[i:i+3]) for i in range(0,192,3)),
                key=lambda rgb:sum((a-b)**2 for a,b in zip(rgb,(25,8,7))))+(255,)
    ImageDraw.Draw(foundation).line([(67,45),(70,45)],fill=outline)
    save(reference, 'source/reference-native.png')
    gear = {}
    for name, regions in reg['gear_regions'].items():
        mask = Image.new('L', SIZE)
        for points in regions: ImageDraw.Draw(mask).polygon(points, fill=255)
        layer = blank(); layer.paste(reference, (0, 0), mask)
        if name == 'hair-ribbon':
            # Keep only ribbon red and the gold clip, never a copied hair patch.
            for y in range(128):
                for x in range(96):
                    r, g, b, a = layer.getpixel((x, y))
                    red = r > g*1.5 and r > b*1.3 and g < 90 and r > 65
                    gold = r > 160 and g > 95 and b < 130
                    if a and not (red or gold): layer.putpixel((x, y), (0, 0, 0, 0))
        if name in ['earrings','shawl','belt-accessories','towel']:
            material=set()
            for y in range(128):
                for x in range(96):
                    r,g,b,a=layer.getpixel((x,y))
                    gold=r>140 and g>85 and g<205 and b<130 and r>g*1.13
                    red=r>g*1.5 and g<95 and b<100 and r>65
                    cream=r>175 and g>145 and b>110
                    leather=r<175 and r>g*1.12 and g>b*.85
                    green=g>r*1.05 and g>b*1.1
                    keep={'earrings':gold,'shawl':red or gold,
                          'belt-accessories':gold or leather or green,
                          'towel':cream or red}[name]
                    if a and keep:material.add((x,y))
            for y in range(128):
                for x in range(96):
                    px=layer.getpixel((x,y))
                    border=px[3] and max(px[:3])<100 and any((x+dx,y+dy) in material for dx,dy in [(0,1),(0,-1),(1,0),(-1,0)])
                    if (x,y) not in material and not border:layer.putpixel((x,y),(0,0,0,0))
        if name=='tankard':
            for y in range(45,56):
                for x in range(66,71):
                    r,g,b,a=layer.getpixel((x,y))
                    if a and r>200 and 100<g<205 and b<160:layer.putpixel((x,y),(0,0,0,0))
            for point in [(68,48),(69,48),(68,50),(69,50),(68,52),(69,52)]:
                if foundation.getpixel(point)[3]:layer.putpixel(point,(0,0,0,0))
        gear[name] = layer

    # Restore reference facial interior only; keep the complete reconstructed outer hair.
    face_mask=Image.new('L',SIZE)
    ImageDraw.Draw(face_mask).polygon([(50,19),(54,20),(55,23),(55,26),(56,28),
        (56,33),(55,35),(55,38),(53,40),(49,41),(45,40),(42,38),
        (40,35),(39,31),(39,28),(43,25),(47,23)],fill=255)
    for y in range(128):
        for x in range(96):
            if face_mask.getpixel((x,y)) and foundation.getpixel((x,y))[3] and reference.getpixel((x,y))[3]:
                foundation.putpixel((x,y),reference.getpixel((x,y)))
    face_mask_rgba=blank();face_mask_rgba.putalpha(face_mask)
    save(face_mask_rgba,'source/face-interior-mask.png')
    # The source beauty mark falls between sample centers; retain it as one clear pixel.
    mole=min((tuple(palette.getpalette()[i:i+3]) for i in range(0,192,3)),
             key=lambda rgb:sum((a-b)**2 for a,b in zip(rgb,(115,43,22))))+(255,)
    foundation.putpixel((42,34),mole)
    eyes, mouth = blank(), blank()
    # Facial components retain source positions; the head behind them stays solid skin.
    eyes_mask = Image.new('L', SIZE)
    ed = ImageDraw.Draw(eyes_mask)
    ed.rectangle((39, 26, 48, 33), fill=255)
    ed.rectangle((52, 27, 55, 34), fill=255)
    ed.rectangle((43, 24, 48, 25), fill=255)
    ed.rectangle((53, 25, 55, 26), fill=255)
    mouth_mask = Image.new('L', SIZE)
    ImageDraw.Draw(mouth_mask).polygon([(44, 35), (47, 35), (48, 36), (52, 36),
                                      (52, 38), (47, 38), (44, 36)], fill=255)
    # Component selection is refined in native coordinates before freezing this baseline.
    for y in range(18, 43):
        for x in range(33, 60):
            p = foundation.getpixel((x, y))
            r, g, b, a = p
            if not a: continue
            eye_feature = max(r, g, b) < 175 or (r > 215 and g > 195 and b > 160)
            lip_feature = r > g*1.4 and g < 125 and max(r, g, b) < 210
            target = eyes if eyes_mask.getpixel((x, y)) and eye_feature else (
                mouth if mouth_mask.getpixel((x, y)) and lip_feature else None)
            if target is not None:
                target.putpixel((x, y), p)
                # Fill a feature with the closest skin row on the same cheek.
                candidates = [foundation.getpixel((xx, yy)) for yy in range(max(18,y-3),min(43,y+4))
                              for xx in range(max(33,x-3),min(60,x+4))]
                skin = [q for q in candidates if q[3] and q[0]>210 and 120<q[1]<205 and q[2]<155]
                assert skin, (x,y)
                foundation.putpixel((x, y), min(skin, key=lambda q:abs(q[1]-175)))

    # Keep hanging curls with the head while the neck and blouse belong to the upper body.
    head_mask=Image.new('L',SIZE);hd=ImageDraw.Draw(head_mask)
    hd.polygon([(0,0),(80,0),(80,40),(64,40),(62,43),(61,48),(57,49),
                (54,46),(53,42),(45,42),(44,47),(41,49),(37,47),(34,43),
                (29,41),(0,40)],fill=255)
    head, upper, lower = blank(), blank(), blank()
    for y in range(128):
        for x in range(96):
            px=foundation.getpixel((x,y))
            hanging_hair=px[0]<210 and px[1]<125 and px[2]<100 and px[0]>px[1]*1.1
            is_head=head_mask.getpixel((x,y)) and (y<41 or hanging_hair)
            target = head if is_head else upper if y < 72 else lower
            target.putpixel((x, y), foundation.getpixel((x, y)))
    save(head, 'head-base.png');save(upper, 'body/upper-base.png');save(lower, 'body/lower-base.png')
    save(eyes, 'eyes/open.png');save(mouth, 'mouth/rest.png')
    base = compose([lower, upper, head, eyes, mouth])
    full = compose([base] + [gear[n] for n in reg['equipment_order']])
    for name, image in gear.items(): save(image, f'equipment/{name}.png')
    save(base, 'base-neutral.png');save(full, 'neutral.png')
    save(base, 'source/complete-underlayer-native.png')
    save(embed(base), 'frames/base/neutral.png');save(embed(full), 'frames/neutral.png')
    save(embed(base), 'poses/neutral/base.png');save(embed(full), 'poses/neutral/composed.png')
    layers = {'lower':'body/lower-base.png','upper':'body/upper-base.png','head':'head-base.png',
              'eyes':'eyes/open.png','mouth':'mouth/rest.png'}
    for name,path in layers.items(): save(embed(load(path)), f'poses/neutral/layers/{name}.png')
    for name,image in gear.items(): save(embed(image), f'poses/neutral/equipment/{name}.png')
    write('poses/recipes.json', {'canvas':[128,160],'offset_from_base':[16,16],'poses':{
        'neutral':{'layers':{n:f'poses/neutral/layers/{n}.png' for n in layers},
                   'equipment':{n:f'poses/neutral/equipment/{n}.png' for n in gear},
                   'base_order':list(layers),'composition_order':list(layers)+reg['equipment_order']}}})
    write('expressions/recipes.json', {'canvas':[96,128],'eyes':{'open':'eyes/open.png'},
        'mouth':{'rest':'mouth/rest.png'},'recipes':{'neutral':{'eyes':'open','mouth':'rest'}}})

    # Fixed cell, 1px extrusion and a separate transparent gutter on each side.
    page = blank((144,176));frame = embed(full);page.paste(frame,(2,2))
    for y in range(-1,161):
        for x in range(-1,129):
            if x in (-1,128) or y in (-1,160):
                page.putpixel((x+2,y+2),frame.getpixel((max(0,min(127,x)),max(0,min(159,y)))))
    save(page,'atlas.png')
    write('atlas-manifest.json',{'version':1,'mode':'baked','usage':'neutral_authoring_not_integrated',
        'sampler':'nearest','mipmaps':False,'extrude_px':1,'gutter_px':1,
        'pages':[{'id':'tavern-madam-0','path':'atlas.png','size':[144,176]}],
        'sprites':[{'id':'neutral','page':'tavern-madam-0','rect':[2,2,128,160],
                    'source_size':[128,160],'trim_offset':[0,0],'pivot':[64,140],
                    'source':'frames/neutral.png','rotated':False}],'clips':{}})
    for name,image in [('neutral',embed(full)),('base-neutral',embed(base))]:
        save(image.resize((512,640),Image.Resampling.NEAREST),f'previews/{name}-4x.png')
    for dark in [False,True]:
        bg,fg = ('#302b32','#eee3cc') if dark else ('#eee3cc','#211510')
        preview = Image.new('RGB',(1072,690),bg);draw=ImageDraw.Draw(preview)
        for x,label,image in [(12,'Complete base / empty hands',embed(base)),(548,'Equipped / neutral',embed(full))]:
            draw.text((x,8),label,fill=fg);image=image.resize((512,640),Image.Resampling.NEAREST)
            preview.paste(image,(x,32),image)
        save(preview,f'previews/neutral-comparison{"-dark" if dark else ""}.png')
    write('animation.json',{'version':1,'id':'tavern-madam','canvas':[128,160],
          'foot_anchor_px':[64,140],'frames':['neutral'],'clips':{}})
    preview_items={**layers,**{n:f'equipment/{n}.png' for n in gear}}
    urls={}
    for name,path in preview_items.items():
        buffer=io.BytesIO();embed(load(path)).save(buffer,format='PNG')
        urls[name]='data:image/png;base64,'+base64.b64encode(buffer.getvalue()).decode()
    gear_names=reg['equipment_order']
    html='<!doctype html><html lang="en"><meta charset="utf-8"><title>Tavern madam · neutral</title><style>body{font:16px system-ui;background:#24232a;color:#eee;margin:24px}nav{display:flex;gap:16px;flex-wrap:wrap;margin:18px 0}section{display:flex;gap:24px;overflow:auto}figure{margin:0}canvas{display:block;background:#eee3cc;image-rendering:pixelated;width:512px;height:640px}body.dark canvas{background:#302b32}label,button,select{font:inherit}figcaption{padding:8px}</style><h1>Tavern madam · neutral pose</h1><nav><label>Scale <select id="scale"><option value="1">1×</option><option value="4" selected>4×</option></select></label><button id="bg">Light / dark</button></nav><nav id="gear"></nav><section><figure><figcaption>Complete base / empty hands</figcaption><canvas id="base" width="128" height="160"></canvas></figure><figure><figcaption>Equipment</figcaption><canvas id="full" width="128" height="160"></canvas></figure></section><script>'
    html+='const urls='+json.dumps(urls)+';const baseOrder='+json.dumps(list(layers))+';const gearOrder='+json.dumps(gear_names)+';'
    html+='const images={};for(const [name,url]of Object.entries(urls)){const im=new Image();im.src=url;images[name]=im;}function draw(){for(const id of ["base","full"]){const c=document.getElementById(id).getContext("2d");c.imageSmoothingEnabled=false;c.clearRect(0,0,128,160);for(const name of [...baseOrder,...(id==="full"?gearOrder.filter(n=>document.getElementById("g-"+n).checked):[])])if(images[name].complete)c.drawImage(images[name],0,0);}}for(const name of gearOrder){const label=document.createElement("label"),input=document.createElement("input");input.type="checkbox";input.id="g-"+name;input.checked=true;input.onchange=draw;label.append(input,document.createTextNode(name));document.getElementById("gear").append(label);}Promise.all(Object.values(images).map(im=>im.decode())).then(draw);document.getElementById("scale").onchange=e=>{const s=Number(e.target.value);for(const canvas of document.querySelectorAll("canvas")){canvas.style.width=128*s+"px";canvas.style.height=160*s+"px";}};document.getElementById("bg").onclick=()=>document.body.classList.toggle("dark");</script></html>'
    (ROOT/'previews/index.html').write_text(html)
    for dark in [False,True]:
        bg,fg=('#302b32','#eee3cc') if dark else ('#eee3cc','#211510')
        out=Image.new('RGB',(792,390),bg);od=ImageDraw.Draw(out)
        for i,(label,image) in enumerate([('Reference cutout',reference),('Complete base',base),('Equipped',full)]):
            od.text((i*264+8,8),label,fill=fg);image=embed(image).resize((256,320),Image.Resampling.NEAREST)
            out.paste(image,(i*264+4,36),image)
        save(out,f'previews/reference-comparison{"-dark" if dark else ""}.png')
    for name,box in [('face',(16,2,65,49)),('hands',(58,40,85,68)),('hip-hand',(14,54,43,73))]:
        for dark in [False,True]:
            bg,fg=('#302b32','#eee3cc') if dark else ('#eee3cc','#211510')
            w,h=box[2]-box[0],box[3]-box[1];out=Image.new('RGB',(w*8*2+32,h*8+40),bg);od=ImageDraw.Draw(out)
            for i,(label,image) in enumerate([('Base',base),('Equipped',full)]):
                od.text((8+i*(w*8+16),8),label,fill=fg);image=image.crop(box).resize((w*8,h*8),Image.Resampling.NEAREST)
                out.paste(image,(8+i*(w*8+16),30),image)
            save(out,f'previews/{name}-8x{"-dark" if dark else ""}.png')
    eye_near=eyes.crop((0,0,50,128)).getbbox();eye_far=eyes.crop((50,0,96,128)).getbbox()
    spec={'version':1,'id':'tavern-madam','role':'npc','action_set':'neutral_only',
          'grid':'B','grid_unit_px':[16,16],'canvas_cells':[6,8],'native_canvas_px':[96,128],
          'foot_anchor_px':[48,124],'preview_scale':4,
          'pose_canvas':{'grid':'B','canvas_cells':[8,10],'native_canvas_px':[128,160],
                         'base_offset_px':[16,16],'foot_anchor_px':[64,140]},
          'reference_roles':{'identity':'source/reference.png','style':'source/reference.png'},
          'reconstructed_occluded_regions':['complete hair and ear under ribbon/earring','blouse and bodice under shawl',
              'skirt and waist under apron, belt and pouches','empty raised grip and forearm'],
          'baseline':{'complete_underlayer':'base-neutral.png','foundation':'source/complete-underlayer-native.png',
                      'builder':'source/build-character.py','coverage_report':'qa/validation.json'},
          'palette':{'max_opaque_colors':64,'reason':'retain face, embroidered cloth, gold, green bottle, beer and leather',
                      'source':'source/palette.png','alpha_values':[0,255]},
          'identity':{'head_bbox_px':list(head.getbbox()),'head_size_without_hat_px':[head.getbbox()[2]-head.getbbox()[0],head.getbbox()[3]-head.getbbox()[1]],
                      'near_eye_bbox_px':list(eye_near),'far_eye_bbox_px':[eye_far[0]+50,eye_far[1],eye_far[2]+50,eye_far[3]],
                      'mouth_bbox_px':list(mouth.getbbox()),'hair_color':'auburn / copper brown, pinned-up curls',
                      'body_build':'curvy sturdy hostess; original puff sleeves and fitted bodice',
                      'skin_color':'warm peach','beard':'none','eyes':'large brown eyes, looking screen-right',
                      'outline_rule':'native dark 1px contours, material shadows separate',
                      'reference_exact_pixel_identity':False},
          'sockets_px':{'neck':[46,43],'waist':[46,70],'left_hand':[68,50],'right_hand':[33,67]},
          'limb_identity':{'side_convention':'character_anatomical','near_side':'right','far_side':'left'},
          'layers':{'head_base':'head-base.png','upper_base':'body/upper-base.png','lower_base':'body/lower-base.png',
                    'eyes':{'open':'eyes/open.png'},'mouth':{'rest':'mouth/rest.png'},
                    'equipment':[f'equipment/{n}.png' for n in gear_names]},
          'eyebrow_owner':'eyes','composition_order':list(layers)+gear_names,
          'expression_recipes':{'neutral':{'eyes':'open','mouth':'rest'}},
          'production_plan':{'status':'neutral_only_complete','completed':['neutral'],
                             'not_generated':['extra expressions','gestures','walk']},
          'animation':{'frames':['neutral'],'clips':{}},
          'runtime_export':{'status':'not_integrated','manifest':'atlas-manifest.json','mode':'baked',
                            'sampler':'nearest','mipmaps':False,'extrude_px':1,'gutter_px':1,'rotate_packing':False},
          'generation':{'tool':'built-in image_gen','final_prompts':'source/generation-prompts.json',
                        'registration':'source/registration.json'}}
    def span_at(y,left,right,skin_only=False):
        points=[]
        for x in range(left,right):
            p=base.getpixel((x,y))
            if p[3] and (not skin_only or (p[0]>205 and 110<p[1]<205 and p[2]<155)):points.append(x)
        return {'row_y':y,'roi_x':[left,right],'visible_x_interval':[min(points),max(points)+1],
                'width_px':max(points)-min(points)+1} if points else None
    spec['identity'].update(reference_facial_interior_bbox_px=list(face_mask.getbbox()),
        eye_feature_sizes_with_brows_px={'near':[eye_near[2]-eye_near[0],eye_near[3]-eye_near[1]],
                                        'far':[eye_far[2]-eye_far[0],eye_far[3]-eye_far[1]]},
        eye_feature_gap_px=eye_far[0]+50-eye_near[2],
        visible_cross_sections={'neck_skin':span_at(44,35,56,True),
            'puffed_shoulder_silhouette':span_at(49,12,64),
            'bodice_chest':span_at(58,30,60),'waist':span_at(70,32,60),
            'near_arm_skin':span_at(61,12,28,True),'far_forearm_skin':span_at(59,62,73,True)},
        thigh_width_px={'value':None,'observability':'occluded by skirt'},
        full_leg_length_px={'value':None,'observability':'occluded by skirt'},
        visible_legs_and_boots_band_y=[112,124],
        face_source='reference facial interior, registered polygon; complete outer hair reconstructed')
    (ROOT/'character-spec.yaml').write_text(yaml.safe_dump(spec,sort_keys=False,allow_unicode=True))
    print('Built tavern-madam neutral, complete base, seven gear layers, 128x160 frame')


if __name__ == '__main__':
    build()
    import os
    import subprocess
    import sys
    animation_builder=ROOT/'source/build-animation.py'
    if animation_builder.exists() and not os.environ.get('MADAM_BASE_ONLY'):
        subprocess.run([sys.executable,str(animation_builder),'--base-ready'],check=True)
