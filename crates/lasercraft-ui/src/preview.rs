//! The job preview window (`docs/spec/preview.md`). It draws moves the app
//! hands it; planning happens outside the UI.

use egui::{Color32, Pos2, Sense, Shape, Stroke, StrokeKind};
use lasercraft_geom::{Point, Rect};
use lasercraft_project::{Color, Project};

use crate::colors::layer_color;
use crate::view::View;

const BED: Color32 = Color32::from_gray(246);
const BED_EDGE: Color32 = Color32::from_rgb(40, 160, 60);
const UNLIT: Color32 = Color32::from_rgb(255, 140, 0);
const TRAVEL: Color32 = Color32::from_rgb(215, 40, 40);
const HEAD: Color32 = Color32::from_rgb(30, 110, 230);
const ERROR: Color32 = Color32::from_rgb(200, 30, 30);
const FIT_MARGIN_PX: f64 = 16.0;

/// How a preview move is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewKind {
    /// Laser on, in its layer's color.
    Burn(Color),
    /// Laser off along the path (overscan, gaps, tabs).
    Unlit,
    /// Laser off between cuts.
    Travel,
}

/// One move with its place on the job's timeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewMove {
    /// Start, document mm.
    pub from: Point,
    /// End, document mm.
    pub to: Point,
    /// How to draw it.
    pub kind: PreviewKind,
    /// Seconds into the job when it starts.
    pub start: f64,
    /// Seconds into the job when it ends.
    pub end: f64,
}

/// What the preview shows: built by the app from a plan.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preview {
    /// Every move in order.
    pub moves: Vec<PreviewMove>,
    /// Estimated job time, seconds.
    pub total_time: f64,
    /// Statistics, one per line.
    pub lines: Vec<String>,
    /// Why the job cannot run (outside the work area), if so.
    pub error: Option<String>,
    /// Planner warnings.
    pub warnings: Vec<String>,
}

impl Preview {
    /// The head position at `time` and how many moves have started.
    #[must_use]
    pub fn head_at(&self, time: f64) -> Option<(Point, usize)> {
        let first = self.moves.first()?;
        let mut head = (first.from, 0);
        for (i, m) in self.moves.iter().enumerate() {
            if time < m.start {
                break;
            }
            let span = m.end - m.start;
            let t = if span > 0.0 {
                ((time - m.start) / span).clamp(0.0, 1.0)
            } else {
                1.0
            };
            head = (m.from + (m.to - m.from) * t, i + 1);
        }
        Some(head)
    }

    fn bounds(&self) -> Option<Rect> {
        Rect::bounding(self.moves.iter().flat_map(|m| [m.from, m.to]))
    }
}

/// UI state of the preview window.
#[derive(Default)]
pub(crate) struct PreviewWindow {
    preview: Preview,
    /// Scrub position, seconds; `None` shows the whole job.
    time: Option<f64>,
    show_travel: bool,
    view: Option<View>,
    screen: Option<Rect>,
}

fn pos(p: Point) -> Pos2 {
    Pos2::new(p.x as f32, p.y as f32)
}

fn rect_of(r: egui::Rect) -> Rect {
    Rect::from_corners(
        Point::new(f64::from(r.min.x), f64::from(r.min.y)),
        Point::new(f64::from(r.max.x), f64::from(r.max.y)),
    )
}

impl PreviewWindow {
    pub fn new() -> Self {
        Self {
            show_travel: true,
            ..Self::default()
        }
    }

    /// Replaces what is shown, keeping the zoom and options.
    pub fn set(&mut self, preview: Preview) {
        if let Some(t) = self.time {
            self.time = Some(t.min(preview.total_time));
        }
        self.preview = preview;
    }

    /// Draws the window. Returns false once the user closes it.
    pub fn show(&mut self, ctx: &egui::Context, project: &Project) -> bool {
        let mut open = true;
        egui::Window::new("Job preview")
            .open(&mut open)
            .default_size([720.0, 560.0])
            .show(ctx, |ui| self.contents(ui, project));
        open
    }

    fn contents(&mut self, ui: &mut egui::Ui, project: &Project) {
        let total = self.preview.total_time;
        for line in &self.preview.lines {
            ui.label(line);
        }
        match &self.preview.error {
            Some(e) => {
                ui.colored_label(ERROR, format!("Cannot run: {e}"));
            }
            None => {
                ui.label("Fits the work area.");
            }
        }
        for w in &self.preview.warnings {
            ui.weak(format!("Warning: {w}"));
        }
        ui.horizontal(|ui| {
            let mut t = self.time.unwrap_or(total);
            let slider = egui::Slider::new(&mut t, 0.0..=total.max(f64::EPSILON))
                .text("time")
                .suffix(" s")
                .max_decimals(1);
            if ui.add(slider).changed() {
                self.time = Some(t);
            }
            if ui.button("Whole job").clicked() {
                self.time = None;
            }
            ui.checkbox(&mut self.show_travel, "Show travel");
        });
        self.canvas(ui, project);
    }

    fn canvas(&mut self, ui: &mut egui::Ui, project: &Project) {
        let size = ui.available_size().max(egui::vec2(200.0, 200.0));
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let screen = rect_of(response.rect);
        let area = project.work_area();
        let bed = Rect::from_corners(Point::ZERO, Point::new(area.width_mm, area.height_mm));
        let content = self.preview.bounds().map_or(bed, |b| b.union(bed));
        if self.view.is_none() || self.screen != Some(screen) {
            self.view = Some(View::fit(&screen, &content, FIT_MARGIN_PX));
            self.screen = Some(screen);
        }
        let Some(mut view) = self.view else { return };
        if let Some(pointer) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let at = Point::new(f64::from(pointer.x), f64::from(pointer.y));
                view.zoom_about(at, (f64::from(scroll) / 400.0).exp());
                self.view = Some(view);
            }
        }
        let to = |p: Point| pos(view.to_screen(p));
        painter.rect_filled(egui::Rect::from_two_pos(to(bed.min), to(bed.max)), 0.0, BED);
        painter.rect_stroke(
            egui::Rect::from_two_pos(to(bed.min), to(bed.max)),
            0.0,
            Stroke::new(1.5, BED_EDGE),
            StrokeKind::Middle,
        );
        let time = self.time.unwrap_or(f64::INFINITY);
        for m in &self.preview.moves {
            if m.start > time {
                break;
            }
            let end = if time < m.end && m.end > m.start {
                m.from + (m.to - m.from) * ((time - m.start) / (m.end - m.start))
            } else {
                m.to
            };
            let points = [to(m.from), to(end)];
            match m.kind {
                PreviewKind::Burn(color) => {
                    painter.line_segment(points, Stroke::new(1.2, layer_color(color)));
                }
                PreviewKind::Unlit => {
                    painter.extend(Shape::dashed_line(
                        &points,
                        Stroke::new(0.8, UNLIT),
                        3.0,
                        3.0,
                    ));
                }
                PreviewKind::Travel if self.show_travel => {
                    painter.extend(Shape::dashed_line(
                        &points,
                        Stroke::new(0.8, TRAVEL),
                        5.0,
                        4.0,
                    ));
                }
                PreviewKind::Travel => {}
            }
        }
        if self.time.is_some()
            && let Some((head, _)) = self.preview.head_at(time)
        {
            painter.circle_filled(to(head), 4.0, HEAD);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(from: (f64, f64), to: (f64, f64), start: f64, end: f64) -> PreviewMove {
        PreviewMove {
            from: Point::new(from.0, from.1),
            to: Point::new(to.0, to.1),
            kind: PreviewKind::Burn(1),
            start,
            end,
        }
    }

    #[test]
    fn head_follows_the_timeline() {
        let p = Preview {
            moves: vec![
                mv((0.0, 0.0), (10.0, 0.0), 0.0, 1.0),
                mv((10.0, 0.0), (10.0, 10.0), 1.0, 3.0),
            ],
            total_time: 3.0,
            ..Preview::default()
        };
        assert_eq!(p.head_at(0.5), Some((Point::new(5.0, 0.0), 1)));
        assert_eq!(p.head_at(2.0), Some((Point::new(10.0, 5.0), 2)));
        assert_eq!(p.head_at(9.0), Some((Point::new(10.0, 10.0), 2)));
        assert_eq!(Preview::default().head_at(1.0), None);
    }
}
