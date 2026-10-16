# ADR-0005 — In-Memory Serializer Dev-Dependency for Core Round-Trip Tests

## Status
ACCEPTED

## Context
Task T-0004 specifies: "All types compile with serde; round-trip tests via an in-memory serializer dev-dependency." Furthermore, `docs/02-architecture.md` and the architecture linter specify that `serde_json` is strictly reserved for `lc_io`. To enable deterministic, fast in-memory round-trip serialization tests for all data model structs in `lc_core` without violating crate boundaries, a dedicated in-memory serde format is required as a dev-dependency.

## Decision
1. Add `ron` (Rusty Object Notation) as a `[dev-dependencies]` dependency to `crates/lc_core`.
2. `ron` is strictly used in `lc_core/tests/` for validating serde serialization and deserialization invariants. It is not a runtime dependency and does not compile into production binaries.

## Consequences
- All core types are verified to round-trip faithfully through Serde.
- `lc_core` remains completely free from `serde_json`, maintaining the architectural rule that only `lc_io` performs JSON serialization.
