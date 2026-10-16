---
id: T-0109
title: Native and fallback file open/save dialogs in shell
status: todo
owner: -
role: shell
scope: [crates/lc_shell]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Integrate asynchronous native file dialogs (`rfd`) for `File > Open` and `File > Save As`, with a graceful headless fallback when running in non-desktop environments.

## Acceptance criteria
- [ ] `File > Open` opens a native file picker filtered to `*.lcproj` or directories.
- [ ] Selecting a file invokes `app.open_project_from_path(&path)`.
- [ ] `File > Save As` prompts for target file path and saves project.
- [ ] Headless/test mode falls back safely without blocking UI thread or panicking.
- [ ] Tested via headless mock dialog harness in `crates/lc_shell/tests`.

## Out of scope
- In-canvas project browser (handled by Project panel).
- Cloud storage providers.

## Notes
- See ADR-0011 for rationale on deferring `rfd` to avoid Linux desktop-portal/GTK linkage dependencies during contract freeze.
