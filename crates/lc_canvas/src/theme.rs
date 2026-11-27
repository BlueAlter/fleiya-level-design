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

/// Appearance preference selected by the user (PhotoCraft Studio/Pro/Soft palettes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeChoice {
    /// Follow the operating system (PhotoCraft Studio Dark / Light).
    #[default]
    System,
    /// PhotoCraft Studio Dark (near-black surfaces with soft violet accent).
    Dark,
    /// PhotoCraft Studio Light.
    Light,
    /// PhotoCraft Pro Dark (Photoshop-style flat charcoal).
    ProDark,
    /// PhotoCraft Pro Medium Gray (Photoshop default medium gray).
    ProMedium,
    /// PhotoCraft Pro Light.
    ProLight,
    /// PhotoCraft Soft Dark (muted violet-grey low-contrast dark).
    SoftDark,
    /// PhotoCraft Soft Light (pale violet-grey low-contrast light).
    SoftLight,
    /// PhotoCraft Solarized Dark.
    SolarizedDark,
}

impl ThemeChoice {
    /// All choices, in menu order.
    pub const ALL: [Self; 9] = [
        Self::System,
        Self::Dark,
        Self::Light,
        Self::ProDark,
        Self::ProMedium,
        Self::ProLight,
        Self::SoftDark,
        Self::SoftLight,
        Self::SolarizedDark,
    ];

    /// Short label for menus.
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System (Studio)",
            Self::Dark => "Studio (Dark)",
            Self::Light => "Studio (Light)",
            Self::ProDark => "Pro (Dark)",
            Self::ProMedium => "Pro (Medium Gray)",
            Self::ProLight => "Pro (Light)",
            Self::SoftDark => "Soft Dark",
            Self::SoftLight => "Soft Light",
            Self::SolarizedDark => "Solarized Dark",
        }
    }

    /// Resolves the [`Tokens`] palette for this choice given whether OS is dark.
    pub const fn tokens(self, system_is_dark: bool) -> Tokens {
        match self {
            Self::System => {
                if system_is_dark {
                    Tokens::DARK
                } else {
                    Tokens::LIGHT
                }
            }
            Self::Dark => Tokens::DARK,
            Self::Light => Tokens::LIGHT,
            Self::ProDark => Tokens::PRO_DARK,
            Self::ProMedium => Tokens::PRO_MEDIUM,
            Self::ProLight => Tokens::PRO_LIGHT,
            Self::SoftDark => Tokens::SOFT_DARK,
            Self::SoftLight => Tokens::SOFT_LIGHT,
            Self::SolarizedDark => Tokens::SOLARIZED_DARK,
        }
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied_const(r, g, b, a)
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

const ACTIVE_THEME_CHOICE: &str = "lc_active_theme_choice";

impl Tokens {
    /// PhotoCraft Studio (Dark) palette — default dark appearance.
    pub const DARK: Self = Self {
        dark: true,
        bg: rgb(17, 17, 18),
        panel: rgb(26, 26, 28),
        sunken: rgb(35, 35, 38),
        canvas: rgb(14, 14, 15),
        ink: rgb(236, 236, 240),
        ink_strong: rgb(255, 255, 255),
        muted: rgb(150, 150, 158),
        faint: rgb(96, 96, 104),
        line: rgb(40, 40, 44),
        line_strong: rgb(52, 52, 57),
        accent: rgb(139, 124, 246),
        accent_ink: rgb(214, 208, 255),
        invert_bg: rgb(246, 246, 248),
        invert_fg: rgb(12, 12, 14),
        good: rgb(61, 214, 184),
        warn: rgb(240, 190, 90),
        bad: rgb(240, 96, 96),
        grid_minor: rgba(255, 255, 255, 14),
        grid_major: rgba(255, 255, 255, 30),
        axis_x: rgba(240, 96, 96, 90),
        axis_y: rgba(61, 214, 184, 90),
        selection_fill: rgba(139, 124, 246, 46),
        selection_stroke: rgb(214, 208, 255),
    };

    /// PhotoCraft Studio (Light) palette — default light appearance.
    pub const LIGHT: Self = Self {
        dark: false,
        bg: rgb(240, 240, 243),
        panel: rgb(252, 252, 253),
        sunken: rgb(242, 242, 245),
        canvas: rgb(226, 226, 230),
        ink: rgb(24, 24, 28),
        ink_strong: rgb(9, 9, 12),
        muted: rgb(96, 96, 106),
        faint: rgb(150, 150, 160),
        line: rgb(222, 222, 228),
        line_strong: rgb(214, 214, 220),
        accent: rgb(108, 92, 231),
        accent_ink: rgb(80, 64, 200),
        invert_bg: rgb(20, 20, 24),
        invert_fg: rgb(250, 250, 252),
        good: rgb(10, 117, 99),
        warn: rgb(190, 130, 20),
        bad: rgb(210, 60, 60),
        grid_minor: rgba(24, 24, 28, 16),
        grid_major: rgba(24, 24, 28, 34),
        axis_x: rgba(210, 60, 60, 80),
        axis_y: rgba(10, 117, 99, 80),
        selection_fill: rgba(108, 92, 231, 36),
        selection_stroke: rgb(80, 64, 200),
    };

    /// PhotoCraft Pro (Dark) palette — Photoshop-style flat charcoal.
    pub const PRO_DARK: Self = Self {
        dark: true,
        bg: rgb(30, 30, 30),
        panel: rgb(50, 50, 50),
        sunken: rgb(36, 36, 36),
        canvas: rgb(40, 40, 40),
        ink: rgb(222, 222, 222),
        ink_strong: rgb(255, 255, 255),
        muted: rgb(178, 178, 178),
        faint: rgb(128, 128, 128),
        line: rgb(30, 30, 30),
        line_strong: rgb(74, 74, 74),
        accent: rgb(55, 142, 240),
        accent_ink: rgb(255, 255, 255),
        invert_bg: rgb(55, 142, 240),
        invert_fg: rgb(255, 255, 255),
        good: rgb(61, 214, 184),
        warn: rgb(232, 176, 70),
        bad: rgb(236, 91, 98),
        grid_minor: rgba(255, 255, 255, 14),
        grid_major: rgba(255, 255, 255, 30),
        axis_x: rgba(236, 91, 98, 90),
        axis_y: rgba(61, 214, 184, 90),
        selection_fill: rgba(55, 142, 240, 46),
        selection_stroke: rgb(55, 142, 240),
    };

    /// PhotoCraft Pro (Medium Gray) palette — PhotoCraft suite default.
    pub const PRO_MEDIUM: Self = Self {
        dark: true,
        bg: rgb(66, 66, 66),
        panel: rgb(83, 83, 83),
        sunken: rgb(69, 69, 69),
        canvas: rgb(40, 40, 40),
        ink: rgb(238, 238, 238),
        ink_strong: rgb(255, 255, 255),
        muted: rgb(212, 212, 212),
        faint: rgb(160, 160, 160),
        line: rgb(62, 62, 62),
        line_strong: rgb(104, 104, 104),
        accent: rgb(55, 142, 240),
        accent_ink: rgb(255, 255, 255),
        invert_bg: rgb(55, 142, 240),
        invert_fg: rgb(255, 255, 255),
        good: rgb(61, 214, 184),
        warn: rgb(232, 176, 70),
        bad: rgb(236, 91, 98),
        grid_minor: rgba(255, 255, 255, 14),
        grid_major: rgba(255, 255, 255, 30),
        axis_x: rgba(236, 91, 98, 90),
        axis_y: rgba(61, 214, 184, 90),
        selection_fill: rgba(55, 142, 240, 46),
        selection_stroke: rgb(55, 142, 240),
    };

    /// PhotoCraft Pro (Light) palette.
    pub const PRO_LIGHT: Self = Self {
        dark: false,
        bg: rgb(226, 226, 226),
        panel: rgb(240, 240, 240),
        sunken: rgb(255, 255, 255),
        canvas: rgb(214, 214, 214),
        ink: rgb(20, 20, 20),
        ink_strong: rgb(0, 0, 0),
        muted: rgb(64, 64, 64),
        faint: rgb(120, 120, 120),
        line: rgb(214, 214, 214),
        line_strong: rgb(186, 186, 186),
        accent: rgb(55, 142, 240),
        accent_ink: rgb(16, 98, 196),
        invert_bg: rgb(55, 142, 240),
        invert_fg: rgb(255, 255, 255),
        good: rgb(10, 117, 99),
        warn: rgb(190, 130, 20),
        bad: rgb(210, 60, 60),
        grid_minor: rgba(20, 20, 20, 16),
        grid_major: rgba(20, 20, 20, 34),
        axis_x: rgba(210, 60, 60, 80),
        axis_y: rgba(10, 117, 99, 80),
        selection_fill: rgba(55, 142, 240, 40),
        selection_stroke: rgb(16, 98, 196),
    };

    /// PhotoCraft Soft Dark palette (muted violet-grey low-contrast dark).
    pub const SOFT_DARK: Self = Self {
        dark: true,
        bg: rgb(47, 47, 56),
        panel: rgb(54, 54, 64),
        sunken: rgb(64, 64, 75),
        canvas: rgb(40, 40, 48),
        ink: rgb(238, 238, 246),
        ink_strong: rgb(255, 255, 255),
        muted: rgb(212, 212, 224),
        faint: rgb(190, 190, 206),
        line: rgb(70, 70, 82),
        line_strong: rgb(84, 84, 98),
        accent: rgb(138, 154, 232),
        accent_ink: rgb(190, 200, 252),
        invert_bg: rgb(86, 104, 196),
        invert_fg: rgb(255, 255, 255),
        good: rgb(61, 214, 184),
        warn: rgb(240, 200, 130),
        bad: rgb(250, 160, 172),
        grid_minor: rgba(238, 238, 246, 14),
        grid_major: rgba(238, 238, 246, 30),
        axis_x: rgba(250, 160, 172, 90),
        axis_y: rgba(61, 214, 184, 90),
        selection_fill: rgba(138, 154, 232, 48),
        selection_stroke: rgb(190, 200, 252),
    };

    /// PhotoCraft Soft Light palette.
    pub const SOFT_LIGHT: Self = Self {
        dark: false,
        bg: rgb(226, 224, 231),
        panel: rgb(240, 238, 243),
        sunken: rgb(250, 249, 252),
        canvas: rgb(218, 216, 224),
        ink: rgb(44, 42, 56),
        ink_strong: rgb(24, 22, 34),
        muted: rgb(62, 60, 76),
        faint: rgb(80, 78, 96),
        line: rgb(208, 205, 215),
        line_strong: rgb(190, 187, 201),
        accent: rgb(110, 128, 216),
        accent_ink: rgb(46, 58, 146),
        invert_bg: rgb(80, 98, 190),
        invert_fg: rgb(255, 255, 255),
        good: rgb(10, 117, 99),
        warn: rgb(140, 72, 0),
        bad: rgb(166, 44, 62),
        grid_minor: rgba(44, 42, 56, 16),
        grid_major: rgba(44, 42, 56, 34),
        axis_x: rgba(166, 44, 62, 80),
        axis_y: rgba(10, 117, 99, 80),
        selection_fill: rgba(110, 128, 216, 44),
        selection_stroke: rgb(46, 58, 146),
    };

    /// PhotoCraft Solarized Dark palette.
    pub const SOLARIZED_DARK: Self = Self {
        dark: true,
        bg: rgb(0, 37, 47),
        panel: rgb(7, 54, 66),
        sunken: rgb(0, 37, 47),
        canvas: rgb(0, 43, 54),
        ink: rgb(131, 148, 150),
        ink_strong: rgb(253, 246, 227),
        muted: rgb(101, 123, 131),
        faint: rgb(88, 110, 117),
        line: rgb(0, 37, 47),
        line_strong: rgb(21, 80, 95),
        accent: rgb(38, 139, 210),
        accent_ink: rgb(253, 246, 227),
        invert_bg: rgb(38, 139, 210),
        invert_fg: rgb(253, 246, 227),
        good: rgb(42, 161, 152),
        warn: rgb(181, 137, 0),
        bad: rgb(220, 50, 47),
        grid_minor: rgba(131, 148, 150, 16),
        grid_major: rgba(131, 148, 150, 34),
        axis_x: rgba(220, 50, 47, 90),
        axis_y: rgba(42, 161, 152, 90),
        selection_fill: rgba(38, 139, 210, 46),
        selection_stroke: rgb(253, 246, 227),
    };

    /// Tokens for the currently active PhotoCraft theme.
    pub fn get(ctx: &egui::Context) -> Self {
        let system_is_dark = ctx.theme() == egui::Theme::Dark;
        let choice = ctx
            .data(|d| d.get_temp::<ThemeChoice>(Id::new(ACTIVE_THEME_CHOICE)))
            .unwrap_or(ThemeChoice::System);
        choice.tokens(system_is_dark)
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
    ctx.data_mut(|d| d.insert_temp(Id::new(ACTIVE_THEME_CHOICE), choice));
    match choice {
        ThemeChoice::System => {
            ctx.set_visuals_of(egui::Theme::Dark, visuals(&Tokens::DARK));
            ctx.set_visuals_of(egui::Theme::Light, visuals(&Tokens::LIGHT));
            ctx.set_theme(egui::ThemePreference::System);
        }
        other => {
            let t = other.tokens(true);
            if t.dark {
                ctx.set_visuals_of(egui::Theme::Dark, visuals(&t));
                ctx.set_theme(egui::ThemePreference::Dark);
            } else {
                ctx.set_visuals_of(egui::Theme::Light, visuals(&t));
                ctx.set_theme(egui::ThemePreference::Light);
            }
        }
    }

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
        assert_ne!(Tokens::DARK.bg, Tokens::PRO_DARK.bg);
        assert_ne!(Tokens::PRO_DARK.panel, Tokens::PRO_MEDIUM.panel);
        assert_ne!(Tokens::SOFT_DARK.bg, Tokens::SOLARIZED_DARK.bg);
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
