"""Derive four parts from approved part icons, with no new generation.

- wooden_rod: wooden_pole with its middle removed, so the far end cap rejoins a short rod.
- arrow_shaft: three thin shafts cut from the pole's centre line, each with a nock slit.
- leather_wrapped_handle: wooden_handle with a spiral deer-leather wrap over the grip.
- leather_grip: the wrapped grip section on its own, as a short leather tube.

Run after recolor-parts.py (bases must already be on the material palette). Every result gets
the shared 1px outline and is re-centred on the 32x32 cell.
"""
import importlib.util, json, os
from PIL import Image

spec = importlib.util.spec_from_file_location("p", os.path.join(os.path.dirname(__file__), "recolor-parts.py"))
P = importlib.util.module_from_spec(spec); spec.loader.exec_module(P)
m, R, hls = P.m, P.R, P.hls
ICONS = f"{P.PARTS}/icons"
OUTLINE = m.rgb(R["outline"][0]) + (255,)
LEATHER = sorted((m.rgb(h) for h in R["deer-leather"]), key=lambda c: hls(c)[1])

def load(id): return Image.open(f"{ICONS}/{id}.png").convert("RGBA")
def opaque(im): return {(x, y) for y in range(32) for x in range(32) if im.getpixel((x, y))[3]}

def finish(pixels, id):
    """pixels: {(x,y): rgba}. Redraw the edge as outline, centre, save."""
    pts = set(pixels)
    for (x, y) in pts:
        if any(q not in pts for q in ((x+1,y),(x-1,y),(x,y+1),(x,y-1))): pixels[(x, y)] = OUTLINE
    xs = [p[0] for p in pts]; ys = [p[1] for p in pts]
    dx = (32 - (max(xs) - min(xs) + 1)) // 2 - min(xs)
    dy = (32 - (max(ys) - min(ys) + 1)) // 2 - min(ys)
    out = Image.new("RGBA", (32, 32))
    for (x, y), c in pixels.items(): out.putpixel((x + dx, y + dy), c)
    out.save(f"{ICONS}/{id}.png")

def leather_at(c):
    """Map an underlying pixel's lightness onto the deer-leather ramp, one step darker than the
    pale ash wood so the wrap reads against the handle."""
    l = hls(c[:3])[1]
    i = min(len(LEATHER) - 1, int(l * len(LEATHER))) - 1
    return LEATHER[max(0, i)] + (255,)
GROOVE = LEATHER[0]  # darkest leather: the seam between wraps

def rod():
    im = load("wooden_pole"); px = opaque(im)
    s = {p: p[0] - p[1] for p in px}            # position along the lower-left -> upper-right axis
    lo, hi = min(s.values()), max(s.values())
    cut_a = lo + int((hi - lo) * 0.45); cut_b = lo + int((hi - lo) * 0.8)
    k = (cut_b - cut_a) // 2
    out = {p: im.getpixel(p) for p in px if s[p] <= cut_a}
    for p in px:
        if s[p] > cut_b: out[(p[0] - k, p[1] + k)] = im.getpixel(p)
    # fill the 1-step seam the integer shift can leave
    for p in px:
        if cut_a < s[p] <= cut_a + 2 and (p[0] - 1, p[1] + 1) in out: out.setdefault(p, im.getpixel(p))
    finish(out, "wooden_rod")

def shafts():
    im = load("wooden_pole"); px = opaque(im)
    c = round(sum(x + y for x, y in px) / len(px))  # pole centre line: x + y = c
    core = {p: im.getpixel(p) for p in px if abs(p[0] + p[1] - c) <= 1}
    s = [p[0] - p[1] for p in core]; lo, hi = min(s), max(s)
    out = {}
    for off in (-5, 0, 5):                          # three shafts, offset across the axis
        for (x, y), col in core.items():
            if lo + 10 <= x - y <= hi - 8:            # a little shorter than the pole
                q = (x + off // 2, y + off - off // 2)
                if 0 <= q[0] < 32 and 0 <= q[1] < 32: out[q] = col
    # nock: notch the upper-right tip of each shaft
    for off in (-5, 0, 5):
        line = [q for q in out if q[0] + q[1] == c + off]
        if line:
            t = max(line, key=lambda q: q[0] - q[1])
            out.pop(t, None); out.pop((t[0] - 1, t[1] + 1), None)  # 2px-deep nock slit
    finish(out, "arrow_shaft")

def wrap(only_grip):
    im = load("wooden_handle"); px = opaque(im)
    s = {p: p[0] + p[1] for p in px}             # handle runs upper-left (slot) -> lower-right (butt)
    lo, hi = min(s.values()), max(s.values())
    a = lo + int((hi - lo) * 0.3); b = lo + int((hi - lo) * 0.78)
    out = {}
    for p in px:
        col = im.getpixel(p)
        if a <= s[p] <= b:
            band = (2 * s[p] + (p[0] - p[1])) // 2 % 3   # slanted bands = spiral wrap
            out[p] = leather_at(col) if band != 2 else tuple(int(v * 0.75) for v in GROOVE) + (255,)
        elif not only_grip:
            out[p] = col
    finish(out, "leather_grip" if only_grip else "leather_wrapped_handle")

if __name__ == "__main__":
    rod(); shafts(); wrap(False); wrap(True)
    P.build_atlas()
