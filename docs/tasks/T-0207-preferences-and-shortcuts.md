---
id: T-0207
title: User preferences persistence and customizable keyboard shortcuts
status: todo
owner: -
role: shell
scope: [crates/lc_shell, crates/lc_ui_api]
depends_on: [T-0190]
skills: [incremental-implementation, test-driven-development]
---

## Goal
Provide a preferences settings interface and configuration persistence for user keybindings, grid defaults, and theme options.

## Acceptance criteria
- [ ] User preferences schema stored in application config directory.
- [ ] Keyboard shortcuts table with rebinding capability and collision detection.
- [ ] Project settings / preferences view with interactive shortcut editor.
- [ ] Headless tests verifying key event dispatch against custom bindings.

## Out of scope
- Cloud sync of user preferences.
