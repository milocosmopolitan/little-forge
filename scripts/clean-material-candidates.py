import json, os, sys
from PIL import Image
import argparse
_a = argparse.ArgumentParser(description="Clean Sprite Fusion picks into 32x32 icons and rebuild the atlas.")
_a.add_argument("--set", default="materials", choices=["materials", "parts"])
_set = _a.parse_args().set
ROOT = f"assets/items/{_set}"
SRC = f"{ROOT}/source/spritefusion-p0"
OUT = f"{ROOT}/icons"
GROUND = 30            # last opaque row for resting families
REST = {"ore","ingot","log","leather","fuel"}
cat = json.load(open(f"{ROOT}/" + ("material-catalog.json" if _set == "materials" else "parts-catalog.json")))
items = {i["id"]: i for i in cat["items"]}
def kmeans(pts, k, it=30):
    pts.sort(key=lambda t: -t[1])
    cents = [list(pts[0][0])]
    while len(cents) < min(k, len(pts)):   # farthest-point init keeps outliers (accents)
        far = max(pts, key=lambda t: min(sum((a-b)**2 for a,b in zip(t[0],c)) for c in cents) * t[1])
        cents.append(list(far[0]))
    for _ in range(it):
        acc = [[0,0,0,0] for _ in cents]
        for c, w in pts:
            j = min(range(len(cents)), key=lambda i: sum((a-b)**2 for a,b in zip(c,cents[i])))
            for d in range(3): acc[j][d] += c[d]*w
            acc[j][3] += w
        cents = [[round(a[d]/a[3]) for d in range(3)] if a[3] else cents[i] for i,a in enumerate(acc)]
    # snap each centroid to the nearest real source colour so no invented in-between tones
    real = [c for c,_ in pts]
    cents = [min(real, key=lambda r: sum((a-b)**2 for a,b in zip(r,c))) for c in cents]
    img = Image.new("P", (1,1)); flat = [v for c in cents for v in c]
    img.putpalette(flat + flat[:3]*(256-len(cents))); return img
import os; os.makedirs(OUT, exist_ok=True)
atlas = Image.new("RGBA", tuple(cat["atlas_size"]))
report = []
for id in sorted(f[:-4] for f in os.listdir(SRC) if f.endswith(".png")):
    im = Image.open(f"{SRC}/{id}.png").convert("RGBA")
    a = im.getchannel("A")
    rgb = Image.new("RGB", im.size, (0, 0, 0)); rgb.paste(im.convert("RGB"), mask=a)
    # palette from opaque pixels only, no dithering
    # k-means over pixels weighted by sqrt(frequency): large areas still lead, small accents
    # (rust vein, cord, glints) keep their own cluster instead of merging into the mass.
    from collections import Counter
    cnt = Counter(p[:3] for p in im.getdata() if p[3])
    pts = [(c, max(1, round(n ** 0.5))) for c, n in cnt.items()]
    pal = kmeans(pts, 16)
    q = rgb.quantize(palette=pal, dither=Image.Dither.NONE).convert("RGB")
    out = Image.new("RGBA", im.size, (0, 0, 0, 0)); out.paste(q, mask=a)
    # integer re-placement: centre horizontally, rest on GROUND or centre vertically
    x0, y0, x1, y1 = out.getbbox()
    crop = out.crop((x0, y0, x1, y1))
    if crop.width > 30:
        # Fill-the-canvas picks (the leather rolls): drop the outermost column on each side
        # and redraw the new edge column in the icon's darkest colour, keeping a 1px outline.
        dark = min({p[:3] for p in crop.getdata() if p[3]}, key=sum)
        px = crop.load(); W = crop.width
        for xd, xe in ((0, 1), (W - 1, W - 2)):
            for y in range(crop.height):
                if px[xe, y][3]: px[xe, y] = dark + (255,)
        crop = crop.crop((1, 0, W - 1, crop.height))
    while crop.height > 30:
        # Too tall (ear tips, tails): drop the sparser end row and outline the new edge row.
        dark = min({p[:3] for p in crop.getdata() if p[3]}, key=sum)
        px = crop.load(); W, H = crop.size
        count = lambda y: sum(1 for x in range(W) if px[x, y][3])
        top = count(0) <= count(H - 1)
        drop, edge = (0, 1) if top else (H - 1, H - 2)
        for x in range(W):
            if px[x, edge][3] and px[x, drop][3]: px[x, edge] = dark + (255,)
        crop = crop.crop((0, 1, W, H) if top else (0, 0, W, H - 1))
    w, h = crop.size
    nx = (32 - w) // 2
    ny = GROUND + 1 - h if (items[id].get("rest") or items[id]["family"] in REST) else (32 - h) // 2
    ny = max(ny, 0)
    final = Image.new("RGBA", (32, 32)); final.paste(crop, (nx, ny))
    final.save(f"{OUT}/{id}.png")
    c, r = items[id]["cell"]; atlas.paste(final, (32 * c, 32 * r))
    fb = final.getbbox()
    ok = fb[0] >= 1 and fb[1] >= 1 and fb[2] <= 31 and fb[3] <= 31
    report.append((id, (w, h), fb, len({p[:3] for p in final.getdata() if p[3]}), "OK" if ok else "RIM"))
# Rebuild the atlas from every icon, including ones derived by scripts/recolor-logs.py.
atlas = Image.new("RGBA", tuple(cat["atlas_size"]))
for f in sorted(os.listdir(OUT)):
    if f[:-4] in items:
        c, r = items[f[:-4]]["cell"]; atlas.paste(Image.open(f"{OUT}/{f}"), (32 * c, 32 * r))
atlas.save(f"{ROOT}/{_set}-v1.png")
for r in report: print(*r)

