"""Alpha-connected components of a source sheet (stdlib + Pillow only)."""
from PIL import Image


def components(path, threshold=16, min_area=400, gap=6):
    """Return bounding boxes (l, t, r, b exclusive) of opaque islands, sorted top-left.

    A max filter of `gap` px merges strokes of one control (frame corners, rivets)
    without joining neighbouring controls.
    """
    alpha = Image.open(path).convert('RGBA').getchannel('A').point(lambda v: 255 if v > threshold else 0)
    grown = alpha.filter(__import__('PIL.ImageFilter', fromlist=['MaxFilter']).MaxFilter(gap * 2 + 1))
    w, h = grown.size
    px = grown.load()
    seen = bytearray(w * h)
    boxes = []
    for y in range(h):
        for x in range(w):
            if px[x, y] and not seen[y * w + x]:
                stack = [(x, y)]
                seen[y * w + x] = 1
                l, t, r, b, n = x, y, x, y, 0
                while stack:
                    cx, cy = stack.pop()
                    n += 1
                    l, t, r, b = min(l, cx), min(t, cy), max(r, cx), max(b, cy)
                    for nx, ny in ((cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)):
                        if 0 <= nx < w and 0 <= ny < h and px[nx, ny] and not seen[ny * w + nx]:
                            seen[ny * w + nx] = 1
                            stack.append((nx, ny))
                if n >= min_area:
                    boxes.append((l, t, r + 1, b + 1))
    # tighten to the real (un-grown) alpha
    out = []
    for l, t, r, b in boxes:
        bb = alpha.crop((l, t, r, b)).getbbox()
        out.append((l + bb[0], t + bb[1], l + bb[2], t + bb[3]))
    return sorted(out, key=lambda q: (round(q[1] / 60), q[0]))
