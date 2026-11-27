//! Application modes (Editor, Layers, Entities, Tilesets, Enums, Settings) and vertical left workspace rail.

use std::borrow::Cow;
use std::sync::{Arc, OnceLock};

use egui::load::Bytes;
use egui::{
    vec2, Color32, CornerRadius, CursorIcon, ImageSource, Rect, Response, Sense, Stroke,
    StrokeKind, Ui,
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

/// Custom SVG icon asset rendered inside the vertical left workspace rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RailSvgIcon {
    /// `assets/Icons/Editor.svg`
    Editor,
    /// `assets/Icons/Layers.svg`
    Layers,
    /// `assets/Icons/Entity.svg`
    Entity,
    /// `assets/Icons/Tileset.svg`
    Tileset,
    /// `assets/Icons/Enum.svg`
    Enum,
    /// `assets/Icons/Setting.svg`
    Setting,
}

impl RailSvgIcon {
    /// Stable virtual URI used by `egui_extras::SvgLoader` to cache rasterized sizes.
    pub const fn uri(self) -> &'static str {
        match self {
            Self::Editor => "bytes://workspace_rail/editor.svg",
            Self::Layers => "bytes://workspace_rail/layers.svg",
            Self::Entity => "bytes://workspace_rail/entity.svg",
            Self::Tileset => "bytes://workspace_rail/tileset.svg",
            Self::Enum => "bytes://workspace_rail/enum.svg",
            Self::Setting => "bytes://workspace_rail/setting.svg",
        }
    }

    /// Embedded raw SVG text from `assets/Icons/`.
    pub const fn raw_svg(self) -> &'static str {
        match self {
            Self::Editor => include_str!("../../../../../assets/Icons/Editor.svg"),
            Self::Layers => include_str!("../../../../../assets/Icons/Layers.svg"),
            Self::Entity => include_str!("../../../../../assets/Icons/Entity.svg"),
            Self::Tileset => include_str!("../../../../../assets/Icons/Tileset.svg"),
            Self::Enum => include_str!("../../../../../assets/Icons/Enum.svg"),
            Self::Setting => include_str!("../../../../../assets/Icons/Setting.svg"),
        }
    }

    /// Returns a tintable `ImageSource` (white fill normalized once for theme tinting).
    pub fn image_source(self) -> ImageSource<'static> {
        static EDITOR: OnceLock<Arc<[u8]>> = OnceLock::new();
        static LAYERS: OnceLock<Arc<[u8]>> = OnceLock::new();
        static ENTITY: OnceLock<Arc<[u8]>> = OnceLock::new();
        static TILESET: OnceLock<Arc<[u8]>> = OnceLock::new();
        static ENUM: OnceLock<Arc<[u8]>> = OnceLock::new();
        static SETTING: OnceLock<Arc<[u8]>> = OnceLock::new();

        let slot = match self {
            Self::Editor => &EDITOR,
            Self::Layers => &LAYERS,
            Self::Entity => &ENTITY,
            Self::Tileset => &TILESET,
            Self::Enum => &ENUM,
            Self::Setting => &SETTING,
        };
        let bytes = slot.get_or_init(|| normalize_svg_for_tint(self.raw_svg()));
        ImageSource::Bytes {
            uri: Cow::Borrowed(self.uri()),
            bytes: Bytes::Shared(Arc::clone(bytes)),
        }
    }
}

/// Converts black `#000000` fills in source SVGs to `#ffffff` so `egui::Image::tint`
/// applies theme token colours cleanly without altering the source `.svg` files.
fn normalize_svg_for_tint(raw_svg: &str) -> Arc<[u8]> {
    let normalized = raw_svg
        .replace("#000000", "#ffffff")
        .replace("#000;", "#fff;")
        .replace("#000}", "#fff}");
    Arc::from(normalized.into_bytes().into_boxed_slice())
}

/// Specification for a workspace button in the vertical left rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceRailItem {
    /// Target mode if implemented; `None` for upcoming workspaces.
    pub mode: Option<AppMode>,
    /// Semantic icon identifier preserved for metadata/uniqueness checks.
    pub icon: Icon,
    /// Custom SVG icon asset rendered inside the square rail button.
    pub svg_icon: RailSvgIcon,
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
        svg_icon: RailSvgIcon::Editor,
        tooltip: "Editor Workspace (Disabled)",
        enabled: false,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Layers),
        icon: Icon::Layer,
        svg_icon: RailSvgIcon::Layers,
        tooltip: "Layers Workspace (Ctrl+2)",
        enabled: true,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Entities),
        icon: Icon::Entity,
        svg_icon: RailSvgIcon::Entity,
        tooltip: "Entities Workspace (Ctrl+3)",
        enabled: true,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Tilesets),
        icon: Icon::Tileset,
        svg_icon: RailSvgIcon::Tileset,
        tooltip: "Tilesets Workspace (Ctrl+4)",
        enabled: true,
    },
    WorkspaceRailItem {
        mode: Some(AppMode::Enums),
        icon: Icon::Enum,
        svg_icon: RailSvgIcon::Enum,
        tooltip: "Enums Workspace (Ctrl+5)",
        enabled: true,
    },
];

/// Settings workspace button pinned to the bottom of the vertical left rail.
pub const SETTINGS_RAIL_ITEM: WorkspaceRailItem = WorkspaceRailItem {
    mode: Some(AppMode::Settings),
    icon: Icon::Settings,
    svg_icon: RailSvgIcon::Setting,
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
    egui_extras::install_image_loaders(ui.ctx());

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
                item.svg_icon,
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
            SETTINGS_RAIL_ITEM.svg_icon,
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
    svg_icon: RailSvgIcon,
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

    // Custom SVG icon centered inside the rail button, tinted to match theme tokens.
    let icon_rect = Rect::from_center_size(rect.center(), vec2(18.0, 18.0));
    egui::Image::new(svg_icon.image_source())
        .tint(icon_color)
        .paint_at(ui, icon_rect);

    // Unsaved indicator dot
    if show_unsaved {
        let dot_pos = egui::pos2(rect.max.x - 5.0, rect.min.y + 5.0);
        let dot_color = if is_active { t.invert_fg } else { t.warn };
        ui.painter().circle_filled(dot_pos, 3.0, dot_color);
    }

    response
}
