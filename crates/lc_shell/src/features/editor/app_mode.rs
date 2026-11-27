//! Application modes (Editor, Enums, Tilesets) and vertical left workspace rail.

use egui::{
    vec2, Align2, Color32, CornerRadius, CursorIcon, FontId, Response, Sense, Stroke, StrokeKind,
    Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;

/// Fixed width of the vertical left workspace rail.
pub const WORKSPACE_RAIL_WIDTH: f32 = 44.0;

/// Top-level application mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Level and world editor with dock panels (currently disabled in UI).
    Editor,
    /// Dedicated layer definitions management mode.
    Layers,
    /// Dedicated entity definitions management mode.
    Entities,
    /// Dedicated enum definitions management mode.
    #[default]
    Enums,
    /// Dedicated tileset definitions and atlas management mode.
    Tilesets,
    /// Dedicated application and project settings workspace.
    Settings,
}

impl AppMode {
    /// Returns the workspace display title shown centered in the top bar.
    pub const fn workspace_title(self) -> &'static str {
        match self {
            Self::Editor => "EDITOR WORKSPACE",
            Self::Layers => "LAYER WORKSPACE",
            Self::Entities => "ENTITY WORKSPACE",
            Self::Enums => "ENUM WORKSPACE",
            Self::Tilesets => "TILESET WORKSPACE",
            Self::Settings => "SETTINGS WORKSPACE",
        }
    }
}

/// Specification for a workspace button in the vertical left rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceRailItem {
    /// Target mode if implemented; `None` for upcoming workspaces.
    pub mode: Option<AppMode>,
    /// Icon rendered inside the square rail button.
    pub icon: Icon,
    /// Tooltip describing the workspace and shortcut.
    pub tooltip: &'static str,
    /// Whether this workspace can currently be activated.
    pub enabled: bool,
}

/// Workspace items in vertical rail order: Editor, Layers, Entities, Tilesets, Enums.
pub const WORKSPACE_RAIL_ITEMS: [WorkspaceRailItem; 5] = [
    WorkspaceRailItem {
        mode: Some(AppMode::Editor),
        icon: Icon::Level,
        tooltip: "Editor Workspace (Disabled)",
        enabled: false,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Layers),
        icon: Icon::Layer,
        tooltip: "Layers Workspace (Ctrl+2)",
        enabled: true,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Entities),
        icon: Icon::Entity,
        tooltip: "Entities Workspace (Ctrl+3)",
        enabled: true,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Tilesets),
        icon: Icon::Tileset,
        tooltip: "Tilesets Workspace (Ctrl+4)",
        enabled: true,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Enums),
        icon: Icon::Enum,
        tooltip: "Enums Workspace (Ctrl+5)",
        enabled: true,
    },
];

/// Settings workspace button pinned to the bottom of the vertical left rail.
pub const SETTINGS_RAIL_ITEM: WorkspaceRailItem = WorkspaceRailItem {
    mode: Some(AppMode::Settings),
    icon: Icon::Settings,
    tooltip: "Settings Workspace (Ctrl+6)",
    enabled: true,
};

/// Returns true if the unsaved indicator dot should be shown for the given mode.
pub fn shows_unsaved(mode: AppMode, enum_unsaved: bool) -> bool {
    matches!(mode, AppMode::Enums) && enum_unsaved
}

/// Renders the vertical icon-only workspace switcher rail on the left edge.
pub fn render_workspace_rail(
    ui: &mut Ui,
    active_mode: &mut AppMode,
    enum_unsaved: bool,
    t: &Tokens,
) {
    let button_size = 30.0;
    let gap = 6.0;
    let rail_rect = ui.available_rect_before_wrap();

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.spacing_mut().item_spacing.y = gap;

        for item in WORKSPACE_RAIL_ITEMS {
            let is_active = item.enabled && item.mode == Some(*active_mode);
            let show_dot = item
                .mode
                .is_some_and(|mode| shows_unsaved(mode, enum_unsaved));

            let resp = render_rail_button(
                ui,
                item.icon,
                is_active,
                item.enabled,
                show_dot,
                button_size,
                item.tooltip,
                t,
            );
            if item.enabled && resp.clicked() {
                if let Some(mode) = item.mode {
                    *active_mode = mode;
                }
            }
        }
    });

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(8.0);
        let is_active = SETTINGS_RAIL_ITEM.enabled && SETTINGS_RAIL_ITEM.mode == Some(*active_mode);
        let resp = render_rail_button(
            ui,
            SETTINGS_RAIL_ITEM.icon,
            is_active,
            SETTINGS_RAIL_ITEM.enabled,
            false,
            button_size,
            SETTINGS_RAIL_ITEM.tooltip,
            t,
        );
        if SETTINGS_RAIL_ITEM.enabled && resp.clicked() {
            if let Some(mode) = SETTINGS_RAIL_ITEM.mode {
                *active_mode = mode;
            }
        }
    });

    // Draw 1 px vertical hairline along the right border of the rail.
    ui.painter()
        .vline(rail_rect.max.x, rail_rect.y_range(), t.hairline());
}

#[allow(clippy::too_many_arguments)]
fn render_rail_button(
    ui: &mut Ui,
    icon: Icon,
    is_active: bool,
    enabled: bool,
    show_unsaved: bool,
    size: f32,
    tooltip: &str,
    t: &Tokens,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), sense);
    let mut response = response.on_hover_text(tooltip);
    if enabled {
        response = response.on_hover_cursor(CursorIcon::PointingHand);
    }

    let (bg, stroke, icon_color) = if !enabled {
        (Color32::TRANSPARENT, Stroke::NONE, t.faint)
    } else if is_active {
        (t.accent, Stroke::NONE, t.invert_fg)
    } else if response.hovered() {
        (t.sunken, t.hairline(), t.ink_strong)
    } else {
        (t.panel, t.hairline(), t.muted)
    };

    ui.painter()
        .rect(rect, CornerRadius::ZERO, bg, stroke, StrokeKind::Inside);

    // Icon glyph
    let icon_pos = rect.center();
    let glyph = icon.glyph();
    let font_id = FontId::new(15.0, lc_canvas::icons::family(ui.ctx()));
    ui.painter()
        .text(icon_pos, Align2::CENTER_CENTER, glyph, font_id, icon_color);

    // Unsaved indicator dot
    if show_unsaved {
        let dot_pos = egui::pos2(rect.max.x - 5.0, rect.min.y + 5.0);
        let dot_color = if is_active { t.invert_fg } else { t.warn };
        ui.painter().circle_filled(dot_pos, 3.0, dot_color);
    }

    response
}
