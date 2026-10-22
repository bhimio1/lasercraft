//! Widgets LaserCraft paints itself from the theme tokens
//! (`docs/spec/style/visual-language.md`, "Widget catalogue"): the pill
//! switch, filled buttons, and status bands.

use egui::{Color32, Response, Sense, Stroke, Ui, Vec2};

use crate::theme::{ROUNDED, Tokens, size, weight};

/// A pill switch left of its label; clicking either toggles it. Returns the
/// response of the whole control (`changed` when toggled).
pub fn switch(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    let t = Tokens::of(ui.ctx());
    let height = size::ICON_SMALL + 2.0;
    let track = Vec2::new(height * 2.0, height);
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::TextStyle::Body.resolve(ui.style()),
        t.text,
    );
    let text_gap = if label.is_empty() {
        0.0
    } else {
        size::GAP_TIGHT
    };
    let desired = Vec2::new(
        track.x + text_gap + galley.size().x,
        size::ROW.max(galley.size().y),
    );
    let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let enabled = ui.is_enabled();
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Switch, enabled, *on, label));
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let pill = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - track.y / 2.0),
            track,
        );
        let hovered = response.hovered() && enabled;
        let fill = match (*on, enabled) {
            (true, true) if hovered => t.selection,
            (true, true) => t.accent,
            (true, false) => t.accent_soft,
            (false, true) if hovered => t.text_muted,
            (false, true) => t.switch_off,
            (false, false) => t.text_disabled,
        };
        let painter = ui.painter();
        painter.rect_filled(pill, track.y / 2.0, fill);
        let radius = track.y / 2.0 - 2.0;
        let x = egui::lerp(
            (pill.left() + track.y / 2.0)..=(pill.right() - track.y / 2.0),
            how_on,
        );
        let knob = egui::pos2(x, pill.center().y);
        let knob_fill = if enabled {
            t.field
        } else {
            t.field.gamma_multiply(0.8)
        };
        painter.circle_filled(knob, radius, knob_fill);
        if !*on {
            painter.circle_stroke(knob, radius, Stroke::new(weight::THIN, t.border));
        }
        if response.has_focus() && ui.memory(|m| m.focused().is_some()) {
            painter.rect_stroke(
                pill.expand(2.0),
                track.y / 2.0 + 2.0,
                Stroke::new(weight::NORMAL, t.accent),
                egui::StrokeKind::Outside,
            );
        }
        let color = if enabled { t.text } else { t.text_disabled };
        painter.galley(
            egui::pos2(
                pill.right() + text_gap,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            color,
        );
    }
    response
}

/// How a filled button looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// The window's one main action: `accent`.
    Primary,
    /// Stop and destructive actions: `danger`.
    Danger,
}

/// A button filled with `accent` or `danger` and a white label, at least
/// `min` in size.
pub fn filled(ui: &mut Ui, label: &str, fill: Fill, min: Vec2) -> Response {
    let t = Tokens::of(ui.ctx());
    let color = match fill {
        Fill::Primary => t.accent,
        Fill::Danger => t.danger,
    };
    let text = egui::RichText::new(label).color(t.text_on_strong);
    ui.scope(|ui| {
        let v = &mut ui.style_mut().visuals.widgets;
        for (state, shade) in [
            (&mut v.inactive, color),
            (&mut v.hovered, darker(color, 0.85)),
            (&mut v.active, darker(color, 0.7)),
        ] {
            state.bg_fill = shade;
            state.weak_bg_fill = shade;
            state.bg_stroke = Stroke::NONE;
            state.fg_stroke = Stroke::new(weight::THIN, t.text_on_strong);
        }
        ui.add(egui::Button::new(text).corner_radius(ROUNDED).min_size(min))
    })
    .inner
}

fn darker(c: Color32, f: f32) -> Color32 {
    // Scales each channel down; the result stays in range.
    let s = |v: u8| (f32::from(v) * f) as u8;
    Color32::from_rgb(s(c.r()), s(c.g()), s(c.b()))
}

/// What a status band says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// A hint.
    Hint,
    /// Something went well.
    Success,
    /// Something to check.
    Warning,
    /// Something failed or was refused.
    Error,
}

/// A one-message band: soft fill of its kind, a mark, `text` on it
/// (`visual-language.md`, "Status bands"). Long text wraps.
pub fn band(ui: &mut Ui, kind: Band, text: &str) -> Response {
    let t = Tokens::of(ui.ctx());
    let (fill, mark, mark_color) = match kind {
        Band::Hint => (t.info_soft, "", t.text_muted),
        Band::Success => (t.success_soft, "✔", t.success),
        Band::Warning => (t.warning_soft, "⚠", t.warning),
        Band::Error => (t.danger_soft, "⊗", t.danger),
    };
    egui::Frame::new()
        .fill(fill)
        .corner_radius(ROUNDED)
        .inner_margin(egui::Margin::symmetric(
            size::GAP as i8,
            size::GAP_TIGHT as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                if !mark.is_empty() {
                    ui.label(egui::RichText::new(mark).color(mark_color));
                }
                ui.label(egui::RichText::new(text).color(t.text));
            });
        })
        .response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &egui::Context, events: Vec<egui::Event>, on: &mut bool) -> bool {
        let mut changed = false;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(300.0, 100.0),
            )),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            changed = switch(ui, on, "Output").changed();
        });
        output.textures_delta.clear();
        changed
    }

    #[test]
    fn clicking_the_switch_or_its_label_toggles_it() {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut on = false;
        frame(&ctx, Vec::new(), &mut on);
        let click = |x: f32, pressed: bool| egui::Event::PointerButton {
            pos: egui::pos2(x, 10.0),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        for x in [10.0, 45.0] {
            let before = on;
            frame(
                &ctx,
                vec![egui::Event::PointerMoved(egui::pos2(x, 10.0))],
                &mut on,
            );
            frame(&ctx, vec![click(x, true)], &mut on);
            let changed = frame(&ctx, vec![click(x, false)], &mut on);
            assert!(changed && on != before, "click at {x}");
        }
    }
}
