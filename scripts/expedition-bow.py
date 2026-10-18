"""Registered arm studies from the supplied motion reference; canonical body is read-only."""
from PIL import Image, ImageDraw
import json, math

# Source rectangles share one scale (4 reference pixels per native pixel).
# Masks own sleeves, forearms and gloves only, never the reference head or outfit.
STUDIES = [
    ((0, 0), (0, 12),
     [(187,252),(208,247),(230,267),(250,271),(270,274),(307,281),(323,279),(329,294),(322,309),(310,315),(274,313),(240,303),(214,302),(197,291),(189,279)],
     [(293,253),(311,255),(321,276),(346,283),(373,288),(382,296),(378,315),(366,325),(345,320),(324,311),(312,296),(301,285)]),
    ((512, 0), (0, 14),
     [(198,239),(219,230),(233,234),(238,243),(257,242),(285,242),(315,240),(327,242),(329,261),(318,272),(286,277),(258,277),(234,272),(220,274),(205,266),(198,257)],
     [(326,237),(344,239),(357,250),(376,259),(401,258),(414,262),(415,279),(406,291),(390,296),(370,290),(350,287),(330,280),(321,268)]),
    ((1024, 0), (8, 16),
     [(164,218),(179,214),(190,219),(196,223),(215,223),(243,218),(266,217),(278,222),(278,241),(269,250),(244,256),(217,260),(189,260),(175,263),(162,254),(158,239)],
     [(324,239),(336,232),(349,236),(355,244),(377,245),(400,237),(419,238),(430,245),(430,261),(419,272),(398,274),(375,277),(354,275),(338,278),(324,268)]),
    ((0, 512), (8, 20),
     [(112,171),(125,166),(143,169),(159,164),(183,164),(209,166),(217,171),(217,188),(206,196),(185,199),(161,200),(144,205),(131,210),(119,208),(109,201),(104,186)],
     [(311,191),(324,184),(339,190),(345,195),(364,193),(392,188),(414,188),(433,193),(437,203),(433,216),(419,226),(395,228),(370,229),(345,234),(328,234),(312,224),(306,209)]),
    ((512, 512), (8, 20),
     [(116,181),(130,175),(145,178),(157,187),(164,203),(169,219),(167,231),(157,240),(143,243),(128,236),(114,224),(108,207),(108,193)],
     [(320,194),(332,187),(346,190),(356,196),(378,191),(405,189),(422,190),(434,198),(433,215),(420,225),(396,229),(370,231),(349,235),(332,234),(321,226)]),
    ((1024, 512), (0, 12),
     [(194,236),(211,231),(226,238),(233,253),(245,267),(263,274),(285,281),(301,288),(309,300),(302,312),(289,317),(272,311),(251,300),(231,291),(218,283),(204,273),(195,257)],
     [(324,263),(337,259),(350,268),(361,281),(379,287),(393,287),(402,294),(400,309),(390,321),(375,322),(359,312),(345,306),(332,291),(321,278)]),
]


def build(root, pose, palette, save):
    source = Image.open(root / 'source/bow-motion-reference.png').convert('RGB')
    colors = list(dict.fromkeys(palette.getdata()))
    def closest(p):
        return min(colors, key=lambda c: sum((a-b)**2 for a,b in zip(c,p)))
    def part(origin, offset, poly):
        crop = source.crop((origin[0], origin[1], origin[0]+512, origin[1]+512))
        mask = Image.new('L',(512,512)); ImageDraw.Draw(mask).polygon(poly,fill=255)
        crop = crop.resize((128,128), Image.Resampling.NEAREST)
        mask = mask.resize((128,128), Image.Resampling.NEAREST)
        rgba=Image.new('RGBA',(128,128))
        rgba.putdata([closest(p)+(255,) if a else (0,0,0,0) for p,a in zip(crop.getdata(),mask.getdata())])
        out=Image.new('RGBA',(128,160));out.alpha_composite(rgba,offset);return out
    # Bow grip stays at the far hand; the nock follows the drawing hand until release.
    keys=[
        ('bow-ready',(91,88),(78,84)),
        ('bow-draw',(102,82),(80,78)),
        ('bow-pull',(113,79),(76,74)),
        ('bow-aim',(114,72),(60,65)),
        ('bow-release',(114,72),(43,70)),
        ('bow-recovery',(98,89),(73,88)),
    ]
    dark=closest((44,29,22))+(255,); wood=closest((113,72,40))+(255,)
    light=closest((166,115,65))+(255,); string=closest((208,190,145))+(255,)
    steel=closest((213,214,184))+(255,)
    cloth=Image.open(root.parent/'npc/hunter/poses/neutral/layers/upper.png').convert('RGBA')
    def sleeve(start,end):
        # Sample the approved sleeve's fold texture into a small joint bridge.
        out=Image.new('RGBA',(128,160)); mask=Image.new('L',out.size);d=ImageDraw.Draw(mask)
        dx,dy=end[0]-start[0],end[1]-start[1];length=max(1,math.hypot(dx,dy));nx,ny=-dy/length,dx/length
        d.polygon([(round(x+side*nx*4),round(y+side*ny*4)) for (x,y),side in [(start,1),(end,1),(end,-1),(start,-1)]],fill=255)
        edge=ImageDraw.Draw(out)
        edge.polygon([(round(x+side*nx*5),round(y+side*ny*5)) for (x,y),side in [(start,1),(end,1),(end,-1),(start,-1)]],fill=dark)
        for y in range(160):
            for x in range(128):
                if mask.getpixel((x,y)):
                    u=((x-start[0])*dx+(y-start[1])*dy)/(length*length)
                    v=(x-start[0])*nx+(y-start[1])*ny
                    p=cloth.getpixel((42+min(7,max(0,round(u*7))),78+min(6,max(0,round(v+3)))))
                    out.putpixel((x,y),p if p[3] else closest((189,169,131))+(255,))
        return out
    frames={}; sockets={}
    elbows=[(78,80),(84,79),(92,80),(88,74),(90,74),(83,82)]
    for index,((name,grip,nock),(origin,offset,nearpoly,farpoly)) in enumerate(zip(keys,STUDIES)):
        base=pose('neutral',combat=True)
        if index==0:save(base,'hunter/layers/bow-body-underlayer.png')
        near=part(origin,offset,nearpoly);far=part(origin,offset,farpoly)
        bridge=sleeve((78,78),elbows[index]);bridge.alpha_composite(far);far=bridge
        if index==3:
            bridge=sleeve((50,77),(39,68));bridge.alpha_composite(near);near=bridge
        weapon=Image.new('RGBA',(128,160));d=ImageDraw.Draw(weapon)
        gx,gy=grip; stretched=name in ('bow-pull','bow-aim')
        top=(gx-9 if stretched else gx-8,gy-38)
        bottom=(gx-10,gy+40)
        curve=[top,(gx-10,gy-30),(gx-5,gy-19),(gx-2,gy-9),grip,(gx-2,gy+12),(gx-7,gy+26),bottom]
        d.line(curve,fill=dark,width=3);d.line(curve,fill=wood,width=1)
        d.line([(x-1,y) for x,y in curve[1:-1]],fill=light,width=1)
        anchor=nock if name not in ('bow-release','bow-recovery','bow-ready') else (gx-9,gy)
        d.line([top,anchor,bottom],fill=string,width=1)
        if name not in ('bow-release','bow-recovery','bow-ready'):
            end=(min(125,nock[0]+64),nock[1])
            d.line([nock,end],fill=light,width=1)
            d.polygon([(end[0]-1,end[1]-2),(end[0]+2,end[1]),(end[0]-1,end[1]+2)],fill=steel)
        d.line([(gx-1,gy-4),(gx-1,gy+4)],fill=dark,width=3)
        base.alpha_composite(far);base.alpha_composite(weapon);base.alpha_composite(near)
        # Face/hat are the canonical foreground, including when the string crosses their plane.
        neutral=pose('neutral')
        for y in range(16,65):
            for x in range(36,89):
                if neutral.getpixel((x,y))[3]:base.putpixel((x,y),neutral.getpixel((x,y)))
        frames[name]=base
        for suffix,layer in [('near-arm',near),('far-arm',far),('bow',weapon)]:
            save(layer,f'hunter/layers/{name}-{suffix}.png')
        sockets[name]={'bow_grip':grip,'drawing_hand':nock,'source_origin':origin,'registration_offset':offset}
    (root/'hunter/bow-sockets.json').write_text(json.dumps(sockets,indent=2)+'\n')
    contact=Image.new('RGBA',(128*6,320))
    playback=[]
    for index,frame in enumerate(frames.values()):
        panel=Image.new('RGBA',(256,160))
        for column,bg in enumerate([(28,34,40,255),(237,229,210,255)]):
            contact.paste(bg,(index*128,column*160,(index+1)*128,(column+1)*160))
            contact.alpha_composite(frame,(index*128,column*160))
            panel.paste(bg,(column*128,0,(column+1)*128,160));panel.alpha_composite(frame,(column*128,0))
        playback.append(panel.resize((1024,640),Image.Resampling.NEAREST).convert('RGB'))
    save(contact,'hunter/bow-light-dark-native.png')
    save(contact.resize((1536,640),Image.Resampling.NEAREST),'hunter/bow-light-dark-2x.png')
    for label,factor in [('normal',1),('slow',4)]:
        playback[0].save(root/f'hunter/bow-{label}.gif',save_all=True,append_images=playback[1:],duration=[int(t*factor) for t in [120,180,220,480,180,470]],loop=0,disposal=2)
    (root/'hunter/bow-preview.html').write_text('''<!doctype html><html lang="en"><meta charset="utf-8"><title>Hunter bow motion review</title>
<style>body{background:#32383d;color:#fff;font:16px system-ui;margin:24px}img{image-rendering:pixelated;max-width:100%}button{font:inherit;padding:8px;margin:8px}#frames{display:flex;flex-wrap:wrap}#frames img{width:256px}</style>
<h1>Canonical Hunter · bow motion review</h1><p>Motion reference: supplied six-frame sheet. Canonical face, hat, torso, cloak and feet preserved. Candidate, visual approval pending.</p>
<button onclick="document.getElementById('play').src='bow-normal.gif?'+Date.now()">Normal speed</button><button onclick="document.getElementById('play').src='bow-slow.gif?'+Date.now()">Quarter speed</button>
<p>Left: dark background. Right: light background. Native 128 × 160, nearest 4×.</p><img id="play" src="bow-normal.gif" width="1024" height="640">
<h2>All keys, native pixels and 2×</h2><img src="bow-light-dark-native.png"><br><img src="bow-light-dark-2x.png">
</html>''')
    return frames
