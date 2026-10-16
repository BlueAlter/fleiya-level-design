# ADR-0012 — Toolchain Pinning to Stable Rust 1.99.0

## Status
ACCEPTED

## Context
Initial Phase A reporting referenced an unstable nightly toolchain (`1.89-nightly`).
Level Craft workspace crates require only stable Rust language features and dependencies (`egui 0.36`, `eframe 0.36`, `glam 0.29`, `serde 1.0`).
Nightly toolchains introduce frequent breaking changes, non-reproducible CI runs, and unnecessary maintenance overhead.

## Decision
1. Pin `rust-toolchain.toml` to stable channel `1.99.0`.
2. Verify that all workspace crates, tests, clippy checks, and formatters build cleanly on stable without nightly feature flags.

## Consequences
- Deterministic, reproducible builds across developer workstations and CI runners.
- Zero reliance on unstable rustc compiler flags.
