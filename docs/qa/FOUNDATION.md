# Foundation evidence

Implementation branch: `feat/bevy-foundation`. Native platform acceptance is pending.

## Executed locally

- Rust 1.99.1 / Bevy 0.20.0 dependencies resolved; lockfile committed.
- `cargo test -p forge-core -p forge-content`: 12 tests passed.
- `cargo check -p little-forge`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed after fixes.
- Full workspace tests passed (14 total after save hardening and scheduled sprite tests) using a clean temporary target directory; the original workspace target had corrupt build artifacts.
- Desktop executable builds locally. Local Xvfb cannot bind display sockets, so rendered UI evidence comes from CI, not this container.
- Initial CI identified missing libxkbcommon-x11 runtime dependency and Windows-only unused imports; both corrected in the follow-up.

## Required evidence from CI

Three OS compile/test, portable archives, Linux software-rendered screenshot candidates and headless simulation benchmark. No macOS/Windows UI capability is proven by these builds. Screenshot candidates must be reviewed before approving a visual golden baseline.

## Remaining acceptance gates

- Native macOS Apple Silicon compositing, focus, Retina/mixed-DPI, sleep/wake.
- Windows Shadow PC checklist in issue #10.
- Native work-area adapters and saved monitor placement implemented; actual mixed-DPI/disconnected-display qualification pending.
- Reviewed visual baselines and measured native idle/expanded/unfocused/minimized CPU/GPU/memory.
- Signing/notarization/Steam release outside this Foundation preview.

Do not close the parent epic or native acceptance issues based on code presence alone.

## Simulation-only baseline

`simulation-baseline-linux.json` records 10,000 seeded one-hour simulations on this Linux x86_64 environment, release build: 9.10 ms total, checksum 1,509,897. This small Foundation workload is a reproducibility/sanity baseline, not desktop CPU/GPU power or representative content-scale performance. Raw events show one craft at t=5 and one 12-gold sale at t=12.

## Visual review

The first CI capture (run 38794675720, commit 172491e) revealed a clipped building base, uniform-atlas frame drift, detached hand item and missing middle-dot glyphs. These are corrected in the follow-up: fitted building, explicit source rectangles, bottom-centered character anchors, per-walk-frame hand points and ASCII separators. The initial images are not approved goldens.

Golden candidates are reviewed before copying into `docs/qa/golden/linux-llvmpipe`. CI compares same-platform candidates with channel tolerance 8 and maximum changed-pixel ratio 0.005, preserving a diff image. Hardware/macOS/Windows images require their own approved baselines; do not silently replace these files after a failure.
