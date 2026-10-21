//! Small windows that collect arguments for M3 commands: offset, grid and
//! circular arrays, and text (`docs/spec/arrange.md`, `docs/spec/text.md`).
//! Each produces one invocation when confirmed; none changes the project.

use lasercraft_commands::Invocation;
use lasercraft_geom::Point;
use lasercraft_project::{Color, Project};

/// Corner styles offered for offsets, as `path.offset` names them.
const CORNERS: [&str; 3] = ["round", "miter", "bevel"];

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
        self.open = Some(match action {
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
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                fields(ui, dialog, families);
                confirmed = ui.button("OK").clicked();
            });
        let result = confirmed.then(|| invocation(dialog, color));
        if confirmed || !open {
            self.open = None;
        }
        result
    }
}

fn number(ui: &mut egui::Ui, label: &str, value: &mut f64, speed: f64, unit: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).speed(speed).suffix(unit));
    });
}

fn whole(ui: &mut egui::Ui, label: &str, value: &mut u32, max: u32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(1..=max));
    });
}

fn fields(ui: &mut egui::Ui, dialog: &mut Dialog, families: &[String]) {
    match dialog {
        Dialog::Offset { distance, corner } => {
            number(ui, "Distance (negative: inward)", distance, 0.1, " mm");
            egui::ComboBox::from_label("Corners")
                .selected_text(CORNERS[*corner])
                .show_ui(ui, |ui| {
                    for (i, name) in CORNERS.iter().enumerate() {
                        ui.selectable_value(corner, i, *name);
                    }
                });
        }
        Dialog::Grid { columns, rows, gap } => {
            whole(ui, "Columns", columns, 100);
            whole(ui, "Rows", rows, 100);
            number(ui, "Gap between columns", &mut gap[0], 0.1, " mm");
            number(ui, "Gap between rows", &mut gap[1], 0.1, " mm");
        }
        Dialog::Circular {
            count,
            center,
            sweep,
            rotate,
        } => {
            whole(ui, "Count (with the original)", count, 1000);
            if let Some(c) = center {
                number(ui, "Center x", &mut c.x, 0.5, " mm");
                number(ui, "Center y", &mut c.y, 0.5, " mm");
            }
            number(ui, "Sweep", sweep, 1.0, "°");
            ui.checkbox(rotate, "Turn the copies with the circle");
        }
        Dialog::Text {
            text, font, height, ..
        } => {
            ui.label("Text:");
            ui.add(egui::TextEdit::multiline(text).desired_rows(2));
            egui::ComboBox::from_label("Font")
                .selected_text(font.as_str())
                .height(300.0)
                .show_ui(ui, |ui| {
                    for family in families {
                        ui.selectable_value(font, family.clone(), family);
                    }
                });
            number(ui, "Capital height", height, 0.1, " mm");
        }
    }
}

fn invocation(dialog: &Dialog, color: Color) -> Invocation {
    match dialog {
        Dialog::Offset { distance, corner } => {
            Invocation::new("path.offset", [n(*distance), CORNERS[*corner].to_owned()])
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        }
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
