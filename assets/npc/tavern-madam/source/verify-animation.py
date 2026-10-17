"""Validate authored animation contracts, pixel provenance and packed outputs."""
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw
import hashlib
import importlib.util
import json

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('madam_builder',ROOT/'source/build-animation.py')
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
load=b.load;pixels=b.pixels;compose=b.compose;embed=b.embed;move=b.move


def verify():
    locks=json.loads((ROOT/'source/animation/baseline-lock.json').read_text())
    for path,digest in locks.items():assert hashlib.sha256(pixels(load(path))).hexdigest()==digest,('neutral changed',path)
    animation=json.loads((ROOT/'animation.json').read_text());recipes=json.loads((ROOT/'poses/recipes.json').read_text())['poses']
    assert len(animation['frames'])==23 and set(animation['frames'])==set(recipes)
    colors=set();checked=set();frames={}
    palette=set(b.Builder().colors)
    for name,recipe in recipes.items():
        images={n:load(path) for n,path in {**recipe['layers'],**recipe['equipment']}.items()}
        for n,image in images.items():
            assert image.size==(128,160),(name,n,image.size)
            assert set(image.getchannel('A').getdata())<={0,255},(name,n)
            opaque={p[:3] for p in image.getdata() if p[3]};assert opaque<=palette,(name,n,opaque-palette)
            colors.update(opaque);checked.add(({**recipe['layers'],**recipe['equipment']})[n])
        base=compose([images[n] for n in recipe['base_order']]);full=compose([images[n] for n in recipe['composition_order']])
        for variant,image in [('base',base),('full',full)]:
            path=f'frames/{"base/" if variant=="base" else ""}{name}.png'
            assert pixels(image)==pixels(load(path)),('frame recomposition',path)
            assert image.getchannel('A').getbbox()[3]==140,('foot ground',name,variant,image.getbbox())
            assert pixels(image)==pixels(load(f'poses/{name}/{"base" if variant=="base" else "composed"}.png'))
        assert all(not p[3] or q[3] for p,q in zip(base.getdata(),full.getdata())),('base alpha loss',name)
        assert pixels(full.resize((512,640),Image.Resampling.NEAREST))==pixels(load(f'previews/frames/{name}-4x.png'))
        frames[name]={'base':base,'full':full}
    assert len(colors)<=64
    repair=ROOT/'source/animation/nod-eye-repair'
    before_hashes=json.loads((repair/'before-frame-hashes.json').read_text())
    changed_frames=[path for path,digest in before_hashes.items()
                    if hashlib.sha256((ROOT/path).read_bytes()).hexdigest()!=digest]
    expected_repairs={'frames/nod-down.png','frames/base/nod-down.png'}
    happy_repair=ROOT/'source/animation/happy-eye-repair'
    if (happy_repair/'before-frame-hashes.json').exists():
        expected_repairs.update({'frames/happy.png','frames/base/happy.png'})
    assert set(changed_frames)==expected_repairs,('unregistered frame changed',changed_frames)
    old=load('source/animation/nod-eye-repair/before-registered-face.png')
    new=load('source/animation/nod-down-registered-face.png')
    mask=load('source/animation/nod-eye-repair/adoption-mask.png')
    changes=[(x,y) for y in range(128) for x in range(96) if old.getpixel((x,y))!=new.getpixel((x,y))]
    assert changes and all(mask.getpixel(point)[3] for point in changes),'nod repair escaped registered eye/nose mask'
    assert any(x<50 for x,y in changes) and any(x>50 for x,y in changes),'both eyes must be repaired'
    assert old.getpixel((41,39))==new.getpixel((41,39)),'nod beauty mark changed'
    b.write('qa/nod-eye-repair.json',{'status':'pass','clip':'nod','key_zero_based':2,'key_one_based':3,
            'frame':'nod-down','changed_native_pixels':len(changes),'both_eyes_changed':True,
            'outside_eye_nose_mask':'identical','other_frames':f'{len(before_hashes)-len(expected_repairs)} identical files; registered happy repair excluded',
            'baseline_neutral':'unchanged','prompt':'source/animation/nod-eye-repair/prompt.json',
            'comparison':'previews/nod-eyes-before-after.png'})
    if (repair/'before-nose-face.png').exists():
        before_nose=load('source/animation/nod-eye-repair/before-nose-face.png')
        nose_changes=[(x,y) for y in range(128) for x in range(96) if before_nose.getpixel((x,y))!=new.getpixel((x,y))]
        assert nose_changes and all(50<=x<56 and 39<=y<41 for x,y in nose_changes),'nose repair escaped its small region'
        assert all(new.getpixel((x,39))[0]>205 for x in range(52,55)),'eye/nose skin gap lost'
        assert new.getpixel((51,40))[0]<200 and new.getpixel((53,40))[0]>205,'nose tip not separated'
        assert load(recipes['nod-down']['layers']['head']).getpixel((67,56))==new.getpixel((51,40))
        assert not load(recipes['nod-down']['layers']['mouth']).getpixel((67,56))[3]
        assert not load(recipes['nod-down']['layers']['eyes']).getpixel((67,56))[3]
        b.write('qa/nod-nose-repair.json',{'status':'pass','frame':'nod-down','changed_native_pixels':len(nose_changes),
                'adoption_box_native':[50,39,56,41],'skin_gap_px':1,'nose_translation_x':-2,
                'nose_owner':'head','near_eye_and_other_features':'unchanged from preceding eye repair',
                'other_frames':f'{len(before_hashes)-len(expected_repairs)} identical files; registered happy repair excluded','comparison':'previews/nod-nose-before-after.png',
                'prompt':'source/animation/nod-eye-repair/nose-prompt.json'})
    if (happy_repair/'before-frame-hashes.json').exists():
        happy_hashes=json.loads((happy_repair/'before-frame-hashes.json').read_text())
        happy_changed=[path for path,digest in happy_hashes.items() if hashlib.sha256((ROOT/path).read_bytes()).hexdigest()!=digest]
        assert set(happy_changed)=={'frames/happy.png','frames/base/happy.png'}
        old_eyes=load('source/animation/happy-eye-repair/before-eyes.png');new_eyes=load('eyes/happy-closed.png')
        dark=(77,23,16,255)
        for l,r,dy in [(39,49,2),(52,56,1)]:
            before={(x,y) for y in range(27,34) for x in range(l,r) if old_eyes.getpixel((x,y))==dark}
            after={(x,y) for y in range(27,34) for x in range(l,r) if new_eyes.getpixel((x,y))==dark}
            assert after=={(x,y+dy) for x,y in before},('smile eye rigid translation',l,dy)
        for y in range(128):
            for x in range(96):
                if not (((39<=x<49 or 52<=x<56) and 28<=y<=32) or (x==51 and 30<=y<34)):
                    assert old_eyes.getpixel((x,y))==new_eyes.getpixel((x,y))
        assert all(new_eyes.getpixel((51,y))[3] and new_eyes.getpixel((51,y))[0]>205 for y in range(30,34)),'open-eye remnant beside smile'
        old_happy=load('source/animation/happy-eye-repair/before-frame.png');new_happy=frames['happy']['full']
        assert pixels(old_happy.crop((52,16,76,43)))==pixels(new_happy.crop((52,16,76,43))),'smile repair moved eyebrows'
        assert pixels(old_happy.crop((48,50,76,62)))==pixels(new_happy.crop((48,50,76,62))),'smile repair changed mole/nose/mouth'
        b.write('qa/happy-eye-repair.json',{'status':'pass','near_eye_translation':[0,2],'far_eye_translation':[0,1],
                'curve_shape_width_color':'exactly preserved','eyebrows_nose_mole_mouth':'unchanged',
                'open_eye_remnant_cleanup_native':{'x':51,'rows':[30,31,32,33]},
                'other_frames':'44 identical base/equipped files','canvas':[128,160],
                'comparison':'previews/happy-eyes-before-after.png'})
    expression=json.loads((ROOT/'expressions/recipes.json').read_text())
    eye_mask=load('source/animation/eye-change-mask.png').getchannel('A')
    mouth_mask=load('source/animation/mouth-change-mask.png').getchannel('A')
    original=load('base-neutral.png');neutral_head=load('head-base.png');neutral_mole=original.getpixel((42,34))
    for name in expression['recipes']:
        image=load(f'expressions/base/{name}.png')
        for y in range(128):
            for x in range(96):
                if not eye_mask.getpixel((x,y)) and not mouth_mask.getpixel((x,y)):
                    assert image.getpixel((x,y))==original.getpixel((x,y)),('feature escaped mask',name,x,y)
        assert image.getpixel((42,34))==neutral_mole,('mole changed',name)
        assert pixels(image.crop((52,35,55,36)))==pixels(original.crop((52,35,55,36))),('nose changed',name)
        assert pixels(frames[name]['base'])==pixels(embed(image))
    original_parts={'head':'head-base.png','eyes':'eyes/open.png','mouth':'mouth/rest.png'}
    for name in ['nod-mid','nod-down','turn-left','turn-right']:
        recipe=recipes[name]
        assert pixels(load(recipe['layers']['upper']))==pixels(embed(load('body/upper-base.png'))),('head motion changed clothes',name)
        assert pixels(load(recipe['layers']['lower']))==pixels(embed(load('body/lower-base.png')))
        for gear in ['shawl','apron','belt-accessories','towel','tankard']:
            assert pixels(load(recipe['equipment'][gear]))==pixels(embed(load(f'equipment/{gear}.png')))
        if name=='nod-mid':
            for part,path in original_parts.items():assert pixels(load(recipe['layers'][part]))==pixels(move(embed(load(path)),0,1))
        else:
            assert pixels(compose([load(recipe['layers'][n]) for n in ['head','eyes','mouth']]))==pixels(embed(load(f'source/animation/{name}-registered-face.png')))
    for name in ['give-mid','give-full','give-empty','give-retract','neutral-empty']:
        recipe=recipes[name]
        for n,path in {**original_parts,'lower':'body/lower-base.png'}.items():assert pixels(load(recipe['layers'][n]))==pixels(embed(load(path)))
        assert ('tankard' in recipe['equipment'])==(name in ['give-mid','give-full'])
        if name!='neutral-empty':
            dx,dy=recipe['grip_translation']
            if 'tankard' in recipe['equipment']:assert pixels(load(recipe['equipment']['tankard']))==pixels(move(embed(load('equipment/tankard.png')),dx,dy))
            upper=load(recipe['layers']['upper'])
            assert all(upper.getpixel((x+16+dx,45+16+dy))[3] and max(upper.getpixel((x+16+dx,45+16+dy))[:3])<60 for x in range(67,71))
    give=animation['clips']['give'];assert give['events']==[{'key':3,'name':'transfer','hand':'left','item':'tankard','fires':'once_on_entry'}]
    assert give['frames'][-1]=='neutral-empty' and not give['loop']
    assert animation['clips']['walk']['frames']==[f'walk-{k}' for k in range(6)]
    for k in range(6):
        recipe=recipes[f'walk-{k}'];assert not {'tankard','towel'}&set(recipe['equipment'])
        assert recipe['composition_order'][-1]=='arms_front'
        dy=recipe['head_translation'][1]
        for n,path in original_parts.items():assert pixels(load(recipe['layers'][n]))==pixels(move(embed(load(path)),0,dy))
    gait=[recipes[f'walk-{k}']['gait']['measured_native'] for k in range(6)]
    for k in range(6):
        recipe=recipes[f'walk-{k}'];assert recipe['composition_order'][0]=='shawl-underhem'
        im=load(f'source/animation/walk-{k}-body-registered.png')
        remaining={(x,y) for y in range(128) for x in range(96) if im.getpixel((x,y))[3]};components=[]
        while remaining:
            queue=[remaining.pop()];component=[]
            while queue:
                x,y=queue.pop();component.append((x,y))
                for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                    point=(x+dx,y+dy)
                    if point in remaining:remaining.remove(point);queue.append(point)
            components.append(component)
        assert len(components)==1,('detached walking body pixels',k,[len(c) for c in components])
    opposition=[]
    for a,c in [(0,3),(1,4)]:
        for side in ['near','far']:
            hand_delta=gait[c][side+'_hand'][0]-gait[a][side+'_hand'][0]
            foot_delta=gait[c][side+'_boot']['centroid'][0]-gait[a][side+'_boot']['centroid'][0]
            assert hand_delta*foot_delta<0,('same-side arm/leg phase',a,c,side)
            opposition.append({'keys':[a,c],'side':side,'hand_dx':round(hand_delta,2),'boot_dx':round(foot_delta,2)})
    assert gait[2]['near_boot']['leather_bbox'][3]<124 and gait[5]['far_boot']['leather_bbox'][3]<124,'passing boot never lifts'
    assert gait[3]['near_hand'][0]<gait[2]['near_hand'][0]<gait[0]['near_hand'][0]
    assert gait[3]['near_hand'][0]<gait[5]['near_hand'][0]<gait[0]['near_hand'][0]
    atlas=load('atlas.png');manifest=json.loads((ROOT/'atlas-manifest.json').read_text())
    assert atlas.width%16==atlas.height%16==0 and max(atlas.size)<=2048
    assert len(manifest['sprites'])==23
    for sprite in manifest['sprites']:
        x,y,w,h=sprite['rect'];source=frames[sprite['id']]['full']
        assert sprite['source_size']==[128,160] and sprite['pivot']==[64,140] and not sprite['rotated']
        assert pixels(atlas.crop((x,y,x+w,y+h)))==pixels(source)
        for yy in range(-2,h+2):
            for xx in range(-2,w+2):
                if xx in (-2,w+1) or yy in (-2,h+1):assert atlas.getpixel((x+xx,y+yy))[3]==0
                elif xx in (-1,w) or yy in (-1,h):assert atlas.getpixel((x+xx,y+yy))==source.getpixel((max(0,min(w-1,xx)),max(0,min(h-1,yy))))
    gif_keys=0
    for name,clip in animation['clips'].items():
        assert len(clip['frames'])==len(clip['durations_ms']) and all(t%10==0 and t>0 for t in clip['durations_ms'])
        for slow in [1,4]:
            gif=Image.open(ROOT/f'previews/{name}{"-slow" if slow==4 else ""}.gif')
            assert gif.n_frames==len(clip['frames']),(name,gif.n_frames)
            for k,(frame_name,t) in enumerate(zip(clip['frames'],clip['durations_ms'])):
                gif.seek(k);assert gif.info['duration']==t*slow,(name,k,gif.info['duration'])
                image=gif.convert('RGB')
                for column,variant in enumerate(['base','full']):
                    panel=Image.new('RGB',(512,640),'#eee3cc');im=frames[frame_name][variant].resize((512,640),Image.Resampling.NEAREST);panel.paste(im,(0,0),im)
                    assert image.crop((12+column*536,28,524+column*536,668)).tobytes()==panel.tobytes(),('GIF frame mismatch',name,k,variant)
                gif_keys+=1
    report={'status':'pass','approved_neutral_locks':len(locks),'frames':len(frames),'clips':len(animation['clips']),
            'expressions':8,'native_layer_files':len(checked),'shared_opaque_colors':len(colors),'frame_canvas':[128,160],
            'foot_pivot':[64,140],'binary_alpha':True,'frame_recomposition':'exact','approved_mole_and_nose':'preserved',
            'nod_mid_face':'approved rigid +[0,1]','head_gesture_clothing':'unchanged','tankard':'original pixels and grip holes',
            'give_transfer_events':1,'walk_keys':6,'walk_held_props':False,'walk_same_side_opposition':opposition,
            'passing_boot_lift_and_intermediate_arms':'pass','walk_body_connected_components':1,
            'red_underhem':'registered to walking hem, behind legs','atlas_roundtrip_extrude_gutter':'pass',
            'gif_keys_pixels_and_durations_checked':gif_keys,'nearest_4x':'exact','visual_playback':'see browser-playback.json',
            'runtime':'not_integrated_or_measured',
            'generated_candidate_hashes':{p.name:hashlib.sha256(p.read_bytes()).hexdigest()
                for p in sorted((ROOT/'source/animation/candidates').glob('*.png')) if not p.stem.endswith(('-native','-sample','-patched'))}}
    b.write('qa/animation-validation.json',report);print(json.dumps(report,indent=2))


if __name__=='__main__':verify()
