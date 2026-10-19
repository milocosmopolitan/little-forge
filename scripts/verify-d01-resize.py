#!/usr/bin/env python3
"""Validate native D01 transition logs; this does not replace visual review."""
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("directory", type=Path)
args = parser.parse_args()
rows = [json.loads(line) for line in (args.directory / "host-layout.jsonl").read_text().splitlines()]
assert rows, "No native samples"
opens = closes = visible = 0
for before, row in zip(rows, rows[1:]):
    opens += not before["expanded"] and row["expanded"]
    closes += before["expanded"] and not row["expanded"]
for row in rows:
    if not row["visible"]:
        continue
    visible += 1
    width, height = row["window"]
    assert row["expanded"] and row["native"] == [width, height], row
    assert abs(height / width - 540 / 960) < 0.002, row
    size = row["host_size"]
    center = row["host_center"]
    assert abs(size[0] - width) <= 1 and abs(size[1] - height * 2 / 3) <= 1, row
    assert all(abs(c - s / 2) <= 1 for c, s in zip(center, size)), row
    assert center[1] + size[1] / 2 <= height * 2 / 3 + 1, row
    assert all(c["target"] == f"Some(UVec2({width}, {height}))" for c in row["cameras"]), row
assert opens >= 100 and closes >= 100, (opens, closes)
assert visible >= 100, "A permanently hidden host is not a passing result"
result = {"samples": len(rows), "opens": opens, "closes": closes,
          "visible_samples": visible, "unsafe_visible_samples": 0,
          "qualification": "Native geometry trace; screenshots require separate review"}
print(json.dumps(result, indent=2))
