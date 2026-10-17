#!/usr/bin/env python3
"""Publish native 96x128 authoring results on the shared 128x160 animation canvas."""
import json
import sys
from pathlib import Path

from PIL import Image, ImageSequence, GifImagePlugin

ROOT = Path(__file__).resolve().parents[1]
SIZE = (128, 160)
OFFSET = (16, 16)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def pad(path):
    image = Image.open(path).convert('RGBA')
    if image.size == SIZE:
        return
    assert image.size == (96, 128), (path, image.size)
    result = Image.new('RGBA', SIZE)
    result.paste(image, OFFSET)
    assert result.crop((16, 16, 112, 144)).tobytes() == image.tobytes()
    result.save(path)


def shift(value):
    if value is None:
        return None
    if len(value) == 2 and all(isinstance(v, (int, float)) for v in value):
        return [v + 16 for v in value]
    return [shift(v) for v in value]


def expand(name):
    folder = ROOT / 'assets/npc' / name
    animation_path = folder / 'animation.json'
    animation = json.loads(animation_path.read_text())
    if animation['canvas'] == list(SIZE):
        return
    assert animation['canvas'] == [96, 128]
    paths = sorted(set((folder / 'frames').rglob('*.png')) |
                   set((folder / 'poses').rglob('*.png')))
    for path in paths:
        pad(path)

    recipes_path = folder / 'poses/recipes.json'
    recipes = json.loads(recipes_path.read_text())
    for recipe in recipes['poses'].values():
        layers = []
        for layer in recipe['layers']:
            if not layer.startswith('poses/'):
                target = folder / 'poses/shared/canvas' / layer
                target.parent.mkdir(parents=True, exist_ok=True)
                Image.open(folder / layer).save(target)
                pad(target)
                layer = str(target.relative_to(folder))
            layers.append(layer)
        recipe['layers'] = layers
        recipe['sockets_px'] = {k: shift(v) for k, v in recipe['sockets_px'].items()}
    recipes.update(canvas=list(SIZE), offset_from_base=list(OFFSET))
    write_json(recipes_path, recipes)

    animation.update(canvas=list(SIZE), foot=shift(animation['foot']),
                     foot_anchor_px=[64, 140], base_offset_px=list(OFFSET))
    write_json(animation_path, animation)
    manifest_path = folder / 'atlas-manifest.json'
    manifest = json.loads(manifest_path.read_text())
    # Trimmed atlas pixels stay identical; only their original canvas coordinates move.
    for sprite in manifest['sprites']:
        sprite.update(source_size=list(SIZE), trim_offset=shift(sprite['trim_offset']),
                      pivot=shift(sprite['pivot']))
    write_json(manifest_path, manifest)

    for path in sorted((folder / 'previews').glob('*.gif')):
        with Image.open(path) as gif:
            if gif.size != (384, 512):
                continue
            frames, durations = [], []
            for frame in ImageSequence.Iterator(gif):
                image = frame.convert('RGB')
                result = Image.new('RGB', (512, 640), image.getpixel((0, 0)))
                result.paste(image, (64, 64))
                frames.append(result)
                durations.append(frame.info['duration'])
            loop = gif.info.get('loop', 0)
        # Pillow's save_all merges identical adjacent keys, even with optimize=False.
        # Encode each key separately so transfer/hold timings retain their frame indices.
        indexed = [frame.quantize(colors=256, dither=Image.Dither.NONE) for frame in frames]
        header, _ = GifImagePlugin.getheader(indexed[0], info={'loop': loop})
        with path.open('wb') as output:
            for block in header:
                output.write(block)
            for frame, duration in zip(indexed, durations):
                for block in GifImagePlugin.getdata(frame, duration=duration, disposal=2,
                                                    include_color_table=True):
                    output.write(block)
            output.write(b';')

    html_path = folder / 'previews/animation.html'
    html = html_path.read_text().replace('96×128 원본', '128×160 애니메이션 캔버스')
    html = html.replace('#stage{width:384px;', '#stage{width:512px;')
    for before, after in [(288, 384), (192, 256), (96, 128)]:
        html = html.replace(f'#stage{{width:{before}px}}', f'#stage{{width:{after}px}}')
    html = html.replace('(96*Number(', '(128*Number(')
    html_path.write_text(html)
    spec_path = folder / 'character-spec.yaml'
    spec = spec_path.read_text().split('\n# Shared animation canvas\n')[0].rstrip()
    spec += '\n\n# Shared animation canvas\npose_canvas:\n  grid: B\n  canvas_cells: [8, 10]\n  native_canvas_px: [128, 160]\n  base_offset_px: [16, 16]\n  foot_anchor_px: [64, 140]\n'
    spec_path.write_text(spec)
    report = {'status': 'pass', 'canvas': list(SIZE), 'base_offset_px': list(OFFSET),
              'foot_anchor_px': [64, 140], 'padded_frame_and_pose_files': len(paths),
              'resampling': False, 'original_pixels': 'exact crop roundtrip',
              'trimmed_atlas_pixels': 'unchanged', 'runtime_performance': 'not_measured'}
    write_json(folder / 'qa/canvas-validation.json', report)
    print(f'{name}: shared 128x160 animation canvas, {len(paths)} frame/pose files')


if __name__ == '__main__':
    expand(sys.argv[1])
