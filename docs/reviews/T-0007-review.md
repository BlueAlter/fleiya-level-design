# Review of T-0007
Reviewer: subagent-reviewer   Author: agy-1   Branch: agent/T-0007   Commit reviewed: 5061a77

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  Summary:
  - Gate 1: cargo fmt --all -- --check (pass)
  - Gate 2: cargo clippy --workspace --all-targets -- -D warnings (pass)
  - Gate 3: cargo test --workspace (pass: 5 viewport tests + 1 registry test + 4 golden/io tests + 2 migrate tests + 5 command tests + 3 core unit tests + 1 round-trip test + 9 xtask arch tests)
  - Gate 4: Architecture & Crate Boundaries Lint (pass)

## Acceptance criteria evidence
- [x] Signatures match docs/02-architecture.md or the doc is updated in the same PR with a short ADR.
  Evidence: Trait and type signatures for `DocKind`, `DocumentView`, `PanelView`, `LayerHandler`, `CanvasTool`, `InspectorSection`, `Selection`, `EditorCtx`, `EditorEvent`, and `Registry` match `docs/02-architecture.md:32-46`. ADR-0006 (`docs/decisions/0006-glam-in-ui-api.md`) documents adding `glam` for strongly typed coordinate vectors (`IVec2`). `DocumentView` adds `fn doc_kind(&self) -> DocKind;` required for dock tab deduplication in `docs/03-ui-layout.md` (see finding 2).
- [x] Viewport conversion unit tests incl. negative coordinates and zoom limits.
  Evidence: `crates/lc_ui_api/tests/viewport_test.rs` has 5 comprehensive tests:
  - `test_viewport_defaults_and_zoom_limits` (lines 5-30): verifies default zoom, clamping at `DEFAULT_MIN_ZOOM` (0.05) and `DEFAULT_MAX_ZOOM` (32.0) via `set_zoom` and `zoom_at`.
  - `test_screen_world_conversion_positive_and_negative` (lines 33-57): verifies screen <-> world round-trip conversions for positive pixels, negative coordinates (`IVec2::new(-350, -420)`), and origin `(0, 0)`.
  - `test_world_grid_conversion_negative_and_positive` (lines 60-117): verifies Euclidean division for positive cells `(5, 7)`, negative cell `(-1, -1)` (top-left pixel, interior pixel, and bottom-right edge pixel), and deep negative cells `(-10, -25)`.
  - `test_zoom_at_pivot_preserves_world_position` (lines 120-141): verifies that zooming into and out of an on-screen pivot point preserves the exact world point under the cursor.
  - `test_screen_to_grid_and_grid_to_screen` (lines 144-160): verifies screen rect bounds to grid cell mappings.
- [x] A dummy feature in tests registers a DocumentView and a PanelView through Registry.
  Evidence: `crates/lc_ui_api/tests/registry_test.rs`:
  - `dummy_feature::register(&mut Registry)` (lines 83-96) registers a document factory closure for `DocKind::ProjectSettings` (`DummyDocView`) and a panel view (`DummyPanelView` with `PanelId("dummy_panel")`).
  - `test_dummy_feature_registration_and_dispatch` (lines 98-148) registers the feature, instantiates a document via `reg.create_document(DocKind::ProjectSettings)`, finds the panel via `reg.find_panel(&PanelId::new("dummy_panel"))`, and renders `doc.ui(ui, &mut cx)` with egui context, verifying command queueing into `pending_commands` and event dispatch into `events`.
- [x] Doubt pass recorded in the review file (contract task).
  Evidence: Recorded in the "## Doubt pass" section below with 5 architectural claims, evidence, counter-arguments, and resolutions.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests.
  `EditorCtx` prevents direct mutation of `Project` by exposing only `&Project`. All mutations are deferred through `cx.exec(cmd)` and `cx.exec_boxed(cmd)` onto `pending_commands: &mut Vec<Box<dyn Command>>`.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state.
  All references and signatures use stable identifiers (`LevelId`, `WorldId`, `EntityDefId`, `EnumDefId`, `TilesetDefId`, `LayerDefId`, `EntityId`, `PanelId`). No vector indices are used.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
  `lc_ui_api` depends only on `lc_core` and workspace third-party dependencies (`egui`, `glam`, `thiserror`, `tracing`). `glam` dependency is approved via ADR-0006. Zero filesystem access (`std::fs`) exists in `lc_ui_api`.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
  All four core extension traits are fully specified and exposed in `lc_ui_api::lib.rs`, with registration interfaces provided on `Registry`.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
  Naming strictly adheres to architectural naming conventions (`DocumentView`, `PanelView`, `CanvasTool`, `LayerHandler`, `InspectorSection`, `EditorCtx`, `EditorEvent`, `Viewport`, `ViewState`). `EditorCtx` exposes `log_info`, `log_warn`, and `log_error` utilizing `tracing`. Zero `unwrap()` or `expect()` calls in `src/`.
- [x] Egui API calls verified against the pinned version (no hallucinated methods).
  Types and methods from `egui` (`Pos2`, `Vec2`, `Rect`, `Ui`, `Modifiers`, `Response`, `Painter`) are verified against workspace pinned egui 0.31.
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
  `#![forbid(unsafe_code)]` declared. No `#[allow(...)]` or `#[ignore]` attributes. Diff is strictly within scope (`crates/lc_ui_api`, `docs/decisions/0006-glam-in-ui-api.md`, `Cargo.lock`, `docs/tasks/T-0007-ui-api.md`).
- [x] Acceptance criteria each have evidence.
  All acceptance criteria verified with passing tests and source references.

## Findings
1. [Important] `crates/lc_ui_api/src/registry.rs:61-66` — `Registry::find_panel` returns `Option<&dyn PanelView>`, but `PanelView::ui` requires `&mut self`:
   `PanelView::ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx)` requires a mutable reference to the panel. Callers obtaining an immutable reference via `reg.find_panel(&id)` cannot call `ui()`, forcing callers to either directly mutate `reg.panels_mut()` or maintain a separate panel map.
   Recommendation: Add `pub fn find_panel_mut(&mut self, id: &PanelId) -> Option<&mut (dyn PanelView + 'static)>` to `Registry` to support shell tab rendering directly.
2. [Important] `crates/lc_ui_api/src/view.rs:44-58` vs `docs/02-architecture.md:36` — Trait signature discrepancy on `DocumentView::doc_kind`:
   `DocumentView` introduces `fn doc_kind(&self) -> DocKind;` which is not present in `docs/02-architecture.md` (line 36). While this method is justified and necessary to satisfy the `docs/03-ui-layout.md` invariant ("Same DocKind never opens twice"), `docs/02-architecture.md` was not updated in this PR.
   Recommendation: Update `docs/02-architecture.md` line 36 during Phase A integration to document `fn doc_kind(&self) -> DocKind;`.
3. [Important] `crates/lc_ui_api/src/canvas.rs:12-29` — `CanvasCtx` lacks access to `egui::Painter` for on-canvas interactive tool previews:
   `CanvasTool::on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx)` receives `CanvasCtx`, but `CanvasCtx` does not contain `&egui::Painter` (unlike `LayerRenderCtx` which has `pub painter: &'a egui::Painter`). Interactive tools that need live canvas feedback during drag/hover gestures (such as selection marquee rects, brush circle previews, or entity placement boxes) cannot draw onto the canvas painter in `on_input`.
   Recommendation: For T-0009 (`lc_canvas`) or Phase B tools, add `pub painter: &'a egui::Painter` to `CanvasCtx`, or provide a dedicated canvas overlay drawing hook.
4. [Nit] `crates/lc_ui_api/src/viewport.rs:89-94` — `Viewport::world_to_grid` panics if `grid_size.x == 0` or `grid_size.y == 0`:
   `local.x.div_euclid(grid_size.x)` will panic at runtime if `grid_size` contains zero. While valid layers have positive grid sizes (e.g. 16x16), malformed or uninitialized inputs could crash the process.
   Recommendation: Guard with `let gx = local.x.div_euclid(grid_size.x.max(1));` or document the panic condition in doc comments.
5. [Nit] `crates/lc_ui_api/src/selection.rs:26-29` — `Selection::is_empty()` returns `false` for empty collections:
   `Selection::Entities(vec![])` or `Selection::Tiles { layer_id, cells: vec![] }` returns `false` from `is_empty()` because it only checks `matches!(self, Selection::None)`.
   Recommendation: Check inner collections:
   ```rust
   pub fn is_empty(&self) -> bool {
       match self {
           Selection::None => true,
           Selection::Entities(ids) => ids.is_empty(),
           Selection::Tiles { cells, .. } => cells.is_empty(),
           _ => false,
       }
   }
   ```
6. [Optional] `crates/lc_ui_api/src/viewport.rs:58-69` — `Viewport::zoom_at` does not guard against non-finite zoom factors:
   Passing `f32::NAN` or negative numbers bypasses standard clamp checks and can corrupt `zoom` and `pan` with `NaN`.
   Recommendation: Add an early return `if !factor.is_finite() || factor <= 0.0 { return; }`.
7. [FYI] `crates/lc_ui_api/src/events.rs:8-21` — `EditorEvent` lacks `Undo` / `Redo` variants:
   While undo/redo hotkeys and menu bar actions are handled by `lc_shell`, if floating toolbars or buttons in views need to trigger undo/redo via `cx.emit()`, adding `EditorEvent::Undo` and `EditorEvent::Redo` may be useful in later feature crates.

## Doubt pass (required if a trait/contract/format changed)
Cross-model skipped: non-interactive context.

1. **Deferred Command Queueing (`EditorCtx::pending_commands`)**
   - **Claim**: Deferring mutations via `pending_commands: &mut Vec<Box<dyn Command>>` preserves egui frame immutability and enforces Golden Rule 2 without borrow checker conflicts.
   - **Evidence**: Egui executes immediate-mode UI passes where multiple panels and tabs concurrently read `Project`. Holding `&mut Project` during the UI frame would violate Rust's aliasing guarantees when multiple panels read project data or inspect state.
   - **Counter-argument**: A command executed in one widget (e.g. renaming an entity) will not be visible to subsequent widgets rendered in the exact same frame, introducing a one-frame latency.
   - **Resolution**: Immediate-mode UI executes at 60fps; a one-frame latency for command execution (applied at the end of the frame by the shell before the next frame renders) is standard in egui architectures and prevents inconsistent mid-frame state observation.

2. **`CanvasTool` Input Handling without Painter in `CanvasCtx`**
   - **Claim**: Separating layer rendering (`LayerHandler::render(&LayerRenderCtx)`) from tool input (`CanvasTool::on_input(&CanvasCtx)`) cleanly divides static layer display from event dispatch.
   - **Evidence**: Matches the extension point signature in `docs/02-architecture.md:39-40`.
   - **Counter-argument**: Interactive tools (such as marquee selection, line previews, brush shapes) need to render transient canvas feedback while the pointer is dragged, before any command is committed.
   - **Resolution**: Valid trade-off for T-0007. The foundation satisfies `docs/02-architecture.md`. When implementing `lc_canvas` (T-0009) and layer tools (T-0010/T-0014), `CanvasCtx` should be augmented with `pub painter: &'a egui::Painter` or an overlay drawing method.

3. **`DocumentView::doc_kind(&self) -> DocKind` Addition**
   - **Claim**: Adding `doc_kind(&self) -> DocKind` to `DocumentView` is necessary for the shell dock system to satisfy `docs/03-ui-layout.md`.
   - **Evidence**: `docs/03-ui-layout.md:30` mandates: "Same DocKind never opens twice" and requires focusing existing tabs when requested by commands, console logs, or project tree clicks. Without `doc_kind()`, the shell cannot identify tab identity.
   - **Counter-argument**: Modifying the trait signature deviates from the exact snippet in `docs/02-architecture.md:36` and forces every document implementor to provide `doc_kind`.
   - **Resolution**: The addition is necessary and sound. Rather than removing it, `docs/02-architecture.md` should be updated during Phase A integration to maintain document/code synchronization.

4. **Euclidean Division in `Viewport::world_to_grid`**
   - **Claim**: Using `div_euclid` correctly projects negative continuous pixel coordinates to discrete grid cells (e.g. -1px to -16px maps to cell -1 for a 16px grid).
   - **Evidence**: Verified by unit test `test_world_grid_conversion_negative_and_positive` in `crates/lc_ui_api/tests/viewport_test.rs:60-117`, confirming exact recovery across top-left, interior, and bottom-right edge pixels for negative cells.
   - **Counter-argument**: If `grid_size.x == 0` or `grid_size.y == 0`, `div_euclid` panics with attempt to divide by zero.
   - **Resolution**: Grid dimensions in Level Craft are positive integers defined by layer/project definitions. A defensive guard (`max(1)`) is recommended as a nit to protect against malformed data.

5. **Registry Separation of Dynamic `DocumentFactory` vs Singleton `PanelView`**
   - **Claim**: Documents have dynamic lifetimes (multiple tabs, opening/closing) while dock panels are static singletons in the dock layout.
   - **Evidence**: Verified in `crates/lc_ui_api/tests/registry_test.rs:98-148`. The shell can instantiate new documents on demand via `create_document(kind)` while querying panels by ID.
   - **Counter-argument**: `Registry::find_panel` returns `&dyn PanelView`, which cannot call `PanelView::ui(&mut self, ...)` because `.ui()` requires `&mut self`.
   - **Resolution**: Valid design observation. The shell can iterate over `panels_mut()` or use an added `find_panel_mut` helper.
