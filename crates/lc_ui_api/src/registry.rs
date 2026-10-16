//! Registry for pluggable documents, panels, layer handlers, and inspector sections.

use crate::canvas::LayerHandler;
use crate::inspector::InspectorSection;
use crate::view::{DocumentView, PanelId, PanelView};
use lc_core::command::DocKind;
use lc_core::layer::LayerKind;

/// Factory callback for instantiating document views based on `DocKind`.
pub type DocumentFactory = Box<dyn Fn(DocKind) -> Option<Box<dyn DocumentView>> + Send + Sync>;

/// Central registry populated by feature crates during application initialization.
#[derive(Default)]
pub struct Registry {
    document_factories: Vec<DocumentFactory>,
    panels: Vec<Box<dyn PanelView>>,
    layer_handlers: Vec<Box<dyn LayerHandler>>,
    inspector_sections: Vec<Box<dyn InspectorSection>>,
}

impl Registry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a document factory closure.
    pub fn register_document_factory<F>(&mut self, factory: F)
    where
        F: Fn(DocKind) -> Option<Box<dyn DocumentView>> + Send + Sync + 'static,
    {
        self.document_factories.push(Box::new(factory));
    }

    /// Attempts to instantiate a document view for a given `DocKind` across all registered factories.
    pub fn create_document(&self, kind: DocKind) -> Option<Box<dyn DocumentView>> {
        for factory in &self.document_factories {
            if let Some(doc) = factory(kind) {
                return Some(doc);
            }
        }
        None
    }

    /// Registers a dockable panel view.
    pub fn register_panel(&mut self, panel: Box<dyn PanelView>) {
        self.panels.push(panel);
    }

    /// Returns a slice of all registered panel views.
    pub fn panels(&self) -> &[Box<dyn PanelView>] {
        &self.panels
    }

    /// Returns a mutable slice of all registered panel views.
    pub fn panels_mut(&mut self) -> &mut [Box<dyn PanelView>] {
        &mut self.panels
    }

    /// Finds a registered panel view by its `PanelId`.
    pub fn find_panel(&self, id: &PanelId) -> Option<&dyn PanelView> {
        self.panels
            .iter()
            .find(|p| p.id() == *id)
            .map(|p| p.as_ref())
    }

    /// Finds a registered panel view mutably by its `PanelId`.
    pub fn find_panel_mut(&mut self, id: &PanelId) -> Option<&mut (dyn PanelView + 'static)> {
        for panel in &mut self.panels {
            if panel.id() == *id {
                return Some(panel.as_mut());
            }
        }
        None
    }

    /// Registers a layer handler for a specific `LayerKind`.
    pub fn register_layer_handler(&mut self, handler: Box<dyn LayerHandler>) {
        self.layer_handlers.push(handler);
    }

    /// Finds the registered layer handler for a specific `LayerKind`.
    pub fn layer_handler(&self, kind: LayerKind) -> Option<&dyn LayerHandler> {
        self.layer_handlers
            .iter()
            .find(|h| h.kind() == kind)
            .map(|h| h.as_ref())
    }

    /// Alias for [`Registry::layer_handler`], finding the registered layer handler for a specific `LayerKind`.
    pub fn find_layer_handler(&self, kind: LayerKind) -> Option<&dyn LayerHandler> {
        self.layer_handler(kind)
    }

    /// Returns a slice of all registered layer handlers.
    pub fn layer_handlers(&self) -> &[Box<dyn LayerHandler>] {
        &self.layer_handlers
    }

    /// Registers an inspector section for the floating properties window.
    pub fn register_inspector_section(&mut self, section: Box<dyn InspectorSection>) {
        self.inspector_sections.push(section);
    }

    /// Returns a slice of all registered inspector sections.
    pub fn inspector_sections(&self) -> &[Box<dyn InspectorSection>] {
        &self.inspector_sections
    }

    /// Returns a mutable slice of all registered inspector sections.
    pub fn inspector_sections_mut(&mut self) -> &mut [Box<dyn InspectorSection>] {
        &mut self.inspector_sections
    }
}
