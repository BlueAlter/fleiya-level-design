# ADR-0001 — Default decisions (status: PROPOSED, human may override by editing this file)
1. File format: multi-file JSON (git friendly), `version` in every file, relative asset paths.
2. Export: engine-agnostic flat JSON later. No runtime loader crate in v1.
3. Platform: desktop only. All filesystem access isolated in `lc_io` so a web port stays possible.
4. Auto-layer rules: deferred. `LayerKind::Auto` / `LayerData::Auto` reserved in the data model, no behavior in v1.
5. Level size: dense grids, target up to 1024x1024 cells per layer. Revisit chunking only with a profile showing a problem.
6. Import of `.ldtk` / `.tmx`: not in v1.
7. Undo: one global stack per project.
