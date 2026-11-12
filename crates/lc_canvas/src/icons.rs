//! Semantic icon set for the editor.
//!
//! Icons come from [Phosphor](https://phosphoricons.com/) (MIT) through the
//! `egui-phosphor` crate. UI code should depend on [`Icon`] instead of the
//! upstream codepoint constants so the icon pack can be replaced without
//! touching feature crates.

use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, RichText, Ui};
use egui_phosphor::regular;

use crate::theme::{self, Tokens};

/// Font family registered by [`crate::theme::install_fonts`].
pub const FONT_FAMILY: &str = "phosphor";

macro_rules! define_icons {
    ($( $variant:ident => $glyph:ident, $doc:literal; )*) => {
        /// A semantic editor icon.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Icon {
            $(
                #[doc = $doc]
                $variant,
            )*
        }

        impl Icon {
            /// Every icon in the set, in declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// The Phosphor glyph for this icon.
            pub const fn glyph(self) -> &'static str {
                match self {
                    $(Self::$variant => regular::$glyph,)*
                }
            }

            /// Stable name used in tests and the icon gallery.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)*
                }
            }
        }
    };
}

define_icons! {
    Add => PLUS, "Create or add an item.";
    Remove => MINUS, "Remove an item or collapse a value.";
    Close => X, "Close a tab, dialog or panel.";
    Check => CHECK, "Confirm or mark as done.";
    Search => MAGNIFYING_GLASS, "Search or filter.";
    Settings => GEAR, "Open settings.";
    Help => QUESTION, "Open help.";
    Open => FOLDER_OPEN, "Open a file or project.";
    Save => FLOPPY_DISK, "Save the project.";
    Undo => ARROW_COUNTER_CLOCKWISE, "Undo the last command.";
    Redo => ARROW_CLOCKWISE, "Redo the last undone command.";
    Copy => COPY, "Copy.";
    Paste => CLIPBOARD, "Paste.";
    Duplicate => SQUARES_FOUR, "Duplicate.";
    Delete => TRASH, "Delete permanently.";
    Folder => FOLDER, "A folder.";
    Project => FOLDER_SIMPLE, "Project settings.";
    World => GLOBE, "A world.";
    Level => MAP_TRIFOLD, "A level.";
    Layer => STACK, "A layer.";
    Tileset => IMAGE_SQUARE, "A tileset.";
    Entity => CUBE, "An entity definition or instance.";
    Enum => LIST_BULLETS, "An enum definition.";
    Eye => EYE, "Visible.";
    EyeOff => EYE_SLASH, "Hidden.";
    Lock => LOCK, "Locked.";
    Unlock => LOCK_OPEN, "Unlocked.";
    Grid => GRID_NINE, "Grid or snap settings.";
    Tiles => SQUARES_FOUR, "A tile layer.";
    IntGrid => GRID_FOUR, "An IntGrid layer.";
    AutoLayer => MAGIC_WAND, "An automatic layer.";
    Brush => PAINT_BRUSH, "Paint tiles.";
    Pencil => PENCIL, "Draw freely.";
    Rectangle => RECTANGLE, "Draw a rectangle.";
    Eraser => ERASER, "Erase.";
    Picker => EYEDROPPER, "Pick a tile or colour.";
    Bucket => PAINT_BUCKET, "Fill an area.";
    Palette => PALETTE, "Tile palette.";
    ZoomIn => MAGNIFYING_GLASS_PLUS, "Zoom in.";
    ZoomOut => MAGNIFYING_GLASS_MINUS, "Zoom out.";
    Fit => CORNERS_OUT, "Fit the view.";
    Snap => MAGNET, "Snap to the grid.";
    Play => PLAY, "Play or resume.";
    Pause => PAUSE, "Pause.";
    Info => INFO, "Informational message.";
    Warning => WARNING, "Warning message.";
    Error => X_CIRCLE, "Error message.";
    Debug => BUG, "Debug message.";
    Clear => BROOM, "Clear the output.";
    Drag => DOTS_SIX_VERTICAL, "Drag handle.";
    Collapse => ARROWS_IN_LINE_VERTICAL, "Collapse.";
    Expand => ARROWS_OUT_LINE_VERTICAL, "Expand.";
    ChevronDown => CARET_DOWN, "Expand or move down.";
    ChevronRight => CARET_RIGHT, "Collapsed or move right.";
    ChevronLeft => CARET_LEFT, "Move left.";
    ChevronUp => CARET_UP, "Collapse or move up.";
    Image => IMAGE, "An image asset.";
    File => FILE, "A file.";
    NewFile => FILE_PLUS, "Create a file or document.";
    Cursor => CURSOR, "Selection cursor.";
    Hand => HAND, "Pan the view.";
    Select => SELECTION, "Select objects.";
    Move => ARROWS_OUT_CARDINAL, "Move objects.";
    Shape => SHAPES, "Shape or entity bounds.";
    Home => HOUSE, "Home or workspace.";
    Link => LINK, "Linked reference.";
    Unlink => LINK_BREAK, "Broken link.";
    Refresh => ARROW_CLOCKWISE, "Refresh or reload.";
    Star => STAR, "Favourite or pinned.";
    Tag => TAG, "Tag or metadata.";
    SplitHorizontal => SQUARE_SPLIT_HORIZONTAL, "Split horizontally.";
    SplitVertical => SQUARE_SPLIT_VERTICAL, "Split vertically.";
    Tree => TREE_STRUCTURE, "Hierarchy.";
    Window => APP_WINDOW, "A window or dialog.";
    Swatches => SWATCHES, "Color or tile swatches.";
    Sliders => SLIDERS_HORIZONTAL, "Sliders or settings adjustments.";
    Code => CODE, "Code or script.";
    MapPin => MAP_PIN, "Map pin marker.";
    Compass => COMPASS, "Compass navigation.";
    Sparkle => SPARKLE, "Sparkle or auto-rule.";
    ListPlus => LIST_PLUS, "Add item to list.";
    DotsThree => DOTS_THREE, "More actions.";
    DotsThreeVertical => DOTS_THREE_VERTICAL, "Three vertical dots (more actions).";
    Export => EXPORT, "Export file or item.";
    Import => ARROW_SQUARE_IN, "Import file or item.";
    PlusCircle => PLUS_CIRCLE, "Add with circle.";
    MinusCircle => MINUS_CIRCLE, "Remove with circle.";
    CheckCircle => CHECK_CIRCLE, "Confirmed with circle.";
}

impl Icon {
    /// Rich text for this icon in the icon font.
    pub fn rich(self, ctx: &egui::Context, size: f32, color: Color32) -> RichText {
        RichText::new(self.glyph())
            .font(FontId::new(size, family(ctx)))
            .color(color)
    }
}

/// Font family holding the icon glyphs.
///
/// Falls back to the proportional family in headless tests where
/// [`crate::theme::install_fonts`] has not run.
pub fn family(ctx: &egui::Context) -> FontFamily {
    theme::family_for(ctx, FONT_FAMILY, FontFamily::Proportional)
}

/// Paints an icon centred on `center`.
pub fn paint_icon(painter: &Painter, center: Pos2, icon: Icon, size: f32, color: Color32) {
    painter.text(
        center,
        Align2::CENTER_CENTER,
        icon.glyph(),
        FontId::new(size, family(painter.ctx())),
        color,
    );
}

/// Draws a compact gallery of every icon with its stable name.
///
/// Used by the project settings page to make the set inspectable without a
/// separate tool.
pub fn gallery(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal_wrapped(|ui| {
        for &icon in Icon::ALL {
            let response = egui::Frame::new()
                .fill(t.canvas)
                .stroke(t.hairline())
                .inner_margin(egui::Margin::symmetric(7, 3))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(icon.rich(ui.ctx(), 15.0, t.ink));
                        ui.label(
                            RichText::new(icon.name())
                                .font(FontId::new(10.0, theme::mono_for(ui.ctx())))
                                .color(t.faint),
                        );
                    });
                })
                .response;
            response.on_hover_text(icon.name());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_a_single_glyph() {
        for &icon in Icon::ALL {
            let glyph = icon.glyph();
            assert_eq!(
                glyph.chars().count(),
                1,
                "{} must be exactly one codepoint",
                icon.name()
            );
            assert!(!icon.name().is_empty());
        }
    }

    #[test]
    fn icon_family_falls_back_before_fonts_are_installed() {
        let ctx = egui::Context::default();
        assert_eq!(family(&ctx), FontFamily::Proportional);
    }

    #[test]
    fn gallery_renders_without_installed_fonts() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), gallery);
        output.textures_delta.clear();
    }
}
