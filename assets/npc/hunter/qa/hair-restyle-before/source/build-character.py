"""Build the hunter neutral layers from the reference and complete generated underlayer."""
from pathlib import Path
from collections import deque
from PIL import Image, ImageDraw
import base64
import hashlib
import io
import itertools
import json
import yaml

ROOT=Path(__file__).resolve().parents[1]
SIZE=(128,160)


def load(path):return Image.open(ROOT/path).convert('RGBA')
def blank():return Image.new('RGBA',SIZE)
def save(image,path):
    target=ROOT/path;target.parent.mkdir(parents=True,exist_ok=True);image.save(target);return image
def write(path,value):
    target=ROOT/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(json.dumps(value,indent=2)+'\n')
def pixels(image):return bytes(v for p in image.getdata() for v in (p if p[3] else (0,0,0,0)))
def compose(images):
    out=blank()
    for image in images:out.alpha_composite(image)
    return out
def move(image,dx,dy):
    out=blank();out.paste(image,(dx,dy));return out
def mask_polygons(polygons):
    mask=Image.new('L',SIZE);d=ImageDraw.Draw(mask)
    for polygon in polygons:d.polygon(polygon,fill=255)
    return mask
def region(image,mask):
    out=blank();out.paste(image,(0,0),mask);return out
def largest_component(image):
    points={(x,y) for y in range(160) for x in range(128) if image.getpixel((x,y))[3]};groups=[]
    while points:
        queue=[points.pop()];group=[]
        while queue:
            x,y=queue.pop();group.append((x,y))
            for dx,dy in itertools.product([-1,0,1],repeat=2):
                p=x+dx,y+dy
                if p in points:points.remove(p);queue.append(p)
        groups.append(group)
    keep=set(max(groups,key=len));out=blank()
    for p in keep:out.putpixel(p,image.getpixel(p))
    return out


GEAR_REGIONS={
 'bow':[[(28,47),(32,47),(35,50),(36,56),(36,63),(38,68),(42,71),(44,73),(43,76),(40,77),(36,73),(34,69),(33,63),(33,55),(31,52),(28,51)]],
 'quiver-and-arrows':[[(44,59),(47,58),(49,60),(51,59),(53,62),(52,66),(49,69),(46,68),(43,65),(42,63)]],
 'fur-hat':[[(42,39),(44,33),(48,28),(54,25),(60,22),(65,20),(69,20),(69,22),(73,22),(74,20),
              (76,20),(76,23),(81,24),(86,28),(87,31),(88,35),(88,39),(87,41),(83,40),(78,39),
              (74,39),(69,41),(64,43),(60,43),(57,45),(54,46),(51,47),(49,47),(48,50),(46,53),
              (46,58),(44,62),(43,65),(40,65),(37,62),(36,58),(35,54),(35,49),(36,46),(38,43),(39,41)]],
 'cloak':[[(51,60),(57,60),(63,62),(67,64),(70,63),(75,61),(79,62),(80,66),(83,68),(83,72),
            (85,75),(84,79),(81,78),(78,75),(74,74),(71,73),(68,74),(64,76),(59,77),(55,77),
            (50,77),(46,78),(41,78),(40,74),(44,69),(46,66),(49,65),(49,62)],
          [(64,72),(68,71),(71,75),(71,78),(68,80),(65,79),(63,76)],
          [(67,60),(76,59),(80,61),(81,66),(72,67),(68,65)]] ,
 'leather-outfit':[
    [(50,64),(53,63),(58,67),(65,72),(69,77),(75,81),(78,85),(77,88),(73,86),(69,83),(65,79),(60,75),(56,72),(52,69)],
    [(53,76),(58,74),(63,77),(68,79),(74,75),(78,74),(79,82),(78,89),(79,94),(78,98),(81,104),
     (79,107),(73,107),(69,105),(65,99),(63,101),(61,106),(56,108),(52,107),(54,100),(53,96),(52,92),(54,89),(52,85)],
    [(51,87),(58,87),(63,88),(70,88),(76,87),(80,89),(79,94),(74,94),(70,94),(64,94),(59,93),(54,94),(51,92)],
    [(43,92),(49,91),(54,95),(56,100),(55,106),(53,109),(48,111),(42,109),(39,106),(39,102),(40,98),(42,97)],
    [(77,84),(80,84),(81,87),(81,94),(83,98),(83,105),(81,109),(78,108),(77,103),(77,97),(77,94),(76,92)]],
 'gloves':[[(46,86),(49,87),(50,88),(53,88),(56,90),(57,92),(56,94),(54,95),(52,95),(48,94),(47,92),(44,91),(43,89)],
           [(83,94),(87,94),(89,97),(89,101),(90,105),(88,109),(85,110),(81,108),(80,105),(81,102),(81,99),(82,97)]],
 'boot-fur-cuffs':[[(44,115),(48,116),(54,117),(60,116),(62,117),(63,119),(63,122),(59,124),(54,124),(47,123),(43,121),(43,117)],
                   [(68,116),(73,117),(80,117),(83,116),(85,119),(84,122),(81,124),(74,124),(69,123),(68,120)]]}


def build():
    contract=json.loads((ROOT/'source/production-contract.json').read_text())
    reference=load('source/reference.png');rgb=reference.copy();w,h=rgb.size;seen=set();queue=deque()
    for x in range(w):queue.extend([(x,0),(x,h-1)])
    for y in range(h):queue.extend([(0,y),(w-1,y)])
    while queue:
        x,y=queue.popleft()
        if (x,y) in seen or not (0<=x<w and 0<=y<h) or max(rgb.getpixel((x,y))[:3])>8:continue
        seen.add((x,y));queue.extend([(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
    for point in seen:rgb.putpixel(point,(0,0,0,0))
    reference=rgb.transform(SIZE,Image.Transform.AFFINE,tuple(contract['reference_transform']),Image.Resampling.NEAREST)
    raw=load('source/underlayer-candidate.png');foundation=raw.resize(SIZE,Image.Resampling.NEAREST)
    foundation.putalpha(foundation.getchannel('A').point(lambda a:255 if a>=128 else 0))
    # Remove generated color specks before selecting the connected character silhouette.
    for y in range(160):
        for x in range(128):
            r,g,b,a=foundation.getpixel((x,y))
            chroma=(g>r*1.5 and g>b*1.2) or (b>r*1.3 and b>g*1.2) or (r>160 and b>100 and g<100)
            if a and chroma:foundation.putpixel((x,y),(0,0,0,0))
    foundation=largest_component(foundation);dy=140-foundation.getchannel('A').getbbox()[3];foundation=move(foundation,0,dy)
    colors=[p[:3] for im in [reference,foundation] for p in im.getdata() if p[3]]
    strip=Image.new('RGB',(len(colors),1));strip.putdata(colors);palette=strip.quantize(colors=61,method=Image.Quantize.MEDIANCUT)
    entries=palette.getpalette()
    for i,rgb in enumerate([(22,12,10),(255,245,228),(202,143,58)],61):entries[i*3:i*3+3]=rgb
    entries[192:]=entries[:3]*192;palette.putpalette(entries);save(palette,'source/palette.png')
    def native(image):
        out=image.convert('RGB').quantize(palette=palette,dither=Image.Dither.NONE).convert('RGBA');out.putalpha(image.getchannel('A'))
        for y in range(160):
            for x in range(128):
                if not out.getpixel((x,y))[3]:out.putpixel((x,y),(0,0,0,0))
        return out
    def color(target):
        colors=[tuple(entries[i:i+3]) for i in range(0,192,3)]
        return min(colors,key=lambda c:sum((a-b)**2 for a,b in zip(c,target)))+(255,)
    reference=native(reference);foundation=native(foundation);save(reference,'source/reference-native.png')
    face_polygon=[(66,43),(73,42),(77,42),(78,45),(80,45),(80,48),(81,50),(81,54),(79,57),
                  (76,59),(71,61),(66,60),(62,59),(59,56),(59,52),(60,48),(62,45)]
    face_mask=mask_polygons([face_polygon])
    for y in range(160):
        for x in range(128):
            if face_mask.getpixel((x,y)) and reference.getpixel((x,y))[3]:foundation.putpixel((x,y),reference.getpixel((x,y)))
    for x in range(76,82):foundation.putpixel((x,56),(0,0,0,0))
    # Reconstruct jaw, neck and collar together instead of cutting a neck from cloak pixels.
    repair_source=load('source/neck-repair/candidate.png')
    repair=repair_source.resize(SIZE,Image.Resampling.NEAREST)
    repair.putalpha(repair.getchannel('A').point(lambda a:255 if a>=128 else 0))
    repair=native(repair)
    repair_polygon=[(58,57),(79,57),(79,64),(83,68),(79,74),(75,79),(65,79),(60,73),(56,68)]
    repair_mask=mask_polygons([repair_polygon])
    for y in range(160):
        for x in range(128):
            if repair_mask.getpixel((x,y)):
                foundation.putpixel((x,y),repair.getpixel((x,y)))
                face_mask.putpixel((x,y),0)
    save(repair_mask,'source/neck-repair/repair-mask.png')
    save(region(repair,repair_mask),'source/neck-repair/registered.png')
    hair_source=load('source/hair-hand-repair/hair-candidate.png')
    hair_native=hair_source.resize(SIZE,Image.Resampling.NEAREST)
    hair_native.putalpha(hair_native.getchannel('A').point(lambda a:255 if a>=128 else 0))
    hair_native=native(hair_native)
    hair_polygon=[(38,19),(90,19),(90,35),(84,39),(67,39),(54,42),(44,44),(38,42)]
    hair_mask=mask_polygons([hair_polygon]);applied_hair=Image.new('L',SIZE)
    for y in range(19,45):
        for x in range(38,91):
            if not hair_mask.getpixel((x,y)):continue
            # Fit the generated crown to a fuller 22..42 native-pixel arc; leave the fringe registered.
            source_y=round(25+(y-22)*17/20) if y<=42 else y
            q=hair_native.getpixel((x,source_y))
            if not q[3]:continue
            r,g,b,_=q
            if (g>r*1.5 and g>b*1.2) or (b>r*1.3 and b>g*1.2) or (r>160 and b>100 and g<100):continue
            if not foundation.getpixel((x,y))[3] and not reference.getpixel((x,y))[3]:continue
            foundation.putpixel((x,y),q);applied_hair.putpixel((x,y),255)
    hand_fills=[((45,91),(46,91)),((61,92),(60,92))]
    for point,neighbor in hand_fills:foundation.putpixel(point,foundation.getpixel(neighbor))
    save(applied_hair,'source/hair-hand-repair/hair-mask.png')
    save(region(foundation,face_mask),'source/reference-face-interior.png')
    save(foundation,'source/complete-underlayer-native.png')
    head_mask=mask_polygons([[(0,0),(127,0),(127,59),(79,59),(76,62),(63,62),(60,60),(59,64),(55,64),(51,61),(0,59)]])
    head=region(foundation,head_mask);body=foundation.copy()
    for y in range(160):
        for x in range(128):
            if head_mask.getpixel((x,y)):body.putpixel((x,y),(0,0,0,0))
    upper=body.copy();lower=body.copy()
    for y in range(160):
        for x in range(128):
            (lower if y<97 or (x>=82 and y<111) else upper).putpixel((x,y),(0,0,0,0))
    eyes=blank();mouth=blank();source_head=head.copy()
    eye_boxes=[(61,44,71,54),(76,45,81,54),(61,42,69,44),(76,42,81,45)]
    mouth_box=(67,54,74,57)
    for y in range(40,60):
        for x in range(58,82):
            p=source_head.getpixel((x,y));r,g,b,a=p
            eye=any(l<=x<rr and t<=y<bb for l,t,rr,bb in eye_boxes)
            lip=mouth_box[0]<=x<mouth_box[2] and mouth_box[1]<=y<mouth_box[3]
            feature=eye and (max(r,g,b)<190 or (r>210 and g>195 and b>150)) or lip and r<205 and r>g*1.3
            if a and feature:
                (eyes if eye else mouth).putpixel((x,y),p)
                neighbors=[source_head.getpixel((xx,yy)) for yy in range(max(0,y-3),min(160,y+4)) for xx in range(max(0,x-3),min(128,x+4))]
                skin=[q for q in neighbors if q[3] and q[0]>215 and 120<q[1]<205 and 65<q[2]<160]
                head.putpixel((x,y),min(skin,key=lambda q:abs(q[1]-177)) if skin else color((252,180,114)))
    parts={'lower':lower,'upper':upper,'head':head,'eyes':eyes,'mouth':mouth}
    assert pixels(compose(list(parts.values())))==pixels(foundation),'layer separation changed the underlayer'
    base_paths={'lower':'body/lower-base.png','upper':'body/upper-base.png','head':'head-base.png','eyes':'eyes/open.png','mouth':'mouth/rest.png'}
    for name,path in base_paths.items():save(parts[name],path)
    save(foundation,'base-neutral.png')
    gear={}
    for name,polygons in GEAR_REGIONS.items():
        layer=region(reference,mask_polygons(polygons))
        if name in ['cloak','leather-outfit','gloves']:
            for y in range(160):
                for x in range(128):
                    r,g,b,a=layer.getpixel((x,y))
                    if not a:continue
                    skin=r>225 and 130<g<205 and b>70 and r-g<115
                    if name=='cloak':
                        green=g>=r*.90 and b<g*.95 and r<150
                        clasp=63<=x<=71 and 71<=y<=80 and r>135 and g>80 and b<135
                        keep=not skin
                    else:
                        cream=r>190 and g>165 and b>125
                        green=g>r*1.03 and g>b*1.10
                        keep=not (skin or cream or green)
                    if not keep:layer.putpixel((x,y),(0,0,0,0))
        if name=='leather-outfit':
            glove_mask=mask_polygons(GEAR_REGIONS['gloves'])
            for y in range(160):
                for x in range(128):
                    if glove_mask.getpixel((x,y)):layer.putpixel((x,y),(0,0,0,0))
        gear[name]=save(layer,f'equipment/{name}.png')
    hands=blank()
    for y in range(83,111):
        for x in range(38,92):
            if not (x<60 and y<99 or x>=80):continue
            p=foundation.getpixel((x,y));r,g,b,a=p
            skin=a and r>205 and 100<g<205 and 55<b<165
            neighbor=any(0<=x+dx<128 and 0<=y+ddy<160 and (q:=foundation.getpixel((x+dx,y+ddy)))[3]
                         and q[0]>205 and 100<q[1]<205 and 55<q[2]<165 for dx,ddy in [(1,0),(-1,0),(0,1),(0,-1)])
            if skin or (a and max(r,g,b)<160 and neighbor):hands.putpixel((x,y),p)
    save(hands,'body/hands-front.png')
    order=['bow','quiver-and-arrows','lower','upper','head','eyes','mouth','fur-hat','cloak','leather-outfit','boot-fur-cuffs','hands-front','gloves']
    all_layers={**parts,**gear,'hands-front':hands};full=compose([all_layers[n] for n in order]);save(full,'neutral.png')
    for name,image in all_layers.items():save(image,f'poses/neutral/{"equipment" if name in gear else "layers"}/{name}.png')
    for path,image in [('frames/neutral.png',full),('frames/base/neutral.png',foundation),('poses/neutral/base.png',foundation),('poses/neutral/composed.png',full)]:save(image,path)
    for name,image in [('neutral',full),('base-neutral',foundation)]:save(image.resize((512,640),Image.Resampling.NEAREST),f'previews/{name}-4x.png')
    sockets={'neck':[68,62],'waist':[65,91],'right_hand':[53,93],'left_hand':[86,104],'head':[67,44]}
    recipe={'layers':{n:f'poses/neutral/layers/{n}.png' for n in parts}|{'hands-front':'poses/neutral/layers/hands-front.png'},
            'equipment':{n:f'poses/neutral/equipment/{n}.png' for n in gear},'base_order':list(parts),'composition_order':order,'sockets_px':sockets}
    write('poses/recipes.json',{'canvas':[128,160],'foot_anchor_px':[64,140],'poses':{'neutral':recipe}})
    write('animation.json',{'version':1,'id':'hunter','canvas':[128,160],'foot_anchor_px':[64,140],'frames':['neutral'],'clips':{}})
    atlas=Image.new('RGBA',(144,176));atlas.paste(full,(2,2))
    for yy in range(-1,161):
        for xx in range(-1,129):
            if xx in (-1,128) or yy in (-1,160):atlas.putpixel((xx+2,yy+2),full.getpixel((max(0,min(127,xx)),max(0,min(159,yy)))))
    save(atlas,'atlas.png');write('atlas-manifest.json',{'version':1,'mode':'baked','usage':'neutral_authoring_not_integrated','sampler':'nearest',
        'mipmaps':False,'extrude_px':1,'gutter_px':1,'pages':[{'id':'hunter-0','path':'atlas.png','size':[144,176]}],
        'sprites':[{'id':'neutral','page':'hunter-0','rect':[2,2,128,160],'source_size':[128,160],'trim_offset':[0,0],'pivot':[64,140],
                    'source':'frames/neutral.png','rotated':False}],'clips':{}})
    registration={'reference_transform':contract['reference_transform'],'underlayer_source_size':list(raw.size),'underlayer_native_size':list(SIZE),
        'underlayer_translation':[0,dy],'face_polygon':face_polygon,'eye_boxes':eye_boxes,'mouth_box':list(mouth_box),'gear_regions':GEAR_REGIONS,
        'equipment_order':[n for n in order if n in gear],'composition_order':order,'alpha':'binary; largest connected underlayer after color-speck cleanup',
        'hair_hand_repair':{'source':'source/hair-hand-repair/hair-candidate.png','mask':'source/hair-hand-repair/hair-mask.png','crown_native_registration':{'source_y':[25,42],'target_y':[22,42],'sampling':'nearest integer coordinates'},'hand_holes_filled':[[45,91],[61,92]],'face_neck':'unchanged'},
        'neck_repair':{'source':'source/neck-repair/candidate.png','native_size':[128,160],'translation':[0,0],'polygon':repair_polygon,'method':'one coherent generated jaw-neck-collar region; no source cloak or row-shaped neck fill'},
        'leather_grouping':'coat, strap, belt, pouches and sheathed knife toggle together; no fictional bare coat under a removed belt'}
    write('source/registration.json',registration)
    measurements={'head_bbox_px':list(source_head.getbbox()),'head_size_without_hat_px':[source_head.getbbox()[2]-source_head.getbbox()[0],source_head.getbbox()[3]-source_head.getbbox()[1]],
                  'face_bbox_px':list(face_mask.getbbox()),'eye_bbox_with_brows_px':{side:list(region(eyes,mask_polygons([[(l,t),(r-1,t),(r-1,b-1),(l,b-1)]])).getbbox()) for side,(l,t,r,b) in {'near':(61,42,71,54),'far':(76,42,81,54)}.items()},
                  'eye_white_iris_design':'large brown pupils with cream highlights; reference facial pixels retained',
                  'hair_color':'short tousled dark brown, restored under fur hat','beard':'subtle warm jaw stubble retained from reference',
                  'body_build':'youthful stocky hunter, original large head and short sturdy legs',
                  'outline_rule':'native dark contour; separate fur/leather shading'}
    def span(image,y,left,right):
        xs=[x for x in range(left,right) if image.getpixel((x,y))[3]]
        return {'y':y,'x_interval':[min(xs),max(xs)+1],'width_px':max(xs)-min(xs)+1} if xs else None
    measurements['eye_size_with_brows_px']={name:[b[2]-b[0],b[3]-b[1]] for name,b in measurements['eye_bbox_with_brows_px'].items()}
    measurements['eye_gap_px']=measurements['eye_bbox_with_brows_px']['far'][0]-measurements['eye_bbox_with_brows_px']['near'][2]
    measurements['neck_visible_skin_rows_px']=[60,68]
    measurements['neck_reconstruction']='generated coherent jaw, warm stubble, neck and cream collar; source cloak excluded'
    measurements['leg_span_to_sole_px']=[97,140]
    measurements['visible_cross_sections']={'neck':span(upper,63,60,80),'shoulders':span(foundation,70,32,88),
        'chest':span(foundation,78,50,82),'waist':span(foundation,95,50,80),'near_thigh':span(lower,108,40,64),
        'far_thigh':span(lower,108,66,88),'near_arm':span(upper,84,30,50)}
    spec={'version':1,'id':'hunter','role':'npc','action_set':'neutral_only','grid':'B','grid_unit_px':[16,16],
          'canvas_cells':[8,10],'native_canvas_px':[128,160],'origin_px':[0,0],'foot_anchor_px':[64,140],'preview_scale':4,
          'pose_canvas':{'grid':'B','canvas_cells':[8,10],'native_canvas_px':[128,160],'base_offset_px':[0,0],'foot_anchor_px':[64,140]},
          'reference_roles':{'identity':'source/reference.png','style':'source/reference.png'},
          'reconstructed_occluded_regions':['complete hair under fur hat','cream shirt under cloak and leather outfit','bare hands under fingerless gloves','plain boot shafts under fur cuffs'],
          'baseline':{'complete_underlayer':'base-neutral.png','foundation':'source/complete-underlayer-native.png','builder':'source/build-character.py','coverage_report':'qa/validation.json'},
          'palette':{'max_opaque_colors':64,'reason':contract['palette_reason'],'source':'source/palette.png','alpha_values':[0,255]},
          'identity':measurements,'sockets_px':sockets,'limb_identity':{'side_convention':'character_anatomical','near_side':'right','far_side':'left'},
          'layers':{'head_base':'head-base.png','upper_base':'body/upper-base.png','lower_base':'body/lower-base.png','hands_front':'body/hands-front.png',
                    'eyes':{'open':'eyes/open.png'},'mouth':{'rest':'mouth/rest.png'},'equipment':[f'equipment/{n}.png' for n in gear]},
          'eyebrow_owner':'eyes','composition_order':order,'expression_recipes':{'neutral':{'eyes':'open','mouth':'rest'}},
          'production_plan':{'status':'neutral_only_complete','completed':['neutral'],'not_generated':['extra expressions','nod','shake_head','give','walk']},
          'animation':{'frames':['neutral'],'clips':{}},'runtime_export':{'status':'not_integrated','mode':'baked','manifest':'atlas-manifest.json','sampler':'nearest',
          'mipmaps':False,'extrude_px':1,'gutter_px':1,'rotate_packing':False},'generation':{'tool':'built-in image_gen','final_prompts':'source/generation-prompts.json','registration':'source/registration.json'}}
    (ROOT/'character-spec.yaml').write_text(yaml.safe_dump(spec,sort_keys=False,allow_unicode=True))
    previews(reference,foundation,full,all_layers,order,gear)
    print('Built hunter neutral, complete gear-free body, seven removable equipment groups; 128x160')


def previews(reference,base,full,layers,order,gear):
    def contact(items,path,columns,scale=4,bg='#eee3cc',crop=None):
        box=crop or (0,0,128,160);w,h=(box[2]-box[0])*scale,(box[3]-box[1])*scale;out=Image.new('RGB',((w+16)*columns,(h+36)*((len(items)+columns-1)//columns)),bg);d=ImageDraw.Draw(out)
        for k,(name,im) in enumerate(items):
            x,y=k%columns*(w+16),k//columns*(h+36);d.text((x+8,y+8),name,fill='#eee3cc' if bg=='#302b32' else '#211510');im=im.crop(box).resize((w,h),Image.Resampling.NEAREST);out.paste(im,(x+8,y+28),im)
        save(out,path)
    contact([('Reference',reference),('Complete base',base),('Equipped',full)],'previews/comparison-4x.png',3)
    contact([('Complete base',base),('Equipped',full)],'previews/face-8x.png',2,8,crop=(38,18,94,68))
    contact([('Complete base',base),('Equipped',full)],'previews/hands-8x.png',2,8,crop=(34,78,94,114))
    for bg in ['light','dark']:
        variants=[('Base',base),('All equipment',full),('No hat',compose([layers[n] for n in order if n!='fur-hat'])),
                  ('No gloves',compose([layers[n] for n in order if n!='gloves']))]
        contact(variants,f'previews/toggles-{bg}.png',4,4,'#302b32' if bg=='dark' else '#eee3cc')
    contact(list(gear.items()),'previews/equipment-4x.png',4)
    def uri(image):
        out=io.BytesIO();image.save(out,format='PNG');return 'data:image/png;base64,'+base64.b64encode(out.getvalue()).decode()
    data={n:uri(im) for n,im in layers.items()}
    html='''<!doctype html><html lang="en"><meta charset="utf-8"><title>Hunter neutral review</title><style>body{font:15px system-ui;background:#e5dac5;color:#30231e;margin:20px}nav{display:flex;gap:14px;flex-wrap:wrap;margin:14px 0}section{display:flex;gap:20px;overflow:auto;background:#eee3cc;padding:12px}figure{margin:0}canvas{display:block;image-rendering:pixelated;width:512px;height:640px}body.dark{background:#201e24;color:#eedfc8}body.dark section{background:#302b32}button,select{font:inherit}figcaption{margin-bottom:8px}</style><h1>Hunter · neutral pose</h1><nav><label>Scale <select id="scale"><option value="1">1× native</option><option selected value="4">4× nearest</option></select></label><button id="bg">Light / dark</button><button id="none">Equipment off</button><button id="all">All equipment</button></nav><nav id="gear"></nav><section><figure><figcaption>Complete base / bare hands</figcaption><canvas id="base" width="128" height="160"></canvas></figure><figure><figcaption>Equipment</figcaption><canvas id="full" width="128" height="160"></canvas></figure></section><p>128×160 · fixed foot pivot (64,140) · 64-color shared palette · neutral only.</p><p>Leather coat, harness, belt, pouches and sheathed knife toggle as one outfit. No game runtime integration.</p><script>'''
    html+='const urls='+json.dumps(data)+';const order='+json.dumps(order)+';const gear='+json.dumps(list(gear))+';'
    html+='''const images={};for(const [n,url]of Object.entries(urls)){const im=new Image();im.src=url;images[n]=im}const baseOrder=['lower','upper','head','eyes','mouth'];function draw(){for(const id of ['base','full']){const ctx=document.getElementById(id).getContext('2d');ctx.clearRect(0,0,128,160);ctx.imageSmoothingEnabled=false;for(const n of id==='base'?baseOrder:order){if(gear.includes(n)&&!document.getElementById('g-'+n).checked)continue;ctx.drawImage(images[n],0,0)}}}for(const n of gear){const label=document.createElement('label'),input=document.createElement('input');input.type='checkbox';input.id='g-'+n;input.checked=true;input.onchange=draw;label.append(input,document.createTextNode(n));document.getElementById('gear').append(label)}Promise.all(Object.values(images).map(i=>i.decode())).then(draw);document.getElementById('scale').onchange=e=>{for(const c of document.querySelectorAll('canvas')){c.style.width=128*e.target.value+'px';c.style.height=160*e.target.value+'px'}};document.getElementById('bg').onclick=()=>document.body.classList.toggle('dark');for(const [id,checked]of [['none',false],['all',true]])document.getElementById(id).onclick=()=>{for(const n of gear)document.getElementById('g-'+n).checked=checked;draw()};</script></html>'''
    (ROOT/'previews/index.html').write_text(html)


if __name__=='__main__':build()
