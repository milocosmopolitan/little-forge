#!/usr/bin/env python3
"""Read-only sprite/authoring-atlas audit; JSON output, no GPU performance claims."""
import argparse
import hashlib
import json
import math
from pathlib import Path

from PIL import Image


def positive(value):
    result = int(value)
    if result < 1:
        raise argparse.ArgumentTypeError('must be positive')
    return result


def integers(values, count, minimum=0):
    return (isinstance(values, list) and len(values) == count
            and all(type(v) is int and v >= minimum for v in values))


def canonical(im):
    data = bytearray(im.convert('RGBA').tobytes())
    for i in range(0, len(data), 4):
        if not data[i + 3]:
            data[i:i + 3] = b'\0\0\0'
    return bytes(data)


def texture_bytes(width, height, fmt, mipmaps):
    blocks = {'bc3': (4, 4), 'etc2-rgba8': (4, 4),
              'astc-4x4': (4, 4), 'astc-6x6': (6, 6)}
    total = 0
    while True:
        if fmt == 'rgba8':
            total += width * height * 4
        else:
            bw, bh = blocks[fmt]
            total += math.ceil(width / bw) * math.ceil(height / bh) * 16
        if not mipmaps or (width == 1 and height == 1):
            return total
        width, height = max(1, width // 2), max(1, height // 2)


def image_metrics(im):
    raw = im.tobytes()
    alpha = raw[3::4]
    visible = sum(a > 0 for a in alpha)
    bbox = im.getchannel('A').getbbox()
    area = im.width * im.height
    trim_area = (bbox[2] - bbox[0]) * (bbox[3] - bbox[1]) if bbox else 0
    digest = hashlib.sha256(f'{im.width}x{im.height}:'.encode() + canonical(im)).hexdigest()
    return {'size': list(im.size), 'alpha_bbox_xyxy': list(bbox) if bbox else None,
            'visible_pixels': visible, 'transparent_fraction': 1 - visible / area,
            'partial_alpha_pixels': sum(0 < a < 255 for a in alpha),
            'visible_rgb_colors': len({raw[i:i + 3] for i in range(0, len(raw), 4) if raw[i + 3]}),
            'trim_rect_area_fraction': trim_area / area,
            'visible_rgba_sha256': digest}


def overlaps(a, b):
    x, y, w, h = a
    u, v, s, t = b
    return x < u + s and u < x + w and y < v + t and v < y + h


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('images', type=Path, nargs='*')
    parser.add_argument('--manifest', type=Path)
    parser.add_argument('--page-budget', type=positive, default=2048)
    parser.add_argument('--device-max-size', type=positive)
    parser.add_argument('--gpu-format', choices=['rgba8', 'bc3', 'etc2-rgba8', 'astc-4x4', 'astc-6x6'], default='rgba8')
    args = parser.parse_args()
    if not args.images and not args.manifest:
        parser.error('provide PNG images or --manifest')
    errors, warnings = [], []
    report = {'images': [], 'sprites': [], 'errors': errors, 'warnings': warnings,
              'runtime': {'draw_calls': 'not_measured', 'overdraw': 'not_measured',
                          'gpu_format_support': 'not_checked'},
              'memory_note': 'Format arithmetic only; excludes CPU copies, staging, alignment and driver overhead.'}
    cache = {}

    def load(path):
        path = path.resolve()
        if path in cache:
            return cache[path]
        try:
            with Image.open(path) as src:
                if src.format != 'PNG' or src.mode != 'RGBA':
                    errors.append(f'{path}: expected RGBA PNG')
                im = src.convert('RGBA')
            cache[path] = im
            entry = {'path': str(path), 'disk_bytes': path.stat().st_size, **image_metrics(im)}
            entry['rgba8_base_bytes'] = texture_bytes(*im.size, 'rgba8', False)
            entry['selected_format'] = args.gpu_format
            entry['selected_format_base_bytes'] = texture_bytes(*im.size, args.gpu_format, False)
            entry['selected_format_full_mip_chain_bytes'] = texture_bytes(*im.size, args.gpu_format, True)
            report['images'].append(entry)
            if entry['partial_alpha_pixels']:
                warnings.append(f'{path}: partial alpha; check declared effect/pixel-art contract')
            return im
        except (OSError, ValueError, Image.DecompressionBombError) as exc:
            errors.append(f'{path}: {exc}')
            return None

    for path in args.images:
        im = load(path)
        if im is not None:
            if max(im.size) > args.page_budget:
                warnings.append(f'{path}: exceeds review budget {args.page_budget}px')
            if args.device_max_size and max(im.size) > args.device_max_size:
                errors.append(f'{path}: exceeds supplied device limit {args.device_max_size}px')

    if args.manifest:
        try:
            manifest = json.loads(args.manifest.read_text())
            if not isinstance(manifest, dict) or manifest.get('version') != 1:
                raise ValueError('expected authoring manifest version 1')
            pages, sprites, clips = manifest['pages'], manifest['sprites'], manifest['clips']
            if not isinstance(pages, list) or not pages or not isinstance(sprites, list) or not sprites or not isinstance(clips, dict):
                raise ValueError('pages/sprites must be nonempty lists; clips must be an object')
            if manifest.get('sampler') != 'nearest':
                warnings.append('manifest sampler is not nearest; inspect pixel fidelity and bleed')
            if type(manifest.get('mipmaps')) is not bool:
                raise ValueError('mipmaps must be a boolean')
            border = [manifest['extrude_px'], manifest['gutter_px']]
            if not integers(border, 2):
                raise ValueError('extrude_px/gutter_px must be nonnegative integers')
            pad = sum(border)
            root, page_map, packed = args.manifest.resolve().parent, {}, {}
            for page in pages:
                pid = page['id']
                if not isinstance(pid, str) or not pid or pid in page_map:
                    raise ValueError('page IDs must be unique nonempty strings')
                im = load(root / page['path'])
                page_map[pid] = im
                packed[pid] = []
                if im is not None and (not integers(page['size'], 2, 1) or tuple(page['size']) != im.size):
                    errors.append(f'page {pid}: declared size differs from image')
            ids = set()
            for sprite in sprites:
                sid = sprite['id']
                if not isinstance(sid, str) or not sid or sid in ids:
                    raise ValueError('sprite IDs must be unique nonempty strings')
                ids.add(sid)
                pid, rect = sprite['page'], sprite['rect']
                if pid not in page_map:
                    errors.append(f'{sid}: unknown page {pid}')
                    continue
                im = page_map[pid]
                if im is None:
                    continue
                if not integers(rect, 4) or min(rect[2:]) < 1:
                    errors.append(f'{sid}: invalid rect')
                    continue
                x, y, w, h = rect
                if x + w > im.width or y + h > im.height:
                    errors.append(f'{sid}: rect outside page')
                    continue
                reserved = [x - pad, y - pad, w + 2 * pad, h + 2 * pad]
                if min(x - pad, y - pad) < 0 or x + w + pad > im.width or y + h + pad > im.height:
                    errors.append(f'{sid}: declared border outside page')
                for previous_id, previous_rect, previous_reserved in packed[pid]:
                    if rect != previous_rect and overlaps(reserved, previous_reserved):
                        errors.append(f'{sid}/{previous_id}: rect or reserved border overlap')
                packed[pid].append((sid, rect, reserved))
                if sprite.get('rotated', False) is not False:
                    errors.append(f'{sid}: rotated sprites are unsupported by this contract')
                    continue
                size, offset, pivot = sprite['source_size'], sprite['trim_offset'], sprite['pivot']
                if not integers(size, 2, 1) or not integers(offset, 2):
                    errors.append(f'{sid}: invalid source_size/trim_offset')
                    continue
                if (not isinstance(pivot, list) or len(pivot) != 2
                        or not all(type(v) in (int, float) and math.isfinite(v) for v in pivot)):
                    errors.append(f'{sid}: invalid pivot')
                    continue
                ox, oy = offset
                if ox + w > size[0] or oy + h > size[1]:
                    errors.append(f'{sid}: trimmed rect outside logical canvas')
                    continue
                crop = im.crop((x, y, x + w, y + h))
                if not crop.getchannel('A').getbbox():
                    errors.append(f'{sid}: empty sprite; use an explicit hidden state instead')
                metrics = {'id': sid, 'page': pid, 'rect': rect, **image_metrics(crop)}
                metrics['trimmed_pivot'] = [pivot[0] - ox, pivot[1] - oy]
                report['sprites'].append(metrics)
                if sprite.get('source'):
                    source = load(root / sprite['source'])
                    if source is not None:
                        restored = Image.new('RGBA', tuple(size))
                        restored.paste(crop, (ox, oy))
                        if source.size != tuple(size) or canonical(restored) != canonical(source):
                            errors.append(f'{sid}: packed pixels/trim origin differ from source')
                else:
                    warnings.append(f'{sid}: no source supplied; pixel/origin round-trip unverified')
            for name, clip in clips.items():
                if type(clip['loop']) is not bool or not isinstance(clip['frames'], list) or not clip['frames']:
                    errors.append(f'{name}: invalid loop/frames')
                    continue
                for frame in clip['frames']:
                    duration = frame['duration_ms']
                    if frame['sprite'] not in ids or type(duration) not in (int, float) or not math.isfinite(duration) or duration <= 0:
                        errors.append(f'{name}: invalid sprite reference or duration_ms')
            report['pages'] = []
            for pid, im in page_map.items():
                if im is None:
                    continue
                rects = {tuple(rect) for _, rect, _ in packed[pid]}
                area = sum(w * h for _, _, w, h in rects)
                report['pages'].append({'id': pid, 'size': list(im.size),
                    'unique_rect_area_fraction': area / (im.width * im.height),
                    'estimated_texture_bytes': texture_bytes(*im.size, args.gpu_format, manifest['mipmaps'])})
                if max(im.size) > args.page_budget:
                    warnings.append(f'{pid}: exceeds review budget {args.page_budget}px; not a device failure by itself')
                if args.device_max_size and max(im.size) > args.device_max_size:
                    errors.append(f'{pid}: exceeds supplied device limit {args.device_max_size}px')
            report['declared_page_texture_bytes'] = sum(p['estimated_texture_bytes'] for p in report['pages'])
            warnings.append('Extrude/gutter pixel contents, runtime sampling and performance were not measured')
        except (OSError, ValueError, KeyError, TypeError) as exc:
            errors.append(f'manifest: {exc}')

    groups = {}
    for entry in report['images']:
        groups.setdefault(entry['visible_rgba_sha256'], []).append(entry['path'])
    report['duplicate_image_groups'] = [paths for paths in groups.values() if len(paths) > 1]
    report['status'] = 'structural_failure' if errors else 'structural_pass_runtime_unmeasured'
    print(json.dumps(report, indent=2, ensure_ascii=False))
    return 1 if errors else 0


if __name__ == '__main__':
    raise SystemExit(main())
