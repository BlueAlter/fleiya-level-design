---
id: T-0004
title: Core data model for ALL domains + IDs + serde
status: done
owner: agy-1
role: integrator
scope: [crates/lc_core]
depends_on: [T-0002]
skills: [incremental-implementation, test-driven-development, source-driven-development, api-and-interface-design]
---

## Goal
Implement every struct/enum in docs/01-data-model.md (Project, Definitions, World, Level, all Defs, LayerInstance/LayerData, EntityInstance, FieldType/FieldValues,
Grid<T>) with stable ID newtypes and an ID generator. serde derives. Reserved `Auto` variants exist but do nothing.
## Acceptance criteria
- [x] All types compile with serde; round-trip tests via an in-memory serializer dev-dependency.
- [x] ID generator is deterministic in tests.
- [x] Lookup helpers by ID (no index-based public API).
- [x] No egui, no fs in the crate.
## Out of scope
Commands, file layout (T-0005, T-0006).
