//! LaserCraft's theme: every color, type size and spacing the UI uses, as
//! the tokens of `docs/spec/style/visual-language.md`. This is the only place
//! in the code with color or size values; widgets and painters read
//! [`Tokens`]. Only the light theme is installed for now; the dark values
//! exist so every token has both (a later Theme preference picks one).

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Theme, Vec2};

/// The color tokens of one theme (`visual-language.md`, "Tokens").
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)] // Each field is the token of the same name.
pub struct Tokens {
    // Chrome surfaces and text.
    pub window: Color32,
    pub panel: Color32,
    pub panel_title: Color32,
    pub field: Color32,
    pub field_alt: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub text_on_strong: Color32,
    pub text_on_warning: Color32,
    // Accent and states.
    pub accent: Color32,
    pub accent_soft: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub warning: Color32,
    pub warning_soft: Color32,
    pub success: Color32,
    pub success_soft: Color32,
    pub info_soft: Color32,
    /// The off track of a pill switch.
    pub switch_off: Color32,
    // Canvas (the same in both themes: the bed is always near-white).
    pub canvas: Color32,
    pub bed: Color32,
    pub bed_edge: Color32,
    pub grid_minor: Color32,
    pub grid_major: Color32,
    pub origin_marker: Color32,
    pub selection: Color32,
    pub handle_fill: Color32,
    pub marquee: Color32,
    pub guide: Color32,
    pub drawing: Color32,
    pub node: Color32,
    pub node_selected: Color32,
    pub start_marker: Color32,
    pub head_marker: Color32,
    pub frame_preview: Color32,
    // Job preview.
    pub move_unlit: Color32,
    pub move_travel: Color32,
    pub preview_head: Color32,
    // Floating layers.
    pub shadow: Color32,
    pub backdrop: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Canvas tokens shared by both themes.
const BED: Color32 = rgb(247, 247, 246);
const ACCENT_LIGHT: Color32 = rgb(30, 98, 184);
const WARNING_LIGHT: Color32 = rgb(184, 124, 0);

impl Tokens {
    /// The light theme (installed today).
    #[must_use]
    pub const fn light() -> Self {
        Self {
            window: rgb(232, 234, 237),
            panel: rgb(240, 241, 243),
            panel_title: rgb(220, 223, 228),
            field: rgb(255, 255, 255),
            field_alt: rgb(245, 246, 248),
            border: rgb(120, 126, 135),
            text: rgb(28, 31, 35),
            text_muted: rgb(88, 95, 104),
            text_disabled: rgb(160, 165, 172),
            hover: rgb(224, 233, 245),
            pressed: rgb(203, 218, 238),
            text_on_strong: rgb(255, 255, 255),
            text_on_warning: rgb(28, 31, 35),
            accent: ACCENT_LIGHT,
            accent_soft: rgb(212, 227, 247),
            danger: rgb(192, 40, 40),
            danger_soft: rgb(251, 225, 224),
            warning: WARNING_LIGHT,
            warning_soft: rgb(252, 240, 214),
            success: rgb(40, 122, 52),
            success_soft: rgb(222, 241, 224),
            info_soft: rgb(226, 232, 239),
            switch_off: rgb(128, 134, 143),
            canvas: rgb(214, 216, 219),
            bed: BED,
            bed_edge: rgb(78, 84, 92),
            grid_minor: rgb(228, 230, 233),
            grid_major: rgb(201, 205, 210),
            origin_marker: rgb(52, 64, 80),
            selection: rgb(22, 74, 145),
            handle_fill: rgb(255, 255, 255),
            marquee: ACCENT_LIGHT,
            guide: rgb(14, 112, 104),
            drawing: ACCENT_LIGHT,
            node: rgb(255, 255, 255),
            node_selected: ACCENT_LIGHT,
            start_marker: rgb(104, 40, 140),
            head_marker: rgb(52, 64, 80),
            frame_preview: WARNING_LIGHT,
            move_unlit: rgb(138, 138, 138),
            move_travel: rgb(96, 99, 104),
            preview_head: ACCENT_LIGHT,
            shadow: Color32::from_black_alpha(40),
            backdrop: Color32::from_black_alpha(90),
        }
    }

    /// The dark theme (not installed yet; values kept so every token has
    /// both). Canvas tokens are the light ones: the bed stays near-white.
    #[must_use]
    pub const fn dark() -> Self {
        let light = Self::light();
        Self {
            window: rgb(36, 38, 42),
            panel: rgb(44, 46, 51),
            panel_title: rgb(58, 61, 67),
            field: rgb(26, 27, 30),
            field_alt: rgb(34, 36, 40),
            border: rgb(112, 118, 127),
            text: rgb(232, 234, 237),
            text_muted: rgb(170, 176, 184),
            text_disabled: rgb(104, 109, 116),
            hover: rgb(52, 60, 74),
            pressed: rgb(60, 74, 98),
            text_on_strong: rgb(255, 255, 255),
            text_on_warning: rgb(28, 31, 35),
            accent: rgb(92, 150, 230),
            accent_soft: rgb(40, 62, 98),
            danger: rgb(206, 52, 52),
            danger_soft: rgb(72, 36, 34),
            warning: rgb(222, 160, 40),
            warning_soft: rgb(70, 54, 26),
            success: rgb(92, 178, 104),
            success_soft: rgb(32, 60, 38),
            info_soft: rgb(42, 50, 62),
            switch_off: rgb(120, 126, 135),
            canvas: rgb(56, 58, 63),
            shadow: Color32::from_black_alpha(90),
            backdrop: Color32::from_black_alpha(130),
            ..light
        }
    }

    /// The tokens of `ctx`'s current theme.
    #[must_use]
    pub fn of(ctx: &egui::Context) -> Self {
        match ctx.theme() {
            Theme::Light => Self::light(),
            Theme::Dark => Self::dark(),
        }
    }
}

/// Line weights (`visual-language.md`, "Line weights"), in points.
pub mod weight {
    /// The thinnest crisp line.
    pub const HAIRLINE: f32 = 0.75;
    /// Borders, separators, the bed edge.
    pub const THIN: f32 = 1.0;
    /// Focus rings, unselected shapes, boxes, handle outlines.
    pub const NORMAL: f32 = 1.5;
    /// Selected and hovered shapes, current swatch, origin arms.
    pub const HEAVY: f32 = 3.0;
}

/// Dash rhythm (`style/canvas.md`): dash and gap, points.
pub const DASH: (f32, f32) = (6.0, 4.0);

/// Dotted rhythm: spacing and dot radius, points.
pub const DOTS: (f32, f32) = (4.0, 0.6);

/// The one "slightly rounded" corner radius.
pub const ROUNDED: u8 = 3;

/// Spacing tokens, points.
pub mod size {
    /// Standard spacing between controls and form rows.
    pub const GAP: f32 = 6.0;
    /// Inside a group.
    pub const GAP_TIGHT: f32 = 3.0;
    /// Between groups, around window content.
    pub const GAP_WIDE: f32 = 12.0;
    /// Height of a button, field or list row.
    pub const ROW: f32 = 20.0;
    /// Small icons, check marks, band marks.
    pub const ICON_SMALL: f32 = 12.0;
}

/// Type tokens: body, small, heading, mono sizes in points.
const TYPE_BODY: f32 = 13.0;
const TYPE_SMALL: f32 = 11.0;
const TYPE_HEADING: f32 = 16.0;

fn visuals(t: &Tokens, dark: bool) -> egui::Visuals {
    let mut v = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let radius = CornerRadius::same(ROUNDED);
    v.panel_fill = t.panel;
    v.window_fill = t.panel;
    v.window_stroke = Stroke::new(weight::THIN, t.border);
    v.window_corner_radius = radius;
    v.menu_corner_radius = radius;
    v.window_shadow.color = t.shadow;
    v.popup_shadow.color = t.shadow;
    v.extreme_bg_color = t.field;
    v.text_edit_bg_color = Some(t.field);
    v.faint_bg_color = t.field_alt;
    v.code_bg_color = t.field_alt;
    v.selection.bg_fill = t.accent_soft;
    v.selection.stroke = Stroke::new(weight::THIN, t.text);
    v.hyperlink_color = t.text;
    v.warn_fg_color = t.warning;
    v.error_fg_color = t.danger;
    v.override_text_color = None;
    v.weak_text_color = Some(t.text_muted);
    v.striped = false;
    v.slider_trailing_fill = true;
    v.collapsing_header_frame = false;
    v.button_frame = true;
    v.interact_cursor = None;
    v.text_cursor.stroke = Stroke::new(weight::THIN, t.text);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = t.panel;
    w.noninteractive.weak_bg_fill = t.panel;
    w.noninteractive.bg_stroke = Stroke::new(weight::THIN, t.border);
    w.noninteractive.fg_stroke = Stroke::new(weight::THIN, t.text);
    w.inactive.bg_fill = t.field;
    w.inactive.weak_bg_fill = t.field;
    w.inactive.bg_stroke = Stroke::new(weight::THIN, t.border);
    w.inactive.fg_stroke = Stroke::new(weight::THIN, t.text);
    w.hovered.bg_fill = t.hover;
    w.hovered.weak_bg_fill = t.hover;
    w.hovered.bg_stroke = Stroke::new(weight::THIN, t.text_muted);
    w.hovered.fg_stroke = Stroke::new(weight::THIN, t.text);
    w.hovered.expansion = 0.0;
    w.active.bg_fill = t.pressed;
    w.active.weak_bg_fill = t.pressed;
    w.active.bg_stroke = Stroke::new(weight::THIN, t.accent);
    w.active.fg_stroke = Stroke::new(weight::THIN, t.text);
    w.active.expansion = 0.0;
    w.open.bg_fill = t.accent_soft;
    w.open.weak_bg_fill = t.accent_soft;
    w.open.bg_stroke = Stroke::new(weight::THIN, t.accent);
    w.open.fg_stroke = Stroke::new(weight::THIN, t.text);
    for state in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        state.corner_radius = radius;
    }
    v
}

fn style(t: &Tokens, dark: bool) -> egui::Style {
    let mut s = egui::Style {
        visuals: visuals(t, dark),
        ..egui::Style::default()
    };
    s.text_styles = [
        (
            TextStyle::Body,
            FontId::new(TYPE_BODY, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(TYPE_BODY, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(TYPE_SMALL, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(TYPE_HEADING, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(TYPE_BODY, FontFamily::Monospace),
        ),
    ]
    .into();
    let sp = &mut s.spacing;
    sp.interact_size.y = size::ROW;
    sp.item_spacing = Vec2::splat(size::GAP);
    sp.button_padding = Vec2::new(size::GAP, size::GAP_TIGHT);
    sp.window_margin = egui::Margin::same(size::GAP_WIDE as i8);
    sp.menu_margin = egui::Margin::same(size::GAP_TIGHT as i8);
    sp.icon_width = size::ICON_SMALL;
    sp.icon_spacing = size::GAP_TIGHT;
    s.interaction.show_tooltips_only_when_still = true;
    s
}

/// Installs the light theme on `ctx` and stops following the operating
/// system's theme (`docs/spec/ui-shell.md`, "Look").
pub fn install(ctx: &egui::Context) {
    for (theme, tokens, dark) in [
        (Theme::Light, Tokens::light(), false),
        (Theme::Dark, Tokens::dark(), true),
    ] {
        ctx.set_style_of(theme, style(&tokens, dark));
    }
    ctx.set_theme(egui::ThemePreference::Light);
}

/// WCAG relative luminance of a color (alpha ignored).
#[must_use]
pub fn luminance(c: Color32) -> f64 {
    let lin = |v: u8| {
        let s = f64::from(v) / 255.0;
        if s <= 0.040_45 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

/// WCAG contrast ratio between two colors.
#[must_use]
pub fn contrast(a: Color32, b: Color32) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Black or white, whichever reads better on `fill` (swatch numbers).
#[must_use]
pub fn label_on(fill: Color32) -> Color32 {
    if contrast(Color32::BLACK, fill) >= contrast(Color32::WHITE, fill) {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: f64 = 4.5;
    const MARK: f64 = 3.0;

    fn check(name: &str, a: Color32, b: Color32, target: f64, failures: &mut Vec<String>) {
        let c = contrast(a, b);
        if c < target {
            failures.push(format!("{name}: {c:.2} < {target}"));
        }
    }

    /// The contrast table of `visual-language.md`, "How values are chosen".
    fn failures(t: &Tokens) -> Vec<String> {
        let mut f = Vec::new();
        for (bg_name, bg) in [
            ("window", t.window),
            ("panel", t.panel),
            ("panel-title", t.panel_title),
            ("field", t.field),
            ("field-alt", t.field_alt),
            ("hover", t.hover),
            ("pressed", t.pressed),
            ("accent-soft", t.accent_soft),
            ("danger-soft", t.danger_soft),
            ("warning-soft", t.warning_soft),
            ("success-soft", t.success_soft),
            ("info-soft", t.info_soft),
        ] {
            check(&format!("text on {bg_name}"), t.text, bg, TEXT, &mut f);
        }
        for (bg_name, bg) in [
            ("window", t.window),
            ("panel", t.panel),
            ("field", t.field),
            ("field-alt", t.field_alt),
            ("accent-soft", t.accent_soft),
        ] {
            check(
                &format!("text-muted on {bg_name}"),
                t.text_muted,
                bg,
                TEXT,
                &mut f,
            );
        }
        check(
            "text-on-strong on accent",
            t.text_on_strong,
            t.accent,
            TEXT,
            &mut f,
        );
        check(
            "text-on-strong on danger",
            t.text_on_strong,
            t.danger,
            TEXT,
            &mut f,
        );
        check(
            "text-on-warning on warning",
            t.text_on_warning,
            t.warning,
            TEXT,
            &mut f,
        );
        for (bg_name, bg) in [("panel", t.panel), ("window", t.window)] {
            check(&format!("border vs {bg_name}"), t.border, bg, MARK, &mut f);
        }
        for (bg_name, bg) in [("panel", t.panel), ("window", t.window), ("field", t.field)] {
            check(&format!("accent vs {bg_name}"), t.accent, bg, MARK, &mut f);
        }
        for (bg_name, bg) in [("panel", t.panel), ("field", t.field)] {
            check(
                &format!("switch off vs {bg_name}"),
                t.switch_off,
                bg,
                MARK,
                &mut f,
            );
            for (mark, color) in [
                ("success", t.success),
                ("warning", t.warning),
                ("danger", t.danger),
            ] {
                check(&format!("{mark} vs {bg_name}"), color, bg, MARK, &mut f);
            }
        }
        for (name, color) in [
            ("selection", t.selection),
            ("marquee", t.marquee),
            ("drawing", t.drawing),
            ("guide", t.guide),
            ("start-marker", t.start_marker),
            ("origin-marker", t.origin_marker),
            ("head-marker", t.head_marker),
            ("frame-preview", t.frame_preview),
            ("node-selected", t.node_selected),
            ("bed-edge", t.bed_edge),
            ("move-unlit", t.move_unlit),
            ("move-travel", t.move_travel),
        ] {
            check(&format!("{name} vs bed"), color, t.bed, MARK, &mut f);
        }
        f
    }

    #[test]
    fn light_theme_meets_its_contrast_targets() {
        let f = failures(&Tokens::light());
        assert!(f.is_empty(), "{f:#?}");
    }

    #[test]
    fn dark_theme_text_reads_on_its_surfaces() {
        let t = Tokens::dark();
        for bg in [t.window, t.panel, t.field, t.field_alt, t.accent_soft] {
            assert!(contrast(t.text, bg) >= TEXT);
            assert!(contrast(t.text_muted, bg) >= TEXT);
        }
        assert_eq!(t.bed, Tokens::light().bed, "the bed is the same sheet");
    }

    #[test]
    fn canvas_marks_stand_apart_from_the_bed_and_lighter_unlit() {
        let t = Tokens::light();
        assert!(luminance(t.move_unlit) > luminance(t.move_travel));
        assert!(luminance(t.canvas) < luminance(t.bed));
        assert!(luminance(t.grid_minor) > luminance(t.grid_major));
    }

    #[test]
    fn swatch_numbers_pick_the_readable_color() {
        assert_eq!(label_on(Color32::BLACK), Color32::WHITE);
        assert_eq!(label_on(Color32::from_rgb(250, 240, 80)), Color32::BLACK);
        for (_, fill) in crate::colors::all() {
            assert!(contrast(label_on(fill), fill) >= TEXT, "{fill:?}");
        }
    }

    #[test]
    fn installing_makes_the_light_theme_current() {
        let ctx = egui::Context::default();
        install(&ctx);
        assert_eq!(ctx.theme(), Theme::Light);
        assert_eq!(
            ctx.style_of(Theme::Light).visuals.panel_fill,
            Tokens::light().panel
        );
        assert_eq!(Tokens::of(&ctx), Tokens::light());
    }
}
