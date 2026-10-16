---
id: T-0006
title: lc_io: multi-file JSON save/load, versions, golden sample
status: done
owner: agy-1
role: integrator
scope: [crates/lc_io, examples/sample_project]
depends_on: [T-0004]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Save/load per docs/01-data-model.md file layout. Version field + `migrate` module skeleton. Stable key ordering so diffs are minimal.
## Acceptance criteria
- [x] Golden test: load examples/sample_project then save -> byte-identical files.
- [x] Missing/corrupt file gives a typed error with path, never a panic.
- [x] Unknown newer version is rejected with a clear error.
- [x] Only this crate uses std::fs.
