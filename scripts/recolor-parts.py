"""Snap part icons onto the shared material palette, and derive material variants.

Every part is drawn in its base material (wood = ash, metal = iron, leather = deer; see
parts-catalog.json). This script re-samples each icon's non-outline pixels by lightness into
that material's ramp from material-catalog.json, so parts and raw materials share colours.

The same call with another ramp makes a material variant without new art, e.g.
`variant("straight_blade", "steel")` writes icons/variants/steel_straight_blade.png.
"""
import importlib.util, json, os
from PIL import Image

spec = importlib.util.spec_from_file_location("m", os.path.join(os.path.dirname(__file__), "recolor-metals.py"))
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
hls, R = m.hls, m.R
PARTS = "assets/items/parts"
CAT = json.load(open(f"{PARTS}/parts-catalog.json"))
ITEMS = {i["id"]: i for i in CAT["items"]}
# Worked wood is heartwood: the cut-end ramp without a separate pale sapwood stop, plus one
# shadow tone made by darkening the deepest heartwood colour (bark colour is never used, so a
# white-barked birch still gets wood-coloured shadows).
SAPWOOD_LAST = {"walnut", "cherry", "yew", "birch"}
def WOOD(sp):
    heart = R[f"{sp}-end"][:-1] if sp in SAPWOOD_LAST else R[f"{sp}-end"]
    d = min((m.rgb(h) for h in heart), key=lambda c: hls(c)[1])
    return ["#%02x%02x%02x" % tuple(int(v * 0.7) for v in d)] + heart
METAL = lambda mt: R[mt] + ["#c8ced2" if mt == "iron" else R[mt][-1]]
RAMPS = {"wood": WOOD, "metal": METAL, "leather": lambda a: R[f"{a}-leather"]}

def snap(src, dst, ramp):
    im = Image.open(src).convert("RGBA"); px = im.load()
    opaque = {(x, y) for y in range(32) for x in range(32) if px[x, y][3]}
    # the silhouette edge is always the shared 1px near-black outline
    edge = {(x, y) for (x, y) in opaque if any(q not in opaque for q in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)))}
    body = {p for p in opaque - edge if hls(px[p][:3])[1] >= 0.08}
    for p in edge: im.putpixel(p, m.rgb(R["outline"][0]) + (255,))
    px = im.load()
    ramp = sorted(ramp, key=lambda h: hls(m.rgb(h))[1])  # dark -> light, whatever the source order
    out = im.copy(); m.paint(out.load(), px, body, ramp)
    out.save(dst)

def base(id):
    it = ITEMS[id]; fam = it["family"]; mat = CAT["base_materials"][fam]
    p = f"{PARTS}/icons/{id}.png"
    snap(p, p, RAMPS[fam](mat))

def variant(id, material):
    fam = ITEMS[id]["family"]
    os.makedirs(f"{PARTS}/icons/variants", exist_ok=True)
    snap(f"{PARTS}/icons/{id}.png", f"{PARTS}/icons/variants/{material}_{id}.png", RAMPS[fam](material))

def build_atlas():
    atlas = Image.new("RGBA", tuple(CAT["atlas_size"]))
    for f in sorted(os.listdir(f"{PARTS}/icons")):
        if f[:-4] in ITEMS:
            c, r = ITEMS[f[:-4]]["cell"]; atlas.paste(Image.open(f"{PARTS}/icons/{f}"), (32 * c, 32 * r))
    atlas.save(f"{PARTS}/parts-v1.png")

if __name__ == "__main__":
    for f in sorted(os.listdir(f"{PARTS}/icons")):
        # derived icons are built from already-snapped bases by derive-parts.py
        if f[:-4] in ITEMS and "derived_from" not in ITEMS[f[:-4]]: base(f[:-4])
    build_atlas()
