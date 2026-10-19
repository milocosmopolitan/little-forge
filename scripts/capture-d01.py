#!/usr/bin/env python3
"""Capture actual D01 scenes; ECS assertions alone cannot qualify a blank render."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from PIL import Image

parser = argparse.ArgumentParser()
parser.add_argument("executable")
parser.add_argument("--output", default="qa-output/d01")
args = parser.parse_args()
output = Path(args.output).resolve()
output.mkdir(parents=True, exist_ok=True)
exe = str(Path(args.executable).resolve())
report = {"commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
          "executable_sha256": hashlib.sha256(Path(exe).read_bytes()).hexdigest(),
          "input": "Production Activate handlers, not native mouse/keyboard",
          "clock": "Core and animation advance 1/30 second per rendered frame",
          "visual_approval": False, "characters": {}}
for sex, scenario in [("male", "d01"), ("female", "d01-female")]:
    prefix = output / f"{sex}.png"
    with (output / f"{sex}.log").open("w") as log:
        subprocess.run([exe, "--qa", scenario, "--save-dir", str(output / f"session-{sex}"),
                        "--screenshot", str(prefix)], stdout=log, stderr=subprocess.STDOUT,
                       check=True, timeout=300)
    trace = json.loads(Path(f"{prefix}.json").read_text())
    assert trace["result"] == "passed" and trace["cycles"] == 100
    captures = {}
    for stage in ["ambient", "management", "heating", "hammering", "quenching", "rest",
                  "active-expanded", "active-collapsed"]:
        path = output / f"{sex}-{stage}.png"
        with Image.open(path) as image:
            rgb = image.convert("RGB")
            assert image.width * 180 == image.height * 960 or image.width * 540 == image.height * 960, (stage, image.size)
            expected_height = {"ambient": 180, "management": 540,
                               "active-expanded": 540, "active-collapsed": 180}.get(stage)
            if expected_height:
                assert image.width * expected_height == image.height * 960, (stage, image.size)
            strip_height = round(image.width * 180 / 960)
            strip = rgb.crop((0, image.height - strip_height, image.width, image.height))
            colors = strip.getcolors(maxcolors=16)
            assert colors is None and max(hi for _, hi in strip.getextrema()) > 100, f"{path}: blank/degenerate forge capture"
            captures[path.name] = {"size": list(image.size), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    samples = trace["samples"]
    report["characters"][sex] = {"result": "captures and ECS assertions passed", "cycles": trace["cycles"],
        "samples": len(samples), "actor_ids": sorted({s["entity"] for s in samples}),
        "captures": captures, "save_reload": trace["save_reload"]}
    print(f"{sex}: 100 transitions, work stages and nonblank forge captures passed", flush=True)
(output / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
