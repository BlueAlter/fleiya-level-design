---
id: T-0109
title: Native and fallback file open/save dialogs in shell
status: done
owner: opencode
role: shell
scope: [crates/lc_shell]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Integrate asynchronous native file dialogs (`rfd`) for `File > Open` and `File > Save As`, with a graceful headless fallback when running in non-desktop environments.

## Acceptance criteria
- [x] `File > Open` opens a native file picker filtered to `*.lcproj` or directories.
      Evidence: `menu.rs` calls `request_open_project_dialog`; the `rfd` picker adds the `lcproj` filter (`file_dialog.rs`).
- [x] Selecting a file invokes `app.open_project_from_path(&path)`.
      Evidence: `file_dialog_test::open_dialog_selection_loads_project`.
- [x] `File > Save As` prompts for target file path and saves project.
      Evidence: `file_dialog_test::save_as_dialog_writes_project_and_sets_path`,
      `save_as_adds_missing_lcproj_extension`.
- [x] Headless/test mode falls back safely without blocking UI thread or panicking.
      Evidence: `file_dialog::tests::unavailable_service_returns_immediately`,
      `file_dialog_test::unavailable_dialog_reports_headless_fallback`,
      `polling_does_not_block_while_dialog_is_pending`.
- [x] Tested via headless mock dialog harness in `crates/lc_shell/tests`.
      Evidence: `crates/lc_shell/tests/file_dialog_test.rs` (7 integration tests).

## Out of scope
- In-canvas project browser (handled by Project panel).
- Cloud storage providers.

## Notes
- See ADR-0011 for rationale on deferring `rfd` to avoid Linux desktop-portal/GTK linkage dependencies during contract freeze.
- Dialogs run on a worker thread and are polled from the shell once per frame; the UI thread never blocks.
- Linux/BSD headless sessions (no `DISPLAY`/`WAYLAND_DISPLAY`) or `LC_DISABLE_FILE_DIALOGS=1` report `Unavailable` immediately.
- `cargo xtask check`: `ALL GATES PASSED` on this branch (2026-10-26).
