# ADR-0016 — egui_commonmark for rendering in-app documentation

## Status
ACCEPTED

## Context
The editor ships long-form documentation (this task adds the enum `DOCUMENT`
sub-tab, later tasks add more pages). The pages are authored as Markdown so that
writers do not need Rust, and so translations can be plain files under
`assets/localization/<lang>/docs/`. The editor therefore needs a Markdown
renderer that works inside egui without a browser engine.

Constraints:
- The workspace runs egui `0.36` (see `Cargo.lock`).
- Golden Rule 8: no new dependency without an ADR.
- Architecture boundaries: Markdown rendering is UI presentation and may only
  live in the shell (ADR-0004 keeps the xtask lint strict for every other crate).
- Golden Rule 7: the API must be verified against the pinned version before use.

## Decision
1. Add `egui_commonmark = "0.25.0"` to `[workspace.dependencies]` in the root
   `Cargo.toml`. That release declares `egui = "0.36.0"` and
   `pulldown-cmark = "0.13"`; `0.26` targets egui 0.37 and is explicitly not an
   option until egui is bumped.
2. Use it with `egui_commonmark = { workspace = true }` **only** in
   `crates/lc_shell/Cargo.toml`. The xtask architecture lint rejects it in every
   other crate.
3. Render documents with `CommonMarkViewer::new().show(ui, &mut cache, &text)`
   inside a `ScrollArea`. The `CommonMarkCache` is view state owned by the tab,
   never stored in `Project`.
4. Treat this dependency as part of the egui version lock: it is upgraded in the
   same change that bumps egui, never on its own.

## Alternatives considered
- **In-house Markdown renderer.** Rejected: CommonMark (headings, nested lists,
  emphasis, code spans, links) is a moving target and maintaining a correct
  subset would cost more than the dependency and still render worse.
- **Pre-rendered HTML / web view.** Rejected: adds a browser engine to a native
  editor and duplicates styling that must follow the Fleiya theme tokens.
- **Plain text documents.** Rejected: the task requires structured headings and
  numbered steps, and unstyled text would not match the rest of the window.

## Consequences
- Documentation rendering is available without leaving the egui ecosystem and
  picks up egui styling, so the theme tokens can style the page.
- `egui_commonmark` must be bumped together with egui; a mismatch is a compile
  error, which the version pin makes visible early.
- Default features enable image loading (`egui_extras/image`), so the dependency
  tree gains `pulldown-cmark` 0.13. No runtime crate outside `lc_shell` links it.