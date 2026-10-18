"""Native contracts, canonical preservation, atlas roundtrip and animation coverage."""
from pathlib import Path
from PIL import Image
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];BASE=ROOT/'assets/expedition'
for path,digest in json.loads((BASE/'canonical-hashes.json').read_text()).items():
 assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,path
for name,size,foot in [('hunter',(128,160),[64,140]),('wolf',(128,96),[64,88]),('boar',(128,96),[64,88])]:
 meta=json.loads((BASE/name/'animation.json').read_text());assert meta['canvas']==list(size);assert meta['foot_anchor_px']==foot
 atlas=Image.open(BASE/name/'atlas.png');colors=set()
 for i,frame in enumerate(meta['frames']):
  im=Image.open(BASE/name/'frames'/f'{frame}.png');assert im.mode=='RGBA' and im.size==size
  assert set(im.getchannel('A').getdata())<={0,255}
  colors.update(p[:3] for p in im.getdata() if p[3]);x=i%8*size[0];y=i//8*size[1]
  assert atlas.crop((x,y,x+size[0],y+size[1])).tobytes()==im.tobytes()
  if name=='hunter' and frame.startswith('bow-'):
   neutral=Image.open(BASE/name/'frames/neutral.png')
   assert im.crop((36,16,89,65)).tobytes()==neutral.crop((36,16,89,65)).tobytes(),'head identity drift'
 for clip in meta['clips'].values():assert len(clip['frames'])==len(clip['durations_ms']) and all(t>0 for t in clip['durations_ms']) and all(f in meta['frames'] for f in clip['frames'])
 assert len(colors)<=64,(name,len(colors))
 print(name, len(meta['frames']),'frames;',len(colors),'colors; atlas and alpha valid')
tile=Image.open(BASE/'forest/ground.png');assert list(tile.crop((0,0,1,32)).getdata())==list(tile.crop((63,0,64,32)).getdata())
print('Canonical source hashes unchanged; structural validation only. Visual approval is separate.')
