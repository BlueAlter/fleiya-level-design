---
id: T-0208
title: Level and world export pipelines (LDtk, Tiled, and unified JSON)
status: todo
owner: -
role: integrator
scope: [crates/lc_io, crates/lc_shell]
depends_on: [T-0190]
skills: [incremental-implementation, test-driven-development]
---

## Goal
Implement export commands and serializers to convert Level Craft projects, worlds, and levels into standard formats: LDtk compatible format, Tiled TMX/JSON, and clean monolithic runtime JSON.

## Acceptance criteria
- [ ] Export menu options under File -> Export (LDtk, Tiled, Runtime JSON).
- [ ] Serializer tests validating lossless structure mapping for layers, entities, and tilesets.
- [ ] Round-trip integration tests comparing exported structure against test fixtures.

## Out of scope
- In-place two-way live sync with external editors.
