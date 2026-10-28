---
id: T-0190
title: Cross-crate consistency sweep after first feature batch
status: done
owner: opencode
role: integrator
scope: [docs/, crates/*]
depends_on: [T-0101, T-0102, T-0103, T-0104, T-0105, T-0106, T-0107, T-0108]
skills: [code-review-and-quality, doubt-driven-development, code-simplification]
---

## Goal
Fresh-context review across all feature crates: naming, Command patterns, logging, UI patterns, duplicated helpers that should move to lc_canvas, doc drift.
Produce follow-up tasks (T-02xx) for Polish: multi-tile selection, copy/paste, auto-layer, preferences, shortcuts, export.
## Acceptance criteria
- [x] Sweep report in docs/reviews/; every finding is fixed or turned into a task.

## Notes
- Deduped interaction stroke allocation into `lc_canvas::ids::next_stroke_id`.
- Normalized command label verbs ("Edit", "Remove") and string formats across feature crates.
- Eliminated redundant `layer_kind_of` helper in favor of `Layer::kind()`.
- Removed unused `tracing` dependency from `lc_feature_layers` and `lc_feature_tileset`.
- Removed duplicate `active_tool_name` field in `ShellApp`.
- Corrected doc drift in `02-architecture.md` and `03-ui-layout.md`.
- Produced follow-up tasks T-0205, T-0206, T-0207, and T-0208 for Phase C polish.
- Review documented in `docs/reviews/T-0190-review.md`.
