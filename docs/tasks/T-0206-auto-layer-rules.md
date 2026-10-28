---
id: T-0206
title: Auto-layer rule definitions, pattern evaluation, and automated tile output
status: todo
owner: -
role: layers
scope: [crates/lc_feature_layers, crates/lc_core]
depends_on: [T-0190]
skills: [incremental-implementation, test-driven-development]
---

## Goal
Implement auto-tiling rules for Auto layers evaluated from IntGrid layers, matching patterns (e.g. 3x3 neighbor rules) and producing deterministic tile output.

## Acceptance criteria
- [ ] Auto-layer definition UI for configuring 3x3 pattern rules and source IntGrid layer binding.
- [ ] Rule evaluation engine in core/feature evaluating neighbor states and emitting tile placements.
- [ ] Commands for editing rules and triggering auto-layer rebuild with undo/redo.
- [ ] Unit tests for corner, edge, and inner neighbor pattern matching.

## Out of scope
- Arbitrary scriptable rule engines (strict pattern matrices only).
