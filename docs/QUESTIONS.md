# QUESTIONS (contract-level blockers; humans answer asynchronously)

Format: `- [ ] YYYY-MM-DD T-XXXX <agent>: question / proposed answer`

- [ ] 2026-10-24 T-0201 implementer: `cargo xtask check` passes fmt, clippy (`-D warnings`) and tests, but the architecture lint fails on the pre-existing untracked `crates/lc_app_slint` package (`serde_json` is only allowed in `lc_io`). Proposed answer: remove/exclude `lc_app_slint` from the workspace or add an explicit allowed-dependency rule; T-0201 does not touch that crate.
- [ ] 2026-10-24 T-0202 implementer: `AddLevelCommand::focus()` returns the newly created level, but `ShellApp::process_commands_and_events` captures `cmd.focus()` before `execute` and ignores the `Ok(focus)` returned by `UndoStack::execute`, so adding a level/world does not open its tab. Proposed answer: use the focus returned by `execute` in a follow-up shell fix; T-0202 is visual-only and does not change command behaviour.
