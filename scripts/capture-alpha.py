#!/usr/bin/env python3
"""Replay Alpha 01 through production Bevy controls, with an isolated accelerated clock."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from PIL import Image

parser = argparse.ArgumentParser()
parser.add_argument("executable")
parser.add_argument("--output", default="qa-output/alpha")
args = parser.parse_args()
output = Path(args.output).resolve()
output.mkdir(parents=True, exist_ok=True)
executable = str(Path(args.executable).resolve())
scenes = ["title", "character-selection", "workshop", "chef-commission", "active-crafting",
          "chef-ready", "research-discovered", "research-ready", "research-completed",
          "codex", "journal", "workshop-return"]
reports = {}
for character, scenario in [("male", "alpha"), ("female", "alpha-female")]:
    prefix = output / f"{character}.png"
    with (output / f"{character}.log").open("w") as log:
        subprocess.run([executable, "--safe-window", "--qa", scenario,
                        "--save-dir", str(output / f"session-{character}"),
                        "--screenshot", str(prefix)], stdout=log, stderr=subprocess.STDOUT,
                       check=True, timeout=240)
    report = json.loads(Path(f"{prefix}.json").read_text())
    assert report["result"] == "passed" and report["save_reload"] == "equal"
    state = report["state"]
    assert state["smith"] == character
    assert state["progression"]["research_completed"]
    assert state["progression"]["codex"]["iron_dagger"]["count"] == 1
    files = {}
    for scene in scenes:
        path = output / f"{character}-{scene}.png"
        with Image.open(path) as image:
            assert image.width / image.height == 960 / 540
            assert image.convert("RGB").getextrema()[0][1] > 100
        files[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
    reports[character] = {"result": "passed", "captures": files}
    print(f"{character}: title -> Chef -> research -> Dagger -> Codex -> workshop; reload equal", flush=True)
(output / "summary.json").write_text(json.dumps({
    "base_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "working_tree": "includes local Alpha 01 changes",
    "clock": "0.1 simulation second per rendered frame",
    "input": "Activate events on actual production controls; native input checked separately",
    "characters": reports,
}, indent=2))
