---
id: T-0190
title: Cross-crate consistency sweep after first feature batch
status: todo
owner: -
role: integrator
scope: [docs/, crates/*]
depends_on: [T-0101, T-0102, T-0103, T-0104, T-0105, T-0106, T-0107, T-0108]
skills: [code-review-and-quality, doubt-driven-development, code-simplification]
---

## Goal
Fresh-context review across all feature crates: naming, Command patterns, logging, UI patterns, duplicated helpers that should move to lc_canvas, doc drift.
Produce follow-up tasks (T-02xx) for Polish: multi-tile selection, copy/paste, auto-layer, preferences, shortcuts, export.
## Acceptance criteria
- [ ] Sweep report in docs/reviews/; every finding is fixed or turned into a task.
