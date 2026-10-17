"""Derive tavern-madam facial parts and pose animations from the locked neutral."""
from pathlib import Path
from PIL import Image, ImageDraw, GifImagePlugin
import argparse
import base64
import hashlib
import io
import json
import os
import subprocess
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
SIZE = (128,160)
NATIVE = (96,128)
OFFSET = (16,16)


def load(path): return Image.open(ROOT/path).convert('RGBA')
def blank(size=SIZE): return Image.new('RGBA',size)
def save(image,path):
    target=ROOT/path;target.parent.mkdir(parents=True,exist_ok=True);image.save(target);return image
def write(path,value):
    target=ROOT/path;target.parent.mkdir(parents=True,exist_ok=True)
    target.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')
def pixels(image): return bytes(v for p in image.getdata() for v in (p if p[3] else (0,0,0,0)))
def compose(layers,size=SIZE):
    out=blank(size)
    for layer in layers:out.alpha_composite(layer)
    return out
def embed(image):
    out=blank();out.paste(image,OFFSET);return out
def move(image,dx,dy):
    out=blank(image.size);out.paste(image,(dx,dy));return out
def region(image,points):
    mask=Image.new('L',image.size);ImageDraw.Draw(mask).polygon(points,fill=255)
    out=blank(image.size);out.paste(image,(0,0),mask);return out


class Builder:
    def __init__(self):
        self.palette=Image.open(ROOT/'source/palette.png').convert('P')
        self.colors=[tuple(self.palette.getpalette()[i:i+3]) for i in range(0,192,3)]
        self.parts={k:load(p) for k,p in {'lower':'body/lower-base.png','upper':'body/upper-base.png',
                     'head':'head-base.png','eyes':'eyes/open.png','mouth':'mouth/rest.png'}.items()}
        self.gear={f.stem:load(str(f.relative_to(ROOT))) for f in sorted((ROOT/'equipment').glob('*.png'))}
        self.gear_order=json.loads((ROOT/'source/registration.json').read_text())['equipment_order']
        self.frames={};self.recipes={};self.eyes={};self.mouth={}
        self.sources={}
    def c(self,value):
        rgb=tuple(bytes.fromhex(value.lstrip('#')))
        return min(self.colors,key=lambda p:sum((a-b)**2 for a,b in zip(p,rgb)))+(255,)
    def native(self,image):
        image=image.convert('RGBA');out=image.convert('RGB').quantize(palette=self.palette,dither=Image.Dither.NONE).convert('RGBA')
        out.putalpha(image.getchannel('A').point(lambda a:255 if a>=128 else 0))
        for y in range(out.height):
            for x in range(out.width):
                if not out.getpixel((x,y))[3]:out.putpixel((x,y),(0,0,0,0))
        return out
    def feature_under(self,feature):
        out=blank(NATIVE)
        for y in range(128):
            for x in range(96):
                if feature.getpixel((x,y))[3]:
                    p=self.parts['head'].getpixel((x,y))
                    out.putpixel((x,y),p if p[0]>205 and 110<p[1]<205 and p[2]<155 else self.c('#fdb577'))
        return out
    def expressions(self):
        original=self.parts['eyes'];self.eyes={'open':original}
        under=self.feature_under(original)
        # Occlude any warm sclera retained beneath the original component, within its eye interior.
        for y in range(26,35):
            for x in list(range(39,49))+list(range(52,56)):
                p=self.parts['head'].getpixel((x,y))
                if p[3] and p[0]>215 and p[1]>195 and p[2]>160:under.putpixel((x,y),self.c('#fdb577'))
        brows=blank(NATIVE)
        for y in range(23,27):
            for x in range(39,56):
                if (x<50 and y<=25) or (x>=50 and y<=26):brows.putpixel((x,y),original.getpixel((x,y)))
        for name in ['half','closed','happy-closed','wide']:
            image=under.copy();image.alpha_composite(brows if name!='wide' else move(brows,0,-1));d=ImageDraw.Draw(image)
            if name=='half':
                for y in range(30,35):
                    for x in range(39,56):
                        p=original.getpixel((x,y))
                        if p[3]:image.putpixel((x,y),p)
                d.line([(39,29),(42,29),(46,29),(47,30)],fill=self.c('#4d1710'))
                d.line([(52,30),(55,30)],fill=self.c('#4d1710'))
            elif name=='closed':
                d.line([(39,30),(40,31),(45,31),(47,30)],fill=self.c('#4d1710'))
                d.line([(52,31),(53,32),(55,31)],fill=self.c('#4d1710'))
            elif name=='happy-closed':
                # Keep the smile on the open-eye baseline without moving brows or skin occlusion.
                for y in range(30,34):image.putpixel((51,y),self.c('#fdb577'))
                d.line([(39,32),(40,31),(42,30),(45,30),(47,32)],fill=self.c('#4d1710'))
                d.line([(52,32),(53,31),(54,31),(55,32)],fill=self.c('#4d1710'))
            else:
                d.polygon([(40,26),(45,26),(47,28),(47,31),(46,33),(41,33),(39,31),(39,28)],fill=self.c('#fff2d7'))
                d.line([(39,28),(40,26),(45,26),(47,28)],fill=self.c('#4d1710'))
                d.rectangle((40,28,43,31),fill=self.c('#4d1710'));d.point((41,28),fill=self.c('#fff2d7'))
                d.polygon([(53,27),(55,28),(55,33),(52,34),(52,29)],fill=self.c('#fff2d7'))
                d.line([(52,29),(53,27),(55,28)],fill=self.c('#4d1710'))
                d.rectangle((52,30,53,33),fill=self.c('#4d1710'));d.point((52,30),fill=self.c('#fff2d7'))
            self.eyes[name]=save(image,f'eyes/{name}.png')
        self.mouth={'rest':self.parts['mouth']}
        for name in ['small-open','wide-open','round']:
            image=self.feature_under(self.parts['mouth']);d=ImageDraw.Draw(image)
            if name=='small-open':
                d.polygon([(46,36),(49,36),(50,37),(49,38),(46,38)],fill=self.c('#4d1710'))
                d.line([(47,38),(48,38)],fill=self.c('#b04a23'))
            elif name=='wide-open':
                d.polygon([(45,36),(50,36),(51,37),(50,39),(48,40),(46,39),(45,38)],fill=self.c('#4d1710'))
                d.line([(46,37),(49,37)],fill=self.c('#fff2d7'))
                d.line([(47,39),(49,39)],fill=self.c('#b04a23'))
            else:
                d.polygon([(47,36),(49,36),(50,37),(50,38),(49,39),(47,39),(46,38),(46,37)],fill=self.c('#4d1710'))
                d.line([(48,38),(49,38)],fill=self.c('#b04a23'))
            self.mouth[name]=save(image,f'mouth/{name}.png')
        recipes={'neutral':('open','rest'),'talk-small':('open','small-open'),'talk-wide':('open','wide-open'),
                 'talk-round':('open','round'),'blink':('closed','rest'),'happy':('happy-closed','wide-open'),
                 'surprised':('wide','round'),'relaxed':('half','rest')}
        for name,(eye,mouth) in recipes.items():
            parts={**self.parts,'eyes':self.eyes[eye],'mouth':self.mouth[mouth]}
            base=compose(list(parts.values()),NATIVE);full=compose([base]+[self.gear[n] for n in self.gear_order],NATIVE)
            save(base,f'expressions/base/{name}.png');save(full,f'expressions/{name}.png')
            self.add(name,{n:embed(im) for n,im in parts.items()},{n:embed(im) for n,im in self.gear.items()},
                     sockets={'neck':[62,59],'waist':[62,86],'right_hand':[49,83],'left_hand':[84,66]})
        eye_mask=compose(list(self.eyes.values()),NATIVE);mouth_mask=compose(list(self.mouth.values()),NATIVE)
        save(eye_mask,'source/animation/eye-change-mask.png');save(mouth_mask,'source/animation/mouth-change-mask.png')
        write('expressions/recipes.json',{'canvas':[96,128],'eyes':{n:f'eyes/{n}.png' for n in self.eyes},
              'mouth':{n:f'mouth/{n}.png' for n in self.mouth},
              'recipes':{n:{'eyes':e,'mouth':m} for n,(e,m) in recipes.items()},
              'channels':{'eyes':'independent blink; surprised locks wide','mouth':'independent speech; gestures retain pose-specific rest'}})
        for e in self.eyes:
            for m in self.mouth:
                base=compose([self.parts[n] for n in ['lower','upper','head']]+[self.eyes[e],self.mouth[m]],NATIVE)
                save(embed(compose([base]+[self.gear[n] for n in self.gear_order],NATIVE)),f'previews/combinations/{e}--{m}.png')
    def add(self,name,parts,gear,sockets=None,extra=None):
        back=(extra or {}).get('back_equipment',[])
        order=[n for n in self.gear_order if n in gear]+[n for n in gear if n not in self.gear_order]
        base=compose(list(parts.values()))
        full_order=back+[n for n in parts if n!='arms_front']+[n for n in order if n not in back]+(['arms_front'] if 'arms_front' in parts else [])
        if (extra or {}).get('head_in_front_of_body_gear'):
            full_order=back+[n for n in parts if n not in ['head','eyes','mouth','arms_front']]
            full_order += [n for n in order if n not in ['hair-ribbon','earrings','tankard']+back]
            full_order += ['head','eyes','mouth']
            full_order += [n for n in ['hair-ribbon','earrings','tankard'] if n in gear]
            if 'arms_front' in parts:full_order.append('arms_front')
        full=compose([({**parts,**gear})[n] for n in full_order])
        for n,image in parts.items():save(image,f'poses/{name}/layers/{n}.png')
        for n,image in gear.items():save(image,f'poses/{name}/equipment/{n}.png')
        save(base,f'poses/{name}/base.png');save(full,f'poses/{name}/composed.png')
        save(base,f'frames/base/{name}.png');save(full,f'frames/{name}.png')
        save(full.resize((512,640),Image.Resampling.NEAREST),f'previews/frames/{name}-4x.png')
        self.frames[name]={'base':base,'full':full}
        self.recipes[name]={'layers':{n:f'poses/{name}/layers/{n}.png' for n in parts},
             'equipment':{n:f'poses/{name}/equipment/{n}.png' for n in gear},
             'base_order':list(parts),'composition_order':full_order,'sockets_px':sockets or {},**(extra or {})}

    def candidate(self,name,kind='body'):
        path=f'source/animation/candidates/{name}.png';image=load(path)
        if kind=='head':
            image=self.native(image.resize((64,64),Image.Resampling.NEAREST))
            out=blank(NATIVE);out.paste(image,(12,0))
            registration={'native_cell':[64,64],'paste':[12,0]}
        else:
            transform=(11*image.width/1024,0,-16*image.width/1024,
                       0,11*image.height/1536,64*image.height/1536)
            out=self.native(image.transform(NATIVE,Image.Transform.AFFINE,transform,Image.Resampling.NEAREST))
            registration={'transform':list(transform),'source_size':list(image.size)}
        save(out,f'source/animation/candidates/{name}-native.png')
        self.sources[name]={'path':path,**registration}
        return out

    def split_head(self,image,boxes,mouth_box,mole):
        head=image.copy();eyes=blank(NATIVE);mouth=blank(NATIVE)
        head.putpixel(tuple(mole),self.c('#732b16'))
        source=head.copy()
        for y in range(64):
            for x in range(96):
                p=source.getpixel((x,y));r,g,b,a=p
                eye=any(l<=x<rr and t<=y<bb for l,t,rr,bb in boxes)
                l,t,rr,bb=mouth_box;lip=l<=x<rr and t<=y<bb
                feature=(max(r,g,b)<175 or (r>215 and g>195 and b>160)) if eye else (
                    max(r,g,b)<210 and r>g*1.3) if lip else False
                if a and feature:
                    (eyes if eye else mouth).putpixel((x,y),p)
                    candidates=[source.getpixel((xx,yy)) for yy in range(max(0,y-3),min(64,y+4))
                                for xx in range(max(0,x-3),min(96,x+4))]
                    skin=[q for q in candidates if q[3] and q[0]>205 and 110<q[1]<205 and q[2]<155]
                    head.putpixel((x,y),min(skin,key=lambda q:abs(q[1]-181)) if skin else self.c('#fdb577'))
        assert pixels(compose([head,eyes,mouth],NATIVE))==pixels(source),'pose face separation changed art'
        return {'head':head,'eyes':eyes,'mouth':mouth}

    def head_poses(self):
        neck=blank(NATIVE);d=ImageDraw.Draw(neck)
        # Restore only the short neck attachment hidden by the approved jaw.
        d.polygon([(41,39),(48,39),(51,43),(50,45),(45,44),(41,42)],fill=self.c('#fdb577'))
        d.line([(48,40),(50,43),(50,44)],fill=self.c('#e49a56'))
        save(embed(neck),'source/animation/fixed-neck-support.png')
        configs={
            'nod-down':{'eyes':[(37,33,49,39),(52,34,58,39),(39,29,48,33)],'mouth':[44,41,52,44],
                        'mole':[41,39],'ribbon':[0,1],'earring':[1,3]},
            'turn-left':{'eyes':[(38,28,48,34),(51,28,57,35),(42,25,49,28),(52,25,57,28)],'mouth':[44,36,51,39],
                         'mole':[40,35],'ribbon':[-1,0],'earring':[-1,-1]},
            'turn-right':{'eyes':[(41,29,51,35),(56,29,60,36),(43,26,52,29),(57,27,60,29)],'mouth':[48,37,55,40],
                          'mole':[41,35],'ribbon':[1,0],'earring':[1,0]}}
        original={n:embed(self.parts[n]) for n in ['head','eyes','mouth']}
        parts={'lower':embed(self.parts['lower']),'neck':embed(neck),'upper':embed(self.parts['upper']),
               **{n:move(im,0,1) for n,im in original.items()}}
        gear={n:embed(im) for n,im in self.gear.items()}
        for n in ['hair-ribbon','earrings']:gear[n]=move(gear[n],0,1)
        self.add('nod-mid',parts,gear,extra={'head_translation':[0,1],'face_source':'approved neutral; no feature rescaling','head_in_front_of_body_gear':True})
        for name,config in configs.items():
            source=self.candidate(name,'head')
            if name=='nod-down':
                repair_path='source/animation/nod-eye-repair/candidate.png'
                repaired=self.native(load(repair_path).resize((64,64),Image.Resampling.NEAREST))
                # Adopt only both eye interiors; retain the approved pitched head and face landmarks.
                boxes=[(40,33,49,39),(52,34,57,40)]
                mask=blank(NATIVE)
                for l,t,r,bb in boxes:
                    for y in range(t,bb):
                        for x in range(l,r):
                            p=repaired.getpixel((x-12,y))
                            if p[3] and source.getpixel((x,y))[3]:source.putpixel((x,y),p);mask.putpixel((x,y),(255,255,255,255))
                nose_path='source/animation/nod-eye-repair/nose-candidate.png'
                if (ROOT/nose_path).exists():
                    nose=self.native(load(nose_path).resize((64,64),Image.Resampling.NEAREST))
                    for y in range(39,41):
                        for x in range(50,56):
                            p=nose.getpixel((x-12,y))
                            if p[3] and source.getpixel((x,y))[3]:source.putpixel((x,y),p);mask.putpixel((x,y),(255,255,255,255))
                    self.sources[name]['nose_repair']={'path':nose_path,'native_size':[64,64],'paste':[12,0],
                                                       'adoption_box':[50,39,56,41],'prompt':'source/animation/nod-eye-repair/nose-prompt.json'}
                save(mask,'source/animation/nod-eye-repair/adoption-mask.png')
                self.sources[name]['eye_repair']={'path':repair_path,'native_size':[64,64],'paste':[12,0],
                                                  'eye_boxes':boxes,'prompt':'source/animation/nod-eye-repair/prompt.json'}
            features=self.split_head(source,config['eyes'],config['mouth'],config['mole'])
            save(compose(list(features.values()),NATIVE),f'source/animation/{name}-registered-face.png')
            parts={'lower':embed(self.parts['lower']),'neck':embed(neck),'upper':embed(self.parts['upper']),
                   **{n:embed(im) for n,im in features.items()}}
            gear={n:embed(im) for n,im in self.gear.items()}
            gear['hair-ribbon']=move(gear['hair-ribbon'],*config['ribbon'])
            gear['earrings']=move(gear['earrings'],*config['earring'])
            self.add(name,parts,gear,extra={'face_source':self.sources[name]['path'],
                'feature_boxes':config['eyes'],'mouth_box':config['mouth'],'mole_px':[config['mole'][0]+16,config['mole'][1]+16],
                'head_in_front_of_body_gear':True})
        write('source/animation/head-registration.json',configs)

    def give(self):
        grip_points=[(66,45),(71,45),(72,48),(73,49),(73,51),(71,52),
                     (70,54),(68,54),(67,53),(66,51),(65,50),(65,48)]
        original_grip=region(self.parts['upper'],grip_points)
        save(original_grip,'source/animation/approved-grip.png')
        for name,socket,towel_delta in [('give-mid',(78,61),(4,6)),('give-full',(80,71),(6,12))]:
            candidate=self.candidate(name);upper=self.parts['upper'].copy()
            # Replace the old far arm completely, retaining the approved torso and hip hand.
            for y in range(43,83):
                for x in range(55,96):
                    if (y<50 and x>=63) or (50<=y<72) or (y>=72 and x>=66):
                        upper.putpixel((x,y),candidate.getpixel((x,y)) if y>=50 else (0,0,0,0))
            dx,dy=socket[0]-68,socket[1]-50
            # The generator's fingers are discarded. Keep its sleeve and distal forearm.
            for y in range(43+dy,57+dy):
                for x in range(65+dx, min(96,77+dx)):
                    if x>=67+dx or y<49+dy:upper.putpixel((x,y),(0,0,0,0))
            upper.alpha_composite(move(original_grip,dx,dy))
            # Remove only a wrist outline that became an internal forearm seam.
            before=upper.copy()
            for y in range(49+dy,55+dy):
                for x in range(65+dx,67+dx):
                    p=before.getpixel((x,y))
                    if p[3] and max(p[:3])<100 and all(before.getpixel((x+a,y+b))[3] for a,b in [(1,0),(-1,0),(0,1),(0,-1)]):
                        upper.putpixel((x,y),self.c('#fdb577'))
            parts={**{n:embed(im) for n,im in self.parts.items()},'upper':embed(upper)}
            gear={n:embed(im) for n,im in self.gear.items()}
            gear['tankard']=move(gear['tankard'],dx,dy)
            gear['towel']=move(gear['towel'],*towel_delta)
            sockets={'left_hand':[socket[0]+16,socket[1]+16],'right_hand':[49,83]}
            extra={'arm_source':self.sources[name]['path'],'grip_translation':[dx,dy],
                   'towel_translation':list(towel_delta),'candidate_fingers_adopted':False}
            self.add(name,parts,gear,sockets,extra)
            empty_name='give-empty' if name=='give-full' else 'give-retract'
            self.add(empty_name,parts,{n:im for n,im in gear.items() if n!='tankard'},sockets,extra)
        self.add('neutral-empty',{n:embed(im) for n,im in self.parts.items()},
                 {n:embed(im) for n,im in self.gear.items() if n!='tankard'})

    def walk(self):
        contract=json.loads((ROOT/'source/animation/production-contract.json').read_text())
        self.gait=[]
        for k in range(6):
            name=f'walk-{k}';body=self.candidate(name)
            refinements=[]
            if k in (3,5):
                path=f'source/animation/candidates/{name}-arm.png'
                patch=self.native(load(path).resize((64,64),Image.Resampling.NEAREST))
                body.paste(patch.crop((4,5,38,53)),(8,45));refinements.append(path)
            if k in (2,4,5):
                path=f'source/animation/candidates/{name}-lower.png'
                patch=self.native(load(path).resize((96,64),Image.Resampling.NEAREST))
                body.paste(patch.crop((0,32,96,64)),(0,96));refinements.append(path)
            # Stockings belong to the original costume, including under a lifted hem.
            for y in range(96,128):
                for x in range(96):
                    r,g,b,a=body.getpixel((x,y))
                    if a and r>225 and 110<g<190 and b<155:body.putpixel((x,y),self.c('#9d4c2b' if g>150 else '#813a24'))
            dy=124-body.getchannel('A').getbbox()[3]
            body=move(body,0,dy)
            bob=contract['walk']['keys'][k]['bob_y']
            # Discard the generated head completely; retain the approved identity at integer offsets.
            for y in range(50):
                for x in range(96):
                    if y<41 or (y<44 and x<34) or x>58:body.putpixel((x,y),(0,0,0,0))
            collar=region(self.parts['upper'],[(34,41),(58,41),(61,49),(34,49)])
            body.alpha_composite(move(collar,0,bob))
            remaining={(x,y) for y in range(128) for x in range(96) if body.getpixel((x,y))[3]}
            while remaining:
                queue=[remaining.pop()];component=[]
                while queue:
                    x,y=queue.pop();component.append((x,y))
                    for dx,ddy in [(1,0),(-1,0),(0,1),(0,-1)]:
                        point=(x+dx,y+ddy)
                        if point in remaining:remaining.remove(point);queue.append(point)
                if len(component)<12 and max(y for x,y in component)<50:
                    for point in component:body.putpixel(point,(0,0,0,0))
            upper=body.copy();lower=body.copy()
            for y in range(128):
                for x in range(96):
                    (upper if y>=72 else lower).putpixel((x,y),(0,0,0,0))
            front=blank(NATIVE)
            for y in range(60,96):
                for x in range(7,49):
                    p=body.getpixel((x,y));r,g,b,a=p
                    skin=a and r>205 and 110<g<205 and b<155
                    neighbor=any(0<=x+dx<96 and 0<=y+ddy<128 and
                        (q:=body.getpixel((x+dx,y+ddy)))[3] and q[0]>205 and 110<q[1]<205 and q[2]<155
                        for dx,ddy in [(1,0),(-1,0),(0,1),(0,-1)])
                    if skin or (a and max(r,g,b)<160 and neighbor):front.putpixel((x,y),p)
            parts={'lower':embed(lower),'upper':embed(upper),
                   **{n:move(embed(self.parts[n]),0,bob) for n in ['head','eyes','mouth']},'arms_front':embed(front)}
            gear={n:move(embed(im),0,bob) for n,im in self.gear.items() if n not in ['tankard','towel']}
            # The red underhem follows the walking hem behind the legs, never across a lifted boot.
            shawl=self.gear['shawl'].copy();underhem=blank(NATIVE)
            def hem(image,x):
                ys=[y for y in range(90,117) if (p:=image.getpixel((x,y)))[3] and p[0]>130
                    and 90<p[1]<180 and p[2]>55 and p[0]-p[1]<90]
                return max(ys) if ys else 108
            baseline=load('base-neutral.png');deltas={x:hem(body,x)-hem(baseline,x) for x in range(34,63)}
            for x in range(34,63):
                samples=sorted(deltas[xx] for xx in range(max(34,x-2),min(63,x+3)));delta=samples[len(samples)//2]
                for y in range(110,128):
                    p=shawl.getpixel((x,y))
                    if p[3]:underhem.putpixel((x,y+delta),p);shawl.putpixel((x,y),(0,0,0,0))
            gear['shawl']=move(embed(shawl),0,bob);gear['shawl-underhem']=embed(underhem)
            hand_points=[]
            for l,t,r,bb in [(7,74,49,96),(59,74,89,96)]:
                points=[(x,y) for y in range(t,bb) for x in range(l,r) if (p:=body.getpixel((x,y)))[3]
                        and p[0]>205 and 110<p[1]<205 and p[2]<155]
                bottom=max(y for x,y in points);points=[p for p in points if p[1]>=bottom-3]
                hand_points.append([round(sum(p[i] for p in points)/len(points),2) for i in range(2)])
            foot_rois=[[(18,108,40,124),(54,108,76,124)],[(20,108,42,124),(50,108,76,124)],
                       [(32,103,50,121),(52,108,78,124)],[(54,106,80,124),(20,106,44,124)],
                       [(52,108,72,124),(27,106,44,122)],[(36,108,52,124),(48,106,65,123)]][k]
            feet=[]
            for l,t,r,bb in foot_rois:
                points=[(x,y) for y in range(t,bb) for x in range(l,r) if (p:=body.getpixel((x,y)))[3]
                        and p[0]>p[1]*1.3 and p[1]<130 and p[2]<110]
                feet.append({'centroid':[round(sum(p[i] for p in points)/len(points),2) for i in range(2)],
                             'leather_bbox':[min(p[0] for p in points),min(p[1] for p in points),
                                             max(p[0] for p in points)+1,max(p[1] for p in points)+1]})
            sockets={'neck':[62,59+bob],'waist':[62,86+bob],
                     'right_hand':[round(v)+16 for v in hand_points[0]],
                     'left_hand':[round(v)+16 for v in hand_points[1]]}
            phase={**contract['walk']['keys'][k],'source_ground_translation_y':dy,
                   'refinements':refinements,'source':self.sources[name]['path'],
                   'measured_native':{'near_hand':hand_points[0],'far_hand':hand_points[1],
                                      'near_boot':feet[0],'far_boot':feet[1],'foot_rois':foot_rois}}
            save(body,f'source/animation/{name}-body-registered.png')
            self.add(name,parts,gear,sockets,{'gait':phase,'held_props':False,'head_translation':[0,bob],
                     'head_in_front_of_body_gear':True,'front_arm_occlusion':'near forearm and fingers above apron/belt',
                     'back_equipment':['shawl-underhem'],'underhem_registration':'median-5 column hem offset, drawn behind legs'})
            self.gait.append(phase)

    def contact(self,items,path,columns=4,scale=3,background='#eee3cc'):
        cw,ch=128*scale+16,160*scale+40
        out=Image.new('RGB',(cw*columns,ch*((len(items)+columns-1)//columns)),background);d=ImageDraw.Draw(out)
        for i,(name,image) in enumerate(items):
            x,y=(i%columns)*cw,(i//columns)*ch
            d.text((x+8,y+8),name,fill='#eee3cc' if background=='#302b32' else '#211510')
            image=image.resize((128*scale,160*scale),Image.Resampling.NEAREST);out.paste(image,(x+8,y+30),image)
        save(out,path)

    def clips(self):
        def clip(ids,times,loop=False,events=None):
            return {'frames':ids,'durations_ms':times,'loop':loop,'events':events or []}
        return {
            'idle':clip(['neutral'],[1200],True),
            'blink':clip(['neutral','relaxed','blink','relaxed','neutral'],[1200,50,100,50,500],True),
            'talk':clip(['neutral','talk-small','talk-wide','talk-small','talk-round','neutral'],[160,100,120,100,120,180],True),
            'happy':clip(['neutral','happy','neutral'],[180,700,180]),
            'surprised':clip(['neutral','surprised','neutral'],[160,650,180]),
            'relaxed':clip(['neutral','relaxed','neutral'],[180,700,180]),
            'nod':clip(['neutral','nod-mid','nod-down','nod-mid','neutral'],[120,100,140,100,120]),
            'shake_head':clip(['neutral','turn-left','neutral','turn-right','neutral'],[100,140,100,140,100]),
            'give':clip(['neutral','give-mid','give-full','give-empty','give-retract','neutral-empty'],[120,120,180,120,120,120],
                        events=[{'key':3,'name':'transfer','hand':'left','item':'tankard','fires':'once_on_entry'}]),
            'walk':clip([f'walk-{k}' for k in range(6)],[120]*6,True)}

    def gif(self,ids,times,path,scale=4,slow=1):
        # Write each key explicitly, including repeated start/end images and holds.
        frames=[]
        for name in ids:
            panel=Image.new('RGB',(128*scale*2+48,160*scale+40),'#eee3cc');d=ImageDraw.Draw(panel)
            for k,variant in enumerate(['base','full']):
                d.text((12+k*(128*scale+24),8),f'{name} / {variant}',fill='#211510')
                im=self.frames[name][variant].resize((128*scale,160*scale),Image.Resampling.NEAREST)
                panel.paste(im,(12+k*(128*scale+24),28),im)
            frames.append(panel.quantize(colors=256,dither=Image.Dither.NONE))
        out=io.BytesIO();header,_=GifImagePlugin.getheader(frames[0],info={'loop':0})
        for block in header:out.write(block)
        for image,duration in zip(frames,times):
            for block in GifImagePlugin.getdata(image,include_color_table=True,duration=duration*slow,disposal=2):out.write(block)
        out.write(b';');target=ROOT/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(out.getvalue())

    def export(self):
        clips=self.clips();names=list(self.frames)
        animation={'version':2,'id':'tavern-madam','canvas':[128,160],'foot_anchor_px':[64,140],
                   'frames':names,'clips':clips,'recipes':'poses/recipes.json',
                   'channels':'expressions/recipes.json','scope':'authoring; not game runtime integration'}
        write('animation.json',animation)
        write('poses/recipes.json',{'version':2,'canvas':[128,160],'foot_anchor_px':[64,140],
                                   'side_convention':'anatomical; near=right, far=left','poses':self.recipes})
        write('source/animation/source-registration.json',self.sources)
        write('source/animation/walk-phases.json',self.gait)
        contract=json.loads((ROOT/'source/animation/production-contract.json').read_text())
        contract['give'].update(mid_socket=[78,61],full_socket=[80,71])
        write('source/animation/production-contract.json',contract)
        atlas=blank((1056,496));sprites=[]
        for i,name in enumerate(names):
            x=(i%8)*132+2;y=(i//8)*164+2;im=self.frames[name]['full'];atlas.paste(im,(x,y))
            for yy in range(-1,161):
                for xx in range(-1,129):
                    if xx in (-1,128) or yy in (-1,160):atlas.putpixel((x+xx,y+yy),im.getpixel((max(0,min(127,xx)),max(0,min(159,yy)))))
            sprites.append({'id':name,'page':'tavern-madam-0','rect':[x,y,128,160],'source_size':[128,160],
                            'trim_offset':[0,0],'pivot':[64,140],'source':f'frames/{name}.png','rotated':False})
        save(atlas,'atlas.png')
        write('atlas-manifest.json',{'version':2,'mode':'baked','usage':'common_npc_authoring_not_integrated',
              'sampler':'nearest','mipmaps':False,'extrude_px':1,'gutter_px':1,'rotate_packing':False,
              'pages':[{'id':'tavern-madam-0','path':'atlas.png','size':[1056,496]}],'sprites':sprites,'clips':clips})
        expressions=list(json.loads((ROOT/'expressions/recipes.json').read_text())['recipes'])
        self.contact([(n,self.frames[n]['full']) for n in expressions],'previews/expressions-8.png',4,4)
        self.contact([(n,self.frames[n]['base']) for n in expressions],'previews/expressions-8-base.png',4,4)
        combinations=[(p.stem,Image.open(p).convert('RGBA')) for p in sorted((ROOT/'previews/combinations').glob('*.png'))]
        self.contact(combinations,'previews/expression-combinations-20.png',4,3)
        for variant in ['base','full']:
            for scale in [1,4]:
                for bg in ['light','dark']:
                    self.contact([(f'walk-{k}',self.frames[f'walk-{k}'][variant]) for k in range(6)],
                        f'previews/walk-6-{variant}-{bg}-{scale}x.png',6,scale,'#302b32' if bg=='dark' else '#eee3cc')
            self.contact([(f'walk-{k}',self.frames[f'walk-{k}'][variant]) for k in range(6)],f'previews/walk-6-{variant}.png',6,4)
        self.contact([(n,self.frames[n]['full']) for n in ['neutral','nod-mid','nod-down','turn-left','turn-right']],
                     'previews/head-poses.png',5,4)
        self.contact([(n,self.frames[n]['full']) for n in clips['give']['frames']],'previews/give-6.png',6,4)
        for name,clip in clips.items():
            self.gif(clip['frames'],clip['durations_ms'],f'previews/{name}.gif')
            self.gif(clip['frames'],clip['durations_ms'],f'previews/{name}-slow.gif',slow=4)
        self.html(animation)
        spec=yaml.safe_load((ROOT/'character-spec.yaml').read_text())
        spec.update(action_set='human_common',expression_recipes=json.loads((ROOT/'expressions/recipes.json').read_text())['recipes'],
                    animation=animation,production_plan={'status':'common_set_complete','completed':['neutral','expressions_8','nod','shake_head','give','walk_6'],
                                                       'not_generated':[]})
        spec['layers']['eyes']={n:f'eyes/{n}.png' for n in self.eyes}
        spec['layers']['mouth']={n:f'mouth/{n}.png' for n in self.mouth}
        spec['generation']['animation_prompts']='source/animation/generation-prompts.json'
        spec['generation']['animation_registration']='source/animation/source-registration.json'
        spec['animation_builder']='source/build-animation.py'
        spec['limb_identity'].update(walk_near='right arm and large-buckle boot',walk_far='left arm and small-buckle boot',
                                     held_props_during_walk=False,towel_during_walk=False)
        (ROOT/'character-spec.yaml').write_text(yaml.safe_dump(spec,sort_keys=False,allow_unicode=True))
        (ROOT/'ANIMATION.md').write_text('''# Tavern madam animation assets

Open `previews/animation.html` for normal/slow playback, key stepping, independent eye/mouth combinations, base/equipment, light/dark and 1x/4x views.

All 23 frame and pose canvases are 128x160 with foot pivot (64,140). The approved 96x128 authoring layers are embedded once at (16,16); the fourteen approved neutral images remain pixel-identical. 8 expressions, nod, shake_head, give and six-key walk are complete. The 64-color shared palette uses binary alpha and nearest sampling.

`animation.json` owns clip timing, repeat policy and the one `transfer` event on give key 3. Give ends empty-handed. Preview replay restarts the demonstration with the original tankard; this does not create an inventory item. Walk frees both arms and omits the tankard and wrist towel. Anatomical near side is right; the large boot buckle belongs to that leg as it crosses the screen. All frame layers, sockets and occlusion orders are in `poses/recipes.json`.

Build: `python3 source/build-animation.py`. The neutral builder also regenerates animations automatically after the baseline. Check: `python3 source/verify-character.py` then `python3 source/verify-animation.py`. Generated candidates, exact prompts, registration, baseline hashes and gait phases are retained in `source/animation/`.

`atlas-manifest.json` is an authoring manifest. No game catalog or runtime asset has been replaced, and game runtime performance has not been measured.
''')

    def html(self,animation):
        def uri(image):
            stream=io.BytesIO();image.save(stream,format='PNG');return 'data:image/png;base64,'+base64.b64encode(stream.getvalue()).decode()
        data={'animation':animation,'frames':{n:{v:uri(im) for v,im in variants.items()} for n,variants in self.frames.items()},
              'parts':{n:uri(embed(im)) for n,im in self.parts.items()},
              'eyes':{n:uri(embed(im)) for n,im in self.eyes.items()},'mouth':{n:uri(embed(im)) for n,im in self.mouth.items()},
              'equipment':{n:uri(embed(self.gear[n])) for n in self.gear_order}}
        template=(ROOT/'source/animation/preview-template.html').read_text()
        (ROOT/'previews/animation.html').write_text(template.replace('__DATA__',json.dumps(data,separators=(',',':'))))


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--base-ready',action='store_true');args=parser.parse_args()
    if not args.base_ready:
        subprocess.run([sys.executable,str(ROOT/'source/build-character.py')],check=True,env={**os.environ,'MADAM_BASE_ONLY':'1'})
    builder=Builder();builder.expressions();builder.head_poses();builder.give();builder.walk();builder.export()
    print(f'Built {len(builder.frames)} tavern-madam frames, 8 expressions, nod/shake/give and six-key walk')


if __name__=='__main__':main()

