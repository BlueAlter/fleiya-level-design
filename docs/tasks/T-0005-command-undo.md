---
id: T-0005
title: Command trait, UndoStack, first commands
status: done
owner: agy-1
role: integrator
scope: [crates/lc_core]
depends_on: [T-0004]
skills: [incremental-implementation, test-driven-development, source-driven-development, api-and-interface-design]
---

## Goal
`Command` trait as in docs/02-architecture.md, global `UndoStack` (undo, redo, merge window, clear on load), and a command executor `Project`-side.
Implement structural commands only: Add/Rename/Remove World, Add/Rename/Remove/Move Level (pos + depth).
## Acceptance criteria
- [x] Property-style test helper: apply then revert yields a Project equal to the original, used by every command test.
- [x] merge() tested (e.g. consecutive MoveLevel on the same level collapse).
- [x] Removal commands revert exactly, including level order.
