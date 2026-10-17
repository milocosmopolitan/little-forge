#!/usr/bin/env python3
"""Read-only checks for modular parts, compositions, gait phases and atlas pixels."""
import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent


def load(name):
    with Image.open(ROOT / name) as src:
        assert src.mode == 'RGBA', (name, src.mode)
        return src.copy()


def pixels(im):
    raw = bytearray(im.tobytes())
    for i in range(0, len(raw), 4):
        if not raw[i + 3]:
            raw[i:i + 3] = b'\0\0\0'
    return bytes(raw)


def compose(paths):
    result = Image.new('RGBA', (96, 128))
    for path in paths:
        result.alpha_composite(load(path))
    return result


def verify():
    animation = json.loads((ROOT / 'animation.json').read_text())
    recipes = json.loads((ROOT / 'expressions/recipes.json').read_text())
    poses = json.loads((ROOT / 'poses/recipes.json').read_text())['poses']
    manifest = json.loads((ROOT / 'atlas-manifest.json').read_text())
    colors = set()
    files = sorted(set(ROOT.glob('eyes/*.png')) | set(ROOT.glob('mouth/*.png'))
                   | set(ROOT.glob('expressions/**/*.png')) | set(ROOT.glob('poses/**/*.png'))
                   | set(ROOT.glob('frames/**/*.png')) | set(ROOT.glob('previews/combinations/*.png'))
                   | set(ROOT.glob('body/*.png')) | set(ROOT.glob('equipment/*.png'))
                   | {ROOT / 'head-base.png', ROOT / 'base-neutral.png', ROOT / 'neutral.png'})
    for path in files:
        im = load(path.relative_to(ROOT))
        assert im.size == (96, 128), path
        raw = im.tobytes()
        assert set(raw[3::4]) <= {0, 255}, path
        assert any(raw[3::4]), path
        colors.update(raw[i:i + 3] for i in range(0, len(raw), 4) if raw[i + 3])
    assert len(colors) <= 64, len(colors)
    assert pixels(load('frames/neutral.png')) == pixels(load('neutral.png'))
    assert pixels(load('frames/base/neutral.png')) == pixels(load('base-neutral.png'))
    mask_a = load('source/expression-eye-mask.png').tobytes()[3::4]
    mask_b = load('source/expression-mouth-mask.png').tobytes()[3::4]
    reference = load('neutral.png').tobytes()
    for name, recipe in recipes['recipes'].items():
        expected = compose(['body/lower-base.png', 'body/upper-base.png', 'head-base.png',
                            recipes['eyes'][recipe['eyes']], recipes['mouth'][recipe['mouth']],
                            'equipment/apron.png', 'equipment/belt-tools.png',
                            'equipment/gloves.png', 'equipment/hat.png'])
        actual = load(f'expressions/{name}.png')
        assert pixels(expected) == pixels(actual), name
        raw = actual.tobytes()
        assert all(raw[i:i + 4] == reference[i:i + 4] for i in range(0, len(raw), 4)
                   if not mask_a[i // 4] and not mask_b[i // 4]), name
    for section in ['eyes', 'mouth']:
        parts = [pixels(load(path)) for path in recipes[section].values()]
        assert len(set(parts)) == len(parts), f'duplicate {section} states'
    for name, recipe in poses.items():
        assert pixels(compose(recipe['layers'])) == pixels(load(f'poses/{name}/composed.png')), name
        base = load(f'poses/{name}/base.png').tobytes()[3::4]
        full = load(f'poses/{name}/composed.png').tobytes()[3::4]
        assert not any(a and not b for a, b in zip(base, full)), f'lost base coverage: {name}'
    gait = []
    for k in range(6):
        leg = load(f'poses/walk-{k}/body/lower-base.png')
        b = load(f'poses/walk-{k}/base.png')
        assert leg.getchannel('A').getbbox()[3] == 125, f'ground anchor: {k}'
        assert all(b.getpixel((x, y))[3] for y in range(76, 81) for x in range(36, 61)), f'waist gap: {k}'
        if k in (2, 5):
            assert poses[f'walk-{k}']['arm_source'] == 'neutral'
        else:
            gloves = load(f'poses/walk-{k}/equipment/gloves.png')
            hand = [x for y in range(65, 92) for x in range(60) if gloves.getpixel((x, y))[3]]
            buckle = [x for y in range(104, 128) for x in range(96)
                      if (lambda p: p[3] and p[0] > 180 and p[1] > 125 and p[2] < 110)(leg.getpixel((x, y)))]
            assert hand and buckle, k
            gait.append({'key': k, 'near_hand_x': sum(hand) / len(hand), 'near_boot_buckle_x': sum(buckle) / len(buckle)})
    for a, b in [(gait[0], gait[2]), (gait[1], gait[3])]:
        assert (a['near_hand_x'] - b['near_hand_x']) * (a['near_boot_buckle_x'] - b['near_boot_buckle_x']) < 0, 'ipsilateral arm/leg swing'
    original = load('source/poses/turn-away.png').resize((64, 64), Image.Resampling.NEAREST)
    blank = load('source/poses/turn-away-blank-fixed.png').resize((64, 64), Image.Resampling.NEAREST)
    assert not any(original.getpixel((x, y))[3] >= 128 and blank.getpixel((x, y))[3] < 128
                   for y in range(32, 40) for x in range(45, 51)), 'far-eye silhouette cut away'
    assert sum(f.get('event') == 'transfer' for f in animation['clips']['give']['frames']) == 1
    for name, clip in animation['clips'].items():
        for frame in clip['frames']:
            assert frame['sprite'] in animation['frames'] and frame['duration_ms'] > 0, name
    page = load('atlas.png')
    for sprite in manifest['sprites']:
        x, y, w, h = sprite['rect']
        ox, oy = sprite['trim_offset']
        source = load(sprite['source'])
        crop = page.crop((x, y, x + w, y + h))
        restored = Image.new('RGBA', tuple(sprite['source_size']))
        restored.paste(crop, (ox, oy))
        assert pixels(restored) == pixels(source), sprite['id']
        for dy in range(-2, h + 2):
            for dx in range(-2, w + 2):
                p = page.getpixel((x + dx, y + dy))
                if dx in (-2, w + 1) or dy in (-2, h + 1):
                    assert p[3] == 0, f'gutter: {sprite["id"]}'
                elif dx in (-1, w) or dy in (-1, h):
                    expected = crop.getpixel((max(0, min(w - 1, dx)), max(0, min(h - 1, dy))))
                    assert p == expected, f'extrude: {sprite["id"]}'
    result = {'native_files_checked': len(files), 'shared_rgb_colors': len(colors),
              'expressions': len(recipes['recipes']), 'pose_recipes': len(poses),
              'clips': len(animation['clips']), 'atlas_sprites': len(manifest['sprites']),
              'baseline_unchanged': True, 'expression_changes_outside_feature_masks': 0,
              'lost_pose_base_pixels': 0, 'far_eye_silhouette_lost_pixels': 0,
              'atlas_round_trip_and_borders': 'pass', 'gait_contact_and_support_centroids': gait,
              'runtime_gpu_performance': 'not_measured'}
    print(json.dumps(result, indent=2))
    return result


if __name__ == '__main__':
    verify()
