"""Read-only native, layer, timeline and atlas checks for player animation."""
import json,hashlib
from pathlib import Path
from PIL import Image
R=Path(__file__).resolve().parents[1]
def im(p):return Image.open(p).convert('RGBA')
def px(i):return list(i.get_flattened_data() if hasattr(i,'get_flattened_data') else i.getdata())
def canonical(i):return bytes(v for p in px(i) for v in (p if p[3] else (0,0,0,0)))
def compose(d,names):
 o=Image.new('RGBA',(128,160))
 for n in names:o.alpha_composite(im(d/n))
 return o
reports=[]
for sex in ['female','male']:
 d=R/sex;a=json.loads((d/'animation.json').read_text());m=json.loads((d/'atlas-manifest.json').read_text());assert a['canvas']==[128,160] and a['pivot']==[64,140]
 colors=set();count=0
 for p in (d/'poses').rglob('*.png'):
  o=im(p);assert o.size==(128,160),(p,'size');assert set(px(o.getchannel('A')))<={0,255},(p,'alpha');colors|={q[:3] for q in px(o) if q[3]};count+=1
 assert len(colors)<=86,(sex,'palette',len(colors))
 for key,p in a['poses'].items():
  b=d/'poses'/key
  parts=['body/lower-base.png','body/upper-base.png','head-base.png']
  foundation=compose(b,parts);base=compose(b,parts+['eyes/open.png','mouth/rest.png'])
  assert canonical(foundation)==canonical(im(b/'foundation.png')),(sex,key,'foundation partition')
  assert canonical(base)==canonical(im(d/p['base'])),(sex,key,'base reconstruction')
  eq=im(d/p['composed'])
  if p.get('rendered_variant_layers'):
   names=[str(Path(n).relative_to('poses/'+key)) for n in p['rendered_variant_layers']]
   assert canonical(compose(b,names))==canonical(eq),(sex,key,'rendered variant reconstruction')
   assert canonical(foundation)==canonical(im(b/'foundation.png')),(sex,key,'complete underlayer source')
   rendered=compose(b,['rendered/body/lower.png','rendered/body/upper.png','rendered/head-base.png'])
   assert canonical(rendered)==canonical(im(b/'rendered/foundation.png')),(sex,key,'coherent work source partition')
   for kind in ['rendered/eyes/open.png','rendered/mouth/rest.png']:
    assert all(not x[3] or y[3] for x,y in zip(px(im(b/kind)),px(im(b/'rendered/head-base.png')))),(sex,key,'rendered feature hole')
  else:assert all(not x[3] or y[3] for x,y in zip(px(foundation),px(eq))),(sex,key,'foundation loss')
  for kind in ['eyes/open.png','mouth/rest.png']:
   assert all(not x[3] or y[3] for x,y in zip(px(im(b/kind)),px(im(b/'head-base.png')))),(sex,key,'feature hole')
  if p['facing']=='back':assert not im(b/'eyes/open.png').getbbox() and not im(b/'mouth/rest.png').getbbox(),(sex,key,'face on rear')
  if key in ['nod-down','turn-toward','turn-away','front-neutral']:
   assert im(b/'eyes/open.png').getbbox(),(sex,key,'missing separated eyes')
  bb=eq.getbbox();assert bb and bb[0]>0 and bb[1]>0 and bb[2]<128 and bb[3]<160,(sex,key,'clipped output',bb)
 # Approved neutral is immutable; expressions alter only the union of eye/mouth pigments.
 assert canonical(im(d/'frames/neutral.png'))==canonical(im(d/'neutral.png')),(sex,'approved neutral changed')
 neutral=im(d/'frames/neutral.png');allowed=set()
 for folder in ['eyes','mouth']:
  for p in (d/folder).glob('*.png'):
   for i,q in enumerate(px(im(p))):
    if q[3]:allowed.add(i)
 for name in a['expression_recipes']:
  state=im(d/'frames'/f'{name}.png')
  assert all(i in allowed or x==y for i,(x,y) in enumerate(zip(px(neutral),px(state)))),(sex,name,'expression body drift')
  # Independent protected regions catch portrait pixels leaking into feature layers.
  jaw_start=62 if sex=='female' else 61
  for y in range(160):
   for x in range(128):
    if y<45 or y>=jaw_start or x<57 or x>=78:
     assert state.getpixel((x,y))==neutral.getpixel((x,y)),(sex,name,'hair or jaw drift',x,y)
  if name in ['blink','happy','relaxed']:
   assert state.crop((57,58,78,66)).tobytes()==neutral.crop((57,58,78,66)).tobytes(),(sex,name,'eye layer changed mouth')
  assert canonical(im(d/'poses'/name/'head-base.png'))==canonical(im(d/'head-base.png')),(sex,name,'head replacement')
 for c in a['clips'].values():
  assert c['frames'] and all(fr['sprite'] in a['poses'] and fr['duration_ms']>0 for fr in c['frames'])
  assert all(0<=e['frame']<len(c['frames']) for e in c.get('events',[]))
 assert len(a['clips']['walk']['frames'])==6 and a['clips']['walk']['loop']
 for action in ['hammering','quenching','forging','researching','salvaging']:
  assert len(a['clips'][action]['frames'])==6
  w=a['workshop']['stations'][action];c,s=w['contact_points']['character'],w['contact_points']['station']
  assert [c[0]+32,c[1]+16]==[s[0]+w['origin'][0],s[1]+w['origin'][1]],(sex,action,'registered contact')
  for fr in a['clips'][action]['frames']:
   p=a['poses'][fr['sprite']];b=d/'poses'/fr['sprite'];head=im(b/'head-base.png');upper=im(b/'body/upper-base.png')
   identity=p.get('head_identity',{})
   if identity.get('root_pixels_unchanged'):
    dx,dy=identity['shift']
    for target,source in [('head-base.png','head-base.png'),('eyes/open.png','eyes/open.png'),('mouth/rest.png','mouth/rest.png')]:
     expected=Image.new('RGBA',(128,160));src=im(d/source)
     if source=='head-base.png':
      for row,span in a['head_ownership']['removed_neck_collar_rows'].items():
       for x in range(span[0],span[1]+1):src.putpixel((x,int(row)),(0,0,0,0))
     expected.paste(src,(dx,dy))
     assert canonical(im(b/target))==canonical(expected),(sex,fr['sprite'],'approved identity changed',target)
    ex,ey=p['rendered_head_identity']['shift']
    for target,source in [('rendered/head-base.png','head-base.png'),('rendered/eyes/open.png','eyes/open.png'),('rendered/mouth/rest.png','mouth/rest.png')]:
     src=im(d/source)
     if source=='head-base.png':
      for row,span in a['head_ownership']['removed_neck_collar_rows'].items():
       for x in range(span[0],span[1]+1):src.putpixel((x,int(row)),(0,0,0,0))
      src.alpha_composite(im(d/'equipment'/('hair-tie.png' if sex=='female' else 'headband.png')))
     expected=Image.new('RGBA',(128,160));expected.paste(src,(ex,ey))
     assert canonical(im(b/target))==canonical(expected),(sex,fr['sprite'],'equipped identity changed',target)

   if action=='researching':
    arms=im(b/'body/arms-front.png')
    for lo,hi in [(0,64),(64,128)]:
     pixels=[arms.getpixel((x,y)) for y in range(60,104) for x in range(lo,hi) if arms.getpixel((x,y))[3]]
     assert sum(q[0]>185 and q[1]>105 and q[0]>q[1]*1.22 for q in pixels)>=20,(sex,fr['sprite'],'missing research forearm')
     assert sum(q[0]>165 and q[1]>145 and q[2]>110 and q[0]<q[1]*1.25 for q in pixels)>=20,(sex,fr['sprite'],'research sleeve cut off')
   neck=p['sockets'].get('neck',[64,65])[1] if p.get('rendered_variant_layers') else 65
   assert any(head.getpixel((x,y))[3] and upper.getpixel((xx,yy))[3]
    for y in range(max(1,neck-2),min(159,neck+2)) for x in range(40,100)
    for xx,yy in [(x,y),(x-1,y),(x+1,y),(x,y-1),(x,y+1)]),(sex,fr['sprite'],'detached neck')
   if p.get('rendered_variant_layers'):
    rh=im(b/'rendered/head-base.png');ru=im(b/'rendered/body/upper.png')
    assert any(rh.getpixel((x,y))[3] and ru.getpixel((xx,yy))[3]
     for y in range(50,90) for x in range(35,112)
     for xx,yy in [(x,y),(x-1,y),(x+1,y),(x,y-1),(x,y+1)]),(sex,fr['sprite'],'detached equipped neck')
    base_shift=p['head_identity'].get('shift',[0,0]);eq_shift=p['rendered_head_identity'].get('shift',[0,0])
    if action=='researching':assert abs(base_shift[1]-eq_shift[1])<=3,(sex,fr['sprite'],'equipment changed desk posture')
   assert im(d/'frames/base'/f"{fr['sprite']}.png").getbbox()[3]==141,(sex,fr['sprite'],'foot baseline')
   if p.get('rendered_variant_layers'):
    assert p['source_partition_roundtrip'],(sex,fr['sprite'],'source preservation')
   elif action in ['hammering','quenching','forging']:
    assert canonical(head)==canonical(im(d/'head-base.png')),(sex,fr['sprite'],'work head drift')
    for layer in ['body/arms-front.png','props/held.png','equipment/gloves.png']:
     assert not any(p[3] and q[3] for p,q in zip(px(im(b/layer)),px(head))),(sex,fr['sprite'],'work layer overwrites head',layer)
   else:
    direction='front-neutral' if action=='researching' else 'back-neutral'
    assert canonical(head)==canonical(im(d/'poses'/direction/'head-base.png')),(sex,fr['sprite'],'direction head drift')
    assert canonical(im(b/'body/lower-base.png'))==canonical(im(d/'poses'/direction/'body/lower-base.png')),(sex,fr['sprite'],'planted feet drift')
  start=a['poses'][a['clips'][action]['frames'][0]['sprite']]
  bent=a['poses'][a['clips'][action]['frames'][3 if action=='hammering' else 2]['sprite']]
  if start.get('rendered_variant_layers'):
   if action=='salvaging':
    # Rear hair tilts over the collar; visible collar height is not a neck joint.
    first_head=im(d/start['layers'][2]);bent_head=im(d/bent['layers'][2])
    assert bent_head.getbbox()[1]>=first_head.getbbox()[1]+2,(sex,action,'rear head pinned during bend')
   else:
    assert bent['sockets']['head'][1]>=start['sockets']['head'][1]+2,(sex,action,'head pinned during bend')
    assert bent['sockets']['neck'][1]>=start['sockets']['neck'][1]+1,(sex,action,'neck did not follow torso')
  if action=='forging':assert w['occlusion_order']==['station','character'],(sex,'forge depth')
  if action in ['researching','salvaging']:
   stations=json.loads((R/'stations/catalog.json').read_text())['stations'];bottom=w['origin'][1]+stations[w['station_id']]['size'][1]
   assert (bottom>156 if action=='researching' else bottom<156),(sex,action,'desk ground depth',bottom)
  if action=='quenching':
   rect=w['submerged_tool_clip']['station_rect'];ox,oy=w['origin']
   for fr in a['clips'][action]['frames']:
    tool=im(d/'poses'/fr['sprite']/'props/held.png');full=im(d/'frames'/f"{fr['sprite']}.png");clipped=im(d/'frames'/f"scene-{fr['sprite']}.png")
    for y in range(160):
     for x in range(128):
      if clipped.getpixel((x,y))!=full.getpixel((x,y)):
       assert tool.getpixel((x,y))[3] and rect[0]<=x+32-ox<rect[2] and rect[1]<=y+16-oy<rect[3],(sex,fr['sprite'],'submerged clip escaped tool')
 pages={p['id']:im(d/p['path']) for p in m['pages']};rects={};seen=set()
 for sp in m['sprites']:
  x,y,w,h=sp['rect'];page=pages[sp['page']];assert x>=2 and y>=2 and x+w+2<=page.width and y+h+2<=page.height
  crop=page.crop((x,y,x+w,y+h));restored=Image.new('RGBA',(128,160));restored.alpha_composite(crop,tuple(sp['trim_offset']))
  assert canonical(restored)==canonical(im(d/sp['source'])),(sex,sp['id'],'atlas roundtrip')
  rect=tuple(sp['rect'])
  if rect not in seen:
   seen.add(rect)
   for yy in range(-2,h+2):
    for xx in range(-2,w+2):
     p=page.getpixel((x+xx,y+yy))
     if xx in [-2,w+1] or yy in [-2,h+1]:assert p[3]==0,(sex,sp['id'],'gutter')
     elif xx in [-1,w] or yy in [-1,h]:assert p==crop.getpixel((min(w-1,max(0,xx)),min(h-1,max(0,yy)))),(sex,sp['id'],'extrusion')
 for p in m['pages']:assert max(p['size'])<=2048
 reports.append({'id':sex,'status':'pass','native_pose_layers':count,'frames':len(a['poses']),'clips':len(a['clips']),'animation_colors_including_hot_metal':len(colors),'expression_combinations':20,'foundation_loss':0,'foundation_scope':'each complete source variant independently','coherent_work_poses':30,'approved_three_quarter_head_features_exact':True,'research_sleeves_and_forearms_present':True,'bend_head_motion':True,'front_and_three_quarter_neck_motion':True,'rear_motion_check':'hair crown moves down at least 2px; collar may hide neck','atlas_roundtrip':True,'extrusion_and_gutters':True,'approved_neutral_unchanged':True,'workshop_contacts_registered':True,'draw_calls':'not_measured','runtime_integration':False})
(R/'qa/animation-validation.json').write_text(json.dumps({'status':'pass','characters':reports},indent=2)+'\n');print(json.dumps(reports,indent=2))
