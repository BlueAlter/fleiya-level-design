---
id: T-0010
title: Project panel tree and open-document flow
status: done
owner: agy-1
role: shell
scope: [crates/lc_shell/src/project_panel]
depends_on: [T-0008]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Tree: Project > Worlds > Depth groups > Levels; Definitions > Entities/Enums/Tilesets/Layers. Double-click emits `OpenDocument`. Context menu: rename, duplicate, delete (via Commands).
New project / open project / save wiring via lc_io.
## Acceptance criteria
- [x] Create project, add world + level, open level tab (placeholder DocumentView), save, reload, tree identical.
- [x] Double-click on an open document focuses it, does not duplicate it.
- [x] Drag a level to another depth group issues MoveLevel command and is undoable.
