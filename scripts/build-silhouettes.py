"""Undiscovered-item silhouettes for the Weapon Collection (#115).

Each silhouette is the exact alpha of an approved icon filled with one flat parchment-shadow colour,
so an unknown weapon shows its outline without its materials. The shortbow has no item icon yet; it
uses the bow layer of the hunter's equipment (the art the expedition already shows), centred on a
32x32 canvas.

Run: python3 scripts/build-silhouettes.py -> assets/items/finished/silhouettes/*.png
"""
import os
from PIL import Image

SRC = "assets/items/finished/icons"
OUT = "assets/items/finished/silhouettes"
FILL = (92, 66, 42, 255)        # reads on the locked slot (#1e130c) and on parchment
BOW = ("assets/npc/hunter/equipment/bow.png", (28, 47, 45, 78))


def silhouette(im):
    out = Image.new("RGBA", im.size)
    a = im.getchannel("A").point(lambda v: 255 if v else 0)
    out.paste(Image.new("RGBA", im.size, FILL), mask=a)
    return out


def bow_icon():
    path, box = BOW
    crop = Image.open(path).convert("RGBA").crop(box)
    icon = Image.new("RGBA", (32, 32))
    icon.paste(crop, ((32 - crop.width) // 2, (32 - crop.height) // 2))
    return icon


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for f in sorted(os.listdir(SRC)):
        if f.endswith(".png"):
            silhouette(Image.open(f"{SRC}/{f}").convert("RGBA")).save(f"{OUT}/{f}")
    silhouette(bow_icon()).save(f"{OUT}/shortbow.png")
    print(len(os.listdir(OUT)), "silhouettes")
