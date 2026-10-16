# ADR-0011 — Deferral of Native File Dialog (rfd) to Task T-0109

## Status
ACCEPTED

## Context
Level Craft currently supports project opening and saving via `ShellApp::open_project_from_path` and `save_project_to_path`.
The UI menu bar has `File > Open` and `File > Save As`.
Adding the `rfd` (Rust File Dialog) crate directly during the contract freeze introduces native platform linkage requirements (e.g. `libgtk-3-dev`, `xdg-desktop-portal`, DBus dependencies on Linux), which can cause build/link failures in minimal headless CI environments.

## Decision
1. Retain the existing clean `open_project_from_path` / `save_project_to_path` backend integration in `lc_shell`.
2. Defer native file dialog integration (`rfd`) to a dedicated shell task `T-0109` (`docs/tasks/T-0109-file-dialog.md`).
3. Ensure Task T-0109 includes a headless mock/fallback mode so CI tests remain hermetic and 100% reliable.

## Consequences
- No risky third-party native C library dependencies introduced during contract freeze.
- Clean path forward for desktop UX without CI regressions.
