"""Derive ore and ingot icons by recolouring the approved iron ore and iron ingot.

No new generation. Each base is split into three regions:
- outline: near-black, kept as it is
- body: rock or metal, neutral colours
- accent: the hematite vein on the ore, the rust specks on the ingot

Body and accent keep their relative lightness and are re-sampled from the researched ramps
(material-catalog.json; see "Metal reference" in docs/design/MATERIAL_ATLAS_V1.md). A few
mineral marks (glints, chalcopyrite grains, malachite bands) are placed by rule, not by hand.
"""
import colorsys, json, os
from PIL import Image

ICONS = "assets/items/materials/icons"
CAT = json.load(open("assets/items/materials/material-catalog.json"))
R = CAT["ramps"]

def rgb(h): return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))
def hls(c): return colorsys.rgb_to_hls(*[v / 255 for v in c])
def ramp(stops, n):
    """Spread a short ramp over n flat tones (linear between stops)."""
    s = [rgb(h) for h in stops]
    if n <= len(s): return s
    out = []
    for i in range(n):
        t = i / (n - 1) * (len(s) - 1); a = int(t); b = min(a + 1, len(s) - 1); f = t - a
        out.append(tuple(round(s[a][k] * (1 - f) + s[b][k] * f) for k in range(3)))
    return out

def split(im):
    px = im.load(); W, H = im.size
    outline, body, accent = set(), set(), set()
    for y in range(H):
        for x in range(W):
            p = px[x, y]
            if not p[3]: continue
            h, l, s = hls(p[:3])
            if l < 0.06: outline.add((x, y))
            elif s >= 0.35: accent.add((x, y))
            else: body.add((x, y))
    return px, outline, body, accent

def paint(o, px, pixels, stops, levels=None):
    if not pixels: return
    tones = sorted({hls(px[p][:3])[1] for p in pixels})
    r = ramp(stops, min(levels or len(tones), 7))
    lo, hi = tones[0], tones[-1]
    for p in pixels:
        t = (hls(px[p][:3])[1] - lo) / (hi - lo + 1e-9)
        o[p] = r[min(len(r) - 1, int(t * len(r)))] + (255,)

def ingot(metal):
    im = Image.open(f"{ICONS}/iron_ingot.png").convert("RGBA")
    px, outline, body, accent = split(im)
    out = im.copy(); o = out.load()
    # rust specks belong to iron only: give each speck the lightness of its body neighbours
    src = im.copy(); sp = src.load()
    for (x, y) in accent:
        ns = [sp[q] for q in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)) if q in body]
        if ns: sp[x, y] = max(ns, key=lambda c: hls(c[:3])[1])
    paint(o, sp, body | accent, R[metal])
    if metal == "lead":
        # soft metal: dent the top-right corner by one pixel
        corner = max((p for p in body | outline), key=lambda p: p[0] - p[1])
        o[corner] = (0, 0, 0, 0)
        x, y = corner
        for q in ((x - 1, y), (x, y + 1), (x - 1, y + 1)):
            if q in body: o[q] = rgb(R["outline"][0]) + (255,)
    out.save(f"{ICONS}/{metal}_ingot.png")

ORES = {  # id: (host ramp or None to keep, mineral ramp, marks)
    "copper": (None, "malachite", "copper"),
    "tin": (None, "cassiterite", "glint"),
    "lead": (None, "galena", "glint"),
    "silver": ("rock-dark", "silver", None),
    "gold": ("quartz", "gold", None),
    "zinc": (None, "sphalerite", "resin"),
}

def ore(metal):
    host, mineral, marks = ORES[metal]
    im = Image.open(f"{ICONS}/iron_ore.png").convert("RGBA")
    px, outline, body, accent = split(im)
    out = im.copy(); o = out.load()
    if host: paint(o, px, body, R[host])
    if marks == "copper":
        # malachite: two-tone diagonal banding; chalcopyrite: brassy grains on the lightest spots
        g = ramp(R["malachite"], 4)
        for (x, y) in accent:
            l = hls(px[x, y][:3])[1]
            band = ((x + y) // 2) % 2
            o[x, y] = g[min(3, int(l * 4) + band)] + (255,)
        for p in sorted(accent, key=lambda p: -hls(px[p][:3])[1])[:3]:
            o[p] = rgb(R["chalcopyrite"][0]) + (255,)
    else:
        paint(o, px, accent, R[mineral])
    if marks in ("glint", "resin"):
        tone = R["mineral-glint"][0] if marks == "glint" else R["sphalerite"][-1]
        for p in sorted(accent, key=lambda p: -hls(px[p][:3])[1])[:2]:
            o[p] = rgb(tone) + (255,)
    out.save(f"{ICONS}/{metal}_ore.png")

def build_atlas():
    items = {i["id"]: i for i in CAT["items"]}
    atlas = Image.new("RGBA", tuple(CAT["atlas_size"]))
    for f in sorted(os.listdir(ICONS)):
        if f[:-4] in items:
            c, r = items[f[:-4]]["cell"]; atlas.paste(Image.open(f"{ICONS}/{f}"), (32 * c, 32 * r))
    atlas.save("assets/items/materials/materials-v1.png")

if __name__ == "__main__":
    for m in ["copper", "tin", "lead", "silver", "gold", "zinc", "steel", "bronze", "brass"]:
        ingot(m)
    for m in ORES:
        ore(m)
    build_atlas()
    for f in sorted(os.listdir(ICONS)):
        if f.endswith(("_ingot.png", "_ore.png")):
            im = Image.open(f"{ICONS}/{f}")
            print(f, len({p[:3] for p in im.getdata() if p[3]}), "colors")
