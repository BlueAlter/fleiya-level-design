//! Floating bottom-left animation preview card inside the TILE ATLAS viewport.

use egui::{vec2, Align2, CornerRadius, FontId, Pos2, Rect, StrokeKind, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_ui_api::AssetService;

use super::right_panel::paint_tile_thumbnail;
use super::state::TilesetViewState;

/// Renders the floating animation preview card at the bottom-left corner of the atlas viewport.
pub fn render_animation_preview_card(
    ui: &mut Ui,
    t: &Tokens,
    viewport_rect: Rect,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &TilesetViewState,
) {
    if !state.show_animation_preview {
        return;
    }

    // Determine target anchor to preview: selected anchor, or first available animation
    let active_anchor_id = state
        .selected_animation_anchor
        .or_else(|| tileset.animations.keys().next().copied());

    let Some(anchor_id) = active_anchor_id else {
        return;
    };

    let Some(anim) = tileset.animations.get(&anchor_id) else {
        return;
    };

    if anim.frames.is_empty() {
        return;
    }

    // Request continuous repaint while animation is playing for smooth playback
    if state.is_animation_playing {
        ui.ctx().request_repaint();
    }

    let active_frame_tile = state.active_animation_frame(tileset, anchor_id);
    let frame_index = anim
        .frames
        .iter()
        .position(|&f| f == active_frame_tile)
        .unwrap_or(0);

    let card_w = 88.0;
    let card_h = 96.0;
    let card_pos = Pos2::new(
        viewport_rect.left() + 12.0,
        viewport_rect.bottom() - card_h - 12.0,
    );
    let card_rect = Rect::from_min_size(card_pos, vec2(card_w, card_h));

    let p = ui.painter().with_clip_rect(viewport_rect);

    // Background & outline matching #badge and zoom %
    p.rect_filled(
        card_rect,
        CornerRadius::ZERO,
        lc_canvas::theme::tint(t.bg, 0.90),
    );
    p.rect_stroke(
        card_rect,
        CornerRadius::ZERO,
        t.outline(),
        StrokeKind::Inside,
    );

    // Title label: PREVIEW #<anchor_id>
    let title_text = format!("PREVIEW #{anchor_id}");
    p.text(
        Pos2::new(card_rect.center().x, card_rect.min.y + 10.0),
        Align2::CENTER_CENTER,
        title_text,
        FontId::new(9.5, lc_canvas::theme::mono_for(ui.ctx())),
        t.accent_ink,
    );

    // Center tile preview box
    let thumb_size = 44.0;
    let thumb_rect = Rect::from_min_size(
        Pos2::new(
            card_rect.center().x - thumb_size / 2.0,
            card_rect.min.y + 20.0,
        ),
        vec2(thumb_size, thumb_size),
    );
    p.rect_filled(thumb_rect, CornerRadius::ZERO, t.sunken);
    p.rect_stroke(
        thumb_rect,
        CornerRadius::ZERO,
        t.hairline(),
        StrokeKind::Inside,
    );

    let tile_ref = TileRef {
        tileset_id: tileset.id,
        tile_id: active_frame_tile,
        flip_x: false,
        flip_y: false,
    };
    paint_tile_thumbnail(&p, ui.ctx(), thumb_rect, project, assets, &tile_ref);

    // Frame counter & speed: F{i+1}/{total} (dur ms)
    let frame_info = format!(
        "F{}/{} ({}ms)",
        frame_index + 1,
        anim.frames.len(),
        anim.frame_duration_ms
    );
    p.text(
        Pos2::new(card_rect.center().x, card_rect.max.y - 10.0),
        Align2::CENTER_CENTER,
        frame_info,
        FontId::proportional(10.0),
        t.muted,
    );
}
