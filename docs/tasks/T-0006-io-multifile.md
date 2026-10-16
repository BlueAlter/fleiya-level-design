---
id: T-0006
title: lc_io: multi-file JSON save/load, versions, golden sample
status: todo
owner: -
role: integrator
scope: [crates/lc_io, examples/sample_project]
depends_on: [T-0004]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Save/load per docs/01-data-model.md file layout. Version field + `migrate` module skeleton. Stable key ordering so diffs are minimal.
## Acceptance criteria
- [ ] Golden test: load examples/sample_project then save -> byte-identical files.
- [ ] Missing/corrupt file gives a typed error with path, never a panic.
- [ ] Unknown newer version is rejected with a clear error.
- [ ] Only this crate uses std::fs.
