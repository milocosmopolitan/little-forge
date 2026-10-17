"""Build the neutral carpenter from retained reference and complete underlayer."""
from pathlib import Path
from PIL import Image, ImageDraw
import base64
import hashlib
import io
import json
import math
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
SIZE = (96, 128)
TRANSFORM = (10, 0, 60, 0, 10, 130)

def load(path):
    return Image.open(ROOT/path).convert('RGBA')

def save(image, path):
    path=ROOT/path;path.parent.mkdir(parents=True,exist_ok=True);image.save(path)
    return image

def write(path, value):
    (ROOT/path).write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')

def blank():
    return Image.new('RGBA',SIZE)

def mask(points):
    out=Image.new('L',SIZE);ImageDraw.Draw(out).polygon(points,fill=255);return out

def cut(image, points):
    out=blank();out.paste(image,(0,0),mask(points));return out

def compose(layers):
    out=blank()
    for layer in layers:out.alpha_composite(layer)
    return out

def pixels(image):
    return bytes(v for p in image.getdata() for v in (p if p[3] else (0,0,0,0)))

def sample(image):
    return image.transform(SIZE,Image.Transform.AFFINE,TRANSFORM,Image.Resampling.NEAREST)

def largest_component(image):
    occupied={(x,y) for y in range(128) for x in range(96) if image.getpixel((x,y))[3]}
    groups=[]
    while occupied:
        group={occupied.pop()};todo=list(group)
        while todo:
            x,y=todo.pop()
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                q=(x+dx,y+dy)
                if q in occupied:occupied.remove(q);group.add(q);todo.append(q)
        groups.append(group)
    keep=max(groups,key=len)
    for y in range(128):
        for x in range(96):
            if (x,y) not in keep:image.putpixel((x,y),(0,0,0,0))
    return image

def contact(items,path,scale=4,crop=None,dark=False,columns=3):
    crop=crop or (0,0,96,128);w,h=crop[2]-crop[0],crop[3]-crop[1]
    cw,ch=w*scale+16,h*scale+32
    bg,fg=('#302b32','#eee3cc') if dark else ('#eee3cc','#211510')
    out=Image.new('RGB',(cw*columns,ch*math.ceil(len(items)/columns)),bg);d=ImageDraw.Draw(out)
    for i,(name,im) in enumerate(items):
        x,y=i%columns*cw,i//columns*ch;d.text((x+8,y+8),name,fill=fg)
        im=im.crop(crop).resize((w*scale,h*scale),Image.Resampling.NEAREST);out.paste(im,(x+8,y+26),im)
    save(out,path)

def build():
    for folder in ['body','eyes','mouth','equipment','previews','qa']:(ROOT/folder).mkdir(exist_ok=True)
    reference=sample(load('source/reference.png'))
    raw=sample(load('source/generated-underlayer.png'))
    raw.putalpha(raw.getchannel('A').point(lambda a:255 if a>=128 else 0))
    raw=largest_component(raw)
    # Re-register the generated supporting hand to the reference glove, without a tool.
    hand=cut(raw,[(31,69),(36,68),(43,69),(44,72),(43,76),(40,79),(35,79),(31,76)])
    d=ImageDraw.Draw(raw)
    d.polygon([(31,69),(36,68),(43,69),(44,72),(43,76),(40,79),(35,79),(31,76)],fill='#edcda6')
    d.line([(37,70),(37,74),(35,78)],fill='#cfad87')
    raw.alpha_composite(hand,(-6,0))
    save(raw,'source/underlayer-registered.png')

    polygons={
        'apron-tools': [[(40,58),(64,57),(65,67),(65,77),(67,80),(68,85),(68,92),
            (71,94),(69,97),(63,99),(47,99),(41,98),(40,94),(40,86),(39,81),
            (36,80),(35,76),(37,72),(38,64)]],
        'straps': [[(34,46),(37,45),(40,47),(41,51),(45,53),(46,57),(46,61),
            (42,61),(41,57),(39,54),(37,50),(35,49)],
            [(60,49),(63,50),(65,53),(65,59),(61,60),(59,58),(59,55)]],
        'belt-pouches': [[(38,75),(45,75),(47,78),(46,82),(41,83),(37,80)]],
        'towel': [[(33,78),(37,77),(39,80),(39,87),(38,90),(40,96),(39,99),
            (35,98),(34,96),(30,95),(31,91),(32,86)]],
        'boards': [[(29,60),(35,57),(40,55),(42,56),(43,61),(42,68),(38,71),
            (34,70),(30,67)],[(17,73),(21,71),(25,72),(25,77),(23,80),(18,81),(17,78)]],
        'gloves': [[(27,69),(31,68),(34,71),(36,74),(35,77),(33,79),(29,79),(26,77),(25,73)],
            [(71,74),(74,73),(77,75),(78,78),(78,81),(76,82),(74,82),(71,80),(70,77)]],
        'mallet': [[(79,59),(81,59),(86,62),(92,65),(92,70),(90,73),(85,74),
            (83,72),(81,69),(78,67),(77,63)],
            [(81,70),(84,72),(82,77),(80,83),(78,89),(75,89),(75,86),(78,77)]],
    }
    gear_raw={}
    for name,regions in polygons.items():
        layer=blank()
        for points in regions:layer.alpha_composite(cut(reference,points))
        gear_raw[name]=layer
    # Each visible reference pixel has one equipment owner, including overlap edges.
    owned=Image.new('L',SIZE)
    for name in ['gloves','boards','towel','straps','belt-pouches','apron-tools']:
        layer=gear_raw[name]
        for y in range(128):
            for x in range(96):
                if layer.getpixel((x,y))[3]:
                    if owned.getpixel((x,y)):layer.putpixel((x,y),(0,0,0,0))
                    else:owned.putpixel((x,y),255)
    # Fingerless gloves and the shaft contain hand-colored gaps, never copied skin.
    for name in ['gloves','mallet']:
        layer=gear_raw[name]
        for y in range(65,90):
            for x in range(96):
                p=layer.getpixel((x,y))
                if p[3] and p[0]>210 and p[1]>105 and p[1]<205 and p[2]<150:
                    layer.putpixel((x,y),(0,0,0,0))
    # Lock the approved character palette so local repairs cannot recolor the face.
    palette=Image.open(ROOT/'source/palette-locked.png').convert('P')
    entries=palette.getpalette()
    colors=[tuple(entries[i:i+3]) for i in range(0,96,3)]
    palette.save(ROOT/'source/palette.png')
    def c(value):
        rgb=tuple(bytes.fromhex(value.lstrip('#')))
        return min(colors,key=lambda p:sum((a-b)**2 for a,b in zip(p,rgb)))+(255,)
    def native(im):
        out=im.convert('RGB').quantize(palette=palette,dither=Image.Dither.NONE).convert('RGBA')
        out.putalpha(im.getchannel('A'));return out
    base=native(raw)
    gear={name:native(im) for name,im in gear_raw.items()}
    # Replace clipped black-background fringes with local, closed native contours.
    outlines={
        'boards': [[(29,60),(35,57),(40,55),(42,56),(43,61),(42,68),(38,71),(34,70)],
                   [(17,73),(21,71),(24,72),(25,77),(23,80),(18,81),(17,78),(17,73)]],
        'mallet': [[(79,59),(81,59),(86,62),(92,65),(92,70),(90,73),(85,74),(83,72),
                    (81,69),(78,67),(77,63),(79,59)],
                   [(81,74),(78,84),(76,88),(78,89),(81,82)]],
        'towel': [[(33,79),(32,86),(31,91),(30,95),(34,96),(35,98),(39,99),(40,96)]],
    }
    for name,paths in outlines.items():
        d=ImageDraw.Draw(gear[name])
        for path in paths:d.line(path,fill=c('#281d18'),width=1)
    # Pouches own leather and brass only, never a copied piece of the towel.
    for y in range(78,90):
        for x in range(28,36):
            px=gear['belt-pouches'].getpixel((x,y))
            if px[3] and (min(px[:3])>125 or px[1]>px[0]*0.82):
                gear['belt-pouches'].putpixel((x,y),(0,0,0,0))
    # Restore a fitted shirt edge and compact supporting hand on the native grid.
    bd=ImageDraw.Draw(base)
    bd.rectangle((24,68,44,83),fill=(0,0,0,0))
    shirt=[(35,67),(44,67),(44,83),(40,83),(36,82),(35,80),(35,76),(34,72)]
    bd.polygon(shirt,fill=c('#efcfaa'))
    bd.line([(35,67),(34,72),(35,76),(35,80),(36,82),(40,83),(44,83)],fill=c('#281d18'))
    bd.line([(39,71),(38,74),(39,76)],fill=c('#cfad87'))
    bd.line([(40,79),(43,80)],fill=c('#cfad87'))
    # Keep the original proximal forearm; only the wrist and hand change.
    proximal=cut(native(sample(load('source/generated-underlayer.png'))),
        [(20,65),(25,65),(28,68),(28,72),(24,73),(21,70)])
    proximal.putalpha(proximal.getchannel('A').point(lambda a:255 if a>=128 else 0))
    base.alpha_composite(proximal)
    # Register the generated closed fist at the supporting glove socket.
    candidate=native(load('source/support-hand-candidate-native.png'))
    fist=candidate.crop((13,17,22,27))
    fist_mask=Image.new('L',fist.size)
    ImageDraw.Draw(fist_mask).polygon([(0,0),(3,0),(5,1),(6,2),(7,2),(8,3),
        (8,5),(7,6),(7,7),(5,9),(2,9),(1,7),(0,5)],fill=255)
    fist.putalpha(fist_mask)
    bd=ImageDraw.Draw(base)
    bd.polygon([(24,66),(28,66),(29,67),(30,68),(32,69),(33,71),
                (32,74),(29,75),(26,73),(24,70)],fill=c('#fcb573'))
    bd.line([(28,66),(29,67),(30,68),(32,69),(33,71)],fill=c('#281d18'))
    bd.line([(24,69),(25,72),(27,74),(29,75)],fill=c('#c67f4c'))
    base.alpha_composite(fist,(27,69))
    bd=ImageDraw.Draw(base)
    bd.line([(26,70),(27,71),(28,72)],fill=c('#fcb573'))
    save(cut(base,[(24,66),(28,66),(32,69),(35,71),(35,76),(32,79),
                   (29,79),(26,75),(24,71)]),'source/hand-repair-native.png')
    # Compact curled bare fingers at the actual wooden shaft socket.
    bd.rectangle((74,76,84,86),fill=(0,0,0,0))
    bd.rectangle((73,77,84,86),fill=(0,0,0,0))
    bd.rectangle((78,74,84,76),fill=(0,0,0,0))
    bd.polygon([(75,75),(77,75),(78,76),(79,76),(80,77),(80,80),(79,81),
                (79,82),(78,83),(76,83),(75,82),(74,80),(73,79),(73,77)],fill=c('#ffb574'))
    # Only the exposed perimeter is outlined; the wrist has no internal dark border.
    bd.line([(77,75),(78,76),(79,76),(80,77),(80,80),(79,81),(79,82),
             (78,83),(76,83),(75,82),(74,80),(73,79)],fill=c('#281d18'))
    bd.line([(76,78),(78,78),(78,79),(76,80)],fill=c('#b76c3c'))
    bd.line([(76,82),(78,82)],fill=c('#b76c3c'))
    save(base,'source/underlayer-registered.png')

    # A complete cloth silhouette replaces the cropped, leather-contaminated towel.
    towel=blank();td=ImageDraw.Draw(towel)
    td.polygon([(32,78),(36,78),(37,81),(37,87),(36,90),(39,96),(38,99),
                (34,98),(33,96),(29,95),(29,92),(30,89),(30,84),(31,81)],
               fill=c('#f0debd'),outline=c('#281d18'))
    td.line([(35,80),(35,85),(34,89),(35,93),(37,97)],fill=c('#dfc6a2'))
    # Stripe pixels are clipped to the cloth interior so the outer line stays whole.
    stripe=blank();sd=ImageDraw.Draw(stripe)
    sd.line([(29,88),(37,90)],fill=c('#73774b'),width=2)
    sd.line([(29,93),(38,95)],fill=c('#73774b'),width=2)
    for y in range(78,100):
        for x in range(29,40):
            if stripe.getpixel((x,y))[3] and towel.getpixel((x,y))[3] and towel.getpixel((x,y))!=c('#281d18'):
                towel.putpixel((x,y),stripe.getpixel((x,y)))
    gear['towel']=towel

    # Preserve the visible pocket/tool detail inside a clean complete leather apron.
    apron_points=[(43,58),(62,58),(63,60),(63,73),(64,77),(65,83),(66,90),
                  (69,95),(68,97),(63,99),(44,99),(41,98),(41,90),(40,83),
                  (40,79),(41,76),(42,73),(42,64)]
    apron=blank();ad=ImageDraw.Draw(apron)
    ad.polygon(apron_points,fill=c('#986035'))
    source_native=native(reference)
    for y in range(59,99):
        for x in range(40,69):
            px=source_native.getpixel((x,y))
            covered=any(gear[n].getpixel((x,y))[3] for n in ['boards','gloves','towel','belt-pouches','straps'])
            if apron.getpixel((x,y))[3] and not covered and x>=43 and max(px[:3])>30:
                # Exclude the cream shirt fringe on the far edge.
                if not (px[0]>175 and px[1]>145 and px[2]>100):apron.putpixel((x,y),px)
    ad=ImageDraw.Draw(apron)
    ad.line(apron_points+[apron_points[0]],fill=c('#281d18'),width=1)
    ad.line([(43,97),(47,98),(62,98),(67,96)],fill=c('#6e4228'))
    gear['apron-tools']=apron
    # The belt crosses the gap between the supporting glove and apron buckle.
    belt=gear['belt-pouches'];belt_draw=ImageDraw.Draw(belt)
    belt_draw.polygon([(34,74),(36,73),(39,74),(41,76),(41,78),(38,78),(35,76)],
                      fill=c('#86532f'),outline=c('#281d18'))
    belt_draw.line([(36,74),(39,75)],fill=c('#b87d44'))
    # Complete the hanging leather pouch behind the towel, including its rivet.
    pouch_points=[(36,79),(40,80),(42,82),(42,93),(40,97),(36,96),(35,92),(35,83)]
    pouch=blank();pd=ImageDraw.Draw(pouch)
    pd.polygon(pouch_points,fill=c('#86532f'))
    for y in range(80,97):
        for x in range(35,43):
            px=source_native.getpixel((x,y))
            if pouch.getpixel((x,y))[3] and px[0]>px[1]*1.15 and px[1]<160 and max(px[:3])>35:
                pouch.putpixel((x,y),px)
    pd=ImageDraw.Draw(pouch);pd.line(pouch_points+[pouch_points[0]],fill=c('#281d18'))
    pd.rectangle((38,92,39,93),fill=c('#d29b46'))
    belt.alpha_composite(pouch)



    # The source mallet's near face extends farther left than the former crop.
    # Trace its actual block and bevel, keeping black background out of the head.
    mallet=blank();md=ImageDraw.Draw(mallet)
    shaft=[(79,70),(82,71),(77,90),(73,90),(73,88)]
    md.polygon(shaft,fill=c('#86532f'),outline=c('#281d18'))
    md.line([(80,73),(75,88)],fill=c('#b87d44'))
    mallet_head=[(78,58),(80,58),(82,60),(85,61),(87,63),(90,64),(91,65),
                 (91,69),(89,72),(85,74),(83,74),(81,72),(79,71),(77,70),
                 (76,68),(75,67),(75,64),(76,62),(77,60)]
    head_mask=mask(mallet_head)
    for y in range(58,75):
        for x in range(75,92):
            if head_mask.getpixel((x,y)):mallet.putpixel((x,y),source_native.getpixel((x,y)))
    md=ImageDraw.Draw(mallet);md.line(mallet_head+[mallet_head[0]],fill=c('#281d18'))
    save(mallet,'source/mallet-complete.png')
    # Only the two front finger bands interrupt the shaft, not a rectangular palm cut.
    grip=blank();gd=ImageDraw.Draw(grip)
    gd.line([(77,79),(79,79)],fill='white',width=1)
    gd.line([(76,81),(78,81)],fill='white',width=1)
    for y in range(77,84):
        for x in range(73,81):
            if grip.getpixel((x,y))[3] and base.getpixel((x,y))[3]:mallet.putpixel((x,y),(0,0,0,0))
    save(grip,'source/mallet-finger-mask.png');gear['mallet']=mallet
    # Remove the former glove's isolated finger-colored flecks.
    glove=gear['gloves'];gd=ImageDraw.Draw(glove)
    gd.rectangle((78,78,82,85),fill=(0,0,0,0))

    # Complete only the board underside pixels exposed at the elbow.
    wood=ImageDraw.Draw(gear['boards'])
    wood.line([(29,66),(29,67),(30,68),(31,68)],fill=c('#c67f4c'))
    wood.line([(30,66),(30,67),(31,67)],fill=c('#b36739'))
    # Remove copied cloth at the glove edge and retain a compact exposed thumb.
    for y in range(69,81):
        for x in range(34,38):glove.putpixel((x,y),(0,0,0,0))
    gd=ImageDraw.Draw(glove)
    gd.polygon([(32,70),(34,71),(35,73),(35,76),(33,78),(32,78)],fill=c('#754027'))
    gd.line([(33,70),(34,71),(35,73),(35,76),(33,78)],fill=c('#281d18'))
    gd.line([(34,72),(36,72),(36,73),(35,74)],fill=c('#281d18'))
    gd.line([(34,73),(35,73)],fill=c('#fcb573'))

    # Separate features without cutting alpha holes in the face.
    eyes=blank();mouth=blank();head=blank();upper=blank();lower=blank()
    for y in range(128):
        for x in range(96):
            p=base.getpixel((x,y))
            if y<=44:head.putpixel((x,y),p)
            elif y<84 or (x>=70 and y<=86):upper.putpixel((x,y),p)
            else:lower.putpixel((x,y),p)
    eye_regions=[[(47,26),(50,25),(54,25),(55,27),(55,33),(53,34),(47,34),(46,32),(46,29)],
                 [(60,26),(63,26),(64,29),(64,34),(61,35),(60,33)],
                 [(49,23),(53,22),(56,24),(55,25),(51,25),(48,26)],
                 [(62,25),(64,25),(64,27),(62,27)]]
    eye_mask=Image.new('L',SIZE)
    for points in eye_regions:ImageDraw.Draw(eye_mask).polygon(points,fill=255)
    # Extract only eye whites and brown/dark eye pixels; retain cheek/skin shading.
    for y in range(20,36):
        for x in range(44,66):
            p=head.getpixel((x,y))
            if eye_mask.getpixel((x,y)) and p[3] and (max(p[:3])<150 or min(p[:3])>215):
                eyes.putpixel((x,y),p);head.putpixel((x,y),c('#ffb574'))
    for y in range(35,40):
        for x in range(51,62):
            p=head.getpixel((x,y))
            if p[3] and p[0]<180 and p[1]<110:
                mouth.putpixel((x,y),p);head.putpixel((x,y),c('#ffb574'))
    for name,im in [('head-base.png',head),('body/upper-base.png',upper),('body/lower-base.png',lower),
                    ('eyes/open.png',eyes),('mouth/rest.png',mouth)]:save(im,name)
    reconstructed=compose([lower,upper,head,eyes,mouth]);assert pixels(reconstructed)==pixels(base)
    save(base,'base-neutral.png')
    order=['apron-tools','straps','belt-pouches','towel','boards','mallet','gloves']
    gear={name:gear[name] for name in order}
    for name,im in gear.items():save(im,f'equipment/{name}.png')
    full=compose([base]+[gear[name] for name in order]);save(full,'neutral.png')
    assert all(not p[3] or q[3] for p,q in zip(base.getdata(),full.getdata()))
    for name,im in [('neutral',full),('base-neutral',base)]:
        save(im.resize((384,512),Image.Resampling.NEAREST),f'previews/{name}-4x.png')
    contact([('equipment-free',base),('equipped',full)],'previews/layers-light-4x.png',columns=2)
    contact([('equipment-free',base),('equipped',full)],'previews/layers-dark-4x.png',columns=2,dark=True)
    contact([('equipment-free',base),('equipped',full)],'previews/face-8x.png',columns=2,scale=8,crop=(27,3,71,47))
    contact(list(gear.items()),'previews/equipment-4x.png',columns=4)
    before=ROOT/'source/cleanup-before'
    if before.exists():
        for label,crop,items in [
            ('hands',(18,64,45,85),[('BEFORE',load('source/cleanup-before/base-neutral.png')),('AFTER',base)]),
            ('towel',(28,73,48,101),[('BEFORE',load('source/cleanup-before/neutral.png')),('AFTER',full)]),
            ('mallet',(69,56,94,92),[('BEFORE',load('source/cleanup-before/neutral.png')),('AFTER',full)]),
            ('apron',(33,54,72,102),[('BEFORE',load('source/cleanup-before/equipment/apron-tools.png')),('AFTER',gear['apron-tools'])])]:
            contact(items,f'previews/cleanup-{label}-8x.png',columns=2,scale=8,crop=crop)
    if (ROOT/'source/hand-towel-before').exists():
        for label,crop,old,current in [
            ('hand',(18,64,43,85),'base-neutral.png',base),
            ('towel',(27,75,43,101),'neutral.png',full)]:
            contact([('BEFORE',load('source/hand-towel-before/'+old)),('AFTER',current)],
                    f'previews/hand-towel-{label}-8x.png',columns=2,scale=8,crop=crop)
    # Reference uses the exact same source transform, so visible heights are comparable.
    reference_mask=blank();reference_mask.putalpha(compose([base]+list(gear.values())).getchannel('A'))
    ref_show=reference.copy();ref_show.putalpha(reference_mask.getchannel('A'))
    contact([('reference / same scale',ref_show),('native equipped',full)],'previews/reference-comparison-4x.png',columns=2)
    atlas=Image.new('RGBA',(112,144));atlas.alpha_composite(full,(2,2));save(atlas,'atlas.png')
    digest=hashlib.sha256(pixels(full)).hexdigest()
    manifest={'version':1,'sampler':'nearest','mipmaps':False,'extrude_px':1,'gutter_px':1,'trimmed':False,
        'runtime_status':'authoring_export_not_integrated','pages':[{'id':'carpenter-0','path':'atlas.png','size':[112,144]}],
        'sprites':[{'id':'neutral','page':'carpenter-0','rect':[2,2,96,128],'source_size':[96,128],
                    'trim_offset':[0,0],'pivot':[48,124],'source':'neutral.png','rotated':False,'sha256_pixels':digest}],
        'clips':{'idle':{'loop':False,'frames':[{'sprite':'neutral','duration_ms':120}]}}}
    write('atlas-manifest.json',manifest)
    actual_colors=sorted({p[:3] for im in [base,full,head,upper,lower,eyes,mouth]+list(gear.values()) for p in im.getdata() if p[3]})
    assert len(actual_colors)<=32
    def runs(y,x0=0,x1=96):
        spans=[];start=None
        for x in range(x0,x1+1):
            on=x<x1 and base.getpixel((x,y))[3]
            if on and start is None:start=x
            elif not on and start is not None:spans.append([start,x]);start=None
        return spans
    def span(y,x0=0,x1=96):
        spans=runs(y,x0,x1);return max((b-a for a,b in spans),default=0)
    eye_boxes=[]
    for box in [(45,26,57,35),(59,26,65,36)]:
        bbox=eyes.crop(box).getchannel('A').getbbox()
        eye_boxes.append([bbox[0]+box[0],bbox[1]+box[1],bbox[2]+box[0],bbox[3]+box[1]])
    spec={'version':1,'id':'carpenter','scope':'basic standing neutral pose only','grid':'B','grid_unit_px':[16,16],
        'canvas_cells':[6,8],'native_canvas_px':[96,128],'preview_scale':4,'origin_px':[0,0],'foot_anchor_px':[48,124],
        'reference_roles':{'identity':'source/reference.png','style':'source/reference.png'},
        'baseline':{'complete_underlayer':'base-neutral.png','builder':'source/build-character.py','coverage_report':'qa/validation.json'},
        'palette':{'max_opaque_colors':32,'rgba_hex':['#'+''.join(f'{v:02x}' for v in p)+'ff' for p in actual_colors],'alpha_values':[0,255]},
        'identity':{'head_size_without_hat_px':[head.getbbox()[2]-head.getbbox()[0],head.getbbox()[3]-head.getbbox()[1]],
            'face_bbox_px':[37,19,65,44],'eye_size_px':[[b[2]-b[0],b[3]-b[1]] for b in eye_boxes],'eye_bbox_px':eye_boxes,'eye_spacing_px':eye_boxes[1][0]-eye_boxes[0][2],
            'eye_white_iris_design':'large off-white eyes with brown pupils looking screen-right',
            'shoulder_width_px':span(54),'chest_width_px':span(62,35,65),'waist_width_px':span(82,30,70),'upper_arm_width_px':span(60,10,38),
            'thigh_width_px':span(94,25,49),'measurement_rows_px':{'shoulder':54,'chest':62,'waist':82,'upper_arm_horizontal':60,'near_thigh':94},'neck_visible_length_px':9,'body_build':'slim-to-average young carpenter, original large-head short-leg proportions',
            'hair_color':'dirty blond/light brown with warm gold highlights','beard_and_moustache_mask':'short sparse jaw stubble preserved in head-base',
            'measurement_status':'native authored measurements, not claimed pixel-identical reference anatomy',
            'outline_rule':'native 1px dark contour; material shadows retained'},
        'sockets_px':{'neck':[50,46],'waist':[50,82],'eyes':[[51,30],[62,31]],'mouth':[56,38],
                      'anatomical_right_hand':[31,74],'anatomical_left_hand':[77,80]},
        'limb_identity':{'side_convention':'character_anatomical','near_side':'screen-left anatomical right','far_side':'screen-right anatomical left'},
        'layers':{'head_base':'head-base.png','upper_base':'body/upper-base.png','lower_base':'body/lower-base.png',
                  'eyes':{'open':'eyes/open.png'},'mouth':{'rest':'mouth/rest.png'},'equipment':{n:f'equipment/{n}.png' for n in gear}},
        'eyebrow_owner':'eyes','expression_recipes':{'neutral':{'eyes':'open','mouth':'rest'}},
        'composition_order':['lower_base','upper_base','head_base','eyes','mouth']+order,
        'reconstructed_occluded_regions':['shirt beneath apron','trousers beneath apron','bare hands beneath fingerless gloves and tools',
            'screen-left supporting hand moved on native grid to match original glove'],
        'production_plan':{'status':'basic_pose_generated','expressions':['neutral'],'pose_phases':[['neutral']]},
        'runtime_export':{'status':'not_integrated','mode':'baked','manifest':'atlas-manifest.json','sampler':'nearest','mipmaps':False,
            'extrude_px':1,'gutter_px':1,'trim_supported_by_consumer':False,'rotate_packing':False},
        'generation':{'tool':'built-in image_gen','final_prompts':'source/generation-prompts.json','candidate':'source/generated-underlayer.png'},
        'animation':'not_generated'}
    write('character-spec.yaml',spec)
    report={'status':'pass','native_size':list(SIZE),'opaque_union_colors':len(actual_colors),'base_loss_pixels':0,
        'layer_recomposition':'pixel-identical','head_silhouette_complete':True,'expression_invariance':'not_applicable_one_expression',
        'neutral_visible_bbox':full.getchannel('A').getbbox(),'base_visible_bbox':base.getchannel('A').getbbox(),
        'reference_native_transform':list(TRANSFORM),'reference_exact_anatomy':'not_claimed',
        'files':{n:hashlib.sha256(pixels(im)).hexdigest() for n,im in [('neutral.png',full),('base-neutral.png',base)]}}
    write('qa/validation.json',report)
    def data(im):
        f=io.BytesIO();im.save(f,format='PNG');return 'data:image/png;base64,'+base64.b64encode(f.getvalue()).decode()
    html='''<!doctype html><html lang="ko"><meta charset="utf-8"><title>Carpenter 기본 자세</title><style>body{background:#18191b;color:#eee3cc;font:16px system-ui;margin:32px}button{font:inherit;padding:10px;margin:6px}main{display:flex;gap:20px;flex-wrap:wrap}figure{margin:0}img{background:#eee3cc;image-rendering:pixelated;width:384px;height:512px}body.dark img{background:#302b32}figcaption{padding:10px}aside img{width:96px;height:128px}</style><h1>Carpenter 기본 자세</h1><p>96×128 네이티브 · 기본 자세 1개</p><button onclick="document.body.classList.toggle('dark')">밝은 / 어두운 배경</button><main>'''
    for title,im in [('장비 없는 기본층',base),('장비 착용',full)]:html+=f'<figure><figcaption>{title}</figcaption><img src="{data(im)}"></figure>'
    html+='</main><h2>네이티브 1배</h2><aside>'+f'<img src="{data(base)}"><img src="{data(full)}">'+'</aside></html>'
    (ROOT/'previews/index.html').write_text(html)
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    build()
    animation_builder=ROOT/'source/build-animation.py'
    if animation_builder.exists() and os.environ.get('CARPENTER_BASE_ONLY')!='1':
        subprocess.run([sys.executable,str(animation_builder),'--base-ready'],check=True)
