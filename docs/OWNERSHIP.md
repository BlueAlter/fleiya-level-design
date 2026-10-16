# OWNERSHIP and Cross-Review

## Status: CONTRACT FROZEN (tag `contract-v0.1`)
As of the completion of Phase A (T-0011), the shared contract files (`crates/lc_core/*`, `crates/lc_ui_api/*`, and `crates/lc_io/*`) are **STRICTLY INTEGRATOR-OWNED**.
Feature agents working on Phase B (T-01xx) and subsequent tasks must NOT modify these crates within feature tasks. Any requested contract changes or extensions must be proposed via `docs/QUESTIONS.md` for integrator review.

A task may only edit files within its declared `scope`.

| Area | Crates / paths | Role slot | Reviewed by (neighbour) |
|---|---|---|---|
| Core, io, ui_api, xtask, CI, docs | lc_core, lc_io, lc_ui_api, xtask, docs/ | integrator | any feature reviewer (fresh session) |
| Shell: docks, menu, console, project panel | lc_shell | shell | integrator |
| Level canvas | lc_feature_level, lc_canvas | level | tileset |
| Tilesets + tile painting | lc_feature_tileset | tileset | level |
| Layer defs + Layers panel | lc_feature_layers | layers | tileset |
| Enums | lc_feature_enums | enums | entities |
| Entities, fields, floating inspector content | lc_feature_entities | entities | enums |
| World, depth, multi-world | lc_feature_world | world | level |

Role slots are job descriptions, not specific tools: any agent (agy, Claude Code, Codex, ...) can fill a slot by claiming a task with that `role`.
The reviewer must be a different session from the author, ideally with a fresh context (and optionally a different model).
