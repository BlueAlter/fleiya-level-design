# ADR-0008 — Console Logging Architecture and Tracing Subscriber

## Status
ACCEPTED

## Context
Task T-0009 requires Godot-style console output in the editor shell:
- A `tracing` subscriber layer feeding a bounded ring buffer (default 10,000 lines).
- A UI panel (`ConsolePanel`) implementing `PanelView` with level filter toggles, search text filtering, clear, copy to clipboard, duplicate collapsing, and click-to-focus `DocKind`.
- Logging from worker threads without blocking or contending with the UI thread.
- Rotating file logging to `logs/editor.log` so logs can be inspected outside the graphical environment.
- Preservation of architecture linter rules: no filesystem usage outside `lc_io`.

## Decision
1. `lc_shell` depends on `tracing-subscriber` (with `registry` and `fmt` features enabled in workspace dependencies).
2. The `ConsoleSubscriberLayer` captures `tracing::Event`s into a thread-safe `SharedRingBuffer` using a fast mutex lock, ensuring worker threads never block the UI thread.
3. Rotating file persistence is delegated to `RotatingLogWriter` in `lc_io`, maintaining the architectural invariant that only `lc_io` performs filesystem operations.
4. `EditorCtx::exec` logs command execution at `tracing::Level::DEBUG`.

## Consequences
- Logging is unified across all crates via standard `tracing` macros (`info!`, `warn!`, `error!`, `debug!`, `trace!`).
- The UI remains completely decoupled from filesystem operations while offering real-time streaming console output.
- All gates in `cargo xtask check` remain fully green.
