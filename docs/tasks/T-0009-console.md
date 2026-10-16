---
id: T-0009
title: Console panel with tracing layer, file log
status: done
owner: agy-1
role: shell
scope: [crates/lc_shell/src/console, crates/lc_app]
depends_on: [T-0008]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Console per docs/03-ui-layout.md: tracing Layer -> bounded ring buffer, panel with level filters, search, clear, copy, duplicate collapsing, auto-scroll/pause; also rotating `logs/editor.log`.
## Acceptance criteria
- [x] Unit tests for ring buffer bound, filtering, duplicate collapsing.
- [x] Logging from a worker thread appears without blocking the UI.
- [x] Every Command application logs at debug (hook in EditorCtx::exec).
