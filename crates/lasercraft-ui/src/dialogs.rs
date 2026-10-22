//! Small windows that collect arguments for M3 commands: offset, grid and
//! circular arrays, and text (`docs/spec/arrange.md`, `docs/spec/text.md`);
//! and M7's image adjustments (`docs/spec/images.md`).
//! Each produces one invocation when confirmed; none changes the project.

use lasercraft_commands::Invocation;
use lasercraft_geom::Point;
use lasercraft_project::{Color, Project};

use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Fill};

/// Corner styles offered for offsets: `path.offset` name, label.
const CORNERS: [(&str, &str); 3] = [("round", "Round"), ("miter", "Miter"), ("bevel", "Bevel")];

#[derive(Debug, Clone, PartialEq)]
enum Dialog {
    Offset {
        distance: f64,
        corner: usize,
    },
    Grid {
        columns: u32,
        rows: u32,
        gap: [f64; 2],
    },
    Circular {
        count: u32,
        center: Option<Point>,
        sweep: f64,
        rotate: bool,
    },
    Text {
        at: Point,
        text: String,
        font: String,
        height: f64,
    },
    Adjust {
        id: u32,
        brightness: f64,
        contrast: f64,
        gamma: f64,
        /// The values when the window opened, for Reset all.
        opened: [f64; 3],
    },
}

/// The open dialog, if any.
#[derive(Debug, Default)]
pub(crate) struct Dialogs {
    open: Option<Dialog>,
    /// Installed font families, loaded the first time the text window opens.
    families: Option<Vec<String>>,
}

fn n(v: f64) -> String {
    if v == 0.0 {
        "0".to_owned()
    } else {
        v.to_string()
    }
}

impl Dialogs {
    /// Opens the window for `ui.offset`, `ui.grid_array` or
    /// `ui.circular_array`. Returns whether the name was one of them.
    pub fn open(&mut self, action: &str) -> bool {
        let dialog = match action {
            "ui.offset" => Dialog::Offset {
                distance: 1.0,
                corner: 0,
            },
            "ui.grid_array" => Dialog::Grid {
                columns: 3,
                rows: 2,
                gap: [5.0, 5.0],
            },
            "ui.circular_array" => Dialog::Circular {
                count: 6,
                center: None,
                sweep: 360.0,
                rotate: true,
            },
            _ => return false,
        };
        // One tool window at a time: asking again keeps the open one.
        self.open.get_or_insert(dialog);
        true
    }

    /// Opens the adjustments of the first selected image; `false` if no
    /// image is selected.
    pub fn open_adjust(&mut self, project: &Project) -> bool {
        let found = project.selection().into_iter().find_map(|id| {
            let image = project.shape(id)?.geometry().image()?;
            Some((id, image.adjust))
        });
        let Some((id, a)) = found else {
            return false;
        };
        self.open = Some(Dialog::Adjust {
            id,
            brightness: a.brightness,
            contrast: a.contrast,
            gamma: a.gamma,
            opened: [a.brightness, a.contrast, a.gamma],
        });
        true
    }

    /// Opens the text window for a click at `at`.
    pub fn open_text(&mut self, at: Point) {
        let font = lasercraft_import::fonts::default_family().unwrap_or_default();
        self.open = Some(Dialog::Text {
            at,
            text: String::new(),
            font,
            height: 10.0,
        });
    }

    /// Draws the open window; returns the invocation when confirmed.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        color: Color,
    ) -> Option<Invocation> {
        let dialog = self.open.as_mut()?;
        let title = match dialog {
            Dialog::Offset { .. } => "Offset",
            Dialog::Grid { .. } => "Grid array",
            Dialog::Circular { .. } => "Circular array",
            Dialog::Text { .. } => "Add text",
            Dialog::Adjust { .. } => "Image adjustments",
        };
        if let Dialog::Circular { center, .. } = dialog
            && center.is_none()
        {
            let area = project.work_area();
            *center = Some(Point::new(area.width_mm / 2.0, area.height_mm / 2.0));
        }
        if matches!(dialog, Dialog::Text { .. }) && self.families.is_none() {
            self.families = Some(lasercraft_import::fonts::families());
        }
        let families = self.families.as_deref().unwrap_or_default();
        let mut open = true;
        let mut confirmed = false;
        let mut cancelled = false;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                egui::Grid::new("dialog-form")
                    .num_columns(2)
                    .spacing([size::GAP, size::GAP_TIGHT])
                    .show(ui, |ui| fields(ui, dialog, families));
                let problem = problem(dialog);
                if let Some(problem) = problem {
                    widgets::band(ui, widgets::Band::Error, problem);
                }
                let (ok, cancel) = buttons(ui, "OK", problem.is_none(), problem.unwrap_or(""));
                // Enter commits from a one-line field; in the text box it is
                // a new line, and Ctrl+Enter commits instead.
                let multiline = matches!(dialog, Dialog::Text { .. });
                let enter = ui.input(|i| {
                    i.key_pressed(egui::Key::Enter) && (!multiline || i.modifiers.command)
                });
                confirmed = problem.is_none() && (ok || enter);
                cancelled = cancel || ui.input(|i| i.key_pressed(egui::Key::Escape));
            });
        let result = confirmed.then(|| invocation(dialog, color));
        if confirmed || cancelled || !open {
            self.open = None;
        }
        result
    }
}

/// A form rule the values break; the primary button stays disabled.
fn problem(dialog: &Dialog) -> Option<&'static str> {
    match dialog {
        Dialog::Grid { columns, rows, .. } if columns * rows < 2 => {
            Some("Cannot make the array: it needs at least two blocks.")
        }
        Dialog::Text { text, .. } if text.trim().is_empty() => Some("Type some text first."),
        _ => None,
    }
}

/// The button row: the primary button (OK) and Cancel, right-aligned below a rule.
/// Returns which was clicked.
pub(crate) fn buttons(
    ui: &mut egui::Ui,
    primary: &str,
    ok_enabled: bool,
    why: &str,
) -> (bool, bool) {
    let t = Tokens::of(ui.ctx());
    ui.add_space(size::GAP);
    let rule = ui.available_rect_before_wrap();
    ui.painter().hline(
        rule.x_range(),
        rule.top(),
        egui::Stroke::new(weight::THIN, t.border),
    );
    ui.add_space(size::GAP);
    let mut ok = false;
    let mut cancel = false;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let width = egui::vec2(80.0, size::ROW);
        cancel = ui
            .add(egui::Button::new("Cancel").min_size(width))
            .clicked();
        ok = ui
            .add_enabled_ui(ok_enabled, |ui| {
                widgets::filled(ui, primary, Fill::Primary, width)
            })
            .inner
            .on_disabled_hover_text(why)
            .clicked();
    });
    (ok, cancel)
}

/// A right-aligned label for the form grid.
fn label(ui: &mut egui::Ui, text: &str, tip: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(text).on_hover_text(tip);
    });
}

#[allow(clippy::too_many_arguments)]
fn number(
    ui: &mut egui::Ui,
    text: &str,
    tip: &str,
    value: &mut f64,
    speed: f64,
    range: (f64, f64),
    decimals: usize,
) {
    label(ui, text, tip);
    ui.add(
        egui::DragValue::new(value)
            .speed(speed)
            .range(range.0..=range.1)
            .max_decimals(decimals),
    )
    .on_hover_text(tip);
    ui.end_row();
}

fn whole(ui: &mut egui::Ui, text: &str, tip: &str, value: &mut u32, range: (u32, u32)) {
    label(ui, text, tip);
    ui.add(egui::DragValue::new(value).range(range.0..=range.1))
        .on_hover_text(tip);
    ui.end_row();
}

/// A hint under a field, in small muted text.
fn hint(ui: &mut egui::Ui, text: &str) {
    let t = Tokens::of(ui.ctx());
    ui.label("");
    ui.label(egui::RichText::new(text).small().color(t.text_muted));
    ui.end_row();
}

fn fields(ui: &mut egui::Ui, dialog: &mut Dialog, families: &[String]) {
    const SPAN: (f64, f64) = (-1000.0, 1000.0);
    match dialog {
        Dialog::Offset { distance, corner } => {
            let tip = "How far the new outline lies from the shapes. Negative values shrink them.";
            number(ui, "Distance (mm)", tip, distance, 0.1, SPAN, 2);
            hint(ui, "Negative moves inward");
            label(
                ui,
                "Corners",
                "How the corners of the new outline are shaped.",
            );
            egui::ComboBox::from_id_salt("offset-corners")
                .selected_text(CORNERS[*corner].1)
                .show_ui(ui, |ui| {
                    for (i, (_, name)) in CORNERS.iter().enumerate() {
                        ui.selectable_value(corner, i, *name);
                    }
                });
            ui.end_row();
        }
        Dialog::Grid { columns, rows, gap } => {
            let gap_tip = "Space between neighbouring blocks, edge to edge. \
                           Negative values overlap them.";
            whole(
                ui,
                "Columns",
                "Blocks across, the original included.",
                columns,
                (1, 100),
            );
            whole(
                ui,
                "Rows",
                "Blocks down, the original included.",
                rows,
                (1, 100),
            );
            number(ui, "Column gap (mm)", gap_tip, &mut gap[0], 0.1, SPAN, 2);
            number(ui, "Row gap (mm)", gap_tip, &mut gap[1], 0.1, SPAN, 2);
        }
        Dialog::Circular {
            count,
            center,
            sweep,
            rotate,
        } => {
            let tip = "Blocks in the finished ring, the original included.";
            whole(ui, "Total count", tip, count, (2, 1000));
            hint(ui, "Includes the original");
            if let Some(c) = center {
                let tip = "Point the blocks turn around.";
                number(ui, "Center X (mm)", tip, &mut c.x, 0.5, SPAN, 2);
                number(ui, "Center Y (mm)", tip, &mut c.y, 0.5, SPAN, 2);
            }
            let tip = "Angle the blocks spread over, clockwise. 360 spaces them evenly \
                       round a full turn.";
            number(ui, "Sweep (°)", tip, sweep, 1.0, (-360.0, 360.0), 2);
            ui.label("");
            widgets::switch(ui, rotate, "Rotate copies")
                .on_hover_text("Turn each block as it goes round. Off keeps every block upright.");
            ui.end_row();
        }
        Dialog::Text {
            text, font, height, ..
        } => {
            label(ui, "Text", "The words to add. Enter starts a new line.");
            ui.add(egui::TextEdit::multiline(text).desired_rows(3))
                .on_hover_text("The words to add. Enter starts a new line.");
            ui.end_row();
            label(
                ui,
                "Font",
                "Font family, from the fonts installed on this computer.",
            );
            egui::ComboBox::from_id_salt("text-font")
                .selected_text(font.as_str())
                .height(300.0)
                .show_ui(ui, |ui| {
                    for family in families {
                        ui.selectable_value(font, family.clone(), family);
                    }
                });
            ui.end_row();
            let tip = "Height of capital letters. 0.1–1000.";
            number(
                ui,
                "Capital height (mm)",
                tip,
                height,
                0.1,
                (0.1, 1000.0),
                2,
            );
        }
        Dialog::Adjust {
            brightness,
            contrast,
            gamma,
            opened,
            ..
        } => {
            let t = Tokens::of(ui.ctx());
            ui.label("");
            ui.label(
                egui::RichText::new("For this image only")
                    .heading()
                    .color(t.text),
            );
            ui.end_row();
            for (text, value, range, log) in [
                ("Brightness (%)", &mut *brightness, (-100.0, 100.0), false),
                ("Contrast", &mut *contrast, (-99.0, 99.0), false),
                ("Gamma", &mut *gamma, (0.1, 10.0), true),
            ] {
                label(ui, text, "Double-click the slider to reset it");
                let neutral = if log { 1.0 } else { 0.0 };
                let slider = ui.add(egui::Slider::new(value, range.0..=range.1).logarithmic(log));
                if slider.double_clicked() {
                    *value = neutral;
                }
                ui.end_row();
            }
            ui.label("");
            if ui
                .button("Reset all")
                .on_hover_text(
                    "Put every setting in this window back to how it was when the window opened",
                )
                .clicked()
            {
                [*brightness, *contrast, *gamma] = *opened;
            }
            ui.end_row();
        }
    }
}

fn invocation(dialog: &Dialog, color: Color) -> Invocation {
    match dialog {
        Dialog::Offset { distance, corner } => {
            Invocation::new("path.offset", [n(*distance), CORNERS[*corner].0.to_owned()])
        }
        Dialog::Grid { columns, rows, gap } => Invocation::new(
            "arrange.grid",
            [columns.to_string(), rows.to_string(), n(gap[0]), n(gap[1])],
        ),
        Dialog::Circular {
            count,
            center,
            sweep,
            rotate,
        } => {
            let c = center.unwrap_or(Point::ZERO);
            Invocation::new(
                "arrange.circular",
                [
                    count.to_string(),
                    n(c.x),
                    n(c.y),
                    n(*sweep),
                    if *rotate { "rotate" } else { "upright" }.to_owned(),
                ],
            )
        }
        Dialog::Text {
            at,
            text,
            font,
            height,
        } => Invocation::new(
            "shape.add_text",
            [
                n(at.x),
                n(at.y),
                n(*height),
                font.clone(),
                text.clone(),
                format!("@{color}"),
            ],
        ),
        Dialog::Adjust {
            id,
            brightness,
            contrast,
            gamma,
            ..
        } => {
            // Sliders give long fractions; keep what a person can see.
            let round = |v: f64, places: i32| {
                let f = 10f64.powi(places);
                n((v * f).round() / f)
            };
            Invocation::new(
                "image.adjust",
                [
                    id.to_string(),
                    format!("brightness={}", round(*brightness, 1)),
                    format!("contrast={}", round(*contrast, 1)),
                    format!("gamma={}", round(*gamma, 2)),
                ],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjust_opens_on_a_selected_image_and_rounds_values() {
        use lasercraft_geom::Affine;
        use lasercraft_project::raster::Gray;
        use lasercraft_project::{Geometry, Image};
        let mut p = Project::default();
        let mut d = Dialogs::default();
        assert!(!d.open_adjust(&p), "nothing selected");
        let img = Image::new(2.0, 2.0, Gray::new(1, 1, vec![0]).unwrap());
        let id = p
            .add_shape_on(Geometry::Image(img), Affine::IDENTITY, 0)
            .unwrap();
        p.set_selection([id]).unwrap();
        assert!(d.open_adjust(&p));
        if let Some(Dialog::Adjust {
            brightness, gamma, ..
        }) = d.open.as_mut()
        {
            *brightness = 12.345;
            *gamma = 1.23456;
        }
        let inv = invocation(d.open.as_ref().unwrap(), 0);
        assert_eq!(
            inv.to_string(),
            format!("image.adjust {id} brightness=12.3 contrast=0 gamma=1.23")
        );
    }

    #[test]
    fn dialogs_build_the_matching_commands() {
        let mut d = Dialogs::default();
        assert!(!d.open("ui.nothing"));
        for (action, expected) in [
            ("ui.offset", "path.offset 1 round"),
            ("ui.grid_array", "arrange.grid 3 2 5 5"),
        ] {
            assert!(d.open(action));
            let inv = invocation(d.open.as_ref().unwrap(), 0);
            assert_eq!(inv.to_string(), expected);
            // Asking for another tool window keeps the open one.
            assert!(d.open("ui.circular_array"));
            assert_eq!(
                invocation(d.open.as_ref().unwrap(), 0).to_string(),
                expected
            );
            d.open = None;
        }
        assert_eq!(
            problem(&Dialog::Grid {
                columns: 1,
                rows: 1,
                gap: [0.0, 0.0]
            }),
            Some("Cannot make the array: it needs at least two blocks.")
        );
        let text = Dialog::Text {
            at: Point::new(1.0, 2.0),
            text: "Hi there".to_owned(),
            font: "DejaVu Sans".to_owned(),
            height: 8.0,
        };
        assert_eq!(
            invocation(&text, 3).to_string(),
            r#"shape.add_text 1 2 8 "DejaVu Sans" "Hi there" @3"#
        );
        let circular = Dialog::Circular {
            count: 4,
            center: Some(Point::new(200.0, 200.0)),
            sweep: 360.0,
            rotate: false,
        };
        assert_eq!(
            invocation(&circular, 0).to_string(),
            "arrange.circular 4 200 200 360 upright"
        );
    }
}
