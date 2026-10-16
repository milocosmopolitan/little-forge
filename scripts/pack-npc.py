#!/usr/bin/env python3
"""Mechanical split/registration of generated artwork; never synthesizes animation pixels.
Source frames retain their pixels, with ONE shared scale per source sheet, no per-frame auto-fit.
"""
from pathlib import Path
import json,hashlib
import numpy as np
from scipy import ndimage
from PIL import Image
root=Path(__file__).resolve().parents[1]
folder=root/'assets/npc/farmer'
W,H=384,352
frames=[]
registration=[]
def split(im,rect,scale=1):
    cell=im.crop(rect)
    if scale!=1: cell=cell.resize((round(cell.width*scale),round(cell.height*scale)),Image.Resampling.NEAREST)
    # Keep the character component; discard neighboring-row fragments in generated sheets.
    rgba=np.array(cell)
    labels,count=ndimage.label(rgba[:,:,3]>100)
    sizes=np.bincount(labels.ravel());sizes[0]=0
    keep=labels==sizes.argmax()
    rgba[:,:,3]=np.where(keep,rgba[:,:,3],0)
    cell=Image.fromarray(rgba)
    # Alpha bbox excludes invisible RGB (some PNG viewers incorrectly show that RGB).
    bbox=cell.getchannel('A').point(lambda a:255 if a>100 else 0).getbbox()
    if not bbox: raise ValueError(rect)
    x0,y0,x1,y1=bbox
    # Ground registration from boots, not changing hat or outstretched hand.
    boots=cell.getchannel('A').crop((0,y1-18,cell.width,y1)).point(lambda a:255 if a>100 else 0).getbbox()
    pivot=(boots[0]+boots[2])//2
    offset=(W//2-pivot,H-12-y1)
    out=Image.new('RGBA',(W,H));out.alpha_composite(cell,offset)
    registration.append({'source_rect':rect,'source_scale':scale,'offset':list(offset),'foot':[W//2,H-12]})
    return out
im=Image.open(folder/'source/expressions.png').convert('RGBA')
xs=[0,350,630,930,1254];ys=[0,320,635,932,1254]
for row in range(4):
 for col in range(4):frames.append(split(im,(xs[col],ys[row],xs[col+1],ys[row+1])))
im=Image.open(folder/'source/walk.png').convert('RGBA')
for row in range(2):
 for col in range(3):frames.append(split(im,(col*512,row*512,(col+1)*512,(row+1)*512),.625))
atlas=Image.new('RGBA',(W*4,H*6))
for i,frame in enumerate(frames):atlas.alpha_composite(frame,((i%4)*W,(i//4)*H))
atlas.save(folder/'atlas.png')
# Source layers for the neutral authoring pose, on the SAME registered canvas.
neutral=frames[0]
for name,box in {'head':(0,0,W,180),'body':(0,180,W,H),'eyes':(175,108,238,145),'mouth':(180,143,237,174)}.items():
 layer=Image.new('RGBA',(W,H));layer.alpha_composite(neutral.crop(box),(box[0],box[1]));layer.save(folder/f'{name}.png')
c={'id':'farmer','faces':{'mouths':'npc/farmer/mouths.png','eyes':'npc/farmer/eyes-atlas.png','hands':'npc/farmer/hands.png'},'version':1,'image':'npc/farmer/atlas.png','size':[W*4,H*6],'cell':[W,H],'foot':[W/2,H-12],'head':[W/2,100],'frames':[],'clips':{}}
for i in range(len(frames)):
 c['frames'].append({'rect':[(i%4)*W,(i//4)*H,W,H],'duration':.12,'hand':[250,244]})
for i,hand in {12:[250,244],13:[242,230],14:[294,205],15:[225,244]}.items():c['frames'][i]['hand']=hand
c['frames'][15]['transfer']=True
c['frames'][14]['duration']=.4
for n,ids,loop in [('idle',[0],True),('talk',[0,1,2,1,3,1],True),('blink',[0,4,5,4,0],False),('surprised',[6,7,6,0],False),('nod',[0,8,9,8,0],False),('shake_head',[0,10,0,11,0],False),('walk',[16,17,18,19,20,21],True),('give',[12,13,14,15,0],False)]:c['clips'][n]={'frames':ids,'looping':loop}
hands=Image.new('RGBA',atlas.size)
for i,f in enumerate(frames):
 hx,hy=c['frames'][i]['hand']
 box=(int(hx)-20,int(hy)-10,int(hx)+28,int(hy)+12)
 hands.alpha_composite(f.crop(box),((i%4)*W+box[0],(i//4)*H+box[1]))
hands.save(folder/'hands.png')
(folder/'character.json').write_text(json.dumps(c,indent=2)+'\n')
(folder/'registration.json').write_text(json.dumps(registration,indent=2)+'\n')
# Small, independently addressable face patches for dialogue composition.
face=Image.new('RGBA',(W*4,H))
for j,i in enumerate([0,1,2,3]):
 patch=frames[i].crop((180,143,237,174));face.alpha_composite(patch,(j*W+180,143))
face.save(folder/'mouths.png')
blink=Image.new('RGBA',(W*3,H))
for j,i in enumerate([0,4,5]):
 patch=frames[i].crop((175,108,238,145));blink.alpha_composite(patch,(j*W+175,108))
blink.save(folder/'eyes-atlas.png')
print('Packed',len(frames),'registered frames',atlas.size)
# Runtime prop gets a bounded texture; preserve original generated resolution in source.
prop=Image.open(folder/'source/carrot.png').convert('RGBA')
prop=prop.crop(prop.getchannel('A').getbbox())
prop.thumbnail((112,140),Image.Resampling.NEAREST)
prop.save(folder/'carrot.png')
# Keep the existing manifest as the single integrity inventory.
manifest_path=root/'assets/manifest.json'
m=json.loads(manifest_path.read_text())
m['assets']=[a for a in m['assets'] if not a['id'].startswith('farmer_')]
for f in sorted(folder.glob('*.png')):
 im=Image.open(f);w,h=im.size
 cols,rows=(4,6) if f.name in ['atlas.png','hands.png'] else (4,1) if f.name=='mouths.png' else (3,1) if f.name=='eyes-atlas.png' else (1,1)
 m['assets'].append({'id':'farmer_'+f.stem.replace('-','_'),'path':str(f.relative_to(root/'assets')),'width':w,'height':h,'columns':cols,'rows':rows,'alpha':True,'anchor':[.5,340/352] if f.name!='carrot.png' else [.5,.5],'sha256':hashlib.sha256(f.read_bytes()).hexdigest()})
manifest_path.write_text(json.dumps(m,indent=2)+'\n')
(folder/'source/hashes.json').write_text(json.dumps({f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted((folder/'source').glob('*.png'))},indent=2)+'\n')
