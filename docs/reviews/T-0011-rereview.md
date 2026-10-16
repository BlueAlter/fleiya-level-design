# Review of T-0011 (Independent Re-review)
Reviewer: adversarial-code-reviewer   Author: agy-1   Branch: agent/T-0011 (fix/contract-v0.2)   Commit reviewed: b4cbcf4 (and 0c9c400 / main b0eb5a7)

Verdict: CHANGES_REQUESTED

## Gates
- cargo xtask check: pass
```
 INFO === Gate 1: cargo fmt --all -- --check ===
 INFO === Gate 2: cargo clippy --workspace --all-targets -- -D warnings ===
 INFO === Gate 3: cargo test --workspace ===
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
 INFO === Gate 4: Architecture & Crate Boundaries Lint ===
 INFO === ALL GATES PASSED: fmt, clippy, test, architecture lint ===
```

## Acceptance criteria evidence
- [ ] Docs match code (traits, file layout, crate matrix) — **FAIL / PARTIAL**:
  - `docs/PHASE_A_REPORT.md:142` documents `Registry::find_layer_handler`, but in `crates/lc_ui_api/src/registry.rs:84` the method is named `layer_handler`.
  - Critical contract types are omitted from `docs/PHASE_A_REPORT.md` Section 3: `DocKind`, `Selection`, `CanvasCtx`, `LayerRenderCtx`, `PanelId`, `ViewState`, and core def types.
  - `docs/02-architecture.md:32-35` states `DocKind` is defined in `lc_ui_api`, but it is actually located in `lc_core::command`.
  - Test count drift: `docs/PHASE_A_REPORT.md:249` claims 47 tests, but the actual workspace count is 48 tests.
- [x] All Phase A tasks done; `cargo xtask check` green — **PASS**: Tasks T-0000..T-0011 marked `done` on the board; all 4 gates pass with 48 tests and zero warnings.
- [x] Tag created; docs/OWNERSHIP.md states contract files are now integrator-owned — **PASS**: Tag `contract-v0.1` exists at commit `0c9c400`; `docs/OWNERSHIP.md:4-5` designates `crates/lc_core/*`, `crates/lc_ui_api/*`, and `crates/lc_io/*` as strictly integrator-owned.
- [ ] Phase B tasks reviewed for scope overlap; adjust before releasing them — **FAIL / PARTIAL**:
  - Critical contract gaps identified that will immediately block Phase B:
    1. Lack of shared session / UI state in `EditorCtx` (active level, active layer, tile palette selection, active tool), directly violating prerequisites for `T-0102`, `T-0103`, and `T-0104`.
    2. Lack of an asset/texture loading service in `EditorCtx` / `lc_ui_api`, preventing `T-0102` and `T-0101` from loading PNG assets or rendering textures without violating `CONSTRAINTS.md:26` ("no fs outside lc_io").
    3. `EditorEvent::SelectionChanged` is silently ignored in `crates/lc_shell/src/shell_app.rs:601`.
    4. Scope overlaps between `T-0102` and `T-0103` (`crates/lc_feature_tileset`) and `T-0106` and `T-0107` (`crates/lc_feature_entities`) must be strictly sequenced.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
- [x] Egui API calls verified against the pinned version (no hallucinated methods).
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
- [ ] Acceptance criteria each have evidence: **FAIL** (Verbatim docs drift from real signatures, critical contract gaps block Phase B feature development).

## Findings

### Critical (Must Fix)

1. **[Critical] Method Name Drift in Verbatim Contract: `Registry::find_layer_handler` vs `layer_handler`** (`docs/PHASE_A_REPORT.md:142` vs `crates/lc_ui_api/src/registry.rs:84`)
   The report published `find_layer_handler` while code implements `layer_handler`. Provide `find_layer_handler` as an alias or update docs and code to match.

2. **[Critical] Missing Shared Session State in `EditorCtx` breaks Phase B tasks (T-0102, T-0103, T-0104)** (`crates/lc_ui_api/src/context.rs:11-16`)
   `EditorCtx` lacks session state: active `DocKind`, active `LevelId`, active `LayerDefId`, selection, and tile palette selection. Phase B feature agents cannot share state across crate boundaries.

3. **[Critical] Missing Texture / Asset Service Trait in `lc_ui_api` prevents image loading for Tilesets** (`crates/lc_ui_api/src/context.rs`, `crates/lc_ui_api/src/lib.rs`)
   Features cannot use `std::fs` and cannot depend on `lc_io`. Without an `AssetService` trait exposed through `EditorCtx` and implemented in `lc_shell`, tilesets cannot load or display textures.

### Important (Should Fix)

4. **[Important] Critical Contract Types Omitted from `docs/PHASE_A_REPORT.md` Section 3**
   Omitted: `DocKind`, `Selection`, `CanvasCtx`, `LayerRenderCtx`, `PanelId`, `ViewState`, `DocumentFactory`, `LayerKind`, `TileRef`, `WorldLayout`, `ProjectSettings`.

5. **[Important] Architectural Ambiguity on `DocKind` Definition Location** (`docs/02-architecture.md:32-35`)
   Documented as defined in `lc_ui_api`, but actually defined in `lc_core::command`.

6. **[Important] Silent Drop of `EditorEvent::SelectionChanged` in Shell** (`crates/lc_shell/src/shell_app.rs:601`)
   Selection changes are discarded by shell event loop.

7. **[Important] Missing Test for Command Error Surfacing in Status Bar** (`crates/lc_shell/src/shell_app.rs:583-585`)
   No automated test confirms that failed command applications surface to status bar and log at error level.

8. **[Important] Scope Overlap and Sequencing for Phase B Feature Tasks** (`docs/tasks/T-0102.md`, `T-0103.md`, `T-0106.md`, `T-0107.md`)
   Need clear dependencies to prevent merge conflicts.

### Nit / Optional / FYI
9. **[Nit] `EditorCtx` logging methods omitted from Section 3** (`crates/lc_ui_api/src/context.rs:77-89`)
10. **[Nit] Outdated Test Count in `docs/PHASE_A_REPORT.md`** (47 vs 48)
11. **[FYI] Self-Review Reliance for Late Phase A Tasks**

## Doubt pass
- Claim: Phase A contract freeze at `contract-v0.1` provides a stable foundation for Phase B.
- Evidence: xtask check passes with 48 tests.
- Counter-argument: Signature mismatches, missing session state, and missing asset loading service will immediately break Phase B feature development under architecture constraints.
- Resolution: CHANGES_REQUESTED. Execute `fix/contract-v0.2` addressing all Critical findings before releasing Phase B.
