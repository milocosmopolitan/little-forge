# Body-owned neck and shoulders

The rejected v10 build filled missing alpha while retaining bad source-jaw contours, triangular neck patches and inconsistent head offsets. Its zero-hole count was not anatomical acceptance.

Current production uses `body.mjs` through `work-coherent.mjs`. The approved moving head never enters this body constructor. It preserves source shirt clusters and their local shadows, builds a short 8×6 body-owned neck mount, and restores a complete shoulder plate between the source shirt edges and that mount. Small disconnected old-source head flecks near the neck are removed. Thinking-hand source pixels are protected separately. The body is complete before the head is drawn.

`style-restoration/heads.mjs` uses one root attachment per sex: female `(62,66)`, male `(57,64)`. Each source body has its own collar registration. There are no per-key head translation corrections. Approved eyes and mouth move exactly with the neckless head.

Production diagnostics include `body/neck-skin.png`, `body/source-connection.png`, `body/shoulder-plate.png`, `body/required-coverage.png` and their rendered counterparts. Pose metadata distinguishes the neck rectangle from the shoulder polygon. These are source-specific body registrations, not a universal anatomy template for other characters.

`before/` and `*-before-animation.json` preserve the rejected v10 comparison. `retained-frames.json` guards 120 common/rear frame files. They are historical evidence, not current builder inputs. `prototype.mjs` writes temporary comparisons only; the production entry point remains `node assets/characters/source/animation/build.mjs`.

Run `qa/verify-body-mount.py`, `verify-head-ownership.py`, `verify-neck-assembly.py`, `verify-animation.py` and `verify-neck-material-water.py` from `assets/characters/`. Their pixel assertions do not establish visual acceptance. `review.py` renders current separated layers and all 96 visible work variants on cyan for independent inspection. Research still reuses the approved three-quarter face over the front torso; this repair does not claim a newly authored front-facing head.
