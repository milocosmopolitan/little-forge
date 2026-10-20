"""Layout mockups for UI kit v2 at the real size (#118).

Renders the Orders and Weapon Collection screens in the upper 960x360 management area, on top of
the live 960x180 workshop strip, using only repository art:
- the kit-v2 nine-slices (scripts/draw-ui-kit-v2.py),
- finished goods, parts and material icons at 1x (32px), NPC heads cropped from neutral.png,
- recipes from finished-goods-catalog.json.

Text is drawn the way Bevy would draw it (Noto Sans KR, not baked into the kit). Rewards, counts,
deadlines and quality stars are sample values: these are layout mockups, not production panels.

Run: python3 scripts/mockup-ui-kit-v2.py  -> docs/qa/evidence/ui-kit-v2/
"""
import json, os
from PIL import Image, ImageDraw, ImageFont

KIT = "assets/ui/kit-v2"
OUT = "docs/qa/evidence/ui-kit-v2"
FONT = "assets/ui/fonts/NotoSansKR.ttf"
WORKSHOP = "docs/qa/evidence/atlas-v2/composites/male-forging.png"
M = json.load(open(f"{KIT}/manifest.json"))
SLICES = {a["name"]: a for a in M["assets"]}
T = M["text_colors"]
GOODS = {i["id"]: i for i in json.load(open("assets/items/finished/finished-goods-catalog.json"))["items"]}
PARTS = {i["id"]: i for i in json.load(open("assets/items/parts/parts-catalog.json"))["items"]}


def font(size, weight=500):
    f = ImageFont.truetype(FONT, size)
    try:
        f.set_variation_by_axes([weight])
    except Exception:
        pass
    return f


def kit(name):
    return Image.open(f"{KIT}/{name}.png").convert("RGBA")


def nine(name, w, h):
    """Nine-slice with stretched edges and centre (nearest), integer pixels only."""
    a = SLICES[name]; src = kit(name)
    l, t, r, b = a["slice_ltrb"]; W, H = src.size
    out = Image.new("RGBA", (w, h))
    xs = [(0, l, 0, l), (l, W - r, l, w - r), (W - r, W, w - r, w)]
    ys = [(0, t, 0, t), (t, H - b, t, h - b), (H - b, H, h - b, h)]
    for sx0, sx1, dx0, dx1 in xs:
        for sy0, sy1, dy0, dy1 in ys:
            if sx1 > sx0 and sy1 > sy0 and dx1 > dx0 and dy1 > dy0:
                out.paste(src.crop((sx0, sy0, sx1, sy1)).resize((dx1 - dx0, dy1 - dy0), Image.NEAREST), (dx0, dy0))
    return out


class Screen:
    def __init__(self):
        self.im = Image.new("RGBA", (960, 360))
        self.d = ImageDraw.Draw(self.im)

    def put(self, img, x, y):
        self.im.alpha_composite(img, (x, y))

    def nine(self, name, x, y, w, h):
        self.put(nine(name, w, h), x, y)

    def text(self, x, y, s, size=13, colour="on_parchment", weight=500, anchor="la"):
        self.d.text((x, y), s, font=font(size, weight), fill=T.get(colour, colour), anchor=anchor)

    def width(self, s, size, weight=500):
        return self.d.textlength(s, font=font(size, weight))

    def tag(self, x, y, label, kind):
        w = int(self.width(label, 11, 600)) + 14
        self.nine(f"tag-{kind}", x, y, w, 18)
        self.text(x + w // 2, y + 9, label, 11, "on_wood", 600, "mm")
        return w

    def slot(self, x, y, size, icon=None, state="normal", scale=1, silhouette=False):
        self.nine(f"slot-{state}", x, y, size, size)
        if icon:
            im = Image.open(icon).convert("RGBA")
            if scale != 1:
                im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
            if silhouette:
                a = im.getchannel("A")
                im = Image.new("RGBA", im.size, (92, 66, 42, 255)); im.putalpha(a)
            self.put(im, x + (size - im.width) // 2, y + (size - im.height) // 2)

    def stars(self, x, y, n, of=3):
        for i in range(of):
            self.put(kit("glyph-star" if i < n else "glyph-star-empty"), x + i * 11, y)

    def rail(self, tabs, selected, tab_h, gap, y0=20):
        for i, (label, count) in enumerate(tabs):
            y = y0 + i * (tab_h + gap)
            sel = i == selected
            self.nine("tab-selected" if sel else "tab-normal", 18, y, 172, tab_h)
            big = tab_h >= 52
            self.text(36, y + (12 if big else tab_h // 2), label, 16 if big else 14, "on_wood", 700, "la" if big else "lm")
            if big:
                self.text(36, y + 36, count, 13, "on_wood_muted" if not sel else "on_wood", 500)
            else:
                self.text(176, y + tab_h // 2, count, 12, "on_wood_muted" if not sel else "on_wood", 500, "rm")
            if sel:
                self.put(kit("sparkle"), 14, y + tab_h // 2 - 4)

    def scrollbar(self, x, y, h, frac=0.5, top=0.0):
        self.nine("scrollbar-track", x, y, 8, h)
        th = max(16, int(h * frac))
        self.nine("scrollbar-thumb", x, y + int((h - th) * top), 8, th)

    def mock_label(self):
        self.text(480, 7, "목업 · 예시 수치", 10, "on_wood_muted", 500, "mm")

    def fit(self, s, size, maxw, weight=500):
        """Cut a line to maxw with an ellipsis, as the Bevy label will."""
        if self.width(s, size, weight) <= maxw:
            return s
        while s and self.width(s + "…", size, weight) > maxw:
            s = s[:-1]
        return s + "…"


def icon(kind, id):
    return f"assets/items/{kind}/icons/{id}.png"


def part_ids(item):
    return [r.split(" x")[0] for r in GOODS[item]["recipe"]]


def portrait(npc, size=48):
    """Head-and-shoulders crop of an existing NPC's neutral frame, 1x, no resampling."""
    im = Image.open(f"assets/npc/{npc}/neutral.png").convert("RGBA")
    x0, y0, x1, y1 = im.getbbox()
    cx = (x0 + x1) // 2
    crop = im.crop((cx - size // 2, y0 - 2, cx + size // 2, y0 - 2 + size))
    return crop


def orders():
    s = Screen()
    s.nine("window-frame", 0, 0, 960, 360)
    s.rail([("전체", "6 / 8"), ("일상 의뢰", "4 / 5"), ("특별 의뢰", "2 / 2"), ("완료 기록", "12")], 0, 60, 6)

    # centre: order list
    x, y, w, h = 198, 14, 392, 332
    s.nine("panel-parchment", x, y, w, h)
    s.text(x + 18, y + 16, "진행 중인 의뢰", 17, "on_parchment", 700)
    s.nine("button-normal", x + w - 112, y + 12, 94, 26)
    s.text(x + w - 72, y + 25, "최신순", 12, "on_wood", 600, "mm")
    tx, ty = x + w - 42, y + 23                       # drop-down caret, drawn (the font has no glyph)
    s.d.polygon([(tx, ty), (tx + 8, ty), (tx + 4, ty + 4)], fill=T["on_wood"])
    s.nine("divider", x + 14, y + 44, w - 28, 5)
    rows = [
        ("chef", "요리사 브루노", "요리사의 식칼", "kitchen_knife", "“좋은 식칼이 있으면 요리가 더 맛있어지지!”", "320", "3일 남음"),
        ("farmer", "농부 에린", "농부의 괭이", "farming_hoe", "“밭을 일굴 튼튼한 괭이가 필요해요.”", "180", "2일 남음"),
        ("carpenter", "목수 토마스", "목수의 손도끼", "hand_axe", "“나무를 다듬을 손도끼가 필요하오.”", "260", "4일 남음"),
    ]
    rx, rw = x + 14, w - 28 - 12
    for i, (npc, name, title, item, quote, coin, due) in enumerate(rows):
        ry = y + 54 + i * 86
        s.nine("card-selected" if i == 0 else "card", rx, ry, rw, 80)
        s.slot(rx + 8, ry + 6, 56)
        s.put(portrait(npc, 48), rx + 12, ry + 10)
        s.text(rx + 36, ry + 76, name, 10, "on_parchment_muted", 600, "mb")
        quote = s.fit(quote, 11, rw - 74 - 124)
        s.text(rx + 74, ry + 10, title, 15, "on_parchment", 700)
        s.tag(rx + 80 + int(s.width(title, 15, 700)), ry + 11, "일상 의뢰", "daily")
        s.text(rx + 74, ry + 36, quote, 11, "on_parchment_muted", 500)
        cx = rx + rw - 118
        s.put(kit("glyph-coin"), cx, ry + 10)
        s.text(cx + 14, ry + 15, coin, 12, "on_parchment", 700, "lm")
        s.put(kit("glyph-star"), cx + 52, ry + 10)
        s.text(cx + 64, ry + 15, "+1", 12, "on_parchment", 700, "lm")
        s.slot(cx, ry + 32, 44, icon("finished", item))
        s.put(kit("glyph-hourglass"), cx + 52, ry + 50)
        s.text(cx + 64, ry + 55, due, 12, "deadline", 700, "lm")
    s.scrollbar(x + w - 22, y + 54, 266, 0.75, 0.0)

    # right: selected order detail
    x, y, w, h = 594, 14, 352, 332
    s.nine("panel-parchment", x, y, w, h)
    s.text(x + 18, y + 14, "요리사의 식칼", 19, "on_parchment", 700)
    s.tag(x + 26 + int(s.width("요리사의 식칼", 19, 700)), y + 18, "일상 의뢰", "daily")
    s.nine("divider", x + 14, y + 46, w - 28, 5)
    s.slot(x + 16, y + 56, 60)
    s.put(portrait("chef", 52), x + 20, y + 60)
    for j, line in enumerate(["“좋은 식칼이 있으면 요리가 더 맛있어지지!", "신선한 재료의 맛을 살릴 수 있는,", "튼튼하고 날카로운 식칼을 부탁하네.”"]):
        s.text(x + 88, y + 60 + j * 17, line, 12, "on_parchment_muted", 500)
    s.slot(x + 16, y + 126, 52, icon("finished", "kitchen_knife"), "selected")
    s.text(x + 42, y + 182, "식칼", 11, "on_parchment", 700, "mt")
    for j, (k, v, col) in enumerate([("납품 기한", "3일 남음", "deadline"), ("보상 금화", "320", "on_parchment"), ("평판", "요리사 +1", "on_parchment")]):
        yy = y + 132 + j * 19
        s.text(x + 86, yy, k, 12, "on_parchment_muted", 500)
        s.text(x + 160, yy, v, 12, col, 700)
    s.nine("divider", x + 14, y + 192, w - 28, 5)
    s.text(x + 18, y + 200, "필요 부품", 13, "on_parchment", 700)
    for j, pid in enumerate(part_ids("kitchen_knife")):
        px = x + 18 + j * 80
        s.slot(px, y + 218, 44, icon("parts", pid))
        s.text(px + 22, y + 266, f'{PARTS[pid]["name_ko"]}  1/1', 11, "on_parchment", 600, "mt")
    bx, by = x + 14, y + 296
    for label, kind, bw in (("보류", "button-normal", 84), ("의뢰 수락", "button-primary-normal", 124), ("제작으로 이동", "button-normal", 108)):
        s.nine(kind, bx, by, bw, 28)
        s.text(bx + bw // 2, by + 14, label, 13, "on_wood" if "primary" not in kind else "gold", 700, "mm")
        bx += bw + 4
    s.mock_label()
    return s.im


def collection():
    s = Screen()
    s.nine("window-frame", 0, 0, 960, 360)
    s.rail([("전체", "6 / 8"), ("단검", "2 / 2"), ("도끼", "2 / 2"), ("창", "1 / 3"), ("둔기", "1 / 1"), ("방패", "0 / 1"), ("활", "잠김")], 0, 42, 4)

    x, y, w, h = 198, 14, 392, 332
    s.nine("panel-parchment", x, y, w, h)
    s.text(x + 18, y + 14, "전체 무기", 17, "on_parchment", 700)
    s.text(x + 18, y + 36, "만들었거나 손에 넣은 무기 기록", 11, "on_parchment_muted", 500)
    s.text(x + w - 18, y + 16, "수집 현황  6 / 8", 12, "on_parchment", 700, "ra")
    s.nine("progress-track", x + w - 120, y + 34, 102, 10)
    s.nine("progress-fill", x + w - 119, y + 35, 75, 8)
    s.nine("divider", x + 14, y + 52, w - 28, 5)
    cards = [("dagger", True, 2), ("hunting_knife", True, 2), ("hand_axe", True, 1), ("woodcutting_axe", True, 2),
             ("spear", True, 1), ("pike", False, 0), ("halberd", False, 0), ("wooden_club", True, 1)]
    cw, ch = 86, 124
    for i, (id, found, q) in enumerate(cards):
        cx = x + 16 + (i % 4) * (cw + 4); cy = y + 62 + (i // 4) * (ch + 4)
        sel = id == "hunting_knife"
        s.nine("card-selected" if sel else "card", cx, cy, cw, ch)
        s.slot(cx + (cw - 52) // 2, cy + 10, 52, icon("finished", id), "normal" if found else "locked", silhouette=not found)
        s.text(cx + cw // 2, cy + 72, GOODS[id]["name_ko"] if found else "???", 12, "on_parchment" if found else "on_parchment_muted", 700, "mt")
        if found:
            s.stars(cx + cw // 2 - 16, cy + 96, q)
    s.mock_label()

    x, y, w, h = 594, 14, 352, 332
    s.nine("panel-parchment", x, y, w, h)
    s.text(x + 18, y + 14, "사냥칼", 19, "on_parchment", 700)
    tw = int(s.width("단검", 11, 600)) + 14
    s.tag(x + w - 18 - tw, y + 18, "단검", "rank")
    s.text(x + 18, y + 42, "숲을 걷는 사냥꾼의 첫 번째 동반자.", 11, "on_parchment_muted", 500)
    s.nine("divider", x + 14, y + 60, w - 28, 5)
    s.slot(x + 18, y + 72, 84, icon("finished", "hunting_knife"), scale=2)
    s.stars(x + 116, y + 76, 2)
    for j, line in enumerate(["클립 포인트 외날에 가죽 손잡이를 감은", "사냥용 칼. 손질과 해체에 쓰인다."]):
        s.text(x + 116, y + 94 + j * 17, line, 12, "on_parchment", 500)
    s.text(x + 18, y + 168, "최초 제작", 12, "on_parchment_muted", 500)
    s.text(x + 96, y + 168, "기록 없음", 12, "on_parchment", 700)
    s.text(x + 180, y + 168, "최고 품질", 12, "on_parchment_muted", 500)
    s.stars(x + 252, y + 171, 2)
    s.nine("divider", x + 14, y + 192, w - 28, 5)
    s.text(x + 18, y + 202, "주요 제작 부품", 13, "on_parchment", 700)
    for j, pid in enumerate(part_ids("hunting_knife")):
        px = x + 18 + j * 80
        s.slot(px, y + 222, 44, icon("parts", pid))
        s.text(px + 22, y + 270, PARTS[pid]["name_ko"], 11, "on_parchment", 600, "mt")
    return s.im


def compose(top):
    """Upper 360 management area over the live lower 180 workshop strip."""
    full = Image.new("RGBA", (960, 540), (0, 0, 0, 0))
    full.alpha_composite(top, (0, 0))
    full.alpha_composite(Image.open(WORKSHOP).convert("RGBA"), (0, 360))
    return full


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for name, fn in (("orders", orders), ("collection", collection)):
        top = fn()
        compose(top).save(f"{OUT}/{name}-960x540.png")
        top.resize((1920, 720), Image.NEAREST).save(f"{OUT}/{name}-960x360-2x.png")
        print(name, "ok")


def kit_sheet():
    """Every component at 3x on dark and light, and each nine-slice stretched to three sizes."""
    names = [a["name"] for a in M["assets"]]
    sizes = [(24, 24), (64, 40), (140, 72)]
    W = 800
    rows = [names[i:i + 2] for i in range(0, len(names), 2)]
    heights = [max(max(kit(n).height * 3, 76) for n in r) + 24 for r in rows]
    out = Image.new("RGBA", (2 * W, sum(heights) + 8))
    for half, bg in enumerate(((40, 34, 30, 255), (214, 204, 186, 255))):
        out.paste(Image.new("RGBA", (W, out.height), bg), (half * W, 0))
    d = ImageDraw.Draw(out)
    oy = 8
    for r, hgt in zip(rows, heights):
        for col, n in enumerate(r):
            a = SLICES[n]; src = kit(n)
            for half in (0, 1):
                x = col * 400 + half * W
                out.alpha_composite(src.resize((src.width * 3, src.height * 3), Image.NEAREST), (x + 4, oy + 16))
                if a["slice_ltrb"]:
                    l, t, rr, b = a["slice_ltrb"]
                    xx = x + 4 + src.width * 3 + 8
                    for (w, h) in sizes:
                        w, h = max(w, l + rr + 2), max(h, t + b + 2)
                        if t + b == 0:
                            h = src.height          # horizontal-only slice (divider)
                        if xx + w > x + 396:
                            break
                        out.alpha_composite(nine(n, w, h), (xx, oy + 16)); xx += w + 6
                d.text((x + 4, oy + 2), n, font=font(10, 600), fill=(240, 220, 190) if half == 0 else (60, 40, 20))
        oy += hgt
    return out


if __name__ == "__main__":
    kit_sheet().save(f"{OUT}/kit-sheet.png")
    print("kit sheet ok")
