#!/usr/bin/env python3
"""Offline composites, explicitly distinct from Bevy acceptance captures."""
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT/'assets/forge/atlas-v2'
OUT = ROOT/'docs/qa/evidence/atlas-v2/composites'
OUT.mkdir(parents=True, exist_ok=True)
fixtures = json.loads((ART/'catalog.json').read_text())
by_id = {f['id']: f for f in fixtures}
background = Image.open(ROOT/'assets/forge/interior/background.png').convert('RGBA')
proof = []
occlusion = []

for sex in ('female', 'male'):
    manifest = json.loads((ART/f'{sex}.json').read_text())
    atlas = Image.open(ART/f'{sex}.png').convert('RGBA')
    canonical = json.loads((ROOT/f'assets/characters/{sex}/animation.json').read_text())
    for pose in ('forging','hammering','quenching','researching','salvaging','walk-front','walk-behind','idle','empty'):
        frames = []
        for frame in range(6):
            result = background.copy()
            work = canonical['workshop']['stations'].get(pose)
            fixture_id = None
            if work:
                fixture_id = 'workbench' if pose in ('researching','salvaging') else work['station_id']
                f = by_id[fixture_id]
                target = f['mode_contacts'].get(pose,f['contact'])
                original = work['contact_points']['character']
                exact = [f['origin'][i]+target[i]-original[i] for i in range(2)]
                origin = [int(v+0.5) for v in exact]
            elif pose.startswith('walk'):
                origin = [420+frame*48-64,(148 if pose.endswith('behind') else 178)-140]
                exact = origin
            else:
                origin = [360-64,166-140]
                exact = origin
            foot = [origin[0]+64, origin[1]+140]
            name = f'{pose}-{frame}' if work else f'walk-{frame}' if pose.startswith('walk') else 'neutral'
            if pose == 'quenching':
                name = f'quenching/scene-{frame}'
            index = manifest['ids'].index(name)
            x,y = index%8*128,index//8*160
            character = atlas.crop((x,y,x+128,y+160))
            if pose == 'researching':
                desk = by_id['workbench']
                mask = Image.open(ART/'workbench.png').getchannel('A')
                uncovered = 0
                for cy in range(120,160):
                    for cx in range(128):
                        if character.getpixel((cx,cy))[3]:
                            dx,dy = origin[0]+cx-desk['origin'][0],origin[1]+cy-desk['origin'][1]
                            if not (0<=dx<mask.width and 0<=dy<mask.height and mask.getpixel((dx,dy))==255):
                                uncovered += 1
                assert uncovered == 0, (sex,frame,uncovered)
                occlusion.append(dict(sex=sex,frame=frame,uncovered_lower_body_pixels=uncovered))
            layers = []
            for f in fixtures:
                for part in ('back','front'):
                    z = f['depth']-0.5
                    if f['id']==fixture_id and pose!='salvaging':  # runtime: salvage uses floor depth
                        z = foot[1]+20 if part=='front' or pose=='researching' else foot[1]-20
                    layers.append((z,Image.open(ART/f"{f['id']}-{part}.png").convert('RGBA'),f['origin']))
            if pose!='empty':
                layers.append((foot[1],character,origin))
            if pose=='researching':
                layers.append((foot[1]+40,Image.open(ART/f'arms/{sex}-equipped-researching-{frame}.png').convert('RGBA'),origin))
            for _,image,position in sorted(layers,key=lambda x:x[0]):
                result.alpha_composite(image,tuple(position))
            frames.append(result)
            proof.append(dict(sex=sex,pose=pose,frame=frame,canvas_origin=origin,foot=foot,
                              exact_origin=exact,contact_rounding=[origin[i]-exact[i] for i in range(2)]))
        frames[2].save(OUT/f'{sex}-{pose}.png')
        frames[2].resize((3840,720),Image.Resampling.NEAREST).save(OUT/f'{sex}-{pose}-4x.png')
        frames[0].save(OUT/f'{sex}-{pose}.gif',save_all=True,append_images=frames[1:],duration=160,loop=0,disposal=2)
(OUT/'contacts.json').write_text(json.dumps(proof,indent=2)+'\n')
(OUT.parent/'research-occlusion.json').write_text(json.dumps(occlusion,indent=2)+'\n')
print('Offline composites and six-frame contact sequences exported for both canonical smiths')
