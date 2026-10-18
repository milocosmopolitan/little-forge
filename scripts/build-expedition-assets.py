"""Deterministic expedition derivative assets. Canonical NPC files are read-only inputs."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageOps
import json, hashlib, argparse, shutil
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'assets/expedition'
NPC=ROOT/'assets/npc/hunter'
SIZE=(128,160)
def save(im,path):
 p=OUT/path;p.parent.mkdir(parents=True,exist_ok=True);im.save(p)
def blank(size=SIZE):return Image.new('RGBA',size)
def move(im,x,y):
 out=blank(im.size);out.alpha_composite(im,(x,y));return out
def clean(im,colors=64):
 alpha=im.getchannel('A').point(lambda a:255 if a>=192 else 0)
 im=im.convert('RGB').quantize(colors=colors,dither=Image.Dither.NONE).convert('RGBA');im.putalpha(alpha)
 data=[p if p[3] else (0,0,0,0) for p in im.getdata()];im.putdata(data);return im
def pack(name,frames,clips,foot):
 names=list(frames);w,h=next(iter(frames.values())).size
 atlas=blank((w*min(8,len(names)),h*((len(names)+7)//8)))
 for i,n in enumerate(names):save(frames[n],Path(name)/'frames'/f'{n}.png');atlas.alpha_composite(frames[n],((i%8)*w,(i//8)*h))
 save(atlas,Path(name)/'atlas.png')
 (OUT/name/'animation.json').write_text(json.dumps({'version':1,'canvas':[w,h],'foot_anchor_px':foot,'sampler':'nearest','frames':names,'clips':clips,'visual_approval':'pending'},indent=2)+'\n')
 preview=blank((w*len(names),h))
 for i,n in enumerate(names):preview.alpha_composite(frames[n],(i*w,0))
 save(preview.resize((preview.width*4,h*4),Image.Resampling.NEAREST),Path(name)/'preview-4x.png')

def hunter():
 recipes=json.loads((NPC/'poses/recipes.json').read_text())['poses']
 frames={}
 def pose(n,combat=False):
  r=recipes[n];layers=r['layers']|r['equipment'];out=blank()
  for k in r['composition_order']:
   if k=='bow' or (combat and k in ['hands-front','gloves']):continue
   part=Image.open(NPC/layers[k]).convert('RGBA')
   if combat and k=='upper':
    pd=ImageDraw.Draw(part);pd.polygon([(38,84),(45,83),(53,89),(61,91),(61,98),(48,98),(38,92)],fill=(0,0,0,0));pd.rectangle((82,86,94,114),fill=(0,0,0,0))
   out.alpha_composite(part)
  return out
 for n in ['neutral','blink','happy','surprised']+[f'walk-{i}' for i in range(6)]:frames[n]=pose(n)
 # Preserve the approved head, cape, torso and feet. Arms are local expedition overlays.
 palette=Image.open(NPC/'source/palette.png').convert('RGB');colors=list(dict.fromkeys(palette.getdata()))
 def c(rgb):return min(colors,key=lambda p:sum((a-b)**2 for a,b in zip(p,rgb)))+(255,)
 dark=c((40,28,20));leather=c((101,62,35));light=c((187,146,89));cream=c((210,191,144));skin=c((250,178,118));metal=c((205,207,175))
 for name,pull in [('bow-draw',0),('bow-aim',8),('bow-release',6),('bow-recovery',2)]:
  base=pose('neutral',combat=True);arm=blank();d=ImageDraw.Draw(arm)
  # Far hand holds the bow; the near hand draws the string toward the cheek.
  d.polygon([(80,76),(91,77),(103,74),(105,80),(91,86),(81,83)],fill=cream,outline=dark)
  d.polygon([(99,74),(108,73),(110,77),(107,82),(101,81)],fill=leather,outline=dark)
  d.line([(106,47),(113,55),(117,67),(118,78),(115,93),(108,106)],fill=dark,width=3)
  d.line([(107,48),(113,58),(116,70),(116,82),(112,96),(108,104)],fill=light,width=1)
  d.line([(106,47),(93-pull,77),(108,106)],fill=cream,width=1)
  d.polygon([(46,79),(51,76),(65,79),(91-pull,73),(95-pull,79),(64,87),(49,87)],fill=dark)
  d.polygon([(48,79),(52,78),(64,81),(65,85),(50,85)],fill=cream)
  d.polygon([(65,81),(88-pull,75),(93-pull,75),(94-pull,79),(66,85)],fill=leather)
  d.line([(68,81),(88-pull,76)],fill=light,width=1)
  if name!='bow-release':d.line([(88-pull,77),(123,77)],fill=light,width=1);d.polygon([(123,75),(127,77),(123,79)],fill=metal)
  base.alpha_composite(arm);frames[name]=base;save(arm,Path('hunter/layers')/(name+'.png'))
 for name,up in [('dagger-draw',True),('dagger-slash',False)]:
  base=pose('give-mid' if up else 'give-full');layer=blank();d=ImageDraw.Draw(layer)
  if up:d.polygon([(101,85),(103,61),(107,57),(106,83)],fill=metal,outline=dark);d.line([(99,84),(109,84)],fill=light,width=2)
  else:d.polygon([(114,78),(127,67),(127,72),(118,82)],fill=metal,outline=dark)
  base.alpha_composite(layer);frames[name]=base;save(layer,Path('hunter/layers')/(name+'.png'))
 frames['hit']=pose('surprised');frames['victory']=pose('happy')
 clips={'combat-idle':['neutral','blink','neutral'],'walk':[f'walk-{i}' for i in range(6)],'bow':['bow-draw','bow-aim','bow-release','bow-recovery'],'dagger':['dagger-draw','dagger-slash','neutral'],'hit':['hit','neutral'],'retreat':[f'walk-{i}' for i in range(6)],'victory':['victory','neutral']}
 pack('hunter',frames,{k:{'frames':v,'durations_ms':[150]*len(v),'loop':k in ['walk','retreat','combat-idle']} for k,v in clips.items()},[64,140])
 # Reference comparison is at the same native scale, with light and dark mattes.
 comp=Image.new('RGBA',(128*5,320),(40,48,44,255))
 for i,n in enumerate(['neutral','bow-aim','bow-release','dagger-slash','hit']):
  comp.alpha_composite(frames[n],(i*128,0));comp.paste((225,216,189,255),(i*128,160,(i+1)*128,320));comp.alpha_composite(frames[n],(i*128,160))
 save(comp.resize((1280,640),Image.Resampling.NEAREST),'hunter/comparison-2x.png')

def monsters():
 # One registered silhouette per archetype prevents cross-frame head/body drift in candidate sheets.
 for name,box in [('wolf',(17,48,242,198)),('boar',(52,94,286,265))]:
  source=Image.open(OUT/'source'/f'{name}.png').convert('RGBA');im=source.crop(box).resize(((box[2]-box[0])//2,(box[3]-box[1])//2),Image.Resampling.NEAREST)
  im=clean(im,48);base=blank((128,96));base.alpha_composite(im,(6,88-im.height));base=ImageOps.mirror(base)
  frames={'idle':base}
  # Legs are isolated below the belly; feet stay on y=88. Head and torso pixels never scale.
  for i in range(4):
   f=base.copy();legs=f.crop((0,73,128,89));ImageDraw.Draw(f).rectangle((0,73,127,88),fill=(0,0,0,0))
   for x0,x1,phase in [(0,64,1),(64,128,-1)]:f.alpha_composite(legs.crop((x0,0,x1,16)),(x0+phase*[0,2,0,-2][i],73))
   frames[f'run-{i}']=f
  frames['attack']=move(base,-3,0);frames['hit']=move(base,2,0)
  frames['defeat']=blank((128,96));frames['defeat'].alpha_composite(base.crop((0,0,128,73)),(0,15))
  clips={'idle':['idle'],'run':[f'run-{i}' for i in range(4)],'attack':['idle','attack','idle'],'hit':['hit','idle'],'defeat':['hit','defeat']}
  pack(name,frames,{k:{'frames':v,'durations_ms':[160]*len(v),'loop':k in ['run','idle']} for k,v in clips.items()},[64,88])

def forest():
 src=Image.open(OUT/'source/forest.png').convert('RGBA')
 # Fixed native plate. No scrolling seam or floating platform strips.
 bg=clean(src.resize((480,270),Image.Resampling.NEAREST),96);save(bg,'forest/backdrop.png')
 # Separate static depth ownership without inventing hidden scenery or stretching pixels.
 masks=[Image.new('L',(480,270)) for _ in range(3)]
 for y in range(270):
  for x in range(480):
   layer=2 if y>=218 else (1 if x<105 or x>390 or (325<x<365 and 55<y<125) else 0)
   masks[layer].putpixel((x,y),255)
 for name,mask in zip(['distant','midground','foreground'],masks):
  layer=blank((480,270));layer.paste(bg,(0,0),mask);save(layer,'forest/'+name+'.png')

 # Continuous traversable plane sampled from the path, with explicit edge wrapping.
 tile=bg.crop((140,190,204,222));tile=tile.resize((64,32),Image.Resampling.NEAREST)
 for y in range(32):tile.putpixel((63,y),tile.getpixel((0,y)))
 save(tile,'forest/ground.png')
 prop=Image.open(OUT/'source/props.png').convert('RGBA').crop((595,677,755,788)).resize((80,56),Image.Resampling.NEAREST);save(clean(prop,32),'forest/rock.png')
 arrow=blank((32,16));d=ImageDraw.Draw(arrow);d.line([(3,8),(26,8)],fill='#b58b50',width=1);d.polygon([(25,5),(31,8),(25,11)],fill='#e8dbc1');d.line([(3,5),(7,8),(3,11)],fill='#d6c4a1',width=1);save(arrow,'effects/arrow.png')
 hit=blank((32,32));d=ImageDraw.Draw(hit);d.polygon([(16,2),(19,12),(29,16),(20,19),(16,30),(12,20),(2,16),(12,12)],fill='#efc87b');save(hit,'effects/hit.png')

if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--import-dir',type=Path);args=parser.parse_args()
 if args.import_dir:
  sources={'wolf':'ChatGPT Image Oct 18, 2026, 04_16_23 PM.png','boar':'ChatGPT Image Oct 18, 2026, 04_16_27 PM.png','forest':'ChatGPT Image Oct 18, 2026, 04_16_33 PM.png','props':'ChatGPT Image Oct 18, 2026, 04_16_41 PM.png'}
  (OUT/'source').mkdir(parents=True,exist_ok=True)
  for name,file in sources.items():shutil.copyfile(args.import_dir/file,OUT/'source'/f'{name}.png')
 hunter();monsters();forest()
 manifest={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(NPC.rglob('*.png'))}
 (OUT/'canonical-hashes.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print('Built expedition derivatives; visual approval remains pending.')
