//! Regression test: after Save an enum must be reported clean again.
//!
//! Value edits (reorder, rename, colour...) change `enum_draft` only. The per-enum shadow
//! `enum_drafts[id].def` used by the dirty check went stale, so after Save the
//! Save / Revert buttons stayed enabled (and Revert appeared to do nothing).

use lc_core::defs::{EnumDef, EnumValueDef};
use lc_core::id::EnumDefId;
use lc_core::project::Project;
use lc_shell::enum_view::sync_enum_draft_shadows;
use lc_shell::ShellApp;

fn value(id: &str) -> EnumValueDef {
    EnumValueDef {
        id: id.to_string(),
        name: id.to_string(),
        color: [255, 255, 255, 255],
        tile: None,
    }
}

#[test]
fn reorder_then_save_returns_to_clean() {
    let id = EnumDefId(1);
    let mut project = Project::new("SaveSync");
    project.defs.enums.push(EnumDef {
        id,
        name: "WeaponType".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![value("Sword"), value("Bow"), value("Axe")],
    });
    let mut app = ShellApp::new(project);
    app.selected_enum_def = Some(id);
    assert!(!app.enum_is_dirty(), "freshly loaded enum must be clean");

    // reorder in the working copy only, exactly like the UI does
    let draft = app.enum_draft.iter_mut().find(|e| e.id == id).unwrap();
    draft.values.swap(0, 2);
    app.enum_draft_revision += 1;
    assert!(app.enum_is_dirty(), "reordering must enable Save / Revert");

    // what the Save button does
    let saved = app.enum_draft.iter().find(|e| e.id == id).cloned().unwrap();
    app.project.defs.enums[0] = saved.clone();
    app.enum_saved_defs.insert(id, saved.clone());
    if let Some(slot) = app.enum_saved.iter_mut().find(|e| e.id == id) {
        *slot = saved;
    }
    if let Some(d) = app.enum_drafts.get_mut(&id) {
        d.saved_revision = d.revision;
    }

    // the stale shadow is what used to keep the enum dirty after saving
    assert!(app.enum_is_dirty(), "precondition: stale shadow keeps it dirty");
    sync_enum_draft_shadows(&mut app);
    assert!(!app.enum_is_dirty(), "after Save the enum must be clean again");
}

#[test]
fn sync_keeps_unsaved_edits_dirty() {
    let id = EnumDefId(1);
    let mut project = Project::new("SaveSync2");
    project.defs.enums.push(EnumDef {
        id,
        name: "WeaponType".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![value("Sword"), value("Bow")],
    });
    let mut app = ShellApp::new(project);
    app.selected_enum_def = Some(id);

    let draft = app.enum_draft.iter_mut().find(|e| e.id == id).unwrap();
    draft.values.swap(0, 1);
    app.enum_draft_revision += 1;

    sync_enum_draft_shadows(&mut app);
    assert!(app.enum_is_dirty(), "syncing must never hide real unsaved edits");
}
