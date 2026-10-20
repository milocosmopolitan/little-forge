"""Hand-drawn finished-good icons, drawn on the shared material palette with no generation.

Items marked "source": "hand" in finished-goods-catalog.json are drawn here. The clean and recolor
steps skip them, so re-running the pipeline never overwrites them. Each icon is built from masks
(haft, head), lit from the top-left with the wood and iron ramps, and closed with a 1px outline.

Order: clean-material-candidates.py --set finished -> draw-finished.py -> recolor-finished.py -> qa.
"""
import importlib.util, os
from PIL import Image

spec = importlib.util.spec_from_file_location("p", os.path.join(os.path.dirname(__file__), "recolor-parts.py"))
P = importlib.util.module_from_spec(spec); spec.loader.exec_module(P)
ROOT = "assets/items/finished"
OUTLINE = P.R["outline"][0]
WOOD = P.WOOD("ash")      # dark -> light: 0 shadow, 1 base, 2 light, 3 highlight
IRON = P.METAL("iron")    # dark -> light: 0..4

def rgb(h): return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5)) + (255,)

def finish(px):
    """px: {(x, y): hex}. Adds a 4-neighbour outline outside the filled pixels and centres the result."""
    out = dict(px)
    for (x, y) in px:
        for q in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if q not in px: out[q] = OUTLINE
    xs = [x for x, _ in out]; ys = [y for _, y in out]
    dx = (32 - (max(xs) - min(xs) + 1)) // 2 - min(xs)   # centre the drawing in the cell
    dy = (32 - (max(ys) - min(ys) + 1)) // 2 - min(ys)
    out = {(x + dx, y + dy): c for (x, y), c in out.items()}
    im = Image.new("RGBA", (32, 32))
    for (x, y), c in out.items():
        assert 1 <= x <= 30 and 1 <= y <= 30, f"pixel {x, y} breaks the 1px rim"
        im.putpixel((x, y), rgb(c))
    return im

def hand_axe():
    """Hatchet: short ash haft, small iron head with a flat poll and a short unbearded bit.
    Unlike the woodcutting axe (large dark bearded head on a long haft), the head is small and
    pale, the poll is shallow so it never reads as a second blade, and the bit ends in a bright
    honed bevel.

    The haft runs at 45 degrees, so the head is drawn square to it: every pixel centre is turned
    into head coordinates (u along the head, toward the bit; v along the haft, toward its top) and
    tested against the shapes below. That keeps the head at a right angle to the haft without
    rotating a bitmap."""
    r = 0.5 ** 0.5
    cx, cy = 18.5, 11.5                               # top of the haft, centre of the eye

    def uv(x, y):
        dx, dy = x + 0.5 - cx, y + 0.5 - cy
        return (dx + dy) * r, (dx - dy) * r          # u: down-right (bit), v: up-right (haft top)

    k = 1.15                                         # head scale against the haft

    def bit_half(u):                                 # half-width of the bit: concave cheeks, no beard
        return k * (2.0 + ((u / k - 2) / 6) ** 2 * 2.6)

    def edge(v):                                     # convex cutting edge
        return k * (8.2 + 1.2 * (1 - (v / k / 4.8) ** 2))

    def part(x, y):
        u, v = uv(x, y)
        if -1.6 * k <= u <= 2 * k and abs(v) <= 2.1 * k: return "eye"
        if -3.0 * k <= u < -1.6 * k and abs(v) <= 1.5 * k: return "poll"     # flat butt, narrower than the eye
        if 2 * k < u <= edge(v) and abs(v) <= bit_half(u): return "bit"
        if abs(u) <= 1.1 and -16 <= v <= 2.1 * k + 1.2: return "haft"   # runs through the eye, tip shows
        if abs(u) <= 1.6 and -18 <= v < -16: return "knob"
        return None

    parts = {(x, y): part(x, y) for x in range(-4, 36) for y in range(-4, 36)}
    parts = {p: k for p, k in parts.items() if k}
    px = {}
    for (x, y), kind in parts.items():
        u, v = uv(x, y)
        lit = (x - 1, y) not in parts or (x, y - 1) not in parts      # faces the top-left light
        dark = (x + 1, y) not in parts or (x, y + 1) not in parts
        if kind in ("haft", "knob"):
            px[(x, y)] = WOOD[3] if lit and u < 0 else WOOD[2] if u < 0 else WOOD[1] if not dark else WOOD[0]
        else:
            c = IRON[2]
            if kind == "bit" and u >= edge(v) - 1.8: c = IRON[3]        # honed bevel
            if kind == "bit" and u >= edge(v) - 0.8: c = IRON[4]
            if lit: c = IRON[4]
            elif dark: c = IRON[1]
            if kind == "eye" and 1.7 * k <= u: c = IRON[1]              # shoulder between eye and bit
            px[(x, y)] = c
    return finish(px)

DRAW = {"hand_axe": hand_axe}

if __name__ == "__main__":
    for id, fn in DRAW.items():
        fn().save(f"{ROOT}/icons/{id}.png"); print(id, "drawn")
