"""UI kit v2: hand-drawn nine-slice components for the timber / parchment / gold screens (#118).

Visual target: the owner's Orders and Weapon Collection references in assets/ui/references/.
Colours were sampled from those references and tuned onto the item palette's outline.
Everything is drawn here as code; nothing is generated or cut from the references.

Density: one art pixel = one logical pixel, the same density as the workshop scene and the
32x32 item icons. Bevy draws these with nearest sampling at integer UI scale, as nine-slices with
the insets recorded in assets/ui/kit-v2/manifest.json. Text is never baked into art.

Run: python3 scripts/draw-ui-kit-v2.py  (writes assets/ui/kit-v2/*.png and manifest.json)
"""
import hashlib, json, os
from PIL import Image

OUT = "assets/ui/kit-v2"

# Palette (sampled from the references, then snapped to a small ramp per material)
OUTLINE = "#0b0806"
WOOD = ["#1e1310", "#2e1d14", "#402416", "#4c2916", "#6a3d22"]        # frame and rail planks
WOOD_LIT = ["#5a3520", "#6a4024", "#7a4a2a"]                         # selected / hover plank
IRON = ["#1a1a1e", "#3a3a40", "#5c5c64", "#8e8e96", "#c0c0c6"]         # brackets, rivets
PARCH = ["#a07a4a", "#c9a06a", "#dcbb88", "#ecd1a6", "#f4e0bc"]       # line, edge, mid, fill, light
GOLD = ["#8a5a1e", "#c48a32", "#f2c37f", "#ffe6a8"]                  # selected rim, glow
LINE = ["#5a3a20", "#8a6038", "#b08a5a"]                             # thin brown rules on parchment
SLOT = ["#1e130c", "#2a1c13", "#3a2818", "#4a3420", "#6a4a2c"]
GREEN = ["#2e3824", "#5a6446", "#7f8a64", "#a4ae86"]                 # "daily" tag
RED = ["#4a1c12", "#8a3a28", "#b3624a", "#d08a70"]                   # "special" tag, deadline


def rgba(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5)) + (255,)


class Canvas:
    def __init__(self, w, h):
        self.w, self.h = w, h
        self.im = Image.new("RGBA", (w, h))

    def px(self, x, y, c):
        if 0 <= x < self.w and 0 <= y < self.h:
            self.im.putpixel((x, y), rgba(c))

    def rect(self, x0, y0, x1, y1, c):
        """Filled rectangle, inclusive corners."""
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                self.px(x, y, c)

    def box(self, x0, y0, x1, y1, c):
        """1px rectangle outline, inclusive corners."""
        for x in range(x0, x1 + 1):
            self.px(x, y0, c); self.px(x, y1, c)
        for y in range(y0, y1 + 1):
            self.px(x0, y, c); self.px(x1, y, c)

    def bevel(self, x0, y0, x1, y1, lit, dark):
        """Top-left light, bottom-right shade, on a 1px ring."""
        for x in range(x0, x1 + 1):
            self.px(x, y0, lit); self.px(x, y1, dark)
        for y in range(y0 + 1, y1):
            self.px(x0, y, lit); self.px(x1, y, dark)

    def clear(self, pts):
        for x, y in pts:
            if 0 <= x < self.w and 0 <= y < self.h:
                self.im.putpixel((x, y), (0, 0, 0, 0))

    def round_corners(self, r=1):
        """Cut r pixels diagonally from each outer corner (soft, non-square ends)."""
        pts = []
        for i in range(r):
            for j in range(r - i):
                for cx, cy in ((i, j), (self.w - 1 - i, j), (i, self.h - 1 - j), (self.w - 1 - i, self.h - 1 - j)):
                    pts.append((cx, cy))
        self.clear(pts)


# ---- components -------------------------------------------------------------------------------

def window_frame():
    """Outer timber frame with iron corner brackets. Interior is the dark backing the rail sits on."""
    W = H = 48; I = 16
    c = Canvas(W, H)
    c.rect(0, 0, W - 1, H - 1, OUTLINE)
    c.rect(1, 1, W - 2, H - 2, WOOD[3])
    # plank grain runs along each edge, so stretching the edge keeps it straight
    for y in (3, 7, 10):
        for x in range(1, W - 1):
            c.px(x, y, WOOD[2]); c.px(x, H - 1 - y, WOOD[2])
    for x in (3, 7, 10):
        for y in range(1, H - 1):
            c.px(x, y, WOOD[2]); c.px(W - 1 - x, y, WOOD[2])
    c.bevel(1, 1, W - 2, H - 2, WOOD[4], WOOD[1])
    c.box(12, 12, W - 13, H - 13, OUTLINE)                  # inner lip
    c.bevel(13, 13, W - 14, H - 14, WOOD[0], WOOD[2])        # recessed: dark top-left inside
    c.rect(14, 14, W - 15, H - 15, WOOD[1])                  # backing
    for (bx, by, sx, sy) in ((0, 0, 1, 1), (W - 1, 0, -1, 1), (0, H - 1, 1, -1), (W - 1, H - 1, -1, -1)):
        # L-shaped iron bracket, 13 long, 5 wide, with two rivets per arm
        for i in range(13):
            for j in range(5):
                for (x, y) in ((bx + sx * i, by + sy * j), (bx + sx * j, by + sy * i)):
                    c.px(x, y, IRON[1])
        for i in range(13):
            c.px(bx + sx * i, by + sy * 5, OUTLINE); c.px(bx + sx * 5, by + sy * i, OUTLINE)
            c.px(bx + sx * i, by, OUTLINE); c.px(bx, by + sy * i, OUTLINE)
        c.px(bx + sx * 12, by + sy * 1, OUTLINE); c.px(bx + sx * 1, by + sy * 12, OUTLINE)
        for i in range(1, 12):                                # top-left light on the plate
            c.px(bx + sx * i, by + sy * 1, IRON[2] if sy > 0 else IRON[1])
            c.px(bx + sx * 1, by + sy * i, IRON[2] if sx > 0 else IRON[1])
        for (rx, ry) in ((2, 2), (9, 2), (2, 9)):
            x, y = bx + sx * rx, by + sy * ry
            c.px(x, y, IRON[4]); c.px(x + sx, y, IRON[2]); c.px(x, y + sy, IRON[2]); c.px(x + sx, y + sy, IRON[0])
    return c, (I, I, I, I)


def parchment_panel():
    """Parchment page: timber rim, darker parchment edge band, thin double rule with corner ticks."""
    W = H = 36; I = 12
    c = Canvas(W, H)
    c.rect(0, 0, W - 1, H - 1, OUTLINE)
    c.bevel(1, 1, W - 2, H - 2, WOOD[4], WOOD[2])
    c.bevel(2, 2, W - 3, H - 3, WOOD[3], WOOD[2])
    c.rect(3, 3, W - 4, H - 4, PARCH[1])
    c.rect(4, 4, W - 5, H - 5, PARCH[2])
    c.rect(6, 6, W - 7, H - 7, PARCH[3])
    c.box(7, 7, W - 8, H - 8, LINE[2])                       # outer rule
    c.box(9, 9, W - 10, H - 10, LINE[2])                     # inner rule
    c.rect(10, 10, W - 11, H - 11, PARCH[3])
    for (x, y) in ((8, 8), (W - 9, 8), (8, H - 9), (W - 9, H - 9)):
        c.px(x, y, LINE[1])                                  # corner ticks between the rules
    c.px(4, 4, PARCH[4]); c.px(5, 4, PARCH[4]); c.px(4, 5, PARCH[4])   # top-left light catch
    return c, (I, I, I, I)


def card(selected=False):
    """List row / grid card on parchment. Selected: gold rim with a 1px glow and corner studs."""
    if not selected:
        W = H = 16; I = 4
        c = Canvas(W, H)
        c.rect(0, 0, W - 1, H - 1, LINE[2])
        c.rect(1, 1, W - 2, H - 2, PARCH[3])
        c.rect(1, H - 2, W - 2, H - 2, PARCH[2])             # bottom shade
        c.rect(W - 2, 1, W - 2, H - 2, PARCH[2])
        c.px(1, 1, PARCH[4]); c.px(2, 1, PARCH[4]); c.px(1, 2, PARCH[4])
        c.round_corners(1)
        return c, (I, I, I, I)
    W = H = 20; I = 6
    c = Canvas(W, H)
    c.rect(0, 0, W - 1, H - 1, GOLD[3])                      # glow
    c.rect(1, 1, W - 2, H - 2, GOLD[1])
    c.rect(2, 2, W - 3, H - 3, GOLD[2])
    c.bevel(2, 2, W - 3, H - 3, GOLD[3], GOLD[1])
    c.rect(3, 3, W - 4, H - 4, PARCH[4])
    c.rect(3, H - 4, W - 4, H - 4, PARCH[3])
    for (x, y) in ((3, 3), (W - 4, 3), (3, H - 4), (W - 4, H - 4)):
        c.px(x, y, GOLD[2])                                  # corner studs
    c.round_corners(2)
    return c, (I, I, I, I)


def rail_tab(state):
    """Left rail category plank. normal / hover / selected."""
    W = H = 24; I = 8
    c = Canvas(W, H)
    if state == "selected":
        c.rect(0, 0, W - 1, H - 1, GOLD[3])                  # glow
        c.rect(1, 1, W - 2, H - 2, GOLD[1])
        c.bevel(2, 2, W - 3, H - 3, GOLD[3], GOLD[1])
        c.box(3, 3, W - 4, H - 4, GOLD[2])
        c.rect(4, 4, W - 5, H - 5, WOOD_LIT[1])
        c.rect(4, 4, W - 5, 5, WOOD_LIT[2])                  # lit top band
        c.rect(4, H - 6, W - 5, H - 5, WOOD_LIT[0])
        for (x, y) in ((3, 3), (W - 4, 3), (3, H - 4), (W - 4, H - 4)):
            c.px(x, y, GOLD[3])
        c.round_corners(2)
        return c, (I, I, I, I)
    fill, rule = (WOOD[2], LINE[2]) if state == "hover" else (WOOD[1], LINE[1])
    c.rect(0, 0, W - 1, H - 1, OUTLINE)
    c.bevel(1, 1, W - 2, H - 2, WOOD[3], WOOD[0])
    c.rect(2, 2, W - 3, H - 3, fill)
    c.box(3, 3, W - 4, H - 4, rule)                          # inset rule with clipped corners
    for (x, y) in ((3, 3), (W - 4, 3), (3, H - 4), (W - 4, H - 4)):
        c.px(x, y, fill)
    for (x, y) in ((4, 4), (W - 5, 4), (4, H - 5), (W - 5, H - 5)):
        c.px(x, y, rule)
    c.round_corners(1)
    return c, (I, I, I, I)


def button(state, primary=False):
    """Timber button. normal / hover / pressed / disabled. Primary (the main action) has a warm
    plank and a 2px gold rim, so it reads apart from the plain buttons beside it."""
    W, H = 20, 18; I = 6
    c = Canvas(W, H)
    if state == "disabled":
        c.rect(0, 0, W - 1, H - 1, OUTLINE)
        c.bevel(1, 1, W - 2, H - 2, "#6a605a", "#4a4240")
        c.rect(2, 2, W - 3, H - 3, "#3a3230")
        c.round_corners(1)
        return c, (I, I, I, I)
    if primary:
        fill = {"normal": WOOD_LIT[0], "hover": WOOD_LIT[1], "pressed": WOOD[2]}[state]
        c.rect(0, 0, W - 1, H - 1, GOLD[3] if state == "hover" else OUTLINE)
        c.bevel(1, 1, W - 2, H - 2, GOLD[2], GOLD[1])
        c.bevel(2, 2, W - 3, H - 3, GOLD[1], GOLD[0])
        c.rect(3, 3, W - 4, H - 4, fill)
        if state == "pressed":
            c.rect(3, 3, W - 4, 3, WOOD[1])
        else:
            c.rect(3, 3, W - 4, 3, WOOD_LIT[2])
        c.round_corners(1)
        return c, (I, I, I, I)
    fill = {"normal": WOOD[2], "hover": WOOD_LIT[0], "pressed": WOOD[1]}[state]
    c.rect(0, 0, W - 1, H - 1, OUTLINE)
    c.bevel(1, 1, W - 2, H - 2, LINE[2], LINE[1])
    c.rect(2, 2, W - 3, H - 3, fill)
    if state == "pressed":
        c.rect(2, 2, W - 3, 2, WOOD[0])                      # pressed: shadow on top, no lit band
    else:
        c.rect(2, 2, W - 3, 2, WOOD_LIT[2] if state == "hover" else WOOD[3])
        c.rect(2, H - 3, W - 3, H - 3, WOOD[0])
    c.round_corners(1)
    return c, (I, I, I, I)


def slot(state):
    """Recessed item slot for 32px icons (content box = slot minus 4px each side)."""
    W = H = 16; I = 5
    c = Canvas(W, H)
    if state == "selected":
        c.rect(0, 0, W - 1, H - 1, GOLD[3])
        c.bevel(1, 1, W - 2, H - 2, GOLD[2], GOLD[1])
    else:
        c.rect(0, 0, W - 1, H - 1, OUTLINE)
        c.bevel(1, 1, W - 2, H - 2, SLOT[0], SLOT[4] if state == "normal" else SLOT[2])
    c.rect(2, 2, W - 3, H - 3, SLOT[1] if state != "locked" else SLOT[0])
    c.bevel(2, 2, W - 3, H - 3, SLOT[0], SLOT[3] if state != "locked" else SLOT[1])
    return c, (I, I, I, I)


def tag(kind):
    """Small pill behind a Bevy text label: daily (green), special (red), rank (dark), gold."""
    ramp = {"daily": GREEN, "special": RED, "rank": [OUTLINE, WOOD[1], WOOD[2], LINE[2]], "gold": GOLD}[kind]
    W, H = 12, 12; I = 4
    c = Canvas(W, H)
    c.rect(0, 0, W - 1, H - 1, ramp[0])
    c.bevel(1, 1, W - 2, H - 2, ramp[3], ramp[1])
    c.rect(2, 2, W - 3, H - 3, ramp[2] if kind != "gold" else ramp[1])
    c.round_corners(2)
    return c, (I, I, I, I)


def progress(kind):
    W, H = 12, 8; I = 3
    c = Canvas(W, H)
    if kind == "track":
        c.rect(0, 0, W - 1, H - 1, OUTLINE)
        c.rect(1, 1, W - 2, H - 2, SLOT[1])
        c.rect(1, 1, W - 2, 1, SLOT[0])
    else:                                                    # gold fill, drawn inside the track
        c.rect(0, 0, W - 1, H - 1, GOLD[1])
        c.rect(0, 0, W - 1, 1, GOLD[3])
        c.rect(0, 2, W - 1, 3, GOLD[2])
        c.rect(0, H - 1, W - 1, H - 1, GOLD[0])
    return c, (I, I, I, I)


def scrollbar(kind):
    W, H = 8, 12; I = 3
    c = Canvas(W, H)
    if kind == "track":
        c.rect(0, 0, W - 1, H - 1, LINE[1])
        c.rect(1, 1, W - 2, H - 2, PARCH[1])
        c.rect(1, 1, 1, H - 2, PARCH[0])
    else:
        c.rect(0, 0, W - 1, H - 1, OUTLINE)
        c.bevel(1, 1, W - 2, H - 2, WOOD[4], WOOD[1])
        c.rect(2, 2, W - 3, H - 3, WOOD[3])
        c.round_corners(1)
    return c, (I, I, I, I)


def divider():
    """Thin brown rule with a diamond end cap; stretch the middle."""
    W, H = 16, 5; I = (6, 0, 6, 0)
    c = Canvas(W, H)
    for x in range(3, W - 3):
        c.px(x, 2, LINE[1])
    for x0 in (0, W - 5):
        c.px(x0 + 2, 0, LINE[1]); c.px(x0 + 1, 1, LINE[1]); c.px(x0 + 3, 1, LINE[1])
        c.px(x0, 2, LINE[1]); c.px(x0 + 2, 2, PARCH[4]); c.px(x0 + 4, 2, LINE[1])
        c.px(x0 + 1, 3, LINE[1]); c.px(x0 + 3, 3, LINE[1]); c.px(x0 + 2, 4, LINE[1])
        c.px(x0 + 2, 1, LINE[2]); c.px(x0 + 1, 2, LINE[2]); c.px(x0 + 3, 2, LINE[2]); c.px(x0 + 2, 3, LINE[2])
    return c, I


def sparkle():
    """Four-point glint beside the selected rail tab (decoration, not a slice)."""
    c = Canvas(9, 9)
    for i in range(9):
        c.px(4, i, GOLD[2]); c.px(i, 4, GOLD[2])
    for p in ((4, 3), (4, 5), (3, 4), (5, 4)):
        c.px(*p, GOLD[3])
    c.px(4, 4, "#ffffff")
    c.clear([(4, 0), (4, 8), (0, 4), (8, 4)])
    for p in ((4, 1), (4, 7), (1, 4), (7, 4)):
        c.px(*p, GOLD[1])
    return c, None


def glyph(rows, colours):
    """Small inline glyph from a character map ('.' = transparent)."""
    c = Canvas(len(rows[0]), len(rows))
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch != ".":
                c.px(x, y, colours[ch])
    return c, None


GLYPHS = {
    # coin: gold disc, top-left light, stamped centre
    "glyph-coin": ([
        "...oooo...",
        "..o3322o..",
        ".o332221o.",
        "o33211221o",
        "o32122211o",
        "o32122211o",
        "o22111111o",
        ".o221111o.",
        "..o1111o..",
        "...oooo...",
    ], {"o": OUTLINE, "1": GOLD[0], "2": GOLD[1], "3": GOLD[2]}),
    # hourglass: deadline marker, red frame with sand
    "glyph-hourglass": ([
        "oooooooo",
        "o222222o",
        ".o3333o.",
        "..o33o..",
        "...oo...",
        "..o.3o..",
        ".o.33.o.",
        "o233332o",
        "oooooooo",
    ], {"o": OUTLINE, "2": RED[1], "3": GOLD[2]}),
}


def star(filled):
    """Five-point star, 9x9."""
    rows = [
        "....o....",
        "...o3o...",
        "...o3o...",
        "ooo332ooo",
        "o3332221o",
        ".o32221o.",
        "..o2o21o.",
        ".o2o.o1o.",
        ".oo...oo.",
    ]
    ramp = {"o": OUTLINE, "1": GOLD[0], "2": GOLD[1], "3": GOLD[2]} if filled else \
           {"o": LINE[1], "1": PARCH[1], "2": PARCH[2], "3": PARCH[2]}
    return glyph(rows, ramp)


COMPONENTS = {
    "window-frame": window_frame,
    "panel-parchment": parchment_panel,
    "card": lambda: card(False),
    "card-selected": lambda: card(True),
    "tab-normal": lambda: rail_tab("normal"),
    "tab-hover": lambda: rail_tab("hover"),
    "tab-selected": lambda: rail_tab("selected"),
    "button-normal": lambda: button("normal"),
    "button-hover": lambda: button("hover"),
    "button-pressed": lambda: button("pressed"),
    "button-disabled": lambda: button("disabled"),
    "button-primary-normal": lambda: button("normal", True),
    "button-primary-hover": lambda: button("hover", True),
    "button-primary-pressed": lambda: button("pressed", True),
    "slot-normal": lambda: slot("normal"),
    "slot-selected": lambda: slot("selected"),
    "slot-locked": lambda: slot("locked"),
    "tag-daily": lambda: tag("daily"),
    "tag-special": lambda: tag("special"),
    "tag-rank": lambda: tag("rank"),
    "tag-gold": lambda: tag("gold"),
    "progress-track": lambda: progress("track"),
    "progress-fill": lambda: progress("fill"),
    "scrollbar-track": lambda: scrollbar("track"),
    "scrollbar-thumb": lambda: scrollbar("thumb"),
    "divider": divider,
    "sparkle": sparkle,
    "glyph-coin": lambda: glyph(*GLYPHS["glyph-coin"]),
    "glyph-hourglass": lambda: glyph(*GLYPHS["glyph-hourglass"]),
    "glyph-star": lambda: star(True),
    "glyph-star-empty": lambda: star(False),
}

# Colours Bevy uses for text drawn on this kit (never baked into the art)
TEXT = {
    "on_parchment": LINE[0], "on_parchment_muted": "#7a5e40", "on_wood": PARCH[3],
    "on_wood_muted": "#b89a74", "deadline": RED[1], "gold": GOLD[2],
}


def build():
    os.makedirs(OUT, exist_ok=True)
    assets = []
    for name, fn in COMPONENTS.items():
        c, insets = fn()
        path = f"{OUT}/{name}.png"
        c.im.save(path)
        colours = {p[:3] for p in c.im.getdata() if p[3]}
        alphas = {p[3] for p in c.im.getdata()}
        assert alphas <= {0, 255}, name
        assets.append({
            "name": name, "path": path, "width": c.w, "height": c.h, "colors": len(colours),
            "slice_ltrb": list(insets) if insets else None,
            "sha256": hashlib.sha256(open(path, "rb").read()).hexdigest(),
        })
    manifest = {
        "schema": 1, "issue": 118, "kit": "v2-timber-parchment",
        "status": "hand-drawn-candidate-pending-owner-approval",
        "density": "1 art px = 1 logical px (same as the workshop scene and 32px item icons); nearest sampling",
        "references": ["assets/ui/references/orders-reference.jpg", "assets/ui/references/weapon-collection-reference.jpg"],
        "text_colors": TEXT,
        "slot_content_inset": 4,
        "assets": assets,
    }
    json.dump(manifest, open(f"{OUT}/manifest.json", "w"), indent=2)
    open(f"{OUT}/manifest.json", "a").write("\n")
    return manifest


if __name__ == "__main__":
    m = build()
    for a in m["assets"]:
        print(f'{a["name"]:24s} {a["width"]}x{a["height"]} {a["colors"]:2d} colours slice={a["slice_ltrb"]}')
