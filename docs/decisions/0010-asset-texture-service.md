# ADR-0010 — Asset and Texture Service for Feature Crates

## Status
ACCEPTED

## Context
Feature crates (`lc_feature_tileset`, `lc_feature_level`, `lc_feature_entities`) need to load PNG image assets (tileset textures, entity sprites, icon previews) into GPU textures (`egui::TextureHandle`).
However, Gate 4 architectural rules strictly enforce:
1. Feature crates cannot depend on `lc_io`.
2. Feature crates cannot use `std::fs` (checked by `xtask arch_lint`).

## Decision
1. Define an `AssetService` trait and typed `AssetError` in `lc_ui_api::assets`:
   - `load_texture(&self, ctx: &egui::Context, relative_path: &str) -> Result<egui::TextureHandle, AssetError>`
   - `invalidate(&self, relative_path: &str)`
2. Provide `NoopAssetService` in `lc_ui_api` for unit tests and headless environments.
3. Expose `cx.assets()` through `EditorCtx`.
4. Add `read_bytes(path: &Path) -> Result<Vec<u8>, IoError>` to `lc_io`.
5. Implement `ShellAssetService` in `lc_shell` using `lc_io::read_bytes` and `image` decoding, with thread-safe caching.

## Consequences
- Feature crates load textures cleanly via `cx.assets().load_texture(ui.ctx(), &rel_path)?` without knowing how files are loaded from disk.
- Zero architecture lint violations (`std::fs` remains confined strictly to `lc_io`).
- Texture allocation is cached and deduplicated across frames.
