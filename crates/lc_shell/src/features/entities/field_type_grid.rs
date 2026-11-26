//! Symmetric `+ SINGLE VALUE` & `+ ARRAY` buttons with LDtk's 11-type colored grid popup.

use egui::{
    vec2, Color32, CornerRadius, CursorIcon, Layout, Popup, PopupCloseBehavior, Rect, RichText,
    Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::theme::{self, Tokens};
use lc_core::fields::FieldType;

use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_COMPACT;

struct TypeCardSpec {
    title: &'static str,
    glyph: &'static str,
    hint: &'static str,
    color: Color32,
    kind: TypeCardKind,
}

#[derive(Clone, Copy)]
enum TypeCardKind {
    Int,
    Float,
    Bool,
    String,
    Multilines,
    Color,
    Enum,
    FilePath,
    Tile,
    EntityRef,
    Point,
}

/// Returns the LDtk semantic badge/card color for a base [`FieldType`].
pub fn field_type_color(base: &FieldType) -> Color32 {
    match base {
        FieldType::Int | FieldType::Float => Color32::from_rgb(55, 135, 155),
        FieldType::Bool => Color32::from_rgb(138, 92, 148),
        FieldType::String | FieldType::Multilines => Color32::from_rgb(162, 82, 46),
        FieldType::Color | FieldType::Tile => Color32::from_rgb(108, 154, 62),
        FieldType::Enum(_) => Color32::from_rgb(164, 54, 54),
        FieldType::FilePath | FieldType::EntityRef | FieldType::Point | FieldType::Array(_) => {
            Color32::from_rgb(84, 92, 152)
        }
    }
}

const TYPE_CARDS: [TypeCardSpec; 11] = [
    TypeCardSpec {
        title: "Integer",
        glyph: "123",
        hint: "Integer number",
        color: Color32::from_rgb(55, 135, 155),
        kind: TypeCardKind::Int,
    },
    TypeCardSpec {
        title: "Float",
        glyph: "1.0",
        hint: "Floating-point number",
        color: Color32::from_rgb(55, 135, 155),
        kind: TypeCardKind::Float,
    },
    TypeCardSpec {
        title: "Boolean",
        glyph: "✓",
        hint: "True / False flag",
        color: Color32::from_rgb(138, 92, 148),
        kind: TypeCardKind::Bool,
    },
    TypeCardSpec {
        title: "String",
        glyph: "\"Ab\"",
        hint: "Single-line text",
        color: Color32::from_rgb(162, 82, 46),
        kind: TypeCardKind::String,
    },
    TypeCardSpec {
        title: "Multilines",
        glyph: "\"Ab\\n\"",
        hint: "Multi-line text block",
        color: Color32::from_rgb(162, 82, 46),
        kind: TypeCardKind::Multilines,
    },
    TypeCardSpec {
        title: "Color",
        glyph: "Col",
        hint: "RGBA color value",
        color: Color32::from_rgb(108, 154, 62),
        kind: TypeCardKind::Color,
    },
    TypeCardSpec {
        title: "Enum",
        glyph: "Enu",
        hint: "Value from an Enum definition",
        color: Color32::from_rgb(164, 54, 54),
        kind: TypeCardKind::Enum,
    },
    TypeCardSpec {
        title: "File path",
        glyph: "*.*",
        hint: "Relative file path",
        color: Color32::from_rgb(84, 92, 152),
        kind: TypeCardKind::FilePath,
    },
    TypeCardSpec {
        title: "Tile",
        glyph: "Tile",
        hint: "Tileset tile rectangle",
        color: Color32::from_rgb(108, 154, 62),
        kind: TypeCardKind::Tile,
    },
    TypeCardSpec {
        title: "Entity ref",
        glyph: "Ent",
        hint: "Reference to another entity instance",
        color: Color32::from_rgb(84, 92, 152),
        kind: TypeCardKind::EntityRef,
    },
    TypeCardSpec {
        title: "Point",
        glyph: "X,Y",
        hint: "Grid coordinate point",
        color: Color32::from_rgb(84, 92, 152),
        kind: TypeCardKind::Point,
    },
];

/// Renders symmetric equal-width `+ SINGLE VALUE` and `+ ARRAY` buttons that open
/// LDtk's 5-column colored 11-type grid picker popup.
pub fn render_add_field_buttons(
    ui: &mut Ui,
    app: &ShellApp,
    t: &Tokens,
    left_w: f32,
    entity_id: lc_core::id::EntityDefId,
) -> Option<FieldType> {
    let mut picked: Option<FieldType> = None;
    let gap = 6.0;
    let btn_w = ((left_w - gap) * 0.5).max(80.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if let Some(ft) = render_grid_trigger(
            ui,
            app,
            t,
            ("entity_add_single", entity_id),
            "+ SINGLE VALUE",
            "Add a field that contains a single value.",
            false,
            btn_w,
        ) {
            picked = Some(ft);
        }
        if let Some(ft) = render_grid_trigger(
            ui,
            app,
            t,
            ("entity_add_array", entity_id),
            "+ ARRAY",
            "Add a field that contains an array of multiple values.",
            true,
            btn_w,
        ) {
            picked = Some(ft);
        }
    });

    picked
}

#[allow(clippy::too_many_arguments)]
fn render_grid_trigger(
    ui: &mut Ui,
    app: &ShellApp,
    t: &Tokens,
    salt: impl egui::AsId,
    label: &str,
    tooltip: &str,
    wrap_array: bool,
    width: f32,
) -> Option<FieldType> {
    let btn_id = ui.make_persistent_id(salt);
    let popup_id = btn_id.with("type_grid_popup");

    let (rect, mut resp) =
        ui.allocate_exact_size(vec2(width, CONTROL_HEIGHT_COMPACT), Sense::click());
    resp = resp
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    if ui.is_rect_visible(rect) {
        let is_open = Popup::is_id_open(ui.ctx(), popup_id);
        let fill = if is_open || resp.hovered() {
            theme::tint(t.accent, 0.85)
        } else {
            t.accent
        };
        let border = if is_open || resp.hovered() {
            Stroke::new(1.0, t.ink_strong)
        } else {
            t.hairline()
        };
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::ZERO, fill);
        p.rect_stroke(rect, CornerRadius::ZERO, border, StrokeKind::Inside);
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
        );
        child.add(
            egui::Label::new(RichText::new(label).size(11.0).strong().color(t.invert_fg))
                .selectable(false),
        );
    }

    let mut chosen: Option<FieldType> = None;
    let mut keep_open = false;

    Popup::from_toggle_button_response(&resp)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(404.0);
            let card_w = 74.0;
            let card_h = 66.0;
            let spacing = 6.0;

            let enum_pick_id = popup_id.with("picking_enum");
            let picking_enum = ui
                .ctx()
                .data_mut(|d| d.get_temp::<bool>(enum_pick_id).unwrap_or(false));

            if picking_enum {
                keep_open = true;
                ui.label(
                    RichText::new("PICK ENUM DEFINITION")
                        .size(11.0)
                        .strong()
                        .color(t.accent_ink),
                );
                ui.add_space(4.0);
                if app.project.defs.enums.is_empty() {
                    ui.label(
                        RichText::new("No Enum defined yet. Create one in Enums Workspace.")
                            .size(11.5)
                            .color(t.warn),
                    );
                } else {
                    for ed in &app.project.defs.enums {
                        if ui.button(&ed.name).clicked() {
                            chosen = Some(FieldType::Enum(ed.id));
                            ui.ctx().data_mut(|d| d.insert_temp(enum_pick_id, false));
                            Popup::close_id(ui.ctx(), popup_id);
                        }
                    }
                }
                ui.add_space(6.0);
                if ui.button("<- Back to types").clicked() {
                    ui.ctx().data_mut(|d| d.insert_temp(enum_pick_id, false));
                }
            } else {
                for chunk in TYPE_CARDS.chunks(5) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        for spec in chunk {
                            if render_type_card(ui, t, spec, vec2(card_w, card_h)) {
                                match spec.kind {
                                    TypeCardKind::Int => chosen = Some(FieldType::Int),
                                    TypeCardKind::Float => chosen = Some(FieldType::Float),
                                    TypeCardKind::Bool => chosen = Some(FieldType::Bool),
                                    TypeCardKind::String => chosen = Some(FieldType::String),
                                    TypeCardKind::Multilines => {
                                        chosen = Some(FieldType::Multilines)
                                    }
                                    TypeCardKind::Color => chosen = Some(FieldType::Color),
                                    TypeCardKind::Enum => {
                                        if app.project.defs.enums.len() == 1 {
                                            chosen =
                                                Some(FieldType::Enum(app.project.defs.enums[0].id));
                                        } else {
                                            ui.ctx()
                                                .data_mut(|d| d.insert_temp(enum_pick_id, true));
                                            keep_open = true;
                                        }
                                    }
                                    TypeCardKind::FilePath => chosen = Some(FieldType::FilePath),
                                    TypeCardKind::Tile => chosen = Some(FieldType::Tile),
                                    TypeCardKind::EntityRef => chosen = Some(FieldType::EntityRef),
                                    TypeCardKind::Point => chosen = Some(FieldType::Point),
                                }
                                if !keep_open {
                                    Popup::close_id(ui.ctx(), popup_id);
                                }
                            }
                        }
                    });
                    ui.add_space(spacing);
                }
            }
        });

    if !Popup::is_id_open(ui.ctx(), popup_id) {
        ui.ctx()
            .data_mut(|d| d.insert_temp(popup_id.with("picking_enum"), false));
    }

    chosen.map(|ft| {
        if wrap_array {
            FieldType::Array(Box::new(ft))
        } else {
            ft
        }
    })
}

fn render_type_card(ui: &mut Ui, t: &Tokens, spec: &TypeCardSpec, size: egui::Vec2) -> bool {
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    resp = resp
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(spec.hint);

    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let bg = if resp.hovered() {
            theme::tint(spec.color, 0.92)
        } else {
            theme::tint(spec.color, 0.72)
        };
        p.rect_filled(rect, CornerRadius::ZERO, bg);
        p.rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(
                1.0,
                if resp.hovered() {
                    t.ink_strong
                } else {
                    spec.color
                },
            ),
            StrokeKind::Inside,
        );
        let inner = rect.shrink(4.0);
        p.rect_stroke(
            inner,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 70)),
            StrokeKind::Inside,
        );

        let top_rect = Rect::from_min_max(inner.min, egui::pos2(inner.max.x, inner.min.y + 20.0));
        let mut top_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(top_rect)
                .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
        );
        top_ui.add(
            egui::Label::new(
                RichText::new(spec.title)
                    .size(11.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .selectable(false),
        );

        let bot_rect = Rect::from_min_max(egui::pos2(inner.min.x, inner.min.y + 20.0), inner.max);
        let mut bot_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bot_rect)
                .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
        );
        bot_ui.add(
            egui::Label::new(
                RichText::new(spec.glyph)
                    .size(16.0)
                    .strong()
                    .color(Color32::from_rgba_unmultiplied(255, 255, 255, 190)),
            )
            .selectable(false),
        );
    }

    resp.clicked()
}
