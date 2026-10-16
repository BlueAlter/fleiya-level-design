use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::error::CoreError;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::selection::Selection;
use lc_ui_api::view::{DocumentView, PanelId, PanelView};

#[derive(Debug)]
struct DummyCommand {
    applied: bool,
}

impl Command for DummyCommand {
    fn apply(&mut self, _p: &mut Project) -> Result<(), CoreError> {
        self.applied = true;
        Ok(())
    }

    fn revert(&mut self, _p: &mut Project) {
        self.applied = false;
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Dummy Command".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::ProjectSettings)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct DummyDocView {
    kind: DocKind,
}

impl DocumentView for DummyDocView {
    fn title(&self, _p: &Project) -> String {
        "Dummy Settings".to_string()
    }

    fn ui(&mut self, _ui: &mut egui::Ui, cx: &mut EditorCtx) {
        cx.exec(DummyCommand { applied: false });
        cx.emit(EditorEvent::Status("Dummy doc rendered".to_string()));
    }

    fn is_valid(&self, _p: &Project) -> bool {
        true
    }

    fn doc_kind(&self) -> DocKind {
        self.kind
    }
}

struct DummyPanelView {
    id: PanelId,
}

impl PanelView for DummyPanelView {
    fn id(&self) -> PanelId {
        self.id.clone()
    }

    fn title(&self) -> &str {
        "Dummy Panel"
    }

    fn ui(&mut self, _ui: &mut egui::Ui, cx: &mut EditorCtx) {
        cx.emit(EditorEvent::SelectionChanged(Selection::None));
    }
}

mod dummy_feature {
    use super::*;

    pub fn register(reg: &mut Registry) {
        reg.register_document_factory(|kind| match kind {
            DocKind::ProjectSettings => Some(Box::new(DummyDocView { kind })),
            _ => None,
        });

        reg.register_panel(Box::new(DummyPanelView {
            id: PanelId::new("dummy_panel"),
        }));
    }
}

#[test]
fn test_dummy_feature_registration_and_dispatch() {
    let mut reg = Registry::new();
    dummy_feature::register(&mut reg);

    // 1. Verify DocumentView registration and instantiation
    let doc_opt = reg.create_document(DocKind::ProjectSettings);
    assert!(doc_opt.is_some(), "DocumentView should be created");
    let mut doc = doc_opt.expect("doc exists");

    let project = Project::new("TestProject");
    assert_eq!(doc.title(&project), "Dummy Settings");
    assert!(doc.is_valid(&project));
    assert_eq!(doc.doc_kind(), DocKind::ProjectSettings);

    // 2. Verify PanelView registration and retrieval
    let panel_opt = reg.find_panel(&PanelId::new("dummy_panel"));
    assert!(panel_opt.is_some(), "PanelView should be found by id");
    let panel = panel_opt.expect("panel exists");
    assert_eq!(panel.id(), PanelId::new("dummy_panel"));
    assert_eq!(panel.title(), "Dummy Panel");

    let panel_mut = reg.find_panel_mut(&PanelId::new("dummy_panel"));
    assert!(
        panel_mut.is_some(),
        "PanelView should be found mutably by id"
    );

    // Selection::is_empty checks
    assert!(Selection::None.is_empty());
    assert!(Selection::Entities(vec![]).is_empty());
    assert!(!Selection::Entities(vec![lc_core::id::EntityId(1)]).is_empty());

    // 3. Test EditorCtx interaction with DummyDocView
    let undo_stack = UndoStack::new();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();

    {
        let mut cx = EditorCtx::new(&project, &undo_stack, &mut events, &mut pending_commands);

        // We can test UI method with an egui test context
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                doc.ui(ui, &mut cx);
            });
        });
        output.textures_delta.clear();
    }

    // Verify command was queued
    assert_eq!(pending_commands.len(), 1);
    assert_eq!(pending_commands[0].label(), "Dummy Command");

    // Verify event was emitted
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        EditorEvent::Status("Dummy doc rendered".to_string())
    );

    // 4. Verify LayerHandler registration and find_layer_handler alias
    struct DummyLayerHandler;
    impl lc_ui_api::canvas::LayerHandler for DummyLayerHandler {
        fn kind(&self) -> lc_core::layer::LayerKind {
            lc_core::layer::LayerKind::Tiles
        }
        fn render(&self, _ctx: &mut lc_ui_api::canvas::LayerRenderCtx) {}
        fn tools(&self) -> Vec<Box<dyn lc_ui_api::canvas::CanvasTool>> {
            Vec::new()
        }
    }

    reg.register_layer_handler(Box::new(DummyLayerHandler));
    assert!(reg
        .layer_handler(lc_core::layer::LayerKind::Tiles)
        .is_some());
    assert!(reg
        .find_layer_handler(lc_core::layer::LayerKind::Tiles)
        .is_some());
    assert!(reg
        .find_layer_handler(lc_core::layer::LayerKind::Entities)
        .is_none());
}
