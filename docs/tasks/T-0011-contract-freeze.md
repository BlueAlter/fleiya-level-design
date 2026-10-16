---
id: T-0011
title: CONTRACT FREEZE: integration check and tag contract-v0.1
status: done
owner: agy-1
role: integrator
scope: [docs/, crates/*]
depends_on: [T-0005, T-0006, T-0009, T-0010]
skills: [code-review-and-quality, doubt-driven-development, code-simplification, documentation-and-adrs]
---

## Goal
Fresh-context sweep of everything from Phase A: consistency, dead code, docs vs code drift, ADR updates. Fix drift. Resolve docs/QUESTIONS.md. Tag `contract-v0.1`.
## Acceptance criteria
- [x] Docs match code (traits, file layout, crate matrix).
- [x] All Phase A tasks `done`; `cargo xtask check` green.
- [x] Tag created; docs/OWNERSHIP.md states contract files are now integrator-owned.
- [x] Phase B tasks reviewed for scope overlap; adjust before releasing them.

## Phase A report (REQUIRED, the human will send this file to a reviewer who has not seen the repo)
Write `docs/PHASE_A_REPORT.md`, <= 400 lines, self-contained, with these sections:
1. Summary table of T-0000..T-0011: status, rounds used, reviewer type (independent | self), commit SHA.
2. Pinned versions (rust toolchain, eframe, egui, egui_dock, other deps) and any ADRs added (title + one-line reason).
3. Contract, verbatim: the public signatures of `Command`, `UndoStack`, `DocumentView`, `PanelView`, `LayerHandler`, `CanvasTool`, `InspectorSection`, `EditorCtx`, `EditorEvent`, `Registry`, `Viewport`, and the top-level data structs (Project, Definitions, World, Level, LayerInstance/LayerData, EntityInstance, FieldType). Signatures only, no bodies.
4. T-0008 spike outcome: what egui_dock could and could not do, which approach was chosen, and why.
5. Deviations from docs/01..03 and from AGENTS.md, each with the reason.
6. Test and gate status: output tail of `cargo xtask check` on main, number of tests, anything ignored or allowed (with reasons).
7. Honest risk list: weak spots, things implemented by guess, APIs not verified, tasks reviewed only by self.
8. Open items: unchecked docs/QUESTIONS.md entries and docs/MANUAL_CHECKS.md.
9. Which Phase B tasks (T-0101..T-0108) are ready and any scope overlap you noticed between them.
