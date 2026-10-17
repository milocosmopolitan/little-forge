"""Build the chef foundation, then its expressions and poses when available."""
from pathlib import Path
import hashlib
import json
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
SIZE = (96, 128)
OUTLINE = '#211510'
PALETTE = ['#211510','#38251d','#503729','#6a4933','#855634','#a0693c',
           '#be8348','#d58b48','#e9a15b','#f7b773','#c4783f',
           '#fff0d5','#efd9b8','#d9bb96','#bf9b73','#a88059',
           '#42422d','#57573a','#6e6b43','#83804f',
           '#6b2520','#96332b','#bb4536','#d55b44',
           '#79521d','#bb811e','#edb633','#ffe16c',
           '#433b35','#65594d','#8a7763','#ffffff']
SHIFT = (0, 2)

def blank():
    return Image.new('RGBA', SIZE)

def mask(points):
    result = Image.new('L', SIZE)
    ImageDraw.Draw(result).polygon(points, fill=255)
    return result

def cut(im, points, outline=False):
    result = blank()
    result.paste(im, (0, 0), mask(points))
    if outline:
        ImageDraw.Draw(result).line(points + [points[0]], fill=OUTLINE, width=1)
    return result

def compose(layers):
    result = blank()
    for layer in layers:
        result.alpha_composite(layer)
    return result

def save(im, name):
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    im.save(path)
    return im

# Coordinates trace the native silhouettes; the generated glow is never exported.
HEAD = [(42,4),(51,4),(55,5),(58,7),(60,10),(62,14),
        (63,16),(62,19),(62,24),(64,27),(65,29),(66,31),(65,35),
        (63,38),(59,40),(53,42),(47,41),(42,40),(38,38),(34,37),
        (31,34),(30,29),(28,28),(27,26),(28,22),(30,20),(29,18),
        (29,16),(31,15),(32,11),(34,8),(37,6),(40,5)]
UPPER = [(32,33),(37,35),(44,40),(52,42),(60,40),(63,39),
         (68,41),(72,43),(74,48),(77,53),(79,56),(80,51),
         (85,49),(89,49),(91,51),(92,54),(92,58),(90,61),
         (87,64),(83,64),(80,66),(77,69),(74,69),(73,73),
         (74,79),(72,81),(74,83),(69,85),(63,85),(60,86),
         (52,85),(47,85),(42,84),(37,83),(28,81),(29,78),
         (30,77),(27,75),(25,72),(22,70),(18,69),(14,68),
         (11,66),(9,63),(8,60),(7,58),(7,55),(8,52),
         (11,49),(13,46),(17,43),(21,40),(26,38),(31,37)]
LOWER = [(31,79),(43,80),(50,82),(62,82),(71,82),(71,91),
         (69,98),(69,102),(68,105),(66,108),(67,111),(72,112),
         (76,114),(78,117),(78,120),(68,120),(63,119),(57,119),
         (54,117),(53,112),(54,108),(53,106),(54,101),(53,98),
         (52,93),(51,89),(49,87),(47,91),(45,94),(44,98),
         (42,101),(42,105),(40,108),(40,112),(40,120),
         (37,121),(26,121),(25,119),(25,115),(26,111),
         (27,108),(26,104),(25,102),(26,99),(28,97),(28,92),
         (29,86),(29,82)]

candidate = Image.open(ROOT / 'source/generated-candidate.png').convert('RGB')
reference_source = Image.open(ROOT / 'source/reference.png').convert('RGB')
candidate = candidate.resize((192, 128), Image.Resampling.NEAREST)
native_rgb = candidate.copy()
pal = Image.new('P', (1, 1))
rgb = [tuple(bytes.fromhex(c[1:])) for c in PALETTE]
pal.putpalette([v for c in rgb for v in c] + [0] * (768 - len(rgb)*3))
candidate = candidate.quantize(palette=pal, dither=Image.Dither.NONE).convert('RGBA')
equipped = candidate.crop((0, 0, 96, 128))
base = candidate.crop((96, 0, 192, 128))
head = cut(base, HEAD, True)
upper = cut(base, UPPER, True)
lower = cut(base, LOWER, True)

# Repair only the scalp arc. Row spans remove the former doubled interior step.
scalp = ImageDraw.Draw(head)
scalp.rectangle((0,0,95,12),fill=(0,0,0,0))
arc = {4:(42,51),5:(38,55),6:(36,57),7:(35,58),8:(34,59),
       9:(33,60),10:(32,60),11:(32,61),12:(31,61)}
for y,(left,right) in arc.items():
    scalp.line((left,y,right,y),fill=PALETTE[8],width=1)
    for x in range(left,right+1):
        above = arc.get(y-1)
        if x in (left,right) or above is None or not above[0] <= x <= above[1]:
            scalp.point((x,y),fill=OUTLINE)
scalp.polygon([(37,10),(39,9),(42,9),(42,10),(39,11),(37,11)],fill=PALETTE[9])

# The empty hand comes from the equipment-free source, never from a cut-up grip.
reference_scale = 118 / 1342
skin_palette = Image.new('P',(1,1))
skin_colors = PALETTE[:11]
skin_palette.putpalette([v for c in skin_colors for v in bytes.fromhex(c[1:])] +
                       [0]*(768-len(skin_colors)*3))
bare_arm_source = native_rgb.crop((96,0,192,128)).quantize(
    palette=skin_palette,dither=Image.Dither.NONE).convert('RGBA')
hand_path = [(85,49),(88,49),(89,50),(90,51),(91,53),(92,55),
             (92,57),(91,58),(90,60),(88,62),(87,64),(85,66),
             (83,67),(81,69),(78,70),(76,69),(74,68),(73,66),
             (73,64),(73,61),(74,59),(76,57),(78,56),(79,57),
             (79,55),(79,53),(80,51),(81,50),(82,50),(82,52),
             (83,52),(83,50)]
arm_path = [(75,60),(78,58),(80,61),(80,65),(78,68),
            (76,68),(74,65),(74,62)]
# Rebuild this local region from a single complete bare-arm source.
ImageDraw.Draw(upper).rectangle((79,49,95,71),fill=(0,0,0,0))
ImageDraw.Draw(upper).rectangle((73,57,95,71),fill=(0,0,0,0))
upper.alpha_composite(cut(base,[(72,54),(76,54),(78,55),(78,57),
                              (75,59),(73,61),(72,60)]))
upper.alpha_composite(cut(bare_arm_source,hand_path))
# A narrow dark contour follows the outside only; the source supplies the anatomy.
ImageDraw.Draw(upper).line(hand_path+[hand_path[0]],fill=OUTLINE,width=1)

# Re-sample the actual reference at the common character scale, then trace only
# the boot contours. Local translations register the original boots to the legs.
reference_scale = 118 / 1342
boot_polygons = [
    ([(270,1238),(291,1255),(365,1264),(444,1245),(443,1277),
      (423,1310),(413,1320),(411,1352),(406,1371),(424,1397),
      (422,1466),(409,1478),(254,1478),(247,1467),(249,1433),
      (258,1400),(266,1374),(270,1332),(264,1301),(263,1270)],3),
    ([(547,1270),(574,1277),(624,1283),(682,1270),(724,1254),
      (728,1286),(709,1320),(695,1346),(699,1376),(743,1371),
      (780,1384),(807,1405),(820,1438),(817,1465),(801,1474),
      (565,1474),(547,1466),(545,1428),(550,1386),(554,1359),(551,1321)],5)
]
leather_palette = Image.new('P',(1,1))
leather_colors = PALETTE[:7]+PALETTE[24:32]
leather_palette.putpalette([v for c in leather_colors for v in bytes.fromhex(c[1:])] +
                          [0]*(768-len(leather_colors)*3))
ld = ImageDraw.Draw(lower)
ld.rectangle((0,101,95,127),fill=(0,0,0,0))
ld.polygon([(28,100),(42,100),(42,104),(28,104)],fill=PALETTE[17])
ld.polygon([(54,100),(69,100),(69,105),(55,105)],fill=PALETTE[17])
for points,dx in boot_polygons:
    reference_native = reference_source.transform(SIZE,Image.Transform.AFFINE,
        (1/reference_scale,0,34-(4+dx)/reference_scale,
         0,1/reference_scale,138-4/reference_scale),Image.Resampling.NEAREST)
    reference_native = reference_native.quantize(palette=leather_palette,dither=Image.Dither.NONE).convert('RGBA')
    native_points = [(round(4+(x-34)*reference_scale+dx),
                      round(4+(y-138)*reference_scale)) for x,y in points]
    lower.alpha_composite(cut(reference_native,native_points,True))
complete_before_split = compose([lower,upper,head])

# Eye ownership includes eyebrows. Only dark/white eye pixels move to the overlay.
eyes = blank()
for box in [(43,13,54,19),(43,19,52,26),(57,12,63,18),(57,19,63,25)]:
    for y in range(box[1],box[3]):
        for x in range(box[0],box[2]):
            c = head.getpixel((x,y))
            if c[3] and (max(c[:3]) < 160 or min(c[:3]) > 225):
                eyes.putpixel((x,y),c)
                head.putpixel((x,y),tuple(bytes.fromhex('e9a15b'))+(255,))
mouth = blank()
for y in range(33,35):
    for x in range(53,58):
        c = head.getpixel((x,y))
        if c[3] and max(c[:3]) < 120:
            mouth.putpixel((x,y),c)
            head.putpixel((x,y),tuple(bytes.fromhex('e9a15b'))+(255,))

# Equipment contains only the named garments/props; the complete body stays intact.
gear_polygons = {
 'scarf': [[(30,31),(34,32),(40,36),(47,40),(53,42),(59,41),(63,37),
            (65,39),(65,42),(61,45),(60,47),(62,51),(62,54),(59,54),
            (56,51),(54,47),(52,47),(52,45),(47,46),(42,44),(37,41),(33,38),(29,35)]],
 'straps': [[(34,39),(38,41),(41,44),(44,47),(44,52),(43,57),
             (41,61),(37,64),(34,65),(29,63),(27,60),(30,58),
             (34,61),(36,60),(38,56),(38,52),(37,47),(36,44)],
            [(63,41),(67,42),(69,46),(70,52),(71,58),(68,59),
             (66,55),(65,50),(64,46)]],
 'apron': [[(41,53),(51,53),(57,55),(63,53),(67,55),(70,61),(71,66),
            (72,73),(72,79),(74,83),(75,89),(74,95),(69,97),(42,97),
            (38,95),(38,88),(38,80),(37,72),(38,65),(39,59)]],
 'belt-tools': [[(35,66),(43,66),(50,67),(60,67),(68,66),(72,65),
                 (73,69),(70,72),(71,78),(71,83),(68,82),(65,78),(64,73),
                 (62,72),(63,77),(61,79),(59,76),(59,72),(55,72),(54,77),
                 (54,80),(51,83),(49,84),(47,86),(44,84),(44,81),(39,81),
                 (35,80),(34,75),(34,71)]],
 'towel': [[(28,64),(31,64),(33,66),(35,65),(36,69),(35,74),(35,82),
            (36,89),(34,92),(31,93),(29,90),(27,90),(26,88),(23,88),
            (20,86),(20,83),(22,78),(23,72),(25,68)]],
 'spoon': [[(85,28),(89,29),(92,32),(93,35),(92,39),(89,42),
            (85,45),(84,49),(83,51),(80,51),(81,46),(81,43),(79,40),
            (79,35),(80,32),(82,30)],
           [(80,62),(82,62),(81,72),(79,74),(78,73),(79,67)]]
}
gear = {}
for name, polygons in gear_polygons.items():
    gear[name] = compose([cut(equipped, points) for points in polygons])
    if name == 'scarf':
        allowed = PALETTE[20:24]
        material = {tuple(bytes.fromhex(c[1:])) for c in allowed}
        anchors = {(x,y) for y in range(128) for x in range(96)
                   if gear[name].getpixel((x,y))[3] and gear[name].getpixel((x,y))[:3] in material}
        for y in range(128):
            for x in range(96):
                c = gear[name].getpixel((x,y))
                keep_outline = c[:3] == tuple(bytes.fromhex(OUTLINE[1:])) and any(
                    (x+dx,y+dy) in anchors for dx,dy in [(0,1),(0,-1),(1,0),(-1,0)])
                if (x,y) not in anchors and not keep_outline:
                    gear[name].putpixel((x,y),(0,0,0,0))

# Re-author the narrow leather paths at native scale so their edges stay continuous.
straps = blank()
draw = ImageDraw.Draw(straps)
for path in gear_polygons['straps']:
    draw.polygon(path,fill=PALETTE[3],outline=OUTLINE,width=1)
draw.line([(36,41),(40,46),(41,54),(39,59),(35,63),(29,60)],fill=PALETTE[5],width=1)
draw.line([(65,43),(67,48),(68,55)],fill=PALETTE[5],width=1)
for box in [(38,47,43,52),(65,47,69,51)]:
    draw.rectangle(box,fill=PALETTE[26],outline=OUTLINE,width=1)
    draw.rectangle((box[0]+2,box[1]+2,box[2]-1,box[3]-1),fill=PALETTE[2])
gear['straps'] = straps

# These small equipment contours are authored instead of cut through source pixels.
spoon_reference_points = [(912,427),(951,429),(975,446),(990,470),(995,510),
    (982,550),(955,586),(917,610),(897,630),(890,663),(881,715),
    (872,778),(858,838),(849,910),(837,948),(812,947),(812,922),
    (827,845),(833,792),(843,735),(852,675),(861,631),(856,605),
    (845,577),(842,540),(851,501),(870,466),(889,442)]
spoon_native = reference_source.transform(SIZE,Image.Transform.AFFINE,
    (1/reference_scale,0,34-9/reference_scale,
     0,1/reference_scale,138-4/reference_scale),Image.Resampling.NEAREST)
spoon_native = spoon_native.quantize(palette=leather_palette,dither=Image.Dither.NONE).convert('RGBA')
spoon_points = [(round(9+(x-34)*reference_scale),
                 round(4+(y-138)*reference_scale)) for x,y in spoon_reference_points]
spoon = cut(spoon_native,spoon_points,True)
# The reference hides part of the shaft behind fingers. Restore wood there,
# keeping a complete prop instead of baking a hand-shaped hole into it.
for y in range(48,77):
    row = [x for x in range(96) if spoon.getpixel((x,y))[3]]
    for x in row:
        color = OUTLINE if x in (min(row),max(row)) or y == 76 else PALETTE[5]
        if min(row) < x < max(row) and x == min(row)+1:
            color = PALETTE[6]
        spoon.putpixel((x,y),tuple(bytes.fromhex(color[1:]))+(255,))
complete_spoon = spoon.copy()
# Only the curled grip hides the shaft. The lower forearm is not an occluder.
grip_occlusion = mask([(80,53),(83,52),(87,53),(88,56),
                       (87,59),(84,62),(80,62)])
front_fingers = blank()
for y in range(128):
    for x in range(96):
        if grip_occlusion.getpixel((x,y)) and upper.getpixel((x,y))[3]:
            front_fingers.putpixel((x,y),upper.getpixel((x,y)))
            spoon.putpixel((x,y),(0,0,0,0))
gear['spoon'] = spoon

towel = blank()
td = ImageDraw.Draw(towel)
towel_back = [(28,65),(31,64),(33,66),(33,72),(31,81),
              (29,88),(26,89),(23,88),(21,87),(20,85),(21,81),
              (23,75),(24,69),(26,66)]
towel_front = [(31,65),(34,65),(35,68),(34,74),(34,82),
               (35,88),(34,91),(31,92),(28,91),(28,87),
               (29,80),(29,73),(30,68)]
td.polygon(towel_back,fill=PALETTE[12],outline=OUTLINE,width=1)
td.polygon([(26,68),(29,66),(29,73),(27,80),(25,85),(22,85),
            (24,77)],fill=PALETTE[11])
td.polygon([(21,81),(24,82),(29,83),(29,85),(24,84),(21,83)],fill=PALETTE[22])
td.polygon(towel_front,fill=PALETTE[11],outline=OUTLINE,width=1)
td.line([(32,68),(31,76),(31,82)],fill=PALETTE[13],width=1)
td.polygon([(29,83),(33,84),(33,86),(29,85)],fill=PALETTE[22])
td.polygon([(29,88),(33,89),(33,90),(30,90)],fill=PALETTE[22])
gear['towel'] = towel

scarf = blank()
scd = ImageDraw.Draw(scarf)
# Outline the two collar strips individually, leaving the beard fully visible.
scd.polygon([(31,32),(34,33),(38,36),(43,39),(48,41),(53,42),
             (53,45),(49,45),(44,43),(39,41),(35,38),(31,36),(29,35)],
            fill=PALETTE[22],outline=OUTLINE,width=1)
scd.line([(32,33),(36,36),(41,39),(47,42)],fill=PALETTE[23],width=1)
scd.line([(36,38),(42,41),(48,43),(51,43)],fill=PALETTE[21],width=1)
scd.polygon([(54,42),(59,40),(63,38),(64,39),(64,42),(61,44),(57,46),(54,45)],
            fill=PALETTE[22],outline=OUTLINE,width=1)
scd.line([(57,43),(61,41),(63,39)],fill=PALETTE[23],width=1)
scd.polygon([(54,45),(57,46),(58,49),(61,52),(62,55),(59,55),
             (56,53),(55,50),(53,48)],fill=PALETTE[22],outline=OUTLINE,width=1)
scd.line([(56,48),(57,51),(60,53)],fill=PALETTE[21],width=1)
scd.polygon([(52,43),(54,43),(56,45),(54,48),(52,48),(51,46)],
            fill=PALETTE[21],outline=OUTLINE,width=1)
scd.point((53,45),fill=PALETTE[23])
gear['scarf'] = scarf

def shifted(im):
    out = blank()
    out.paste(im, SHIFT)
    return out

layers = {'body/lower-base.png':lower,'body/upper-base.png':upper,
          'head-base.png':head,'eyes/open.png':eyes,'mouth/rest.png':mouth,
          'body/hand-front-spoon.png':front_fingers}
layers.update({f'equipment/{name}.png':im for name,im in gear.items()})
layers = {name:shifted(im) for name,im in layers.items()}
for name, im in layers.items():
    save(im,name)
save(shifted(complete_spoon),'source/spoon-complete.png')
foundation = compose([layers[n] for n in ['body/lower-base.png','body/upper-base.png','head-base.png']])
under = compose([foundation,layers['eyes/open.png'],layers['mouth/rest.png']])
assert compose([under,layers['equipment/spoon.png']]).tobytes() == compose(
    [under,shifted(complete_spoon),layers['body/hand-front-spoon.png']]).tobytes(), 'Grip mask does not match finger occlusion'
full = compose([under]+[layers[f'equipment/{n}.png'] for n in ['apron','straps','belt-tools','towel','scarf','spoon']])
save(foundation,'source/foundation.png')
save(under,'base-neutral.png')
save(full,'neutral.png')
save(full,'atlas.png')
save(full.resize((384,512),Image.Resampling.NEAREST),'previews/neutral-4x.png')
save(under.resize((384,512),Image.Resampling.NEAREST),'previews/base-neutral-4x.png')

review = Image.new('RGB',(384*4,552),'#eee3cc')
d = ImageDraw.Draw(review)
for i,(im,bg,label) in enumerate([(under,'#eee3cc','Base / light'),(full,'#eee3cc','Equipped / light'),
                                (under,'#302b32','Base / dark'),(full,'#302b32','Equipped / dark')]):
    review.paste(Image.new('RGB',(384,512),bg),(384*i,40))
    review.paste(im.resize((384,512),Image.Resampling.NEAREST),(384*i,40),im.resize((384,512),Image.Resampling.NEAREST))
    d.text((384*i+12,14),label,fill='#211510')
save(review,'previews/layer-check.png')
face = full.crop((24,4,68,45)).resize((352,328),Image.Resampling.NEAREST)
save(face,'previews/face-8x.png')
reference = Image.open(ROOT/'source/reference.png').convert('RGB')
reference = reference.crop((34,138,996,1480))
reference = reference.resize((85,118),Image.Resampling.NEAREST)
comparison = Image.new('RGBA',(192,128),'#eee3cc')
comparison.paste(reference,(4,6))
comparison.alpha_composite(full,(96,0))
save(comparison.resize((768,512),Image.Resampling.NEAREST),'previews/reference-comparison-4x.png')

# Reference details use the same body scale as the asset, with nearest enlargement.
detail = Image.new('RGB',(1000,480),'#eee3cc')
dd = ImageDraw.Draw(detail)
reference_full = Image.new('RGBA',SIZE)
reference_full.paste(reference,(4,6))
for i,(label,reference_box,asset_box) in enumerate([
    ('Spoon',(73,29,92,79),(76,29,95,79)),
    ('Boots',(20,100,76,128),(23,100,79,128))]):
    start = i*400
    dd.text((start+8,8),label,fill=OUTLINE)
    for j,(im,box) in enumerate([(reference_full,reference_box),(full,asset_box)]):
        crop = im.crop(box)
        crop = crop.resize((crop.width*4,crop.height*4),Image.Resampling.NEAREST)
        x = start+8+j*(100 if i == 0 else 232)
        detail.paste(crop,(x,50),crop)
        dd.text((x,30),'REFERENCE' if j == 0 else 'CURRENT',fill=OUTLINE)
save(detail,'previews/reference-details-4x.png')

grip_stack = Image.new('RGB',(864,480),'#eee3cc')
gsd = ImageDraw.Draw(grip_stack)
for row,bg in enumerate(['#eee3cc','#302b32']):
    for col,(label,im) in enumerate([
            ('MASKED SPOON',layers['equipment/spoon.png']),
            ('EMPTY HAND',under),
            ('UNMASKED SPOON',compose([under,shifted(complete_spoon)])),
            ('FINGERS HIDE SHAFT',compose([under,layers['equipment/spoon.png']]))]):
        x,y = col*216,row*240
        grip_stack.paste(Image.new('RGB',(216,240),bg),(x,y))
        gsd.text((x+8,y+12),label,fill=OUTLINE if row == 0 else '#eee3cc')
        crop = im.crop((74,49,95,77)).resize((126,168),Image.Resampling.NEAREST)
        grip_stack.paste(crop,(x+36,y+40),crop)
save(grip_stack,'previews/spoon-layer-order.png')

bare_before = ROOT/'source/bare-hand-repair-before'
if bare_before.exists():
    toggle_review = Image.new('RGB',(864,480),'#eee3cc')
    trd = ImageDraw.Draw(toggle_review)
    states = [('OLD EMPTY',Image.open(bare_before/'base-neutral.png').convert('RGBA')),
              ('CURRENT EMPTY',under),
              ('OLD SPOON',Image.open(bare_before/'neutral.png').convert('RGBA')),
              ('CURRENT SPOON',full)]
    for row,bg in enumerate(['#eee3cc','#302b32']):
        for col,(label,im) in enumerate(states):
            x,y = col*216,row*240
            toggle_review.paste(Image.new('RGB',(216,240),bg),(x,y))
            trd.text((x+12,y+12),label,fill=OUTLINE if row == 0 else '#eee3cc')
            crop = im.crop((70,48,96,78)).resize((156,180),Image.Resampling.NEAREST)
            toggle_review.paste(crop,(x+24,y+40),crop)
    save(toggle_review,'previews/empty-hand-toggle-check.png')

grip_before = ROOT/'source/grip-rebuild-before/neutral.png'
if grip_before.exists():
    grip_review = Image.new('RGB',(648,320),'#eee3cc')
    grd = ImageDraw.Draw(grip_review)
    aligned_reference = reference_source.convert('RGBA').transform(SIZE,Image.Transform.AFFINE,
        (1/reference_scale,0,34-9/reference_scale,
         0,1/reference_scale,138-6/reference_scale),Image.Resampling.NEAREST)
    for i,(label,im) in enumerate([('REFERENCE',aligned_reference),
            ('BEFORE',Image.open(grip_before).convert('RGBA')),('CURRENT',full)]):
        crop = im.crop((71,48,96,78)).resize((200,240),Image.Resampling.NEAREST)
        grip_review.paste(crop,(i*216+8,40),crop)
        grd.text((i*216+8,16),label,fill=OUTLINE)
    save(grip_review,'previews/grip-reference-check.png')

hand_before = ROOT/'source/hand-detail-before/neutral.png'
if hand_before.exists():
    hand_review = Image.new('RGB',(416,256),'#eee3cc')
    hrd = ImageDraw.Draw(hand_review)
    for i,im in enumerate([Image.open(hand_before).convert('RGBA'),full]):
        crop = im.crop((74,49,94,72)).resize((160,184),Image.Resampling.NEAREST)
        hand_review.paste(crop,(i*208+20,40),crop)
        hrd.text((i*208+20,16),'BEFORE' if i == 0 else 'AFTER',fill=OUTLINE)
    save(hand_review,'previews/hand-detail-before-after.png')

before_path = ROOT/'source/review-before/neutral.png'
if before_path.exists():
    before = Image.open(before_path).convert('RGBA')
    regions = [('Scalp',(27,3,65,18)),('Spoon and hand',(72,29,95,78)),
               ('Towel',(18,65,38,96)),('Scarf',(27,31,67,59)),('Boots',(22,100,80,128))]
    comparison = Image.new('RGB',(1920,280),'#eee3cc')
    cd = ImageDraw.Draw(comparison)
    for i,(label,box) in enumerate(regions):
        start = 1280 if i == 4 else i*320
        cd.text((start+8,10),label,fill=OUTLINE)
        for j,im in enumerate([before,full]):
            crop = im.crop(box)
            crop = crop.resize((crop.width*4,crop.height*4),Image.Resampling.NEAREST)
            x = start+j*(244 if i == 4 else 156)+8
            comparison.paste(crop,(x,56),crop)
            cd.text((x,32),'BEFORE' if j == 0 else 'AFTER',fill=OUTLINE)
    save(comparison,'previews/outline-before-after.png')
    allowed = Image.new('L',SIZE)
    ad = ImageDraw.Draw(allowed)
    for _,(x0,y0,x1,y1) in regions:
        ad.rectangle((x0,y0,x1-1,y1-1),fill=255)
    assert all(before.getpixel((x,y)) == full.getpixel((x,y))
               for y in range(128) for x in range(96) if not allowed.getpixel((x,y))), 'Unrelated pixels changed'

opaque = lambda im: {i for i,p in enumerate(im.getdata()) if p[3]}
colors = sorted({p for im in layers.values() for p in im.getdata() if p[3]})
assert len(colors) <= 32
assert not (opaque(under)-opaque(full))
assert compose([layers[n] for n in ['body/lower-base.png','body/upper-base.png','head-base.png','eyes/open.png','mouth/rest.png']]).tobytes() == under.tobytes()
assert shifted(complete_before_split).tobytes() == under.tobytes(), 'Splitting changed the complete base'
arm_mask = mask(arm_path)
arm_interior_dark = [(x,y) for y in range(50,69) for x in range(73,92)
    if all(arm_mask.getpixel((x+dx,y+dy)) for dx in [-1,0,1] for dy in [-1,0,1])
    and upper.getpixel((x,y))[:3] == tuple(bytes.fromhex(OUTLINE[1:]))]
assert not arm_interior_dark, 'A dark seam remains inside the raised forearm'
report = {'native_size':list(SIZE),'union_opaque_colors':len(colors),'alpha_values':[0,255],
          'base_opaque_pixels':len(opaque(under)),'equipped_opaque_pixels':len(opaque(full)),
          'lost_base_pixels':len(opaque(under)-opaque(full)), 'underlayer_recomposition_mismatched_pixels':0,
          'raised_arm_interior_outline_pixels':len(arm_interior_dark),
          'expression_invariance':'Foundation report covers neutral; additional expressions checked in qa/animation-validation.json',
          'sha256_pixels':{n:hashlib.sha256(im.tobytes()).hexdigest() for n,im in layers.items()}}
(ROOT/'source/coverage-check.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))

# The historical entry point must not leave a one-frame atlas over the new export.
if __name__ == '__main__':
    import os
    import subprocess
    import sys
    animation_builder = ROOT/'source/build-animation.py'
    if animation_builder.exists() and not os.environ.get('CHEF_BASE_ONLY'):
        subprocess.run([sys.executable,str(animation_builder),'--base-ready'],check=True)
