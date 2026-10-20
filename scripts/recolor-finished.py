"""Snap finished-good icons onto the shared material palette and rebuild the atlas.

A finished good mixes materials (an ash handle with an iron blade), so each icon is split into
regions by the colour of its cleaned pixels, and each region is re-sampled by lightness into the
ramp of its material, the same way recolor-parts.py does for single-material parts:

- grey   (low saturation)            -> iron, or ash bark on the club
- warm   (brown/tan)                 -> ash heartwood (as the parts)
- strap  (saturated mid brown)      -> deer leather (shield straps), only where the item lists it;
                                        dark plank seams stay wood
- warm_dark (dark brown)             -> deer leather (knife grip, arrow cord), only where listed

The silhouette edge is always the shared outline colour. Which regions an item has is listed under
`regions` in finished-goods-catalog.json. Run after clean-material-candidates.py --set finished and
draw-finished.py. Hand-drawn items ("source": "hand") are already on the palette and are skipped.
"""
import importlib.util, json, os
from PIL import Image

spec = importlib.util.spec_from_file_location("p", os.path.join(os.path.dirname(__file__), "recolor-parts.py"))
P = importlib.util.module_from_spec(spec); spec.loader.exec_module(P)
m, R, hls = P.m, P.R, P.hls
ROOT = "assets/items/finished"
CAT = json.load(open(f"{ROOT}/finished-goods-catalog.json"))
ITEMS = {i["id"]: i for i in CAT["items"]}
OUTLINE = m.rgb(R["outline"][0]) + (255,)
RAMPS = {
    "wood": P.WOOD("ash"),
    "bark": R["ash-bark"],
    "metal": P.METAL("iron"),
    # straps sit on pale ash planks, so they take the darker half of deer leather plus a shadow
    "leather": ["#4e3a22"] + R["deer-leather"][:2],
}
GREY_SAT = 0.22      # below this saturation a pixel is metal (or bark); an item can lower it
                     # with "grey_sat" when its pale ash shaft is nearly grey (spear, pike, halberd)
STRAP = (0.38, 0.50, 0.38)  # lightness band and min saturation of a leather strap pixel
WARM_DARK_L = 0.30  # darker warm pixels are a leather grip or cord, where the item lists warm_dark
OUTLINE_L = 0.08     # interior near-black pixels stay outline (seams, rivet holes)

def classify(c, regions, grey_sat=None):
    h, l, s = hls(c)
    # Items that set their own grey_sat (pale shafts, dark grips) are split by hue as well:
    # blue-grey steel is never wood or leather. Other items keep the plain saturation rule.
    cool = grey_sat is not None and 0.5 <= h <= 0.75
    grey_sat = GREY_SAT if grey_sat is None else grey_sat
    if "grey" in regions and (s < grey_sat or cool or ("warm" not in regions)):
        return regions["grey"]
    if "strap" in regions and STRAP[0] <= l <= STRAP[1] and s >= STRAP[2]:
        return regions["strap"]
    if "warm_dark" in regions and l < WARM_DARK_L:
        return regions["warm_dark"]
    return regions.get("warm", regions.get("grey"))

def strap_rows(groups, px):
    """Horizontal straps: a row that is mostly strap is all strap, except its dark pixels, which
    are rivet heads (iron). The dark rows right above and below a strap are its shadow edge."""
    leather, wood, metal = groups.setdefault("leather", set()), groups.setdefault("wood", set()), groups.setdefault("metal", set())
    body = leather | wood | metal
    rows = {}
    for p in body: rows.setdefault(p[1], []).append(p)
    strap = {y for y, ps in rows.items() if sum(p in leather for p in ps) * 2 >= len(ps)}
    for y in strap:
        for p in rows[y]:
            for g in (leather, wood, metal): g.discard(p)
            (metal if hls(px[p][:3])[1] < 0.3 or p in metal else leather).add(p)
    for y in {y + d for y in strap for d in (-1, 1)} - strap:
        for p in rows.get(y, []):
            if hls(px[p][:3])[1] < 0.2:
                wood.discard(p); leather.add(p)

def snap(id):
    path = f"{ROOT}/icons/{id}.png"
    im = Image.open(path).convert("RGBA"); px = im.load()
    opaque = {(x, y) for y in range(32) for x in range(32) if px[x, y][3]}
    edge = {(x, y) for (x, y) in opaque if any(q not in opaque for q in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)))}
    out = im.copy(); o = out.load()
    for p in edge: o[p] = OUTLINE
    groups = {}
    for p in opaque - edge:
        c = px[p][:3]
        if hls(c)[1] < OUTLINE_L: o[p] = OUTLINE; continue
        groups.setdefault(classify(c, ITEMS[id]["regions"], ITEMS[id].get("grey_sat")), set()).add(p)
    if "strap" in ITEMS[id]["regions"]:
        strap_rows(groups, px)
    for mat, pts in groups.items():
        ramp = sorted(RAMPS[mat], key=lambda h: hls(m.rgb(h))[1])
        m.paint(o, px, pts, ramp)
    out.save(path)
    return {k: len(v) for k, v in groups.items()}

def build_atlas():
    atlas = Image.new("RGBA", tuple(CAT["atlas_size"]))
    for f in sorted(os.listdir(f"{ROOT}/icons")):
        if f[:-4] in ITEMS:
            c, r = ITEMS[f[:-4]]["cell"]; atlas.paste(Image.open(f"{ROOT}/icons/{f}"), (32 * c, 32 * r))
    atlas.save(f"{ROOT}/finished-v1.png")

if __name__ == "__main__":
    for f in sorted(os.listdir(f"{ROOT}/icons")):
        if f[:-4] in ITEMS and "regions" in ITEMS[f[:-4]] and ITEMS[f[:-4]].get("source") != "hand":
            print(f[:-4], snap(f[:-4]))
    build_atlas()
