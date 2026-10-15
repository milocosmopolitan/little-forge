# Native acceptance handoff

This is a Foundation development build, not a Steam release. Native checks below must not be marked passed from CI compilation.

## Windows / Shadow PC

1. Download `little-forge-Windows-X64-<commit>` from the Foundation QA Actions run and extract the contained ZIP completely; keep `assets` next to `little-forge.exe`.
2. In PowerShell in that folder, start the safe path first:
   ```powershell
   .\little-forge.exe --safe-window --save-dir .\qa-save 2>&1 | Tee-Object safe-window.log
   ```
3. Verify C craft → stock → visitor purchase → gold +12 → same dagger carried out. B spends 4 gold for 2 ore. Close and reopen; gold/stock must not duplicate.
4. Start transparent mode, same save folder:
   ```powershell
   .\little-forge.exe --save-dir .\qa-save 2>&1 | Tee-Object transparent.log
   ```
5. F8 activates a 10-second click-through lease. Click a background app; verify input passes through and after 10 seconds the forge accepts input again. Ctrl+Alt+F8 should recover globally; test it while another app is focused. Esc works only when the forge receives keys. If anything traps input, close via Task Manager and restart `--safe-window`.
6. Tab expands upward; test D for window decorations/drag, T for topmost off/on, 100/125/150/200% DPI, second monitor, disconnect, sleep/wake, and application focus. Native work-area detection must be checked; record Dock/taskbar overlaps.
7. Press F12; attach `qa-save/diagnostics.json`, startup logs, screenshot/video, Windows build, GPU/driver, Shadow stream scale and commit SHA to issue #10.
8. Complete every item in `docs/system/PLATFORMS.md`. Shadow streaming cannot prove local GPU power, local input latency or every compositor behavior.

## macOS Apple Silicon

Use the macOS Actions archive (unsigned `.app`) or `cargo run --locked -p little-forge`. For command-line diagnostics run `"Little Forge.app/Contents/MacOS/little-forge" --safe-window`. Verify transparency, focus, Retina scaling, Dock/menu bar placement, monitor movement and sleep/wake. Gatekeeper signing/notarization is not included. Do not disable system-wide security settings to test it.

## Performance capture

Record 60 seconds after warm-up for ambient animated, expanded, unfocused, obscured/minimized and resumed states. Use Activity Monitor (macOS) or Task Manager/Performance Monitor (Windows). Record CPU average/peak, GPU engine usage, working set/RSS, frame rate and test duration. Repeat on the same hardware/build before making optimization claims. `forge-qa` measures only the pure simulation and is not a substitute.

## Steam preparation

Portable package includes executable, assets, build SHA and this guide. Use a private Steam test depot, set executable/working directory, verify a clean machine install, launch/exit and save persistence. Steam SDK/overlay/cloud saves, signing and store publishing are not part of this Foundation implementation.

For repeatable per-process samples: `python -m pip install psutil`, then `python scripts/profile-process.py PID --scenario ambient --seconds 60 --output qa-output/ambient-performance.json`. Press F12 separately for FPS/GPU adapter diagnostics. GPU utilization remains a native-tool measurement.

## Windows black-background regression

The initial Foundation build could select Vulkan and unspecified alpha, leaving a black rectangle even without `--safe-window`. Transparent Windows launches now select DX12, a DirectComposition visual swapchain (`DxgiFromVisual`) and premultiplied alpha. This is configured automatically before renderer initialization. Safe mode retains the automatic backend and opaque background.

On the corrected build, launch without `--safe-window` over a light application window. Empty pixels must show that application; sprites and text must remain visible without black fringes. F12 adapter diagnostics should report `Dx12`. Repeat the global shortcut, 10-second click-through recovery, resizing, DPI and sleep checks. The code/CI change alone does not constitute a native transparency pass. macOS uses its existing Metal/postmultiplied-alpha path and needs separate native evidence if still opaque.
