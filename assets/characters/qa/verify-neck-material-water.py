"""Guard against repainting garments as skin and water hiding the tool too early."""
import json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def image(p):return Image.open(p).convert('RGBA')
variants=0;source_pixels=0;visible_tool_pixels=0;submerged=0
for sex in ['female','male']:
 a=json.loads((ROOT/sex/'animation.json').read_text())
 for name,pose in a['poses'].items():
  if not pose.get('neck_assembly',{}).get('body_owned_mount'):continue
  for equipped in [False,True]:
   p=ROOT/sex/'poses'/name
   raw=image(p/('source/equipped-before-head-restoration.png' if equipped else 'source/body-before-head-restoration.png'))
   upper=image(p/('rendered/body/upper.png' if equipped else 'body/upper-base.png'))
   garment=image(p/('rendered/body/source-connection.png' if equipped else 'body/source-connection.png'))
   for y in range(160):
    for x in range(128):
     pixel=garment.getpixel((x,y))
     if pixel[3]:
      assert pixel==raw.getpixel((x,y)) and upper.getpixel((x,y))==pixel,(sex,name,equipped,'source garment repainted',x,y)
      source_pixels+=1
   variants+=1
 w=a['workshop']['stations']['quenching'];ox,oy=w['origin'];water=image(ROOT/'stations/quenching-tub-water.png')
 for key in ['quenching-1','quenching-2','quenching-3']:
  actor=image(ROOT/sex/'frames'/f'{key}.png');clipped=image(ROOT/sex/'frames'/f'scene-{key}.png');scene=image(ROOT/sex/'previews/scenes'/f'{key}.png');tool=image(ROOT/sex/'poses'/key/'props/held.png')
  for y in range(160):
   for x in range(128):
    sx,sy=x+32-ox,y+16-oy
    if not(tool.getpixel((x,y))[3] and actor.getpixel((x,y))[3] and 0<=sx<64 and 0<=sy<48):continue
    if sy<18 and water.getpixel((sx,sy))[3]:
     assert clipped.getpixel((x,y))==actor.getpixel((x,y)),(sex,key,'tool clipped before entry boundary',x,y)
     assert scene.getpixel((x+32,y+16))==actor.getpixel((x,y)),(sex,key,'back water overpainted visible tool',x,y)
     visible_tool_pixels+=1
    if 6<=sx<59 and 18<=sy<48:
     assert not clipped.getpixel((x,y))[3],(sex,key,'submerged tool not masked',x,y)
     submerged+=1
assert variants==96 and source_pixels>0,(variants,source_pixels)
assert visible_tool_pixels>0 and submerged>0,(visible_tool_pixels,submerged)
report={'status':'pass','source_garment_variants':variants,'garment_pixels_equal_to_same_source_coordinate':source_pixels,'garment_pixels_repainted_as_skin':0,'tool_pixels_visible_over_back_water':visible_tool_pixels,'submerged_tool_pixels_masked':submerged,'water_interface_station_y':18,'scope':'Current complete body garment layers for all visible work keys; both sexes tool occlusion. Body-owned neck pigment interpolation is checked by verify-body-mount.py.'}
(ROOT/'qa/neck-material-water-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
