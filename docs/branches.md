# Branch layout (two-device workflow)

`lc_shell` is split by editor domain so two devices can work in parallel
without touching the same files. `main` is the integration branch; every
`feature/*` branch starts from it.

| Branch | Owns | Device |
| --- | --- | --- |
| `shared/ui-kit` | `crates/lc_shell/src/shared/` — universal widgets and services: drag ghost (`drag_list`), tile bounds, reorder lists, tag input, context menus, tile picker, file dialogs, asset service | both (short-lived fixes) |
| `feature/editor` | `crates/lc_shell/src/features/editor/` — shell app state, docks, menu, console, status bar, inspector | |
| `feature/entity` | `crates/lc_shell/src/features/entities/`, `crates/lc_feature_entities/` | |
| `feature/layer` | `crates/lc_shell/src/features/layers/`, `crates/lc_feature_layers/` | device B |
| `feature/tileset` | `crates/lc_shell/src/features/tileset/`, `crates/lc_feature_tileset/` | device A |
| `feature/enum` | `crates/lc_shell/src/features/enums/`, `crates/lc_feature_enums/` | |
| `feature/project` | `crates/lc_shell/src/features/project/` | |
| `feature/world` | `crates/lc_shell/src/features/world/`, `crates/lc_feature_world/`, `crates/lc_feature_level/` | |
| `feature/settings` | `crates/lc_shell/src/features/settings/` — theme/appearance and project settings | |

## Rules

1. Work only inside your branch's folder (plus its `lc_feature_*` crate and
   tests). Files outside it belong to another branch or to `shared/ui-kit`.
2. Universal changes (drag ghost, tile bounds, context menus, tile picker, ...)
   are **not** made on a feature branch. Do them on `shared/ui-kit`, merge to
   `main`, then merge `main` into your feature branch.
3. `main` never receives feature work directly. Feature branches merge to
   `main` through a PR (see `docs/04-multi-agent-workflow.md`).
4. Keep the gate green on every branch: `cargo xtask check`.
5. `assets/localization/<lang>/editor/<domain>.ron` and
   `assets/localization/<lang>/docs/<domain>/` belong to the domain branch; the
   i18n loader (`lc_ui_api::i18n`, `lc_shell::shared::i18n`), `common.ron` and
   `assets/localization/README.md` belong to `shared/ui-kit`.

## Cross-branch exceptions

- `feature/tileset` task (Tileset workspace mode switch and floating window cutover):
  - `crates/lc_shell/src/features/editor/app_mode.rs`: lines 1, 17, 21-60, 71, 74, 90-104: added `AppMode::Tilesets`, `ModeButton`, `MODE_BUTTONS`, `control_width`, `shows_unsaved`, and data-driven segmented control.
  - `crates/lc_shell/src/features/editor/menu.rs`: lines 105-114, 180, 189-198: replaced `show_tileset_window` checkbox with "Tilesets Workspace  Ctrl+3" button; added `Ctrl+3` shortcut to switch to `AppMode::Tilesets` and select tileset def.
  - `crates/lc_shell/src/features/editor/shell_app.rs`: lines 130-131, 262, 806-810, 1038-1040: replaced `show_tileset_window` field with `tileset_view_state`, redirected `DocKind::TilesetDef` to switch to `AppMode::Tilesets` and select def, removed `render_tileset_window`, and dispatched `AppMode::Tilesets` to `render_tileset_view`.
- `feature/layer` task (Layers workspace mode switch and rail/menu wiring):
  - `crates/lc_shell/src/features/editor/app_mode.rs`: enabled `AppMode::Layers` (`Ctrl+2`) in `WORKSPACE_RAIL_ITEMS` and added `"LAYER WORKSPACE"` title.
  - `crates/lc_shell/src/features/editor/menu.rs`: added `"Layers Workspace  Ctrl+2"` in the `WINDOWS` menu and `Ctrl+2` global shortcut.
  - `crates/lc_shell/src/features/editor/shell_app.rs`: added `layer_view_state`, redirected `DocKind::LayerDef` to `AppMode::Layers`, and dispatched `AppMode::Layers` to `render_layer_view`.
  - `crates/lc_shell/tests/app_mode_test.rs`: updated `WORKSPACE_RAIL_ITEMS[1]` assertion for enabled `AppMode::Layers` and added `Ctrl+2` / `DocKind::LayerDef` assertions.
  - `crates/lc_shell/tests/fleiya_features_test.rs`: updated `DocKind::LayerDef` assertion in `test_definition_tabs_and_windows_behavior` from floating window to `AppMode::Layers`.
  - `crates/lc_core/` & `crates/lc_io/`: extended `LayerDef`, `IntGridValueDef`, `World` (`included_layer_tags` / `excluded_layer_tags`), `WorldFile`, and added `auto_rules` / `auto_rules_eval` (ADR-0017).
  - `crates/lc_feature_world/`: added `SetWorldLayerTagsCommand`, `PruneUnusedWorldLayersCommand`, and `Layer Tags` toolbar drawer in `WorldDocumentView`.

## Daily commands

```sh
git fetch origin
git switch feature/tileset        # device A
# or: git switch feature/layer    # device B

# pick up work merged on main (shared widgets, other features)
git merge origin/main

git add -A && git commit -m "..."
git push origin feature/tileset
```
