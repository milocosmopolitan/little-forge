#!/usr/bin/env python3
"""Pack supplied fixtures and whole-canvas 3/4 character derivatives without reauthoring."""
import argparse
import json
import math
import tempfile
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'assets/forge/atlas-v1/source'
OUT = ROOT / 'assets/forge/atlas-v1'
POSITIONS = {'furnace': [384, 32], 'anvil': [544, 96],
             'quenching-tub': [672, 96], 'workbench': [800, 64], 'counter': [160, 96]}
IDS = {'furnace': 'forge', 'counter': 'display'}


def save_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def build(out):
    out.mkdir(parents=True, exist_ok=True)
    spec = json.loads((SOURCE / 'workshop-atlas.json').read_text())
    rows = []
    for name, s in spec['sprites'].items():
        full = Image.open(SOURCE / f'{name}.png').convert('RGBA')
        back = Image.open(SOURCE / f'{name}-back.png').convert('RGBA')
        front = Image.open(SOURCE / f'{name}-front.png').convert('RGBA')
        assert Image.alpha_composite(back, front).tobytes() == full.tobytes(), name
        assert set(full.getchannel('A').getdata()) <= {0, 255}, name
        # Padding protects the external outline at atlas-tile boundaries.
        layers = []
        for im in (full, back, front):
            padded = Image.new('RGBA', (im.width+2, im.height+2))
            padded.alpha_composite(im, (1, 1))
            layers.append(padded)
        full, back, front = layers
        alpha = full.getchannel('A')
        ring = ImageChops.subtract(alpha.filter(ImageFilter.MaxFilter(3)), alpha)
        front_ring = ImageChops.multiply(ring, front.getchannel('A').filter(ImageFilter.MaxFilter(3)))
        ident = IDS.get(name, name)
        full.save(out / f'{ident}.png')
        for kind, im, mask in [('back', back, ImageChops.subtract(ring, front_ring)),
                               ('front', front, front_ring)]:
            im.save(out / f'{ident}-{kind}.png')
            outline = Image.new('RGBA', full.size, 'white')
            outline.putalpha(mask)
            outline.save(out / f'{ident}-{kind}-outline.png')
        modes = {k: [v[0]+1, v[1]+1] for k, v in s.get('mode_contacts', {}).items()}
        origin = [v-1 for v in POSITIONS[name]]
        rows.append(dict(id=ident, source=f'source/{name}.png', atlas_rect=s['rect'],
                         grid_origin=POSITIONS[name], origin=origin, size=list(full.size),
                         floor=s['floor']+1, depth=POSITIONS[name][1]+s['floor'],
                         contact=[v+1 for v in s['contact']], mode_contacts=modes,
                         hit_mask=[''.join('1' if alpha.getpixel((x,y)) else '0' for x in range(full.width))
                                   for y in range(full.height)]))
    save_json(out/'catalog.json', rows)
    for sex in ('female', 'male'):
        folder = ROOT/'assets/characters'/sex
        manifest = json.loads((folder/'atlas-manifest.json').read_text())
        page = Image.open(folder/'atlas.png').convert('RGBA')
        frames = []
        for frame in manifest['sprites']:
            assert frame['source_size'] == [128, 160]
            x,y,w,h = frame['rect']
            im = Image.new('RGBA', (128,160))
            im.alpha_composite(page.crop((x,y,x+w,y+h)), tuple(frame['trim_offset']))
            frames.append((frame['id'], im.resize((96,120), Image.Resampling.NEAREST)))
        for variant in ('', 'base/'):
            for i in range(6):
                path = folder/(f'frames/scene-quenching-{i}.png' if not variant else f'frames/scene-base/quenching-{i}.png')
                frames.append((f'{variant}quenching/scene-{i}', Image.open(path).convert('RGBA').resize((96,120),Image.Resampling.NEAREST)))
        # Full canvases keep nearest sampling phase stable across differently trimmed poses.
        atlas = Image.new('RGBA', (96*8,120*math.ceil(len(frames)/8)))
        for i, (_, im) in enumerate(frames):
            atlas.alpha_composite(im, ((i%8)*96,(i//8)*120))
        atlas.save(out/f'{sex}-75.png')
        save_json(out/f'{sex}-75.json', dict(canvas=[96,120],pivot=[48,105],columns=8,
                                           rows=math.ceil(len(frames)/8),ids=[x[0] for x in frames]))
        arms_dir = out/'arms'
        arms_dir.mkdir(exist_ok=True)
        for variant in ('base', 'equipped'):
            for i in range(6):
                pose = folder/f'poses/researching-{i}'
                im = Image.open(pose/'body'/('arms-front-base.png' if variant=='base' else 'arms-front.png')).convert('RGBA')
                if variant=='equipped':
                    im.alpha_composite(Image.open(pose/'equipment/gloves.png').convert('RGBA'))
                im.resize((96,120),Image.Resampling.NEAREST).save(arms_dir/f'{sex}-{variant}-researching-{i}.png')
    registrations = []
    for sex in ('female','male'):
        animation = json.loads((ROOT/f'assets/characters/{sex}/animation.json').read_text())
        for mode, reg in animation['workshop']['stations'].items():
            ident = 'workbench' if mode in ('researching','salvaging') else reg['station_id']
            prop = next(row for row in rows if row['id']==ident)
            contact = prop['mode_contacts'].get(mode,prop['contact'])
            exact = [prop['origin'][i]+contact[i]-(reg['contact_points']['character'][i]-[64,140][i])*0.75 for i in range(2)]
            foot = [math.floor(v+0.5) for v in exact]
            registrations.append(dict(character=sex,mode=mode,fixture=ident,foot_exact=exact,
                                      foot=foot,error=[foot[i]-exact[i] for i in range(2)],
                                      canonical_contact=reg['contact_points']['character']))
    save_json(out/'layout.json',dict(format='little-forge-layout-v1', grid=32, viewport=[960,180],
        expanded=[960,540], background='forge/interior/background.png', character_scale=[3,4],
        fixtures=rows, registrations=registrations,
        sampling='Whole 128x160 canvas -> 96x120 nearest; foot snapped once to logical pixel',
        depth='Floor depth for travel; authored work-mode foreground exceptions; outline follows its part'))
    # Legacy visitor is a high-resolution sheet, not native 128x160 character art.
    # Normalize its fixed 3x source density once, then apply the same 3/4 display rule.
    visitor = next(a for a in json.loads((ROOT/'assets/manifest.json').read_text())['assets'] if a['id']=='visitor')
    sheet = Image.open(ROOT/'assets'/visitor['path']).convert('RGBA')
    packed = Image.new('RGBA',(384,360))
    for i, (x0,y0,x1,y1) in enumerate(visitor['frame_rects']):
        crop = sheet.crop((x0,y0,x1,y1))
        w,h = round(crop.width/3), round(crop.height/3)
        canvas = Image.new('RGBA',(128,160))
        canvas.alpha_composite(crop.resize((w,h),Image.Resampling.NEAREST),(64-w//2,140-h))
        packed.alpha_composite(canvas.resize((96,120),Image.Resampling.NEAREST),((i%4)*96,(i//4)*120))
    packed.save(out/'visitor-75.png')
    save_json(out/'visitor-75.json',dict(source=visitor['path'],source_density_divisor=3,
        normalized_canvas=[128,160],display_scale=[3,4],display_canvas=[96,120],pivot=[48,105],
        status='Legacy source density normalization is a visual-review decision, not canonical character reauthoring'))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            build(root)
            for path in root.rglob('*'):
                if path.is_file():
                    assert path.read_bytes() == (OUT/path.relative_to(root)).read_bytes(), str(path)
        print('Atlas exports, binary masks, layer reconstruction and canonical 3/4 derivatives match')
    else:
        build(OUT)
