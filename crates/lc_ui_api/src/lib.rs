//! lc_ui_api: UI contracts, traits, EditorCtx, and events.

#![forbid(unsafe_code)]

pub mod canvas;
pub mod context;
pub mod events;
pub mod inspector;
pub mod registry;
pub mod selection;
pub mod view;
pub mod viewport;

// Re-export core types commonly needed in UI
pub use lc_core::command::{Command, DocKind, UndoStack};
pub use lc_core::id::{EntityId, LayerDefId, LevelId, WorldId};
pub use lc_core::layer::LayerKind;
pub use lc_core::project::Project;

// Re-export ui_api types
pub use canvas::{CanvasCtx, CanvasTool, LayerHandler, LayerRenderCtx};
pub use context::EditorCtx;
pub use events::EditorEvent;
pub use inspector::InspectorSection;
pub use registry::{DocumentFactory, Registry};
pub use selection::Selection;
pub use view::{DocumentView, PanelId, PanelView, ViewState};
pub use viewport::{Viewport, DEFAULT_MAX_ZOOM, DEFAULT_MIN_ZOOM};
