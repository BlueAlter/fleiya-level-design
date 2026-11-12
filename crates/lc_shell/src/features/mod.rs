//! Feature modules, grouped by editor domain.
//!
//! Each folder is owned by one branch (`feature/<name>`) so two devices can
//! work in parallel without touching each other's files:
//!
//! | Folder | Branch |
//! | --- | --- |
//! | `editor/` | `feature/editor` |
//! | `entities/` | `feature/entity` |
//! | `layers/` | `feature/layer` |
//! | `tileset/` | `feature/tileset` |
//! | `enums/` | `feature/enum` |
//! | `project/` | `feature/project` |
//! | `world/` | `feature/world` |
//! | `settings/` | `feature/settings` |

pub mod editor;
pub mod entities;
pub mod enums;
pub mod layers;
pub mod project;
pub mod settings;
pub mod tileset;
pub mod world;
