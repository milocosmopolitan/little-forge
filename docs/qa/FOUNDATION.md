# Foundation evidence

Implementation branch: `feat/bevy-foundation`. Native platform acceptance is pending.

## Executed locally

- Rust 1.99.1 / Bevy 0.20.0 dependencies resolved; lockfile committed.
- `cargo test -p forge-core -p forge-content`: 10 tests passed.
- `cargo check -p little-forge`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed after fixes.
- Full desktop test linking encountered environment artifact errors (`invalid metadata`, zero-length object memory mapping); rerun in clean CI required. This is not a passing runtime test.

## Required evidence from CI

Three OS compile/test, portable archives, Linux software-rendered screenshot candidates and headless simulation benchmark. No macOS/Windows UI capability is proven by these builds. Screenshot candidates must be reviewed before approving a visual golden baseline.

## Remaining acceptance gates

- Native macOS Apple Silicon compositing, focus, Retina/mixed-DPI, sleep/wake.
- Windows Shadow PC checklist in issue #10.
- Native work-area discovery and monitor identity persistence (current conservative inset is documented, not exact Dock/taskbar avoidance).
- Reviewed visual baselines and measured native idle/expanded/unfocused/minimized CPU/GPU/memory.
- Signing/notarization/Steam release outside this Foundation preview.

Do not close the parent epic or native acceptance issues based on code presence alone.
