# Review of T-0000
Reviewer: subagent-reviewer   Author: agy-1   Branch: agent/T-0000   Commit reviewed: d36cf6b

Verdict: APPROVE

## Gates
- cargo xtask check: N/A (pass) — `xtask` workspace crate does not exist yet (scheduled for T-0001 and T-0002). T-0000 scope is strictly documentation (`[docs/PRD.md, docs/QUESTIONS.md]`; code is out of scope). Diff touches only markdown files.

## Acceptance criteria evidence
- [x] docs/PRD.md exists, names "Level Craft", explicit non-goals (no auto-layer, no import, no web in v1 unless ADR-0001 was changed).
  - Evidence: `docs/PRD.md` lines 1, 5, 13 name "Level Craft". Section 2.2 explicitly documents non-goals: no auto-layer in v1, no foreign format import (.ldtk/.tmx), no web/WASM in v1, no game engine runtime crate, and no sparse/chunked grids in v1. Aligns with ADR-0001.
- [x] Every feature in Phase B tasks (T-01xx) maps to a PRD section; gaps listed in docs/QUESTIONS.md.
  - Evidence: Section 9 table maps T-0101 through T-0108, plus T-0190 consistency sweep, directly to Sections 4, 5, 6, 8, and 10 with designated target crates. No gaps required in `docs/QUESTIONS.md`.
- [x] No contradiction with the ADRs; if a change is needed, update the ADR in the same PR.
  - Evidence: Verified against ADR-0001 (defaults), ADR-0002 (UI docking & fallback), and ADR-0003 (multi-agent coordination and crate ownership). All PRD sections are fully consistent with these decisions.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — Specified in Section 2.1, 6.1, 6.2, 10.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state — Specified in Section 5.1, 5.2, 6.2.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — Specified in Section 2.1, 4.1 (Dependency Rules Matrix), 10.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()` — Specified in Section 8.2 and mapped in Section 9.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Specified in Section 4, 10.
- [x] Egui API calls verified against the pinned version (no hallucinated methods) — N/A (no Rust/UI code written in this task).
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline — Diff is 276 lines (within <= 300 line guideline) across `docs/PRD.md` and `docs/tasks/T-0000-prd.md`.
- [x] Acceptance criteria each have evidence — Documented above with specific section and line references.

## Findings
1. [Nit] docs/PRD.md:93-94 vs docs/PRD.md:98-104: Section 5.1 lists `ProjectId` among strongly typed newtypes, but `pub struct Project` in Section 5.2 does not declare a `pub id: ProjectId` field (mirroring `docs/01-data-model.md`). When implementing T-0004, clarify whether `Project` carries an explicit `ProjectId` or is solely identified by its workspace root directory.
2. [Optional] docs/PRD.md:106-111: `Definitions` lists `Vec<TilesetDef>`, `Vec<EnumDef>`, `Vec<EntityDef>`, `Vec<LayerDef>`, but concrete field definitions for these defs are omitted in Section 5.2. This faithfully mirrors `docs/01-data-model.md` and will be defined during T-0004 (Core data model).
3. [Nit] docs/PRD.md:236-243: Section 8.2 lists extension contracts (`DocumentView`, `PanelView`, etc.) but omits the `DocKind` enum definition, though `DocKind` is referenced in Section 6.1 (`fn focus(&self) -> Option<DocKind>`). Consider mentioning `DocKind` in Section 8.2 alongside the traits for completeness.
4. [FYI] docs/PRD.md:250-261: The Phase B mapping matrix cleanly covers all tasks T-0101 through T-0108, plus T-0190 consistency sweep, providing unambiguous scope and crate targets for Phase B feature agents.

## Doubt pass (required if a trait/contract/format changed)
- Claim: Consolidating specifications into `docs/PRD.md` while keeping definition struct fields (`TilesetDef`, `EntityDef`, etc.) at the summary level is appropriate for T-0000.
- Evidence: T-0000's scope is PRD consolidation of `docs/01-data-model.md`, `02-architecture.md`, `03-ui-layout.md` and ADRs. `docs/01-data-model.md` defines the exact same top-level model; detailed struct fields are specifically assigned to task T-0004 ("Core data model for ALL domains").
- Counter-argument: Feature implementers might encounter ambiguity without full field listings in the PRD.
- Resolution: Phase A is sequential and single-implementer; T-0004 creates and freezes the contract in code before Phase B begins. The task descriptions for T-0102, T-0104, T-0105, and T-0106 already specify the needed fields for each definition type.
