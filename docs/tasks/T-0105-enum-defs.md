---
id: T-0105
title: Enum def tab
status: todo
owner: -
role: enums
scope: [crates/lc_feature_enums]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
EnumDef tab: values with colors, optional tile icon, reorder, rename. Deleting a value or enum that is referenced by fields is blocked with a clear message, or cascades via an explicit Command.
## Acceptance criteria
- [ ] Commands + revert tests incl. reference handling.
- [ ] Rename updates dependent displays without breaking IDs.
