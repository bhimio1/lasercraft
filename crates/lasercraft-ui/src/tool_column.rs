//! The drawing tools column (`docs/spec/style/toolbars.md`, "Drawing
//! tools"): icon-only buttons, with Rectangle and Ellipse in the Shape
//! tools flyout.

use egui::{Pos2, Rect, Sense, Stroke, StrokeKind};

use crate::icons;
use crate::interaction::Tool;
use crate::keymap::Keymap;
use crate::theme::{ROUNDED, Tokens, size, weight};

/// One tool: `ui.tool` argument, label, icon id, tool, second tooltip line.
struct Entry {
    name: &'static str,
    label: &'static str,
    icon: &'static str,
    tool: Tool,
    hint: &'static str,
}

const fn entry(
    name: &'static str,
    label: &'static str,
    icon: &'static str,
    tool: Tool,
    hint: &'static str,
) -> Entry {
    Entry {
        name,
        label,
        icon,
        tool,
        hint,
    }
}

const SELECT: Entry = entry("select", "Select", "select", Tool::Select, "");
const LINE: Entry = entry(
    "line",
    "Line",
    "line",
    Tool::Line,
    "Click points; double-click or Enter to finish.",
);
/// The Shape tools flyout's members, in popup order.
const SHAPES: [Entry; 2] = [
    entry("rect", "Rectangle", "rect", Tool::Rect, ""),
    entry("ellipse", "Ellipse", "ellipse", Tool::Ellipse, ""),
];
const NODES: Entry = entry("nodes", "Edit nodes", "edit-nodes", Tool::Nodes, "");
const TEXT: Entry = entry(
    "text",
    "Text",
    "text-tool",
    Tool::Text,
    "Click the bed to add text.",
);

/// UI state of the column.
#[derive(Debug, Default)]
pub(crate) struct ToolColumn {
    /// The Shape tools member shown on its face (index into the members).
    face: usize,
    /// The flyout popup is open.
    open: bool,
}

/// Size of one tool button.
fn button_size() -> egui::Vec2 {
    egui::Vec2::splat(size::ICON + 2.0 * size::GAP_TIGHT)
}

fn tooltip(keymap: &Keymap, e: &Entry) -> String {
    let chord = keymap
        .chord_for(&format!("ui.tool {}", e.name))
        .map(|c| format!(" ({})", c.label()))
        .unwrap_or_default();
    let mut text = format!("{} tool{chord}", e.label);
    if !e.hint.is_empty() {
        text.push('\n');
        text.push_str(e.hint);
    }
    text
}

/// An icon-only tool button with the toolbar states: hover, pressed, and
/// the active tool's `accent-soft` background and `accent` outline.
fn tool_button(ui: &mut egui::Ui, icon: &str, active: bool) -> egui::Response {
    let t = Tokens::of(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(button_size(), Sense::click());
    let painter = ui.painter();
    let background = if active {
        Some(t.accent_soft)
    } else if response.is_pointer_button_down_on() {
        Some(t.pressed)
    } else if response.hovered() {
        Some(t.hover)
    } else {
        None
    };
    if let Some(fill) = background {
        painter.rect_filled(rect, ROUNDED, fill);
    }
    if active {
        painter.rect_stroke(
            rect,
            ROUNDED,
            Stroke::new(weight::THIN, t.accent),
            StrokeKind::Inside,
        );
    }
    let icon_rect = Rect::from_center_size(rect.center(), egui::Vec2::splat(size::ICON));
    icons::paint(painter, icon, icon_rect, t.text, t.accent);
    response
}

/// The flyout marker: a small solid triangle in the lower-right corner.
fn marker(ui: &egui::Ui, rect: Rect) {
    let t = Tokens::of(ui.ctx());
    let r = rect.shrink(2.0);
    let s = size::ICON_SMALL * 0.45;
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            r.right_bottom(),
            r.right_bottom() - egui::vec2(0.0, s),
            r.right_bottom() - egui::vec2(s, 0.0),
        ],
        t.text_muted,
        Stroke::NONE,
    ));
}

impl ToolColumn {
    /// Draws the column for the current tool; returns the `ui.tool`
    /// argument of a tool the user picked.
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        current: Tool,
        keymap: &Keymap,
    ) -> Option<&'static str> {
        // A chord that picks a shape tool also updates the face.
        if let Some(i) = SHAPES.iter().position(|e| e.tool == current) {
            self.face = i;
        }
        let mut picked = None;
        ui.add_space(size::GAP_TIGHT);
        ui.spacing_mut().item_spacing.y = size::GAP_TIGHT;
        for e in [&SELECT, &LINE] {
            picked = picked.or(self.plain(ui, e, current, keymap));
        }
        picked = picked.or(self.shapes(ui, current, keymap));
        for e in [&NODES, &TEXT] {
            picked = picked.or(self.plain(ui, e, current, keymap));
        }
        picked
    }

    fn plain(
        &self,
        ui: &mut egui::Ui,
        e: &Entry,
        current: Tool,
        keymap: &Keymap,
    ) -> Option<&'static str> {
        let response = tool_button(ui, e.icon, current == e.tool).on_hover_text(tooltip(keymap, e));
        response.clicked().then(|| {
            response.surrender_focus();
            e.name
        })
    }

    fn shapes(
        &mut self,
        ui: &mut egui::Ui,
        current: Tool,
        keymap: &Keymap,
    ) -> Option<&'static str> {
        let face = &SHAPES[self.face];
        let active = SHAPES.iter().any(|e| e.tool == current);
        let tip = format!(
            "{}\nRight-click, or click the corner, for more.",
            tooltip(keymap, face)
        );
        let response = tool_button(ui, face.icon, active).on_hover_text(tip);
        marker(ui, response.rect);
        let corner = response
            .interact_pointer_pos()
            .is_some_and(|p| p.x > response.rect.center().x && p.y > response.rect.center().y);
        let mut picked = None;
        if response.secondary_clicked() || (response.clicked() && corner) {
            self.open = !self.open;
        } else if response.clicked() {
            response.surrender_focus();
            picked = Some(face.name);
        }
        if self.open {
            picked = picked.or(self.flyout(ui, response.rect, keymap));
        }
        picked
    }

    /// The popup to the right of the button: one row per member with its
    /// icon, label and chord; a tick on the face member.
    fn flyout(&mut self, ui: &egui::Ui, button: Rect, keymap: &Keymap) -> Option<&'static str> {
        let t = Tokens::of(ui.ctx());
        let mut picked = None;
        let area = egui::Area::new(ui.make_persistent_id("shape-tools-flyout"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::new(button.right() + size::GAP_TIGHT, button.top()))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).fill(t.field).show(ui, |ui| {
                    for (i, e) in SHAPES.iter().enumerate() {
                        let chord = keymap
                            .chord_for(&format!("ui.tool {}", e.name))
                            .map(|c| c.label())
                            .unwrap_or_default();
                        if flyout_row(ui, e, i == self.face, &chord).clicked() {
                            self.face = i;
                            picked = Some(e.name);
                        }
                    }
                });
            });
        let outside = ui.input(|i| i.pointer.any_click())
            && !area.response.contains_pointer()
            && !button.contains(ui.input(|i| i.pointer.interact_pos()).unwrap_or_default());
        if picked.is_some() || outside || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.open = false;
        }
        picked
    }
}

fn flyout_row(ui: &mut egui::Ui, e: &Entry, face: bool, chord: &str) -> egui::Response {
    let t = Tokens::of(ui.ctx());
    let width = 180.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, size::ROW), Sense::click());
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, ROUNDED, t.accent_soft);
    }
    let check = Rect::from_min_size(rect.min, egui::Vec2::splat(size::ROW));
    if face {
        let c = check.center();
        let ink = Stroke::new(weight::NORMAL, t.text);
        painter.line_segment([c + egui::vec2(-4.0, 0.0), c + egui::vec2(-1.0, 3.0)], ink);
        painter.line_segment([c + egui::vec2(-1.0, 3.0), c + egui::vec2(4.0, -3.0)], ink);
    }
    let icon = Rect::from_min_size(
        Pos2::new(
            check.right(),
            rect.center().y - size::ICON_SMALL / 2.0 - 1.0,
        ),
        egui::Vec2::splat(size::ICON_SMALL + 2.0),
    );
    icons::paint(painter, e.icon, icon, t.text, t.accent);
    let font = egui::TextStyle::Body.resolve(ui.style());
    painter.text(
        Pos2::new(icon.right() + size::GAP_TIGHT, rect.center().y),
        egui::Align2::LEFT_CENTER,
        e.label,
        font.clone(),
        t.text,
    );
    painter.text(
        Pos2::new(rect.right() - size::GAP, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        chord,
        font,
        t.text_muted,
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs one frame of the column at the top left; returns the pick.
    fn frame(
        ctx: &egui::Context,
        column: &mut ToolColumn,
        current: Tool,
        events: Vec<egui::Event>,
    ) -> (Option<&'static str>, Vec<String>) {
        let keymap = Keymap::default_keymap();
        let mut picked = None;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 300.0))),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            picked = column.show(ui, current, &keymap);
        });
        output.textures_delta.clear();
        let texts = output
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect();
        (picked, texts)
    }

    fn click(
        ctx: &egui::Context,
        column: &mut ToolColumn,
        at: Pos2,
        button: egui::PointerButton,
    ) -> Option<&'static str> {
        let press = |pressed| egui::Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            ctx,
            column,
            Tool::Select,
            vec![egui::Event::PointerMoved(at)],
        );
        frame(ctx, column, Tool::Select, vec![press(true)]);
        frame(ctx, column, Tool::Select, vec![press(false)]).0
    }

    #[test]
    fn the_shape_tools_flyout_picks_a_member_and_becomes_its_face() {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut column = ToolColumn::default();
        frame(&ctx, &mut column, Tool::Select, Vec::new());
        let side = button_size().y + size::GAP_TIGHT;
        let center = |i: f32| {
            Pos2::new(
                button_size().x / 2.0,
                size::GAP_TIGHT + side * i + button_size().y / 2.0,
            )
        };
        // A plain click on a button picks its tool.
        assert_eq!(
            click(&ctx, &mut column, center(1.0), egui::PointerButton::Primary),
            Some("line")
        );
        assert_eq!(
            click(&ctx, &mut column, center(2.0), egui::PointerButton::Primary),
            Some("rect")
        );
        // A right-click opens the flyout with both members.
        assert_eq!(
            click(
                &ctx,
                &mut column,
                center(2.0),
                egui::PointerButton::Secondary
            ),
            None
        );
        let (_, texts) = frame(&ctx, &mut column, Tool::Select, Vec::new());
        assert!(
            texts.iter().any(|t| t == "Ellipse") && texts.iter().any(|t| t == "Rectangle"),
            "{texts:?}"
        );
        // The Ellipse row sits on the second line of the popup.
        let row = Pos2::new(
            button_size().x + 60.0,
            center(2.0).y - button_size().y / 2.0 + 6.0 + size::ROW * 1.5,
        );
        assert_eq!(
            click(&ctx, &mut column, row, egui::PointerButton::Primary),
            Some("ellipse")
        );
        assert!(!column.open);
        // Ellipse is now the face: a plain click picks it.
        assert_eq!(
            click(
                &ctx,
                &mut column,
                center(2.0) - egui::vec2(6.0, 6.0),
                egui::PointerButton::Primary
            ),
            Some("ellipse")
        );
        // A chord choosing Rectangle moves the face back.
        frame(&ctx, &mut column, Tool::Rect, Vec::new());
        assert_eq!(column.face, 0);
    }

    #[test]
    fn every_tool_has_an_icon() {
        for e in [&SELECT, &LINE, &SHAPES[0], &SHAPES[1], &NODES, &TEXT] {
            assert!(icons::exists(e.icon), "{}", e.icon);
        }
    }
}
