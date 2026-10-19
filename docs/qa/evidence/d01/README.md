# Actual Bevy evidence

See [runtime review](../../D01_RUNTIME_REVIEW.md) for reproduction and qualification limits. These PNGs are current macOS Metal captures at 200% DPI from the delivered Rust source, including legacy scrolling/tab-group and local F8 modifier handling. `source-hashes.json` pins that source; `capture-validation.json` pins the executable and image bytes. The captures were taken with the follow-up changes uncommitted on9742708; later documentation commits do not change the executable. Native female save/restart quenching is separately captured in `female-native-resumed-quench.png`.

`blocked-final-black.png` is retained only as the previous failed-run artifact. It is not current visual evidence. No native recording was obtained. Passing nonblank checks is not visual approval.

![Female ambient](female-ambient.png)

![Male ambient](male-ambient.png)

![Management](female-management.png)

![Heating](female-heating.png)

![Hammering](female-hammering.png)

![Quenching](female-quenching.png)

![Rest](female-rest.png)

![Active expanded](female-active-expanded.png)

![Active collapsed](female-active-collapsed.png)
