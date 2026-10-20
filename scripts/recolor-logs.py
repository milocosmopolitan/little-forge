"""Derive log icons for new species by recolouring an approved log, with no new generation.

Each species takes a base log whose bark texture matches the researched bark. Pixels are split
into three regions:
- outline: dark pixels, kept as they are
- end face: the light cut end with holes filled; its outermost ring is sapwood, the rest heartwood
- bark: everything else

Inside each region a pixel keeps its relative lightness and is re-sampled from the species
ramp. Ramps live in material-catalog.json and follow the "Wood reference" table in
docs/design/MATERIAL_ATLAS_V1.md.
"""
import colorsys, json
from PIL import Image

ICONS = "assets/items/materials/icons"
CAT = json.load(open("assets/items/materials/material-catalog.json"))
SPECIES = {  # id: (base log, extra marks)
    "walnut_log": ("oak_log", None),
    "yew_log": ("oak_log", None),
    "cherry_log": ("ash_log", "lenticels"),
    "mahogany_log": ("pine_log", None),
    "birch_log": ("ash_log", "birch"),
}

def hexrgb(h): return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))
def hls(c): return colorsys.rgb_to_hls(*[v / 255 for v in c])
def sample(ramp, t):
    """Nearest ramp stop for t in [0,1]: flat colours only, no blended tones."""
    return ramp[min(len(ramp) - 1, int(t * len(ramp)))]

def regions(im):
    W, H = im.size; px = im.load()
    opaque = {(x, y) for x in range(W) for y in range(H) if px[x, y][3]}
    outline = {p for p in opaque if hls(px[p][:3])[1] < 0.13}
    seeds = {p for p in opaque - outline if hls(px[p][:3])[1] >= 0.5 and hls(px[p][:3])[2] >= 0.35}
    # largest 4-connected seed blob = cut end
    best, seen = set(), set()
    for s in seeds:
        if s in seen: continue
        comp, stack = set(), [s]
        while stack:
            q = stack.pop()
            if q in comp or q not in seeds: continue
            comp.add(q); x, y = q; stack += [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]
        seen |= comp
        if len(comp) > len(best): best = comp
    # fill holes: non-face pixels inside the blob's bbox that cannot reach the bbox edge
    xs = [p[0] for p in best]; ys = [p[1] for p in best]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    face = set(best); outside = set()
    stack = [(x, y) for x in range(x0, x1 + 1) for y in (y0, y1)] + [(x, y) for y in range(y0, y1 + 1) for x in (x0, x1)]
    while stack:
        q = stack.pop()
        if q in outside or q in best: continue
        x, y = q
        if not (x0 <= x <= x1 and y0 <= y <= y1): continue
        outside.add(q); stack += [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]
    face |= {(x, y) for x in range(x0, x1 + 1) for y in range(y0, y1 + 1)} - outside - outline
    face &= opaque
    rim = {(x, y) for (x, y) in face if any(n not in face for n in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)))}
    bark = opaque - outline - face
    return px, outline, face, rim, bark

def recolor(id, base, marks):
    sp = id.split("_")[0]; R = CAT["ramps"]
    bark_r = [hexrgb(h) for h in R[f"{sp}-bark"]]
    end = [hexrgb(h) for h in R[f"{sp}-end"]]
    has_sap = sp != "mahogany"
    heart, sap = (end[:-1], end[-1]) if has_sap else (end, None)
    im = Image.open(f"{ICONS}/{base}.png").convert("RGBA")
    px, outline, face, rim, bark = regions(im)
    out = im.copy(); o = out.load()
    def lum_map(pixels, ramp):
        ls = {p: hls(px[p][:3])[1] for p in pixels}
        lo, hi = min(ls.values()), max(ls.values())
        for p, l in ls.items():
            o[p] = sample(ramp, (l - lo) / (hi - lo + 1e-9)) + (255,)
    lum_map(bark, bark_r)
    inner = face - rim if has_sap else face
    lum_map(inner, heart)
    if has_sap:
        for p in rim: o[p] = sap + (255,)
    if marks == "lenticels":
        # Black cherry: short pale horizontal lenticels. On this diagonal log, "horizontal
        # around the trunk" runs across the log axis, drawn as 2px dashes in rows.
        light = hexrgb(R["cherry-bark"][-1])
        for (x, y) in sorted(bark):
            if (x * 3 + y * 5) % 11 == 0 and (x + 1, y) in bark and (x - 1, y) in bark:
                o[x, y] = light + (255,); o[x + 1, y] = light + (255,)
    if marks == "birch":
        # Silver birch: white bark, black diamond marks (the base's deepest furrows) and pale
        # grey horizontal lenticels across the log axis.
        ls = sorted({hls(px[p][:3])[1] for p in bark})
        black = hexrgb(R["birch-mark"][0]); grey = hexrgb(R["birch-mark"][1])
        for p in bark:
            if hls(px[p][:3])[1] <= ls[0]: o[p] = black + (255,)
        # sparse short black streaks across the log axis (the dark marks birch is known by)
        for (x, y) in sorted(bark):
            if (x * 3 + y * 5) % 17 == 0 and all((x + d, y) in bark for d in (-1, 1, 2)):
                for d in (0, 1, 2): o[x + d, y] = black + (255,)
                if (x + 1, y - 1) in bark: o[x + 1, y - 1] = grey + (255,)
    out.save(f"{ICONS}/{id}.png")
    n = len({p[:3] for p in out.getdata() if p[3]})
    print(id, "from", base, "colors", n, "face", len(face), "rim", len(rim), "bark", len(bark))

def build_atlas():
    items = {i["id"]: i for i in CAT["items"]}
    atlas = Image.new("RGBA", tuple(CAT["atlas_size"]))
    import os
    for f in sorted(os.listdir(ICONS)):
        id = f[:-4]
        if id in items:
            c, r = items[id]["cell"]; atlas.paste(Image.open(f"{ICONS}/{f}"), (32 * c, 32 * r))
    atlas.save("assets/items/materials/materials-v1.png")

if __name__ == "__main__":
    for id, (base, marks) in SPECIES.items():
        recolor(id, base, marks)
    build_atlas()
