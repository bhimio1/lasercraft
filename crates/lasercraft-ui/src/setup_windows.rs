//! Job setup windows (M9): Rotary attachment, Taper correction, Print
//! registration and its failure message (`docs/spec/rotary.md`,
//! `docs/spec/taper.md`, `docs/spec/registration.md`;
//! `docs/spec/style/later-windows.md`, "Job setups"). They collect
//! arguments; every change is a command.

use lasercraft_commands::Invocation;
use lasercraft_geom::Point;
use lasercraft_geom::taper::{Taper, TopEdge};
use lasercraft_project::{Project, ROTARY_KEYS, Rotary, RotaryKind};

use crate::laser_panel::{Enabled, LaserView};
use crate::shell::Action;
use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Band, Fill};

const NOT_READY: &str = "Needs a connected, ready machine.";

fn right_label(ui: &mut egui::Ui, text: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(text);
    });
}

fn mm(ui: &mut egui::Ui, v: &mut f64, max: f64) -> egui::Response {
    ui.add(
        egui::DragValue::new(v)
            .range(1.0..=max)
            .max_decimals(3)
            .speed(0.1),
    )
}

/// The open setup windows.
#[derive(Default)]
pub(crate) struct SetupWindows {
    rotary: Option<Rotary>,
    taper: Option<Taper>,
    /// The Print registration window, with the page the user stepped back
    /// to (None follows the targets that are set).
    registration: Option<Option<usize>>,
    /// A refused registration step, shown in a message window.
    failed: Option<String>,
}

impl SetupWindows {
    pub(crate) fn open_rotary(&mut self, project: &Project) {
        self.rotary.get_or_insert_with(|| project.rotary());
    }

    pub(crate) fn open_taper(&mut self, project: &Project) {
        let d = project.rotary().object_diameter;
        self.taper.get_or_insert(Taper {
            top: TopEdge::Top,
            top_diameter: d,
            length: 100.0,
            bottom_diameter: (d * 0.8).max(1.0),
        });
    }

    pub(crate) fn open_registration(&mut self) {
        self.registration.get_or_insert(None);
    }

    /// Shows why a registration step was refused.
    pub(crate) fn registration_failed(&mut self, message: String) {
        self.failed = Some(message);
    }

    /// The taper preview: the selection's warped outlines, document mm.
    pub(crate) fn ghost(&self, project: &Project) -> Vec<(Vec<Point>, bool)> {
        let Some(t) = &self.taper else {
            return Vec::new();
        };
        if t.validate().is_err() {
            return Vec::new();
        }
        let ids = project.selection();
        let Ok(Some(b)) = project.bounds_of(&ids) else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|id| project.shape(*id))
            .filter(|s| !s.geometry().is_image())
            .flat_map(|s| s.outlines(0.2))
            .map(|(points, closed)| (t.warp(&points, closed, &b), closed))
            .collect()
    }

    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        laser: &LaserView,
        actions: &mut Vec<Action>,
    ) {
        self.rotary_window(ctx, project, laser, actions);
        self.taper_window(ctx, project, actions);
        self.registration_window(ctx, project, laser, actions);
        self.failed_window(ctx);
    }

    fn rotary_window(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        laser: &LaserView,
        actions: &mut Vec<Action>,
    ) {
        let Some(draft) = &mut self.rotary else {
            return;
        };
        let t = Tokens::of(ctx);
        let mut done = None;
        egui::Window::new("Rotary attachment")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                widgets::switch(ui, &mut draft.enabled, "Use rotary");
                ui.add_space(size::GAP_TIGHT);
                egui::Grid::new("rotary-grid")
                    .num_columns(2)
                    .spacing([size::GAP, size::GAP_TIGHT])
                    .show(ui, |ui| {
                        right_label(ui, "Kind");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut draft.kind, RotaryKind::Rollers, "Rollers");
                            ui.radio_value(&mut draft.kind, RotaryKind::Chuck, "Chuck");
                        });
                        ui.end_row();
                        right_label(ui, "Travel per turn (mm)");
                        mm(ui, &mut draft.mm_per_turn, 10_000.0).on_hover_text(
                            "Y travel the controller makes for one full turn of the roller or chuck",
                        );
                        ui.end_row();
                        if draft.kind == RotaryKind::Rollers {
                            right_label(ui, "Roller diameter (mm)");
                            mm(ui, &mut draft.roller_diameter, 1000.0)
                                .on_hover_text("Diameter of the roller the motor turns");
                            ui.end_row();
                        }
                        right_label(ui, "Object diameter (mm)");
                        mm(ui, &mut draft.object_diameter, 1000.0)
                            .on_hover_text("Measure the widest end of a tapered object");
                        ui.end_row();
                        ui.label("");
                        widgets::switch(ui, &mut draft.mirror, "Turn the other way");
                        ui.end_row();
                    });
                let height = project
                    .bounds_of(&project.shapes().iter().map(|s| s.id()).collect::<Vec<_>>())
                    .ok()
                    .flatten()
                    .map_or(0.0, |b| b.height());
                ui.label(
                    egui::RichText::new(format!(
                        "Circumference {:.1} mm; the design's height turns {:.0}°",
                        draft.circumference(),
                        draft.degrees(height)
                    ))
                    .small()
                    .color(t.text_muted),
                );
                ui.label(
                    egui::RichText::new(
                        "Rotary jobs may go past the bed in Y: turn GRBL's soft limits ($20) off while the rotary is in use.",
                    )
                    .small()
                    .color(t.text_muted),
                );
                ui.add_space(size::GAP_TIGHT);
                let on = Enabled::of(laser, true);
                if ui
                    .add_enabled(on.jog, egui::Button::new("Test rotation"))
                    .on_hover_text("Turn the object once and back with the laser off")
                    .on_disabled_hover_text(NOT_READY)
                    .clicked()
                {
                    // The stored travel per turn is what the test uses.
                    let mut lines = Vec::new();
                    if (draft.mm_per_turn - project.rotary().mm_per_turn).abs() > 1e-12 {
                        lines.push(Invocation::new(
                            "rotary.set",
                            ["mm_per_turn".to_owned(), draft.mm_per_turn.to_string()],
                        ));
                    }
                    lines.push(Invocation::bare("device.rotary_test"));
                    actions.extend(lines.into_iter().map(Action::Run));
                }
                let (ok, cancel) = crate::dialogs::buttons(ui, "OK", true, "");
                if ok {
                    done = Some(true);
                }
                if cancel || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    done = Some(false);
                }
            });
        if let Some(ok) = done {
            if ok {
                actions.extend(rotary_change(&project.rotary(), draft).map(Action::Run));
            }
            self.rotary = None;
        }
    }

    fn taper_window(&mut self, ctx: &egui::Context, project: &Project, actions: &mut Vec<Action>) {
        let Some(taper) = &mut self.taper else {
            return;
        };
        let t = Tokens::of(ctx);
        let ids = project.selection();
        let bounds = project.bounds_of(&ids).ok().flatten();
        let images = ids
            .iter()
            .any(|id| project.shape(*id).is_some_and(|s| s.geometry().is_image()));
        let mut done = None;
        egui::Window::new("Taper correction")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        egui::Grid::new("taper-grid")
                            .num_columns(2)
                            .spacing([size::GAP, size::GAP_TIGHT])
                            .show(ui, |ui| {
                                right_label(ui, "Top of design");
                                egui::ComboBox::from_id_salt("taper-top")
                                    .selected_text(top_word(taper.top))
                                    .show_ui(ui, |ui| {
                                        for e in [TopEdge::Top, TopEdge::Left, TopEdge::Right] {
                                            ui.selectable_value(&mut taper.top, e, top_word(e));
                                        }
                                    })
                                    .response
                                    .on_hover_text(
                                        "Which edge of the design goes on the cup's top end",
                                    );
                                ui.end_row();
                                right_label(ui, "Top diameter (mm)");
                                mm(ui, &mut taper.top_diameter, 1000.0);
                                ui.end_row();
                                right_label(ui, "Length (mm)");
                                mm(ui, &mut taper.length, 2000.0);
                                ui.end_row();
                                right_label(ui, "Bottom diameter (mm)");
                                mm(ui, &mut taper.bottom_diameter, 1000.0);
                                ui.end_row();
                            });
                        let line = match bounds {
                            Some(b) if !images => {
                                format!("Correction {:.1}%", taper.correction(&b) * 100.0)
                            }
                            Some(_) => "Taper correction works on vector shapes only".to_owned(),
                            None => "Select the art to correct".to_owned(),
                        };
                        ui.label(egui::RichText::new(line).small().color(t.text_muted));
                    });
                    cup_drawing(ui, &t);
                });
                let ready = bounds.is_some() && !images && taper.validate().is_ok();
                let (ok, cancel) =
                    crate::dialogs::buttons(ui, "OK", ready, "Select vector shapes first.");
                if ok {
                    done = Some(true);
                }
                if cancel || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    done = Some(false);
                }
            });
        if let Some(ok) = done {
            if ok {
                let n = |v: f64| v.to_string();
                actions.push(Action::Run(Invocation::new(
                    "shape.taper",
                    [
                        taper.top.as_str().to_owned(),
                        n(taper.top_diameter),
                        n(taper.length),
                        n(taper.bottom_diameter),
                    ],
                )));
            }
            self.taper = None;
        }
    }

    fn registration_window(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        laser: &LaserView,
        actions: &mut Vec<Action>,
    ) {
        let Some(back) = &mut self.registration else {
            return;
        };
        let t = Tokens::of(ctx);
        let reg = project.registration();
        let set = reg.targets.len().min(2);
        let page = back.map_or(set, |b| b.min(set));
        let mut open = true;
        let mut close = false;
        let ready = laser.port.is_some() && laser.ready && laser.position.is_some();
        let selection = project.selection();
        let objects = project.objects_of(&selection).map_or(0, |o| o.len());
        let mut missing = Vec::new();
        if laser.port.is_none() {
            missing.push("Connect the machine first.".to_owned());
        } else if !ready {
            missing.push("Wait until the machine reports its position.".to_owned());
        }
        if page < 2 && objects != 1 {
            missing.push("Select exactly one shape or group as the target.".to_owned());
        }
        egui::Window::new("Print registration")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_width(420.0);
                if !missing.is_empty() && page < 2 {
                    widgets::band(ui, Band::Hint, &missing.join(" "));
                }
                ui.label(
                    egui::RichText::new(
                        "Move the head only with the Jog panel or Move head, never by hand.",
                    )
                    .small()
                    .color(t.text_muted),
                );
                ui.add_space(size::GAP);
                let can_set = missing.is_empty();
                let big = egui::vec2(160.0, size::ROW_BIG);
                match page {
                    0 | 1 => {
                        let n = page + 1;
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(200.0);
                                target_pictogram(ui, n == 1, &t);
                                ui.label(format!(
                                    "Select target {n} on the canvas, move the head over its printed mark, then click Set target {n}."
                                ));
                                let label = format!("Set target {n}");
                                if ui
                                    .add_enabled_ui(can_set, |ui| {
                                        widgets::filled(ui, &label, Fill::Primary, big)
                                    })
                                    .inner
                                    .on_disabled_hover_text(missing.join("\n"))
                                    .clicked()
                                {
                                    actions.push(Action::Run(Invocation::new(
                                        "registration.set_target",
                                        [n.to_string()],
                                    )));
                                    *back = None;
                                }
                            });
                            if n == 2 {
                                ui.vertical(|ui| {
                                    ui.set_width(200.0);
                                    jog_pictogram(ui, &t);
                                    ui.label("Move the head to where target 2 should be.");
                                    let go = Enabled::of(laser, true).jog && objects == 1;
                                    if ui
                                        .add_enabled(go, egui::Button::new("Go to expected target 2").min_size(big))
                                        .on_hover_text("Moves the head with the laser off")
                                        .on_disabled_hover_text("Select target 2 and connect a ready machine.")
                                        .clicked()
                                    {
                                        actions.push(Action::Run(Invocation::bare("registration.expected")));
                                    }
                                });
                            }
                        });
                    }
                    _ => {
                        ui.horizontal_top(|ui| {
                            for (fit, title, about) in [
                                ("scaled", "Fit job to targets", "Moves, turns and scales the job to match both targets"),
                                ("rigid", "Place job on targets (no scaling)", "Moves and turns the job only"),
                            ] {
                                ui.vertical(|ui| {
                                    ui.set_width(200.0);
                                    fit_pictogram(ui, fit == "scaled", &t);
                                    ui.label(about);
                                    if ui.add(egui::Button::new(title).min_size(big)).clicked() {
                                        actions.push(Action::Run(Invocation::new("registration.fit", [fit])));
                                        close = true;
                                    }
                                });
                            }
                        });
                    }
                }
                ui.add_space(size::GAP);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!reg.targets.is_empty(), egui::Button::new("Clear registration"))
                        .clicked()
                    {
                        actions.push(Action::Run(Invocation::bare("registration.clear")));
                        *back = None;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            close = true;
                        }
                        if ui.add_enabled(page > 0, egui::Button::new("Back")).clicked() {
                            *back = Some(page - 1);
                        }
                    });
                });
            });
        if !open || close {
            self.registration = None;
        }
    }

    fn failed_window(&mut self, ctx: &egui::Context) {
        let Some(message) = &self.failed else {
            return;
        };
        let mut done = false;
        egui::Window::new("Registration failed")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_max_width(380.0);
                ui.horizontal_top(|ui| {
                    widgets::mark(ui, Band::Error);
                    ui.label(message.as_str());
                });
                ui.add_space(size::GAP);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::filled(ui, "OK", Fill::Primary, egui::vec2(80.0, size::ROW))
                        .clicked()
                        || ui.input(|i| {
                            i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Escape)
                        })
                    {
                        done = true;
                    }
                });
            });
        if done {
            self.failed = None;
        }
    }
}

/// One `rotary.set` with every key that changed.
fn rotary_change(old: &Rotary, new: &Rotary) -> Option<Invocation> {
    let mut args: Vec<String> = Vec::new();
    for key in ROTARY_KEYS {
        let v = new.get(key).unwrap_or_default();
        if old.get(key).unwrap_or_default() == v {
            continue;
        }
        if args.is_empty() {
            args.push(key.to_owned());
            args.push(v);
        } else {
            args.push(format!("{key}={v}"));
        }
    }
    (!args.is_empty()).then(|| Invocation::new("rotary.set", args))
}

fn top_word(e: TopEdge) -> &'static str {
    match e {
        TopEdge::Top => "Top edge",
        TopEdge::Left => "Left edge",
        TopEdge::Right => "Right edge",
    }
}

fn pictogram(ui: &mut egui::Ui) -> (egui::Rect, egui::Painter) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 48.0), egui::Sense::hover());
    (rect, ui.painter().clone())
}

/// A registration sheet with crosshair marks at two corners; the target's
/// corner in `accent` (own design).
fn target_pictogram(ui: &mut egui::Ui, first: bool, t: &Tokens) {
    let (rect, p) = pictogram(ui);
    let sheet = rect.shrink(4.0);
    p.rect_stroke(
        sheet,
        2.0,
        egui::Stroke::new(weight::NORMAL, t.text),
        egui::StrokeKind::Middle,
    );
    let mark = |c: egui::Pos2, color| {
        let s = egui::Stroke::new(weight::NORMAL, color);
        p.circle_stroke(c, 5.0, s);
        p.line_segment([c - egui::vec2(7.0, 0.0), c + egui::vec2(7.0, 0.0)], s);
        p.line_segment([c - egui::vec2(0.0, 7.0), c + egui::vec2(0.0, 7.0)], s);
    };
    let (a, b) = (
        sheet.min + egui::vec2(9.0, 9.0),
        sheet.max - egui::vec2(9.0, 9.0),
    );
    mark(a, if first { t.accent } else { t.text_muted });
    mark(b, if first { t.text_muted } else { t.accent });
}

/// Arrows for moving the head.
fn jog_pictogram(ui: &mut egui::Ui, t: &Tokens) {
    let (rect, p) = pictogram(ui);
    let c = rect.center();
    let s = egui::Stroke::new(weight::NORMAL, t.text);
    for d in [
        egui::vec2(1.0, 0.0),
        egui::vec2(-1.0, 0.0),
        egui::vec2(0.0, 1.0),
        egui::vec2(0.0, -1.0),
    ] {
        let tip = c + d * 18.0;
        p.line_segment([c + d * 4.0, tip], s);
        let side = egui::vec2(-d.y, d.x) * 4.0;
        p.line_segment([tip, tip - d * 5.0 + side], s);
        p.line_segment([tip, tip - d * 5.0 - side], s);
    }
}

/// A frame with scale arrows (fit) or a corner mark sliding into place.
fn fit_pictogram(ui: &mut egui::Ui, scaled: bool, t: &Tokens) {
    let (rect, p) = pictogram(ui);
    let s = egui::Stroke::new(weight::NORMAL, t.text);
    let frame = rect.shrink(8.0);
    p.rect_stroke(frame, 2.0, s, egui::StrokeKind::Middle);
    let a = egui::Stroke::new(weight::NORMAL, t.accent);
    if scaled {
        p.line_segment(
            [frame.center(), frame.right_top() + egui::vec2(6.0, -6.0)],
            a,
        );
        p.line_segment(
            [frame.center(), frame.left_bottom() + egui::vec2(-6.0, 6.0)],
            a,
        );
    } else {
        let c = frame.left_top() + egui::vec2(10.0, 10.0);
        p.line_segment([c, c + egui::vec2(12.0, 0.0)], a);
        p.line_segment([c, c + egui::vec2(0.0, 12.0)], a);
        p.line_segment([c + egui::vec2(24.0, 14.0), c + egui::vec2(8.0, 4.0)], a);
    }
}

/// Our own drawing of a tapered cup with its three measurements.
fn cup_drawing(ui: &mut egui::Ui, t: &Tokens) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 130.0), egui::Sense::hover());
    let p = ui.painter();
    let s = egui::Stroke::new(weight::HEAVY, t.text);
    let top = rect.top() + 20.0;
    let bottom = rect.bottom() - 20.0;
    let cx = rect.center().x;
    let (tl, tr) = (egui::pos2(cx - 40.0, top), egui::pos2(cx + 40.0, top));
    let (bl, br) = (egui::pos2(cx - 28.0, bottom), egui::pos2(cx + 28.0, bottom));
    p.line_segment([tl, bl], s);
    p.line_segment([tr, br], s);
    p.line_segment([bl, br], s);
    p.line_segment([tl, tr], s);
    let font = egui::FontId::proportional(11.0);
    p.text(
        egui::pos2(cx, top - 10.0),
        egui::Align2::CENTER_CENTER,
        "T",
        font.clone(),
        t.text_muted,
    );
    p.text(
        egui::pos2(cx, bottom + 10.0),
        egui::Align2::CENTER_CENTER,
        "B",
        font.clone(),
        t.text_muted,
    );
    p.text(
        egui::pos2(cx + 52.0, rect.center().y),
        egui::Align2::CENTER_CENTER,
        "L",
        font,
        t.text_muted,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotary_ok_sends_only_changed_keys() {
        let old = Rotary::default();
        let mut new = old.clone();
        assert!(rotary_change(&old, &new).is_none());
        new.enabled = true;
        new.object_diameter = 60.0;
        assert_eq!(
            rotary_change(&old, &new).unwrap().to_string(),
            "rotary.set enabled on object_diameter=60"
        );
    }
}
