//! Searchable tileset selection combo based on [`search_picker`].
//!
//! Provides a standardized selector dropdown generic over arbitrary ID types,
//! avoiding any dependency on `Project` or `lc_core`.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::tileset_select::{tileset_select, TilesetSelectItem};
//!
//! let items = vec![TilesetSelectItem::new(1, "WorldTiles"), TilesetSelectItem::new(2, "Dungeon")];
//! let mut selected = Some(1);
//! if tileset_select(ui, &items, &mut selected, true) {
//!     // handle selection change
//! }
//! ```

use egui::{vec2, CornerRadius, Pos2, Rect, RichText, Sense, StrokeKind, Ui};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

use super::search_picker::{
    search_picker, PickerChoice, PickerItem, SearchPickerConfig, SearchPickerState,
};
use super::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Description of an item selectable in [`tileset_select`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TilesetSelectItem<Id: Clone + PartialEq> {
    /// Unique identifier for this tileset option.
    pub id: Id,
    /// User-visible name or label.
    pub label: String,
}

impl<Id: Clone + PartialEq> TilesetSelectItem<Id> {
    /// Constructs a new [`TilesetSelectItem`].
    pub fn new(id: Id, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

/// Renders a searchable selection dropdown backed by [`search_picker`].
///
/// Returns `true` if the selection changed this frame.
pub fn tileset_select<Id: Clone + PartialEq + 'static>(
    ui: &mut Ui,
    items: &[TilesetSelectItem<Id>],
    selected: &mut Option<Id>,
    allow_none: bool,
) -> bool {
    let t = Tokens::get(ui.ctx());
    let state_id = ui.id().with("tileset_select_picker_state");
    let mut picker_state: SearchPickerState = ui
        .ctx()
        .data_mut(|d| d.get_temp(state_id).unwrap_or_default());

    let current_label = selected
        .as_ref()
        .and_then(|id| items.iter().find(|it| it.id == *id))
        .map(|it| it.label.as_str())
        .unwrap_or(if allow_none {
            "-- None --"
        } else {
            "Select Tileset..."
        });

    let is_placeholder = selected.is_none() && !allow_none;
    let text_color = if is_placeholder {
        t.muted
    } else {
        t.ink_strong
    };

    let mut changed = false;

    // Trigger button showing current selection + chevron
    let (rect, resp) = ui.allocate_exact_size(vec2(160.0, CONTROL_HEIGHT_FORM), Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if resp.hovered() { t.panel } else { t.sunken };
        painter.rect_filled(rect, CornerRadius::same(3), bg);
        painter.rect_stroke(
            rect,
            CornerRadius::same(3),
            t.hairline(),
            StrokeKind::Inside,
        );

        let icon_sz = 12.0;
        let right_x = rect.max.x - 12.0;
        let center_y = rect.center().y;
        paint_icon(
            painter,
            Pos2::new(right_x, center_y),
            Icon::ChevronDown,
            icon_sz,
            t.muted,
        );

        let max_text_w = rect.width() - 32.0;
        let text_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 8.0, rect.min.y),
            Pos2::new(rect.min.x + 8.0 + max_text_w, rect.max.y),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
        child.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(RichText::new(current_label).color(text_color));
        });
    }

    let picker_items: Vec<PickerItem<Id>> = items
        .iter()
        .map(|it| PickerItem::new(it.id.clone(), it.label.clone()))
        .collect();

    let mut cfg = SearchPickerConfig::new("Search tilesets...");
    if allow_none {
        cfg.none_label = Some("-- None --");
    }
    cfg.current = selected.clone();

    if let Some(choice) = search_picker(
        &resp,
        ui.id().with("tileset_select_popup"),
        &picker_items,
        &mut picker_state,
        &cfg,
    ) {
        match choice {
            PickerChoice::Item(id) => {
                if *selected != Some(id.clone()) {
                    *selected = Some(id);
                    changed = true;
                }
            }
            PickerChoice::None => {
                if selected.is_some() {
                    *selected = None;
                    changed = true;
                }
            }
        }
    }

    ui.ctx().data_mut(|d| d.insert_temp(state_id, picker_state));
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_tileset_select_item_constructor() {
        let item = TilesetSelectItem::new(42, "Overworld");
        assert_eq!(item.id, 42);
        assert_eq!(item.label, "Overworld");
    }

    #[test]
    fn test_tileset_select_headless_render() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let items = vec![
            TilesetSelectItem::new(1, "WorldTiles"),
            TilesetSelectItem::new(2, "DungeonTiles"),
        ];
        let mut selected = Some(1);

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            let changed = tileset_select(ui, &items, &mut selected, true);
            assert!(!changed);
            assert_eq!(selected, Some(1));
        });

        output.textures_delta.clear();
        assert!(
            !output.shapes.is_empty(),
            "tileset_select should produce render shapes"
        );
    }
}
