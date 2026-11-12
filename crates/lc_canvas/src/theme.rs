//! Level Craft visual design system.
//!
//! The visual language is adapted from ArtCraft Launcher: hard edges, 1 px
//! hairlines, Archivo display type, Inter body text, and Geist Mono HUD labels.
//! Widgets should read [`Tokens::get`] instead of hard-coding a colour.

use std::sync::Arc;

use egui::epaint::text::VariationCoords;
use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, Id, Stroke,
    TextStyle, Visuals,
};

/// Appearance preference selected by the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeChoice {
    /// Follow the operating system.
    #[default]
    System,
    /// Always use the dark palette.
    Dark,
    /// Always use the light palette.
    Light,
}

impl ThemeChoice {
    /// All choices, in menu order.
    pub const ALL: [Self; 3] = [Self::System, Self::Dark, Self::Light];

    /// Short label for menus.
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}

const fn hex(rgb: u32) -> Color32 {
    let [_, r, g, b] = rgb.to_be_bytes();
    Color32::from_rgb(r, g, b)
}

const fn ink_alpha(rgb: u32, alpha: u8) -> Color32 {
    let [_, r, g, b] = rgb.to_be_bytes();
    Color32::from_rgba_unmultiplied_const(r, g, b, alpha)
}

/// Colour tokens for one appearance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    /// Whether this is the dark appearance.
    pub dark: bool,
    /// Page and top-level surface.
    pub bg: Color32,
    /// Panels, tab strips and headers.
    pub panel: Color32,
    /// Wells, pressed states and row hovers.
    pub sunken: Color32,
    /// Canvas background.
    pub canvas: Color32,
    /// Body text.
    pub ink: Color32,
    /// Headings and active labels.
    pub ink_strong: Color32,
    /// Secondary text.
    pub muted: Color32,
    /// Tertiary text.
    pub faint: Color32,
    /// Hairlines.
    pub line: Color32,
    /// Outlined buttons and stronger rules.
    pub line_strong: Color32,
    /// Accent surfaces.
    pub accent: Color32,
    /// Accent text and icons.
    pub accent_ink: Color32,
    /// Primary button fill and inverted hover.
    pub invert_bg: Color32,
    /// Primary button text.
    pub invert_fg: Color32,
    pub good: Color32,
    pub warn: Color32,
    pub bad: Color32,
    /// Minor canvas grid lines.
    pub grid_minor: Color32,
    /// Major canvas grid lines.
    pub grid_major: Color32,
    /// Horizontal world axis.
    pub axis_x: Color32,
    /// Vertical world axis.
    pub axis_y: Color32,
    /// Selection fill.
    pub selection_fill: Color32,
    /// Selection outline.
    pub selection_stroke: Color32,
}

impl Tokens {
    /// Dark editor palette.
    pub const DARK: Self = Self {
        dark: true,
        bg: hex(0x121316),
        panel: hex(0x121316),
        sunken: hex(0x060607),
        canvas: hex(0x0e0f11),
        ink: hex(0xf2f1ee),
        ink_strong: hex(0xffffff),
        muted: ink_alpha(0xf2f1ee, 153),
        faint: ink_alpha(0xf2f1ee, 97),
        line: ink_alpha(0xf2f1ee, 38),
        line_strong: ink_alpha(0xf2f1ee, 102),
        accent: hex(0x2d81ff),
        accent_ink: hex(0x74aaff),
        invert_bg: hex(0xffffff),
        invert_fg: hex(0x0b0b0c),
        good: hex(0x3fd6b8),
        warn: hex(0xffc14d),
        bad: hex(0xff6259),
        grid_minor: ink_alpha(0xf2f1ee, 14),
        grid_major: ink_alpha(0xf2f1ee, 30),
        axis_x: ink_alpha(0xff6259, 90),
        axis_y: ink_alpha(0x3fd6b8, 90),
        selection_fill: ink_alpha(0x2d81ff, 46),
        selection_stroke: hex(0x74aaff),
    };

    /// Light editor palette.
    pub const LIGHT: Self = Self {
        dark: false,
        bg: hex(0xf2f1ee),
        panel: hex(0xf2f1ee),
        sunken: hex(0xe9e8e4),
        canvas: hex(0xf7f6f3),
        ink: hex(0x101014),
        ink_strong: hex(0x000000),
        muted: ink_alpha(0x101014, 158),
        faint: ink_alpha(0x101014, 107),
        line: ink_alpha(0x101014, 41),
        line_strong: ink_alpha(0x101014, 102),
        accent: hex(0x2d81ff),
        accent_ink: hex(0x1659c4),
        invert_bg: hex(0x101014),
        invert_fg: hex(0xf2f1ee),
        good: hex(0x0a7563),
        warn: hex(0x8a5800),
        bad: hex(0xb3261e),
        grid_minor: ink_alpha(0x101014, 16),
        grid_major: ink_alpha(0x101014, 34),
        axis_x: ink_alpha(0xb3261e, 80),
        axis_y: ink_alpha(0x0a7563, 80),
        selection_fill: ink_alpha(0x2d81ff, 40),
        selection_stroke: hex(0x1659c4),
    };

    /// Tokens for the currently selected egui theme.
    pub fn get(ctx: &egui::Context) -> Self {
        if ctx.theme() == egui::Theme::Dark {
            Self::DARK
        } else {
            Self::LIGHT
        }
    }

    /// A 1 px hairline stroke.
    pub fn hairline(&self) -> Stroke {
        Stroke::new(1.0, self.line)
    }

    /// A stronger 1 px stroke for focused or outlined controls.
    pub fn outline(&self) -> Stroke {
        Stroke::new(1.0, self.line_strong)
    }
}

/// Mixes `c` into the background at `alpha` (0-1).
pub fn tint(c: Color32, alpha: f32) -> Color32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Shadow used under menus, dialogs and toasts.
pub fn shadow(t: &Tokens) -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(if t.dark { 140 } else { 36 }),
    }
}

/// Scrim behind a drawer or modal; `amount` runs from 0 (none) to 1.
pub fn scrim(t: &Tokens, amount: f32) -> Color32 {
    let alpha = if t.dark { 170.0 } else { 105.0 } * amount.clamp(0.0, 1.0);
    Color32::from_black_alpha(alpha.round() as u8)
}

/// Builds the egui visuals for one appearance.
fn visuals(t: &Tokens) -> Visuals {
    let mut v = if t.dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    v.panel_fill = t.bg;
    v.window_fill = t.bg;
    v.extreme_bg_color = t.sunken;
    v.text_edit_bg_color = Some(t.sunken);
    v.faint_bg_color = t.sunken;
    v.code_bg_color = t.sunken;
    v.window_stroke = Stroke::new(1.0, t.line_strong);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.window_shadow = shadow(t);
    v.popup_shadow = shadow(t);
    v.override_text_color = Some(t.ink);
    v.hyperlink_color = t.accent_ink;
    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.warn_fg_color = t.warn;
    v.error_fg_color = t.bad;
    v.button_frame = true;
    v.collapsing_header_frame = false;
    v.indent_has_left_vline = true;
    v.striped = false;

    for (w, fill, stroke, fg) in [
        (&mut v.widgets.noninteractive, t.bg, t.line, t.muted),
        (&mut v.widgets.inactive, t.bg, t.line, t.ink),
        (
            &mut v.widgets.hovered,
            t.sunken,
            t.line_strong,
            t.ink_strong,
        ),
        (&mut v.widgets.active, t.sunken, t.line_strong, t.ink_strong),
        (&mut v.widgets.open, t.sunken, t.line_strong, t.ink_strong),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1.0, stroke);
        w.corner_radius = CornerRadius::ZERO;
        w.fg_stroke = Stroke::new(1.0, fg);
        w.expansion = 0.0;
    }

    v
}

/// Applies both appearances and the editor's typography/spacing.
///
/// `ctx.set_fonts` (called by [`install_fonts`]) only takes effect on the next
/// frame. Call this once during startup, before the first frame is drawn.
pub fn apply(ctx: &egui::Context, choice: ThemeChoice) {
    ctx.set_visuals_of(egui::Theme::Dark, visuals(&Tokens::DARK));
    ctx.set_visuals_of(egui::Theme::Light, visuals(&Tokens::LIGHT));
    ctx.set_theme(match choice {
        ThemeChoice::System => egui::ThemePreference::System,
        ThemeChoice::Dark => egui::ThemePreference::Dark,
        ThemeChoice::Light => egui::ThemePreference::Light,
    });

    // Resolve families before taking the style lock: `all_styles_mut` holds the
    // context write lock, and the `*_for` helpers read context data.
    let heading_family = display_for(ctx);
    let button_family = medium_for(ctx);
    let mono_family = mono_for(ctx);

    ctx.all_styles_mut(|style| {
        style.interaction.selectable_labels = false;
        style.interaction.multi_widget_text_select = false;
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.icon_spacing = 8.0;
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
        style.spacing.interact_size.y = CONTROL_H;
        style.spacing.indent = 14.0;
        style.spacing.menu_margin = egui::Margin::same(4);
        style.spacing.window_margin = egui::Margin::same(4);
        style.spacing.scroll.bar_width = 8.0;
        style.spacing.scroll.floating = true;
        style.spacing.scroll.foreground_color = true;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.spacing.scroll.bar_outer_margin = 2.0;
        style.spacing.scroll.handle_min_length = 24.0;
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(20.0, heading_family.clone()),
            ),
            (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(12.0, button_family.clone())),
            (
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional),
            ),
            (TextStyle::Monospace, FontId::new(12.0, mono_family.clone())),
        ]
        .into();
    });
}

/// Height of buttons, inputs and filter cells.
pub const CONTROL_H: f32 = 28.0;
/// Height of the top chrome strip.
pub const TOP_BAR_H: f32 = 40.0;
/// Height of the bottom status strip.
pub const STATUS_H: f32 = 24.0;
/// Height of dock and document tabs.
pub const TAB_H: f32 = 28.0;
/// Height of a tree row.
pub const ROW_H: f32 = 24.0;
/// Inner padding for a side panel.
pub const PANEL_PAD: i8 = 10;
/// Inner padding for a page or dialog.
pub const PAGE_PAD: i8 = 16;
/// The site's HUD label: 10 px Geist Mono 500, upper case, widely spaced.
pub const HUD_SIZE: f32 = 10.0;
pub const HUD_SPACING: f32 = HUD_SIZE * 0.15;
/// Button labels: 10 px Geist Mono 700, upper case.
pub const BUTTON_SPACING: f32 = HUD_SIZE * 0.12;

/// Inter 500.
pub fn medium() -> FontFamily {
    FontFamily::Name("medium".into())
}

/// Inter 600.
pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

/// Archivo 620 at 118 % width.
pub fn display() -> FontFamily {
    FontFamily::Name("display".into())
}

/// Geist Mono 500.
pub fn mono() -> FontFamily {
    FontFamily::Name("mono".into())
}

/// Geist Mono 700.
pub fn mono_bold() -> FontFamily {
    FontFamily::Name("mono-bold".into())
}

const FONTS_INSTALLED: &str = "lc_canvas_fonts_installed";

fn has_custom_fonts(ctx: &egui::Context) -> bool {
    ctx.data(|d| {
        d.get_temp::<bool>(Id::new(FONTS_INSTALLED))
            .unwrap_or(false)
    })
}

/// Returns a named family when [`install_fonts`] has run, otherwise `fallback`.
///
/// Headless tests render the shell without installing the bundled fonts; the
/// fallback keeps those tests from panicking on an unbound font family.
pub fn family_for(ctx: &egui::Context, name: &str, fallback: FontFamily) -> FontFamily {
    if has_custom_fonts(ctx) {
        FontFamily::Name(name.into())
    } else {
        fallback
    }
}

/// Inter 500, or egui's proportional font in headless contexts.
pub fn medium_for(ctx: &egui::Context) -> FontFamily {
    family_for(ctx, "medium", FontFamily::Proportional)
}

/// Inter 600, or egui's proportional font in headless contexts.
pub fn semibold_for(ctx: &egui::Context) -> FontFamily {
    family_for(ctx, "semibold", FontFamily::Proportional)
}

/// Archivo 620, or egui's proportional font in headless contexts.
pub fn display_for(ctx: &egui::Context) -> FontFamily {
    family_for(ctx, "display", FontFamily::Proportional)
}

/// Geist Mono 500, or egui's monospace font in headless contexts.
pub fn mono_for(ctx: &egui::Context) -> FontFamily {
    family_for(ctx, "mono", FontFamily::Monospace)
}

/// Geist Mono 700, or egui's monospace font in headless contexts.
pub fn mono_bold_for(ctx: &egui::Context) -> FontFamily {
    family_for(ctx, "mono-bold", FontFamily::Monospace)
}

type Face<'a> = (&'a str, &'static [u8], &'a [([u8; 4], f32)], FontFamily);

/// Registers the editor's variable fonts and their named families.
pub fn install_fonts(ctx: &egui::Context) {
    const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Variable.ttf");
    const ARCHIVO: &[u8] = include_bytes!("../../../assets/fonts/Archivo-Variable.ttf");
    const GEIST_MONO: &[u8] = include_bytes!("../../../assets/fonts/GeistMono-Variable.ttf");

    let mut fonts = FontDefinitions::default();
    // Keep egui's own fonts behind ours for symbols and emoji ours don't have.
    let fallback: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    // Phosphor supplies editor icons. `add_to_fonts` registers the "phosphor"
    // family and makes it a proportional fallback; the loop below also appends
    // it to every text family so inline icons resolve in HUD/mono labels.
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    let faces: [Face<'_>; 7] = [
        (
            "Inter",
            INTER,
            &[(*b"wght", 400.0), (*b"opsz", 14.0)],
            FontFamily::Proportional,
        ),
        (
            "Inter-Medium",
            INTER,
            &[(*b"wght", 500.0), (*b"opsz", 14.0)],
            medium(),
        ),
        (
            "Inter-SemiBold",
            INTER,
            &[(*b"wght", 600.0), (*b"opsz", 14.0)],
            semibold(),
        ),
        (
            "Archivo-Display",
            ARCHIVO,
            &[(*b"wght", 620.0), (*b"wdth", 118.0)],
            display(),
        ),
        (
            "GeistMono",
            GEIST_MONO,
            &[(*b"wght", 400.0)],
            FontFamily::Monospace,
        ),
        ("GeistMono-Medium", GEIST_MONO, &[(*b"wght", 500.0)], mono()),
        (
            "GeistMono-Bold",
            GEIST_MONO,
            &[(*b"wght", 700.0)],
            mono_bold(),
        ),
    ];

    for (name, bytes, coords, family) in faces {
        // These fonts carry no hinting instructions, so subpixel binning smears
        // stems across two pixels at fractional positions.
        let tweak = FontTweak {
            coords: VariationCoords::new(coords.iter().copied()),
            subpixel_binning: Some(false),
            ..Default::default()
        };
        fonts.font_data.insert(
            name.to_owned(),
            Arc::new(FontData::from_static(bytes).tweak(tweak)),
        );
        let mut stack = vec![name.to_owned(), crate::icons::FONT_FAMILY.to_owned()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(family, stack);
    }

    ctx.set_fonts(fonts);
    ctx.data_mut(|d| d.insert_temp(Id::new(FONTS_INSTALLED), true));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palettes_are_distinct() {
        assert_ne!(Tokens::DARK.bg, Tokens::LIGHT.bg);
        assert_ne!(Tokens::DARK.ink, Tokens::LIGHT.ink);
        assert_ne!(Tokens::DARK.canvas, Tokens::LIGHT.canvas);
    }

    #[test]
    fn apply_sets_square_hairline_visuals() {
        let ctx = egui::Context::default();
        apply(&ctx, ThemeChoice::Dark);

        let visuals = ctx.style_of(egui::Theme::Dark).visuals.clone();
        assert_eq!(visuals.widgets.inactive.corner_radius, CornerRadius::ZERO);
        assert_eq!(visuals.widgets.hovered.expansion, 0.0);
        assert_eq!(visuals.panel_fill, Tokens::DARK.bg);
        assert_eq!(visuals.selection.bg_fill, Tokens::DARK.accent);
    }

    #[test]
    fn install_fonts_does_not_panic() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.label(
                egui::RichText::new("LEVEL CRAFT")
                    .font(FontId::new(18.0, display()))
                    .size(18.0),
            );
            ui.label(
                egui::RichText::new("HUD 100%")
                    .font(FontId::new(10.0, mono()))
                    .size(10.0),
            );
            ui.label(crate::icons::Icon::Layer.rich(&ctx, 14.0, Color32::WHITE));
        });
        assert_eq!(
            crate::icons::family(&ctx),
            FontFamily::Name(crate::icons::FONT_FAMILY.into())
        );
        output.textures_delta.clear();
    }
}
