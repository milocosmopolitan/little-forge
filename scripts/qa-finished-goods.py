"""QA images for the finished-goods atlas: 3x atlas, 1x/2x/3x on dark and light slots, and the
P0 goods next to the parts they are made from. Checks the rim and binary alpha first."""
import json, os
from PIL import Image

ROOT = "assets/items/finished"
OUT = "docs/qa/evidence/finished-goods"
cat = json.load(open(f"{ROOT}/finished-goods-catalog.json"))
os.makedirs(OUT, exist_ok=True)
atlas = Image.open(f"{ROOT}/finished-v1.png")
done = [i for i in cat["items"] if os.path.exists(f"{ROOT}/icons/{i['id']}.png")]
for it in done:
    im = Image.open(f"{ROOT}/icons/{it['id']}.png"); b = im.getbbox()
    assert b[0] >= 1 and b[1] >= 1 and b[2] <= 31 and b[3] <= 31, (it["id"], b)
    assert {im.getpixel((x, y))[3] for y in range(32) for x in range(32)} <= {0, 255}, it["id"]
    assert len({im.getpixel((x, y)) for y in range(32) for x in range(32)} - {(0, 0, 0, 0)}) <= 16, it["id"]

DARK, LIGHT = (60, 52, 46, 255), (226, 214, 190, 255)
bg = Image.new("RGBA", atlas.size, DARK); bg.alpha_composite(atlas)
bg.resize((atlas.width * 3, atlas.height * 3), Image.NEAREST).save(f"{OUT}/finished-atlas-3x.png")

rows = sorted({i["cell"][1] for i in done})
col_w, gap = 32 * 3 + 8, 8
band_h = 8 + sum(32 * s + 4 for s in (1, 2, 3)) + 4
sheet = Image.new("RGBA", (8 * col_w + gap, band_h * 2 * len(rows)), DARK)
y = 0
for r in rows:
    for bgc in (DARK, LIGHT):
        band = Image.new("RGBA", (sheet.width, band_h), bgc); yy = 8
        for s in (1, 2, 3):
            for it in (i for i in done if i["cell"][1] == r):
                c = it["cell"][0]
                ic = atlas.crop((32 * c, 32 * r, 32 * c + 32, 32 * r + 32)).resize((32 * s, 32 * s), Image.NEAREST)
                band.alpha_composite(ic, (gap + c * col_w, yy))
            yy += 32 * s + 4
        sheet.alpha_composite(band, (0, y)); y += band_h
sheet.save(f"{OUT}/scales-dark-light.png")
print(f"{len(done)} icons OK ->", OUT)
