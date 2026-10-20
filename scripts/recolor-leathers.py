"""Derive deer, bear and tiger leather by recolouring the approved wolf_leather roll.

No new generation. Regions:
- outline: near-black, kept as it is
- cord: the saturated tan tie, kept so every roll shares the same cord
- leather: re-sampled by lightness from the species ramp in material-catalog.json

Tiger keeps faint stripe ghosts, drawn by rule as darker diagonal dashes. Edge thickness is
the base's: a recolour cannot make bear leather thicker.
"""
import importlib.util, json, os
from PIL import Image

spec = importlib.util.spec_from_file_location("m", os.path.join(os.path.dirname(__file__), "recolor-metals.py"))
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
ICONS, R, hls, rgb = m.ICONS, m.R, m.hls, m.rgb

def leather(animal):
    im = Image.open(f"{ICONS}/wolf_leather.png").convert("RGBA")
    px = im.load(); W, H = im.size
    body = set()
    for y in range(H):
        for x in range(W):
            p = px[x, y]
            if not p[3]: continue
            h, l, s = hls(p[:3])
            if l >= 0.10 and s < 0.3: body.add((x, y))
    out = im.copy(); o = out.load()
    m.paint(o, px, body, R[f"{animal}-leather"])
    if animal == "tiger":
        ghost = rgb(R["tiger-leather"][0]) + (255,)
        for (x, y) in body:
            if (x + 2 * y) % 9 == 0 and hls(px[x, y][:3])[1] > 0.3:
                o[x, y] = ghost
                if (x + 1, y + 1) in body: o[x + 1, y + 1] = ghost
    out.save(f"{ICONS}/{animal}_leather.png")

if __name__ == "__main__":
    for a in ("deer", "bear", "tiger"):
        leather(a)
    m.build_atlas()
