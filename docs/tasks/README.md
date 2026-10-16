# Tasks
One file per task: `T-XXXX-slug.md` with front matter (`status`, `owner`, `role`, `scope`, `depends_on`, `skills`).
Status values: todo | in_progress | in_review | done. A task whose dependencies are not `done` cannot be claimed (claim.sh enforces it).
Add new tasks by copying `_TEMPLATE.md`. Keep tasks small: one session, <= ~300 changed lines.
