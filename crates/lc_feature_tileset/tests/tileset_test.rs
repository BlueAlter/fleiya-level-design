//! Integration and headless UI tests for lc_feature_tileset.

use glam::UVec2;
use lc_core::command::{DocKind, UndoStack};
use lc_core::defs::TilesetDef;
use lc_core::id::{IdGenerator, TilesetDefId};
use lc_core::project::Project;
use lc_feature_tileset::commands::{
    AddTilesetDefCommand, RemoveTilesetDefCommand, UpdateTilesetDefCommand,
};
use lc_feature_tileset::palette::TilePalettePanel;
use lc_feature_tileset::view::TilesetDocumentView;
use lc_ui_api::assets::{AssetError, AssetService, NoopAssetService};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::session::{SessionState, TileSelection};
use lc_ui_api::view::{DocumentView, PanelView};

struct MockTextureService {
    size: [usize; 2],
}

impl AssetService for MockTextureService {
    fn load_texture(
        &self,
        ctx: &egui::Context,
        relative_path: &str,
    ) -> Result<egui::TextureHandle, AssetError> {
        let pixels = vec![egui::Color32::WHITE; self.size[0] * self.size[1]];
        let image = egui::ColorImage::new(self.size, pixels);
        Ok(ctx.load_texture(relative_path, image, egui::TextureOptions::NEAREST))
    }

    fn invalidate(&self, _relative_path: &str) {}
}

fn create_test_project_with_tileset() -> (Project, TilesetDefId) {
    let mut gen = IdGenerator::new();
    let tileset_id = gen.next_tileset_def_id();
    let tileset = TilesetDef {
        id: tileset_id,
        name: "Dungeon Tiles".to_string(),
        path: "tiles/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };

    let mut project = Project::new("Test Project");
    project.defs.tilesets.push(tileset);
    (project, tileset_id)
}

#[test]
fn test_factory_and_panel_registration() {
    let mut reg = Registry::new();
    lc_feature_tileset::register(&mut reg);

    // Factory creates TilesetDocumentView for DocKind::TilesetDef
    let doc = reg.create_document(DocKind::TilesetDef(TilesetDefId(42)));
    assert!(doc.is_some());
    assert_eq!(
        doc.unwrap().doc_kind(),
        DocKind::TilesetDef(TilesetDefId(42))
    );

    // Factory returns None for other DocKinds
    assert!(reg.create_document(DocKind::ProjectSettings).is_none());

    // Panel is registered
    let panel = reg.find_panel(&TilePalettePanel::PANEL_ID.into());
    assert!(panel.is_some());
    assert_eq!(panel.unwrap().title(), "Tile Palette");
}

#[test]
fn test_tileset_document_view_metadata_and_validity() {
    let (mut project, tileset_id) = create_test_project_with_tileset();
    let view = TilesetDocumentView::new(tileset_id);

    assert_eq!(view.doc_kind(), DocKind::TilesetDef(tileset_id));
    assert!(view.is_valid(&project));
    assert_eq!(view.title(&project), "Tileset: Dungeon Tiles");

    // Remove tileset definition
    project.defs.tilesets.clear();
    assert!(!view.is_valid(&project));
    assert_eq!(view.title(&project), "Tileset (deleted)");
}

#[test]
fn test_tileset_document_view_ui_rendering() {
    let (project, tileset_id) = create_test_project_with_tileset();
    let mut view = TilesetDocumentView::new(tileset_id);

    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = MockTextureService { size: [64, 64] };

    let ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        let mut output = ctx.run_ui(Default::default(), |ui| {
            view.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    // Now test with fallback / error asset service
    let noop_assets = NoopAssetService;
    {
        let mut cx2 = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &noop_assets,
        );

        let mut output2 = ctx.run_ui(Default::default(), |ui| {
            view.ui(ui, &mut cx2);
        });
        output2.textures_delta.clear();
    }
}

#[test]
fn test_tile_palette_panel_empty_state() {
    let mut panel = TilePalettePanel::new();
    assert_eq!(panel.id().as_str(), TilePalettePanel::PANEL_ID);
    assert_eq!(panel.title(), "Tile Palette");

    let project = Project::new("Empty Project");
    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        let mut output = ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    assert!(events.is_empty());
}

#[test]
fn test_tile_palette_panel_selection_events_and_cx_readability() {
    let (project, tileset_id) = create_test_project_with_tileset();
    let mut panel = TilePalettePanel::new();

    let undo = UndoStack::new();
    let mut session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    // 64x64 texture with 16x16 tiles = 4 cols x 4 rows = 16 tiles
    let assets = MockTextureService { size: [64, 64] };

    // Initial render and emit single selection
    let single_sel = TileSelection::single(tileset_id, 5);
    let ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        let mut output = ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();

        cx.emit(EditorEvent::SetTileSelection(Some(single_sel.clone())));
    }

    assert_eq!(
        events,
        vec![EditorEvent::SetTileSelection(Some(single_sel.clone()))]
    );

    // Update session state as shell does and verify readability via cx.tile_selection()
    session.tile_selection = Some(single_sel.clone());
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.tile_selection(), Some(&single_sel));
        assert_eq!(
            cx.tile_selection().unwrap().primary_tile().unwrap().tile_id,
            5
        );
    }

    // Emit rectangular tile selection and verify
    let rect_sel = TileSelection::rect(tileset_id, vec![0, 1, 4, 5], 2);
    events.clear();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        cx.emit(EditorEvent::SetTileSelection(Some(rect_sel.clone())));
    }

    assert_eq!(
        events,
        vec![EditorEvent::SetTileSelection(Some(rect_sel.clone()))]
    );

    session.tile_selection = Some(rect_sel.clone());
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.tile_selection(), Some(&rect_sel));
        assert_eq!(cx.tile_selection().unwrap().width, 2);
        assert_eq!(cx.tile_selection().unwrap().tiles.len(), 4);
    }

    // Clear selection
    events.clear();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        cx.emit(EditorEvent::SetTileSelection(None));
    }
    assert_eq!(events, vec![EditorEvent::SetTileSelection(None)]);

    session.tile_selection = None;
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.tile_selection(), None);
    }
}

#[test]
fn test_tileset_commands_undo_redo_round_trip() {
    let mut project = Project::new("Command Test");
    let mut undo_stack = UndoStack::new();

    let def = TilesetDef {
        id: TilesetDefId(10),
        name: "World Tiles".to_string(),
        path: "tiles/world.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };

    // 1. Add
    let add_cmd = Box::new(AddTilesetDefCommand::new(def.clone()));
    let focus = undo_stack
        .execute(add_cmd, &mut project)
        .expect("add should execute");
    assert_eq!(focus, Some(DocKind::TilesetDef(TilesetDefId(10))));
    assert_eq!(project.defs.tilesets.len(), 1);

    // 2. Edit
    let mut updated_def = def.clone();
    updated_def.name = "World Tiles Renamed".to_string();
    let edit_cmd = Box::new(UpdateTilesetDefCommand::new(updated_def.clone()));
    undo_stack
        .execute(edit_cmd, &mut project)
        .expect("edit should execute");
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles Renamed"
    );

    // 3. Undo edit
    let _ = undo_stack.undo(&mut project);
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles"
    );

    // 4. Redo edit
    let _ = undo_stack.redo(&mut project);
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles Renamed"
    );

    // 5. Remove
    let remove_cmd = Box::new(RemoveTilesetDefCommand::new(TilesetDefId(10)));
    undo_stack
        .execute(remove_cmd, &mut project)
        .expect("remove should execute");
    assert_eq!(project.defs.tilesets.len(), 0);

    // 6. Undo remove
    let _ = undo_stack.undo(&mut project);
    assert_eq!(project.defs.tilesets.len(), 1);
    assert_eq!(
        project.defs.tileset(TilesetDefId(10)).unwrap().name,
        "World Tiles Renamed"
    );
}
