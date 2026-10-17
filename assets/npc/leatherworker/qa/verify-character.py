"""Read-only asset checks, followed by a deterministic rebuild comparison."""
import hashlib
import itertools
import json
import subprocess
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
PARTS = ["body/lower-base.png", "body/upper-base.png", "head-base.png",
         "eyes/open.png", "mouth/rest.png"]
GEAR = ["equipment/apron.png", "equipment/scarf.png", "equipment/belt-tools.png",
        "equipment/gloves.png", "equipment/hairpin.png", "props/leather-hide.png"]

def pixels(image):
    return image.get_flattened_data() if hasattr(image, "get_flattened_data") else image.getdata()

def read(name):
    image = Image.open(ROOT / name)
    assert image.mode == "RGBA", (name, image.mode)
    assert image.size == (96, 128), (name, image.size)
    assert set(pixels(image.getchannel("A"))) <= {0, 255}, name
    assert image.getbbox(), name
    return image

def composite(names):
    out = Image.new("RGBA", (96, 128))
    for name in names:
        out = Image.alpha_composite(out, read(name))
    return out

def hashes():
    return {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in ROOT.rglob("*.png")}

files = PARTS + GEAR + ["base-neutral.png", "neutral-no-prop.png", "neutral.png",
                        "source/foundation.png", "source/native-underlayer.png",
                        "source/native-reference.png"]
colors = set()
for name in files:
    image = read(name)
    colors.update(pixel[:3] for pixel in pixels(image) if pixel[3])
assert len(colors) <= 80, len(colors)
palette = {tuple(bytes.fromhex(c[1:])) for c in json.loads((ROOT / "source/palette.json").read_text())}
assert colors <= palette
foundation = read("source/foundation.png")
assert composite(PARTS[:3]).tobytes() == foundation.tobytes()
assert read("source/native-underlayer.png").tobytes() == foundation.tobytes()
assert composite(PARTS).tobytes() == read("base-neutral.png").tobytes()
assert composite(PARTS + GEAR[:5]).tobytes() == read("neutral-no-prop.png").tobytes()
assert composite(PARTS + GEAR).tobytes() == read("neutral.png").tobytes()
base_alpha = list(pixels(foundation.getchannel("A")))
for toggles in itertools.product((False, True), repeat=len(GEAR)):
    enabled = [name for name, enabled in zip(GEAR, toggles) if enabled]
    composed = composite(PARTS + enabled)
    assert all(not a or b for a, b in zip(base_alpha, pixels(composed.getchannel("A"))))
head_alpha = list(pixels(read("head-base.png").getchannel("A")))
for name in PARTS[3:]:
    assert all(not a or b for a, b in zip(pixels(read(name).getchannel("A")), head_alpha)), name
for name in ["body/lower-base.png", "body/upper-base.png", "eyes/open.png", "mouth/rest.png"]:
    assert read(name).tobytes() == read("source/revision-1-before/" + name).tobytes(), ("accepted part changed", name)
old_head = read("source/revision-1-before/head-base.png")
new_head = read("head-base.png")
blush_mask = read("source/corrections/blush-mask.png")
blush_changes = sum(a != b for a,b in zip(pixels(old_head), pixels(new_head)))
assert blush_changes > 0
assert all(a == b or m[3] for a,b,m in zip(pixels(old_head), pixels(new_head), pixels(blush_mask)))
assert old_head.getchannel("A").tobytes() == new_head.getchannel("A").tobytes()
hide = read("props/leather-hide.png")
full = read("neutral.png")
assert all(not a[3] or a == b for a,b in zip(pixels(hide), pixels(full))), "Hand shows through the front leather fold"
# Flood exterior transparency; an enclosed transparent island would expose hand.
alpha = hide.getchannel("A")
outside = {(x,y) for y in range(128) for x in range(96) if (x in (0,95) or y in (0,127)) and not alpha.getpixel((x,y))}
queue = list(outside)
for x,y in queue:
    for xx,yy in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
        if 0 <= xx < 96 and 0 <= yy < 128 and (xx,yy) not in outside and not alpha.getpixel((xx,yy)):
            outside.add((xx,yy)); queue.append((xx,yy))
hide_holes = sum(not alpha.getpixel((x,y)) and (x,y) not in outside for y in range(128) for x in range(96))
assert hide_holes == 0, hide_holes
for stem in ["neutral", "neutral-no-prop", "base-neutral"]:
    expected = read(stem + ".png").resize((384, 512), Image.Resampling.NEAREST)
    preview = Image.open(ROOT / "previews" / (stem + "-4x.png")).convert("RGBA")
    assert expected.tobytes() == preview.tobytes(), stem
before = hashes()
subprocess.run(["node", str(ROOT / "source/build-character.mjs")], check=True, stdout=subprocess.DEVNULL)
assert before == hashes(), "PNG output changed on rebuild"
result = {"native_files_checked": len(files), "shared_opaque_colors": len(colors),
          "equipment_combinations_checked": 64, "foundation_lost_pixels": 0,
          "face_layers_contained": True, "nearest_previews_exact": True,
          "png_rebuild_byte_identical": True,
          "accepted_body_hands_eyes_mouth_unchanged": True,
          "head_changed_only_in_blush_mask": True, "blush_pixels_restored": blush_changes,
          "hide_interior_transparent_pixels": hide_holes, "hide_front_occlusion_exact": True,
          "visual_review": "Reference/base/equipped, face, and individual layers on light/dark backgrounds reviewed separately"}
(ROOT / "qa/validation-report.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
