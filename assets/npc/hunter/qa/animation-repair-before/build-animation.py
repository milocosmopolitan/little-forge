"""Build hunter expressions and gestures from the locked neutral and generated pose sources."""
from pathlib import Path
from PIL import Image, ImageDraw, GifImagePlugin
import base64, hashlib, io, json, math, yaml

ROOT=Path(__file__).resolve().parents[1]
SIZE=(128,160)
def load(p):return Image.open(ROOT/p).convert('RGBA')
def blank():return Image.new('RGBA',SIZE)
def save(im,p):
    p=ROOT/p;p.parent.mkdir(parents=True,exist_ok=True);im.save(p);return im
def write(p,value):
    p=ROOT/p;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(value,indent=2)+'\n')
def compose(layers):
    out=blank()
    for im in layers:out.alpha_composite(im)
    return out
def move(im,dx,dy):
    out=blank();out.paste(im,(round(dx),round(dy)));return out
def region(im,polygon):
    mask=Image.new('L',SIZE);ImageDraw.Draw(mask).polygon(polygon,fill=255)
    out=blank();out.paste(im,(0,0),mask);return out
def skin(p):return p[3] and p[0]>185 and 85<p[1]<210 and p[2]<170 and p[0]>p[1]*1.2

class Builder:
    def __init__(self):
        self.palette=Image.open(ROOT/'source/palette.png').convert('P')
        self.colors=[tuple(self.palette.getpalette()[i:i+3]) for i in range(0,192,3)]
        self.parts={n:load(p) for n,p in {'lower':'body/lower-base.png','upper':'body/upper-base.png','head':'head-base.png','eyes':'eyes/open.png','mouth':'mouth/rest.png'}.items()}
        # Retain the approved neutral appearance while moving residual sclera into the eye channel.
        for y in range(45,54):
            for x in list(range(60,71))+list(range(74,81)):
                q=self.parts['head'].getpixel((x,y))
                if q[3] and ((q[0]>215 and q[1]>=185 and q[2]>=120) or (x==75 and 46<=y<=51 and max(q[:3])<190)):
                    self.parts['eyes'].putpixel((x,y),q)
                    self.parts['head'].putpixel((x,y),self.c((250,178,118)))
        save(self.parts['eyes'],'eyes/animation-open.png')
        self.gear={p.stem:load(str(p.relative_to(ROOT))) for p in (ROOT/'equipment').glob('*.png')}
        self.order=json.loads((ROOT/'source/registration.json').read_text())['composition_order']
        self.hands=load('body/hands-front.png');self.frames={};self.recipes={};self.registration={}
        self.neutral=load('base-neutral.png');self.sockets={'neck':[67,65],'waist':[65,94],'right_hand':[55,94],'left_hand':[86,105]}
    def c(self,rgb):return min(self.colors,key=lambda q:sum((a-b)**2 for a,b in zip(q,rgb)))+(255,)
    def native(self,im):
        alpha=im.getchannel('A').point(lambda v:255 if v>=128 else 0)
        out=im.convert('RGB').quantize(palette=self.palette,dither=Image.Dither.NONE).convert('RGBA');out.putalpha(alpha)
        for y in range(im.height):
            for x in range(im.width):
                if not alpha.getpixel((x,y)):out.putpixel((x,y),(0,0,0,0))
        return out
    def sheet(self,name,k,cols,rows,size,paste=(0,0),scale=1,offset=(0,0)):
        raw=load(f'source/animation/{name}-candidate.png');x,y=k%cols,k//cols
        crop=raw.crop((round(x*raw.width/cols),round(y*raw.height/rows),round((x+1)*raw.width/cols),round((y+1)*raw.height/rows)))
        im=crop.resize(size,Image.Resampling.NEAREST)
        # Only remove out-of-palette generation specks, never erode artwork contours.
        for yy in range(im.height):
            for xx in range(im.width):
                r,g,b,a=im.getpixel((xx,yy))
                if a and ((g>r*1.5 and g>b*1.2) or (b>r*1.3 and b>g*1.2) or (r>160 and b>100 and g<100)):im.putpixel((xx,yy),(0,0,0,0))
        if scale!=1:im=im.resize((round(size[0]*scale),round(size[1]*scale)),Image.Resampling.NEAREST)
        out=blank();out.paste(self.native(im),(paste[0]+offset[0],paste[1]+offset[1]))
        key=f'{name}-{k}';save(out,f'source/animation/registered/{key}.png')
        self.registration[key]={'source':f'source/animation/{name}-candidate.png','cell':[x,y,cols,rows],'native_size':list(size),'common_scale':scale,'paste':[paste[0]+offset[0],paste[1]+offset[1]]}
        return out
    def add(self,name,parts,gear=None,hands=None,extra=None):
        gear=self.gear.copy() if gear is None else gear
        parts=parts.copy();parts['hands-front']=blank() if hands is None else hands
        order=[n for n in self.order if n in parts or n in gear]
        order += [n for n in parts if n not in order]
        base=compose([parts[n] for n in ['lower','upper','head','eyes','mouth'] if n in parts])
        # Remove detached generator specks only; do not erode the connected character.
        if name not in ['neutral','talk-small','talk-wide','talk-round','blink','happy','surprised','relaxed']:
            points={(x,y) for y in range(160) for x in range(128) if base.getpixel((x,y))[3]};groups=[]
            while points:
                stack=[points.pop()];group=[]
                while stack:
                    x,y=stack.pop();group.append((x,y))
                    for dx in [-1,0,1]:
                        for dy in [-1,0,1]:
                            q=x+dx,y+dy
                            if q in points:points.remove(q);stack.append(q)
                groups.append(group)
            if groups:
                groups.sort(key=len,reverse=True)
                for group in groups[1:]:
                    if len(group)<=12:
                        for point in group:
                            for im in parts.values():im.putpixel(point,(0,0,0,0))
                base=compose([parts[n] for n in ['lower','upper','head','eyes','mouth'] if n in parts])
        # A foreground hand is a duplicate subset of the complete pose, not additional base anatomy.
        full=compose([({**parts,**gear})[n] for n in order])
        for n,im in parts.items():save(im,f'poses/{name}/layers/{n}.png')
        for n,im in gear.items():save(im,f'poses/{name}/equipment/{n}.png')
        for p,im in [(f'frames/base/{name}.png',base),(f'frames/{name}.png',full),(f'poses/{name}/base.png',base),(f'poses/{name}/composed.png',full)]:save(im,p)
        self.frames[name]={'base':base,'full':full,'parts':parts,'gear':gear}
        self.recipes[name]={'layers':{n:f'poses/{name}/layers/{n}.png' for n in parts},'equipment':{n:f'poses/{name}/equipment/{n}.png' for n in gear},'base_order':['lower','upper','head','eyes','mouth'],'composition_order':order,'sockets_px':self.sockets.copy(),**(extra or {})}
        self.recipes[name]['sockets_px']['eyes']=[round((parts['eyes'].getbbox()[0]+parts['eyes'].getbbox()[2])/2),round((parts['eyes'].getbbox()[1]+parts['eyes'].getbbox()[3])/2)] if parts['eyes'].getbbox() else None
        self.recipes[name]['sockets_px']['mouth']=[70,55]
        if name.startswith('give-'):
            points=[(x,y) for y in range(70,105) for x in range(91,127) if skin(base.getpixel((x,y)))]
            if points:
                tip=max(x for x,y in points);palm=[p for p in points if p[0]>=tip-7]
                self.recipes[name]['sockets_px']['left_hand']=[tip-4,round(sum(y for x,y in palm)/len(palm))]
        if name.startswith('walk-'):
            gait=(extra or {})['gait'];hands=gait['hands_measured_screen_rois'];boots=gait['boots_measured_screen_rois']
            near_hand=hands[1] if gait['id']==0 else hands[0];far_hand=hands[0] if gait['id']==0 else hands[1]
            near_boot=boots[0] if gait['id']<3 else boots[1];far_boot=boots[1] if gait['id']<3 else boots[0]
            gait['anatomical_contacts']={'right_hand':near_hand,'left_hand':far_hand,'right_boot':near_boot,'left_boot':far_boot}
            self.recipes[name]['sockets_px'].update(right_hand=near_hand,left_hand=far_hand)

    def expressions(self):
        # Native feature motifs refined from the generated expression sheet, on the locked face.
        eyes={'open':self.parts['eyes']};brows=blank()
        for y in range(42,45):
            for x in range(61,82):
                if y<44 or x>=76:brows.putpixel((x,y),self.parts['eyes'].getpixel((x,y)))
        dark=self.c((65,34,24));white=self.c((255,245,228));lip=self.c((163,70,40))
        for name in ['half','closed','happy-closed','wide']:
            im=brows.copy();d=ImageDraw.Draw(im)
            if name=='half':
                for y in range(49,54):
                    for x in range(61,81):im.putpixel((x,y),self.parts['eyes'].getpixel((x,y)))
                d.line([(62,48),(69,48),(70,49)],fill=dark);d.line([(77,49),(80,49)],fill=dark)
            elif name=='closed':
                d.line([(62,49),(63,50),(68,50),(70,49)],fill=dark);d.line([(77,50),(78,51),(80,50)],fill=dark)
            elif name=='happy-closed':
                d.line([(62,50),(64,48),(68,48),(70,50)],fill=dark);d.line([(77,50),(78,49),(79,49),(80,50)],fill=dark)
            else:
                d.polygon([(62,45),(64,44),(68,44),(70,46),(70,51),(68,53),(63,53),(61,51),(61,47)],fill=white)
                d.line([(61,47),(62,45),(64,44),(68,44),(70,46)],fill=dark)
                d.rectangle((66,46,69,51),fill=dark);d.point((67,46),fill=white)
                d.polygon([(77,46),(79,45),(80,46),(80,52),(77,53),(76,51)],fill=white)
                d.rectangle((78,47,80,51),fill=dark);d.point((78,47),fill=white)
            eyes[name]=save(im,f'eyes/{name}.png')
        mouths={'rest':self.parts['mouth']}
        for name in ['small-open','wide-open','round']:
            im=blank();d=ImageDraw.Draw(im)
            if name=='small-open':
                d.polygon([(68,54),(72,54),(73,55),(72,56),(69,56)],fill=dark);d.line([(70,56),(71,56)],fill=lip)
            elif name=='wide-open':
                d.polygon([(67,54),(73,54),(74,55),(73,57),(69,57),(67,56)],fill=dark)
                d.line([(68,55),(72,55)],fill=white);d.line([(70,57),(72,57)],fill=lip)
            else:
                d.polygon([(70,54),(72,54),(73,55),(73,57),(71,57),(69,56),(69,55)],fill=dark);d.point((71,57),fill=lip)
            mouths[name]=save(im,f'mouth/{name}.png')
        recipes={'neutral':('open','rest'),'talk-small':('open','small-open'),'talk-wide':('open','wide-open'),'talk-round':('open','round'),'blink':('closed','rest'),'happy':('happy-closed','wide-open'),'surprised':('wide','round'),'relaxed':('half','rest')}
        for n,(e,m) in recipes.items():
            self.add(n,{**self.parts,'eyes':eyes[e],'mouth':mouths[m]},hands=self.hands)
            save(self.frames[n]['base'],f'expressions/base/{n}.png');save(self.frames[n]['full'],f'expressions/{n}.png')
        self.eye_bank=eyes;self.mouth_bank=mouths
        write('expressions/recipes.json',{'canvas':[128,160],'eyes':{n:('eyes/animation-open.png' if n=='open' else f'eyes/{n}.png') for n in eyes},'mouth':{n:f'mouth/{n}.png' for n in mouths},'recipes':{n:{'eyes':e,'mouth':m} for n,(e,m) in recipes.items()},'channels':{'eyes':'independent blink; surprised locks wide','mouth':'independent speech'},'source':'source/animation/expressions-candidate.png','refinement':'native feature-only motifs registered on approved face; no skin patches'})
        for e in eyes:
            for m in mouths:
                parts={**self.parts,'eyes':eyes[e],'mouth':mouths[m]};im=compose(list(parts.values()))
                save(im,f'previews/combinations/{e}--{m}.png')
    def ownership(self):
        head=compose([self.parts[n] for n in ['head','eyes','mouth']]);motion=head.copy();excluded=[]
        for y in range(60,66):
            for x in range(50,83):
                p=motion.getpixel((x,y))
                if p[3] and ((skin(p) and not (y==60 and x>=68)) or (y>=63 and x>=60)):
                    motion.putpixel((x,y),(0,0,0,0));excluded.append([x,y])
        neck_mask=region(Image.new('RGBA',SIZE,(255,255,255,255)),[(60,60),(67,61),(71,65),(59,65)])
        for y in range(60,66):
            for x in range(50,83):
                p=motion.getpixel((x,y));cream=p[3] and p[0]>145 and p[1]>110 and p[2]>90 and not skin(p)
                if p[3] and (neck_mask.getpixel((x,y))[3] or cream):
                    motion.putpixel((x,y),(0,0,0,0));excluded.append([x,y])
        torso=self.neutral.copy()
        for y in range(160):
            for x in range(128):
                if motion.getpixel((x,y))[3]:torso.putpixel((x,y),(0,0,0,0))
        save(motion,'source/animation/motion-head.png');save(torso,'source/animation/fixed-body.png')
        write('source/animation/head-ownership.json',{'neck_pixels_excluded_from_motion_head':excluded,'neutral_unchanged':True})
        self.motion_head=motion;self.fixed_body=torso
        self.motion_head_base=self.parts['head'].copy()
        for point in excluded:self.motion_head_base.putpixel(tuple(point),(0,0,0,0))
        save(self.motion_head_base,'source/animation/motion-head-base.png')
    def split_face(self,head,boxes,mouth_box):
        h=head.copy();eyes=blank();mouth=blank()
        for y in range(40,61):
            for x in range(54,85):
                p=head.getpixel((x,y));r,g,b,a=p;eye=any(l<=x<rr and t<=y<bb for l,t,rr,bb in boxes);l,t,rr,bb=mouth_box
                feature=eye and (max(r,g,b)<190 or (r>215 and g>=185 and b>=120)) or l<=x<rr and t<=y<bb and r<205 and r>g*1.3
                if a and feature:
                    (eyes if eye else mouth).putpixel((x,y),p);h.putpixel((x,y),self.c((250,178,118)))
        return h,eyes,mouth
    def head_poses(self):
        self.ownership()
        # Separate neck ownership prevents collar movement and old-jaw remnants.
        for name,k,shift in [('nod-down',0,(0,1)),('turn-left',1,(-1,0)),('turn-right',2,(1,0))]:
            source=move(self.sheet('head-poses',k,3,1,(48,45),paste=(42,21)),0,2)
            head=source.copy();support=blank()
            for y in range(160):
                for x in range(128):
                    p=head.getpixel((x,y))
                    # Keep the complete generated face/hair and only neck skin below the jaw.
                    limit=61
                    if y>=limit:
                        if skin(p) and 60<=x<=73 and y<65:support.putpixel((x,y),p)
                        head.putpixel((x,y),(0,0,0,0))
            body=self.fixed_body.copy();body.alpha_composite(support)
            lower=region(body,[(0,97),(127,97),(127,159),(0,159)]);upper=body.copy()
            for y in range(97,160):
                for x in range(128):
                    if not (x>=82 and y<111):upper.putpixel((x,y),(0,0,0,0))
            boxes=[(59,48,70,55),(73,48,81,56),(60,45,70,48),(74,45,81,48)] if k==0 else [(60,44,71,53),(74,44,83,54),(61,42,71,45),(74,42,83,45)]
            mouth_box=(66,55,75,60) if k==0 else (67,53,77,57)
            h,e,m=self.split_face(head,boxes,mouth_box)
            gear=self.gear.copy();gear['fur-hat']=move(gear['fur-hat'],*shift)
            self.add(name,{'lower':lower,'upper':upper,'head':h,'eyes':e,'mouth':m},gear,self.hands,{'source':f'head-poses-{k}','head_only':True,'fixed_clothing':True,'neck_support':'generated skin only; original collar fixed'})
        head=move(self.motion_head,0,1);h=move(self.motion_head_base,0,1);e=move(self.parts['eyes'],0,1);m=move(self.parts['mouth'],0,1)
        body=self.fixed_body;lower=self.parts['lower'];upper=body.copy()
        for y in range(97,160):
            for x in range(128):
                if not (x>=82 and y<111):upper.putpixel((x,y),(0,0,0,0))
        gear=self.gear.copy();gear['fur-hat']=move(gear['fur-hat'],0,1)
        self.add('nod-mid',{'lower':lower,'upper':upper,'head':h,'eyes':e,'mouth':m},gear,self.hands,{'head_translation':[0,1],'source':'locked neutral preparation','fixed_clothing':True})
    def front_hands(self,body):
        out=blank()
        for y in range(73,115):
            for x in range(20,121):
                p=body.getpixel((x,y));near=any(skin(body.getpixel((x+dx,y+dy))) for dx,dy in [(0,0),(1,0),(-1,0),(0,1),(0,-1)])
                if skin(p) or (p[3] and max(p[:3])<160 and near):out.putpixel((x,y),p)
        return out
    def give(self):
        for k,name in enumerate(['give-mid','give-full']):
            candidate=self.sheet('give',k,2,1,(128,160),scale=.85,offset=(12,13))
            xs=[x for y in range(20,52) for x in range(128) if candidate.getpixel((x,y))[3]]
            candidate=move(candidate,66-round((min(xs)+max(xs))/2),0)
            upper=self.parts['upper'].copy()
            arm_mask=[(78,72),(86,72),(95,76),(123,70),(124,99),(92,103),(81,112),(78,105)]
            masked=region(candidate,arm_mask)
            for y in range(72,112):
                for x in range(79,128):upper.putpixel((x,y),masked.getpixel((x,y)))
            parts={**self.parts,'upper':upper};gear=self.gear.copy()
            # Far glove follows the open palm; keep the hip glove in its original place.
            near=region(gear['gloves'],[(0,0),(65,0),(65,159),(0,159)])
            gear['gloves']=near
            # A fingerless glove uses the generated palm with the cuff behind it.
            body=compose(list(parts.values()));front=self.front_hands(body);front.alpha_composite(self.hands)
            points=[(x,y) for y in range(70,104) for x in range(91,127) if skin(body.getpixel((x,y)))]
            if points:
                tip=max(x for x,y in points);palm=[p for p in points if p[0]>=tip-7];wrist_y=round(sum(y for x,y in palm)/len(palm))+1
                cuff=self.gear['gloves'].crop((81,94,90,102)).rotate(90,expand=True,resample=Image.Resampling.NEAREST)
                worn=blank();worn.paste(cuff,(tip-14,wrist_y-4))
                for yy in range(160):
                    for xx in range(128):
                        if not body.getpixel((xx,yy))[3] or xx>tip-8:worn.putpixel((xx,yy),(0,0,0,0))
                gear['gloves']=compose([near,worn])
            self.add(name,parts,gear,front,{'giving_hand':'left','palm':'open; no held prop','source':f'give-{k}'})
        self.add('give-empty',self.frames['give-full']['parts'],self.frames['give-full']['gear'],self.frames['give-full']['parts']['hands-front'],{'transfer_event_only':True})
        self.add('give-retract',self.frames['give-mid']['parts'],self.frames['give-mid']['gear'],self.frames['give-mid']['parts']['hands-front'])
    def walk(self):
        phases=json.loads((ROOT/'source/animation/production-contract.json').read_text())['walk']['keys'];self.gait=[]
        contacts=[self.sheet('walk-contacts',k,2,1,(128,160),scale=.85,offset=(10,13)) for k in range(2)]
        for k,phase in enumerate(phases):
            body=self.sheet('walk',k,3,2,(128,160),scale=.80,offset=(7,20))
            if k in (0,3):body=contacts[k//3].copy()
            xs=[x for y in range(20,53) for x in range(128) if body.getpixel((x,y))[3]]
            registration_dx=66-round((min(xs)+max(xs))/2)
            body=move(body,registration_dx,0)
            # Fix a single shared ground registration, retaining each leg's authored pose.
            bbox=body.getbbox();dy=140-bbox[3];body=move(body,0,dy)
            bob=phase['bob']
            if k not in (0,3):
                torso=blank();legs=blank()
                for yy in range(160):
                    for xx in range(128):
                        q=body.getpixel((xx,yy));near_skin=any(skin(body.getpixel((max(0,min(127,xx+dx)),max(0,min(159,yy+ddy))))) for dx,ddy in [(0,0),(1,0),(-1,0),(0,1),(0,-1)])
                        arm=yy<111 and (xx<55 or xx>78) and q[3] and (skin(q) or any(skin(body.getpixel((max(0,min(127,xx+dx)),max(0,min(159,yy+ddy))))) for dx in range(-2,3) for ddy in range(-2,3)))
                        if yy>=94 and not arm:legs.putpixel((xx,yy),q)
                        if yy<97 or arm:torso.putpixel((xx,yy),q)
                body=compose([legs,move(torso,0,-3)])
            source_body=body.copy()
            for y in range(67):
                for x in range(128):
                    p=source_body.getpixel((x,y))
                    def cream(q):return q[3] and q[0]>145 and q[1]>110 and q[2]>70 and not skin(q)
                    border=p[3] and max(p[:3])<160 and any(cream(source_body.getpixel((xx,yy))) for xx,yy in [(max(0,x-1),y),(min(127,x+1),y),(x,max(0,y-1)),(x,min(159,y+1))])
                    neck=skin(p) and 60<=x<=72 and y>=60
                    if y<58 or not (cream(p) or border or neck):body.putpixel((x,y),(0,0,0,0))
            head_dy=bob+(0 if k in (0,3) else 1)
            collar=move(region(self.fixed_body,[(57,60),(76,60),(82,68),(82,77),(56,77),(53,68)]),0,head_dy)
            collar_mask=move(region(Image.new('RGBA',SIZE,(255,255,255,255)),[(57,60),(76,60),(82,68),(82,77),(56,77),(53,68)]),0,head_dy)
            for yy in range(58,80):
                for xx in range(51,85):
                    if collar_mask.getpixel((xx,yy))[3] and collar.getpixel((xx,yy))[3]:body.putpixel((xx,yy),collar.getpixel((xx,yy)))
            head=move(self.motion_head,0,head_dy);h=move(self.motion_head_base,0,head_dy);e=move(self.parts['eyes'],0,head_dy);m=move(self.parts['mouth'],0,head_dy)
            upper=body.copy();lower=body.copy()
            for y in range(160):
                for x in range(128):(lower if y<97 else upper).putpixel((x,y),(0,0,0,0))
            parts={'lower':lower,'upper':upper,'head':h,'eyes':e,'mouth':m}
            gear={n:move(im,0,bob) for n,im in self.gear.items() if n not in ['gloves','boot-fur-cuffs']}
            # Gloves and boot cuffs are pose variants: place their original material at measured limb ends.
            boots=[]
            for left,right in [(25,64),(64,108)]:
                points=[(x,y) for y in range(112,141) for x in range(left,right) if (p:=body.getpixel((x,y)))[3] and p[0]>p[1]*1.3 and p[1]<145]
                boots.append([round(sum(x for x,y in points)/len(points)),min(y for x,y in points)+3] if points else [left+15,120])
            cuffs=blank()
            for i,(cx,cy) in enumerate(boots):
                patch=region(self.gear['boot-fur-cuffs'],[(35 if i==0 else 65,112),(65 if i==0 else 90,112),(65 if i==0 else 90,126),(35 if i==0 else 65,126)])
                cuffs.alpha_composite(move(patch,cx-([53,77][i]),cy-120))
            for yy in range(160):
                for xx in range(128):
                    if not body.getpixel((xx,yy))[3]:cuffs.putpixel((xx,yy),(0,0,0,0))
            gear['boot-fur-cuffs']=cuffs
            front=self.front_hands(body);hands=[]
            for left,right in [(20,64),(64,116)]:
                points=[(x,y) for y in range(81,113) for x in range(left,right) if skin(body.getpixel((x,y)))]
                if points:
                    end=max(y for x,y in points);p=[q for q in points if q[1]>=end-3];hands.append([round(sum(x for x,y in p)/len(p)),round(sum(y for x,y in p)/len(p))])
                else:hands.append([left+12,95])
            # Keep finger silhouettes readable; glove material covers only wrist/palm, never the fingertips.
            gloves=blank()
            for i,(cx,cy) in enumerate(hands):
                patch=region(self.gear['gloves'],[(0 if i==0 else 70,80),(65 if i==0 else 100,80),(65 if i==0 else 100,113),(0 if i==0 else 70,113)])
                patch=move(patch,cx-([52,85][i]),cy-([92,104][i]))
                for y in range(160):
                    for x in range(128):
                        if not body.getpixel((x,y))[3] or y>cy:patch.putpixel((x,y),(0,0,0,0))
                gloves.alpha_composite(patch)
            gear['gloves']=gloves
            entry={**phase,'source_ground_translation_y':dy,'source_alignment_dx':registration_dx,'hands_measured_screen_rois':hands,'boots_measured_screen_rois':boots,'contact_source':k in (0,3),'source_head_discarded':True}
            self.add(f'walk-{k}',parts,gear,front,{'gait':entry,'head_translation':[0,head_dy],'source_head':'locked neutral; no rescaling'})
            self.gait.append(entry)
        write('source/animation/walk-phases.json',self.gait)
    def clips(self):
        def clip(ids,times,loop=False,events=None):return {'frames':ids,'durations_ms':times,'loop':loop,'events':events or []}
        return {'idle':clip(['neutral','relaxed','blink','relaxed','neutral'],[1400,40,80,40,1600],True),
          'blink':clip(['neutral','relaxed','blink','relaxed','neutral'],[120,40,80,40,120]),
          'talk':clip(['neutral','talk-small','talk-wide','talk-small','talk-round','neutral'],[120,100,120,100,100,140],True),
          'happy':clip(['neutral','happy','neutral'],[160,700,160]),'surprised':clip(['neutral','surprised','neutral'],[160,600,180]),
          'relaxed':clip(['neutral','relaxed','neutral'],[160,600,180]),
          'nod':clip(['neutral','nod-mid','nod-down','nod-mid','neutral'],[120,100,140,100,120]),
          'shake_head':clip(['neutral','turn-left','neutral','turn-right','neutral'],[100,140,100,140,100]),
          'give':clip(['neutral','give-mid','give-full','give-empty','give-retract','neutral'],[120,120,180,120,120,120],events=[{'key':3,'name':'transfer','hand':'left','item':None,'fires':'once_on_entry'}]),
          'walk':clip([f'walk-{k}' for k in range(6)],[120]*6,True)}
    def contact(self,items,path,cols=4,scale=3,bg='#eee3cc',crop=None):
        box=crop or (0,0,128,160);w=(box[2]-box[0])*scale;h=(box[3]-box[1])*scale
        out=Image.new('RGB',((w+16)*cols,(h+36)*math.ceil(len(items)/cols)),bg);d=ImageDraw.Draw(out)
        for i,(name,im) in enumerate(items):
            x=i%cols*(w+16);y=i//cols*(h+36);d.text((x+8,y+8),name,fill='#eee3cc' if bg=='#302b32' else '#211510');a=im.crop(box).resize((w,h),Image.Resampling.NEAREST);out.paste(a,(x+8,y+28),a)
        save(out,path)
    def export(self):
        clips=self.clips();names=list(self.frames);animation={'version':2,'id':'hunter','canvas':[128,160],'foot_anchor_px':[64,140],'frames':names,'clips':clips,'recipes':'poses/recipes.json','channels':'expressions/recipes.json','scope':'authoring only; not game integration'}
        write('animation.json',animation);write('poses/recipes.json',{'canvas':[128,160],'foot_anchor_px':[64,140],'poses':self.recipes});write('source/animation/registration.json',self.registration)
        cols=8;rows=math.ceil(len(names)/cols);atlas=Image.new('RGBA',(1056,math.ceil(rows*164/16)*16));sprites=[]
        for i,n in enumerate(names):
            x=i%cols*132+2;y=i//cols*164+2;im=self.frames[n]['full'];atlas.paste(im,(x,y))
            for yy in range(-1,161):
                for xx in range(-1,129):
                    if xx in (-1,128) or yy in (-1,160):atlas.putpixel((x+xx,y+yy),im.getpixel((max(0,min(127,xx)),max(0,min(159,yy)))))
            sprites.append({'id':n,'page':'hunter-0','rect':[x,y,128,160],'source_size':[128,160],'trim_offset':[0,0],'pivot':[64,140],'source':f'frames/{n}.png','rotated':False})
        save(atlas,'atlas.png');write('atlas-manifest.json',{'version':2,'sampler':'nearest','mipmaps':False,'extrude_px':1,'gutter_px':1,'pages':[{'id':'hunter-0','path':'atlas.png','size':list(atlas.size)}],'sprites':sprites,'clips':clips,'usage':'authoring_not_integrated'})
        expression_names=list(json.loads((ROOT/'expressions/recipes.json').read_text())['recipes'])
        for v in ['base','full']:
            self.contact([(n,self.frames[n][v]) for n in expression_names],f'previews/expressions-8-{v}.png',4,4)
            self.contact([(n,self.frames[n][v]) for n in ['neutral','nod-mid','nod-down','turn-left','turn-right']],f'previews/head-poses-{v}.png',5,4)
            self.contact([(f'walk-{k}',self.frames[f'walk-{k}'][v]) for k in range(6)],f'previews/walk-6-{v}.png',6,4)
            self.contact([(n,self.frames[n][v]) for n in clips['give']['frames']],f'previews/give-6-{v}.png',6,4)
        self.contact([(n,self.frames[n]['base']) for n in expression_names],'previews/faces-8.png',4,8,crop=(57,39,84,62))
        self.contact([(p.stem,Image.open(p).convert('RGBA')) for p in (ROOT/'previews/combinations').glob('*.png')],'previews/expression-combinations-20.png',4,4,crop=(57,39,84,62))
        for name,clip in clips.items():
            for slow in [1,4]:
                ims=[]
                for n in clip['frames']:
                    canvas=Image.new('RGB',(1072,680),'#eee3cc');d=ImageDraw.Draw(canvas)
                    for j,v in enumerate(['base','full']):
                        d.text((12+j*536,8),f'{name} / {n} / {v}',fill='#211510');a=self.frames[n][v].resize((512,640),Image.Resampling.NEAREST);canvas.paste(a,(12+j*536,28),a)
                    ims.append(canvas.quantize(colors=256,dither=Image.Dither.NONE))
                out=io.BytesIO();header,_=GifImagePlugin.getheader(ims[0],info={'loop':0})
                for block in header:out.write(block)
                for im,ms in zip(ims,clip['durations_ms']):
                    for block in GifImagePlugin.getdata(im,include_color_table=True,duration=ms*slow,disposal=2):out.write(block)
                out.write(b';');p=ROOT/f'previews/{name}{"-slow" if slow==4 else ""}.gif';p.write_bytes(out.getvalue())
        spec=yaml.safe_load((ROOT/'character-spec.yaml').read_text());spec.update(action_set='human_common',expression_recipes=json.loads((ROOT/'expressions/recipes.json').read_text())['recipes'],animation=animation,animation_builder='source/build-animation.py',production_plan={'status':'generated','completed':['neutral','8 expressions','nod','shake_head','give','walk6'],'not_generated':[]})
        spec['generation']['animation_prompts']='source/animation/generation-prompts.json';spec['generation']['animation_registration']='source/animation/registration.json'
        spec['expression_channels']={'eyes':'independent blink; wide locked during surprise','mouth':'independent speech; no jaw/beard replacement'}
        spec['layers']['eyes']={n:('eyes/animation-open.png' if n=='open' else f'eyes/{n}.png') for n in self.eye_bank};spec['layers']['mouth']={n:f'mouth/{n}.png' for n in self.mouth_bank}
        (ROOT/'character-spec.yaml').write_text(yaml.safe_dump(spec,sort_keys=False,allow_unicode=True));self.html(animation)
        (ROOT/'ANIMATION.md').write_text('# Hunter animation\n\n128×160; foot pivot (64,140). Eight expressions, independent eye/mouth channels, nod, shake_head, open-palm give gesture and six-key in-place walk. Near anatomical side is right, far side is left. Give has one transfer cue without a baked item. All approved neutral PNGs remain unchanged.\n\nBuild: `rtk proxy python3 assets/npc/hunter/source/build-animation.py`. Review: `previews/animation.html`. Sources, exact prompts and registration are retained in `source/animation/`. Authoring assets only, no game runtime integration.\n')
        index=ROOT/'previews/index.html'
        text=index.read_text()
        if 'href="animation.html"' not in text:index.write_text(text.replace('</h1>','</h1><p><a href="animation.html">Expressions and motions</a></p>',1))
        self.contact([(n,self.frames[n]['parts'][part]) for n in ['neutral','nod-mid','nod-down','turn-left','turn-right'] for part in ['head','upper']], 'previews/head-body-ownership.png',5,5,crop=(45,38,88,80))
        print(f'Built {len(names)} frames, {len(clips)} clips')
    def html(self,animation):
        def uri(im):
            out=io.BytesIO();im.save(out,format='PNG');return 'data:image/png;base64,'+base64.b64encode(out.getvalue()).decode()
        images={n:{v:uri(self.frames[n][v]) for v in ['base','full']} for n in self.frames}
        for n,frame in self.frames.items():
            layers=frame['parts']|frame['gear'];images[n]['nohat']=uri(compose([layers[k] for k in self.order if k!='fur-hat' and k in layers]))
        expression_recipes=json.loads((ROOT/'expressions/recipes.json').read_text())['recipes']
        for e,eye in self.eye_bank.items():
            for m,mouth in self.mouth_bank.items():
                parts={**self.parts,'eyes':eye,'mouth':mouth,'hands-front':self.hands}
                base=compose([parts[n] for n in ['lower','upper','head','eyes','mouth']])
                full=compose([({**parts,**self.gear})[n] for n in self.order])
                images[e+'--'+m]={'base':uri(base),'full':uri(full),'nohat':uri(compose([({**parts,**self.gear})[n] for n in self.order if n!='fur-hat']))}
        (ROOT/'previews/animation.html').write_text('''<!doctype html><meta charset="utf-8"><title>Hunter animation review</title><style>body{font:15px system-ui;background:#eee3cc;color:#30231e;margin:20px}nav{display:flex;gap:12px;flex-wrap:wrap;margin:14px 0}section{display:flex;gap:24px;overflow:auto}canvas{display:block;width:512px;height:640px;image-rendering:pixelated}body.dark{background:#302b32;color:#eee3cc}button,select{font:inherit}</style><h1>Hunter · expressions and motions</h1><nav><label>Clip <select id="clip"></select></label><button id="play">Pause</button><button id="restart">Restart</button><button id="next">Next key</button><label>Key <input id="key" type="range" min="0" value="0"></label></nav><nav><label>Speed <select id="speed"><option value="1">Normal</option><option value="0.25">Slow ¼</option></select></label><label>Scale <select id="scale"><option value="4">4× nearest</option><option value="1">1× native</option></select></label><button id="bg">Light / dark</button><label><input type="checkbox" id="hat" checked>Hat on equipped preview</label><label><input type="checkbox" id="blink" checked>Independent blink during idle/talk</label></nav><nav><label><input id="face" type="checkbox">Face combinations</label><label>Eyes <select id="eye"><option>open</option><option>half</option><option>closed</option><option>happy-closed</option><option>wide</option></select></label><label>Mouth <select id="mouth"><option>rest</option><option>small-open</option><option>wide-open</option><option>round</option></select></label></nav><section><div>Complete base<canvas id="base" width="128" height="160"></canvas></div><div>Equipped<canvas id="full" width="128" height="160"></canvas></div></section><p id="status"></p><p>128×160 · pivot (64,140) · 64-color shared palette · authoring only.</p><script>const animation='''+json.dumps(animation)+''';const urls='''+json.dumps(images)+''';const expressionRecipes='''+json.dumps(expression_recipes)+''';const bank={};let playing=true,key=0,elapsed=0,last=performance.now(),transfer=0,total=0;for(const[n,variants]of Object.entries(urls)){bank[n]={};for(const[v,url]of Object.entries(variants)){const im=new Image();im.src=url;bank[n][v]=im}}const $=id=>document.getElementById(id);for(const n of Object.keys(animation.clips))$('clip').add(new Option(n,n));function reset(){key=0;elapsed=0;total=0;transfer=0;$('key').max=animation.clips[$('clip').value].frames.length-1;draw()}function draw(){const c=animation.clips[$('clip').value];let name=c.frames[key];if($('blink').checked&&['idle','talk'].includes($('clip').value)&&total%3400>3080&&total%3400<3240)name=(total%3400<3120||total%3400>3200?'half':'closed')+'--'+(expressionRecipes[name]?.mouth||'rest');if($('face').checked)name=$('eye').value+'--'+$('mouth').value;for(const v of ['base','full']){const ctx=$(v).getContext('2d');ctx.clearRect(0,0,128,160);ctx.imageSmoothingEnabled=false;const variant=v==='full'&&!$('hat').checked?'nohat':v;if(bank[name][variant].complete)ctx.drawImage(bank[name][variant],0,0)}$('key').value=key;$('status').textContent=$('clip').value+' · key '+(key+1)+'/'+c.frames.length+' · '+name+' · '+c.durations_ms[key]+' ms · transfer cues: '+transfer+(playing?' · playing':' · paused')}function enter(next,events){key=next;const c=animation.clips[$('clip').value];if(events)for(const e of c.events)if(e.key===key&&e.name==='transfer')transfer++}function tick(now){let dt=Math.min(100,now-last)*Number($('speed').value);last=now;if(playing){elapsed+=dt;total+=dt;const c=animation.clips[$('clip').value];while(elapsed>=c.durations_ms[key]){elapsed-=c.durations_ms[key];if(key===c.frames.length-1){if(c.loop)enter(0,true);else{playing=false;$('play').textContent='Play';break}}else enter(key+1,true)}}draw();requestAnimationFrame(tick)}$('clip').onchange=()=>{playing=true;$('play').textContent='Pause';reset()};$('play').onclick=()=>{playing=!playing;$('play').textContent=playing?'Pause':'Play'};$('restart').onclick=()=>{playing=true;$('play').textContent='Pause';reset()};$('next').onclick=()=>{playing=false;$('play').textContent='Play';enter((key+1)%animation.clips[$('clip').value].frames.length,false);elapsed=0;draw()};$('key').oninput=e=>{playing=false;$('play').textContent='Play';enter(Number(e.target.value),false);elapsed=0;draw()};$('bg').onclick=()=>document.body.classList.toggle('dark');$('scale').onchange=e=>{for(const c of document.querySelectorAll('canvas')){c.style.width=128*e.target.value+'px';c.style.height=160*e.target.value+'px'}};const params=new URLSearchParams(location.search);if(animation.clips[params.get('clip')])$('clip').value=params.get('clip');if(params.get('speed'))$('speed').value=params.get('speed');reset();if(params.has('key')){key=Math.min(Number(params.get('key')),animation.clips[$('clip').value].frames.length-1);playing=false;$('play').textContent='Play'}if(params.get('dark')==='1')document.body.classList.add('dark');if(params.get('hat')==='0')$('hat').checked=false;if(params.get('scale')==='1'){$('scale').value='1';$('scale').onchange({target:$('scale')})}if(params.get('face')==='1'){$('face').checked=true;if([...$('eye').options].some(o=>o.value===params.get('eyes')))$('eye').value=params.get('eyes');if([...$('mouth').options].some(o=>o.value===params.get('mouth')))$('mouth').value=params.get('mouth')}Promise.all(Object.values(bank).flatMap(v=>Object.values(v)).map(i=>i.decode())).then(()=>requestAnimationFrame(tick));</script>''')

if __name__=='__main__':
    b=Builder();b.expressions();b.head_poses();b.give();b.walk();b.export()
