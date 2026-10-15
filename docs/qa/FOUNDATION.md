# Foundation evidence

Implementation branch: `feat/bevy-foundation`. Native platform acceptance is pending.

## Executed locally

- Rust 1.99.1 / Bevy 0.20.0 dependencies resolved; lockfile committed.
- `cargo test -p forge-core -p forge-content`: 12 tests passed.
- `cargo check -p little-forge`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed after fixes.
- Full workspace tests passed (13 total after save hardening) using a clean temporary target directory; the original workspace target had corrupt build artifacts.
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
