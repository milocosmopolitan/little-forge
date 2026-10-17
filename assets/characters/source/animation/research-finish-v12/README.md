# Research layer finish v12

Scope: research only. Approved head, eyes and mouth stay byte-identical. Every other action frame is protected by a pre-edit hash inventory.

The desktop front pass previously reused the base variant's arm mask for the independently registered equipment variant. Broad side strips also included shirt tails and towel pixels. Build a separate skin-connected forearm/hand pass for each variant and exclude isolated copper fittings. The complete character remains behind the desk. The user also requested removing the misplaced male left hanging cloth itself; its exact removed pixels are recorded in `removed-research-prop.png`, constrained to x38–53, y93–109 of equipped research bodies.

The female thinking key's source hand already ended in a point. Reuse the complete generated left resting forearm and fingers from research key 0 at native coordinates, preserving the original sleeve/cuff. This is existing-source recomposition, with no facial redraw or image rescaling.

The inferred female shoulder plate added black horizontal ledges and discontinuous pigment. Only those added plate pixels now continue adjacent bright shirt pigment. Source garment pixels and required body coverage retain their existing checks.

Build: `node assets/characters/source/animation/build.mjs`, then `node assets/characters/source/animation/scenes.mjs`.

Focused regression: `python3 assets/characters/qa/verify-research-finish.py`. Also run the existing animation, head ownership, neck garment, body coverage, water material and deterministic rebuild checks. Pixel checks do not establish user acceptance.
