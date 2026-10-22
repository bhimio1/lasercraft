//! The job preview window (`docs/spec/preview.md`). It draws moves the app
//! hands it; planning happens outside the UI.

use egui::{Pos2, Sense, Shape, Stroke, StrokeKind};
use lasercraft_geom::{Point, Rect};
use lasercraft_project::{Color, Project};

use crate::colors::layer_color;
use crate::laser_panel::clock;
use crate::theme::{DASH, DOTS, Tokens, size, weight};
use crate::view::View;
use crate::widgets::Bounded;
use crate::widgets::{self, Band};

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
    /// The user zoomed or panned; until then the view follows the window.
    moved: bool,
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
        let mut close = false;
        let frame = ctx.content_rect();
        egui::Window::new("Job preview")
            .open(&mut open)
            .collapsible(false)
            .default_size([frame.width() * 0.6, frame.height() * 0.85])
            .bounded(ctx)
            .show(ctx, |ui| close = self.contents(ui, project));
        open && !close
    }

    /// The window's parts, top to bottom; returns whether Close was clicked
    /// (or Esc pressed).
    fn contents(&mut self, ui: &mut egui::Ui, project: &Project) -> bool {
        let t = Tokens::of(ui.ctx());
        let mut close = false;
        egui::Panel::bottom("preview-text")
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.time_row(ui);
                let n = self.preview.lines.len();
                for (i, line) in self.preview.lines.iter().enumerate() {
                    let text = egui::RichText::new(line);
                    // The bounds line is a detail: small and muted.
                    let text = if i + 1 == n && n > 2 {
                        text.small().color(t.text_muted)
                    } else {
                        text.color(t.text)
                    };
                    ui.label(text);
                }
                match &self.preview.error {
                    Some(e) => {
                        widgets::band(ui, Band::Error, &format!("Cannot run: {e}"));
                    }
                    None => {
                        widgets::band(ui, Band::Success, "Fits the work area.");
                    }
                }
                for w in &self.preview.warnings {
                    widgets::band(ui, Band::Warning, &format!("Warning: {w}"));
                }
                self.playback_row(ui);
                ui.add_space(size::GAP);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    close = ui
                        .add(egui::Button::new("Close").min_size(egui::vec2(80.0, size::ROW)))
                        .on_hover_text("Close the job preview")
                        .clicked();
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| self.canvas(ui, project));
        close || ui.input(|i| i.key_pressed(egui::Key::Escape))
    }

    fn time_row(&mut self, ui: &mut egui::Ui) {
        let total = self.preview.total_time;
        ui.horizontal(|ui| {
            ui.label("Time (s)");
            let mut time = self.time.unwrap_or(total);
            ui.spacing_mut().slider_width = (ui.available_width() - 220.0).max(120.0);
            let slider = egui::Slider::new(&mut time, 0.0..=total.max(f64::EPSILON))
                .max_decimals(1)
                .show_value(true);
            if ui.add(slider).changed() {
                self.time = Some(time);
            }
            if ui
                .add_enabled(self.time.is_some(), egui::Button::new("Show whole job"))
                .clicked()
            {
                self.time = None;
            }
        });
    }

    fn playback_row(&mut self, ui: &mut egui::Ui) {
        let t = Tokens::of(ui.ctx());
        let total = self.preview.total_time;
        let time = self.time.unwrap_or(total);
        let started = self.preview.head_at(time).map_or(0, |(_, n)| n);
        ui.horizontal(|ui| {
            widgets::switch(ui, &mut self.show_travel, "Show travel")
                .on_hover_text("Draw laser-off travel moves.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} of {} · move {started} of {}",
                        clock(time),
                        clock(total),
                        self.preview.moves.len()
                    ))
                    .small()
                    .color(t.text_muted),
                );
            });
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui, project: &Project) {
        let t = Tokens::of(ui.ctx());
        let size = ui.available_size().max(egui::vec2(200.0, 200.0));
        let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
        let screen = rect_of(response.rect);
        let area = project.work_area();
        let bed = Rect::from_corners(Point::ZERO, Point::new(area.width_mm, area.height_mm));
        let content = self.preview.bounds().map_or(bed, |b| b.union(bed));
        if self.view.is_none() || (!self.moved && self.screen != Some(screen)) {
            self.view = Some(View::fit(&screen, &content, FIT_MARGIN_PX));
        }
        let Some(mut view) = self.view else { return };
        if self.moved
            && let Some(old) = self.screen
            && old != screen
        {
            // After a zoom, resizing keeps it; the drawing stays centred.
            view.pan(screen.center() - old.center());
        }
        self.screen = Some(screen);
        if let Some(pointer) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let at = Point::new(f64::from(pointer.x), f64::from(pointer.y));
                view.zoom_about(at, (f64::from(scroll) / 400.0).exp());
                self.moved = true;
            }
        }
        let panning = response.dragged_by(egui::PointerButton::Middle)
            || (response.dragged() && ui.input(|i| i.key_down(egui::Key::Space)));
        if panning {
            let d = response.drag_delta();
            view.pan(Point::new(f64::from(d.x), f64::from(d.y)));
            self.moved = true;
        }
        self.view = Some(view);
        let painter = painter.with_clip_rect(response.rect);
        painter.rect_filled(response.rect, 0.0, t.canvas);
        let to = |p: Point| pos(view.to_screen(p));
        let bed_screen = egui::Rect::from_two_pos(to(bed.min), to(bed.max));
        painter.rect_filled(bed_screen, 0.0, t.bed);
        crate::canvas::paint_grid(&painter, &view, &bed, &t);
        painter.rect_stroke(
            bed_screen,
            0.0,
            Stroke::new(weight::THIN, t.bed_edge),
            StrokeKind::Middle,
        );
        crate::canvas::paint_origin(ui, &painter, bed_screen, area.origin, &t);
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
                    painter.line_segment(points, Stroke::new(weight::NORMAL, layer_color(color)));
                }
                PreviewKind::Unlit => {
                    painter.extend(Shape::dotted_line(&points, t.move_unlit, DOTS.0, DOTS.1));
                }
                PreviewKind::Travel if self.show_travel => {
                    painter.extend(Shape::dashed_line(
                        &points,
                        Stroke::new(weight::HAIRLINE, t.move_travel),
                        DASH.0,
                        DASH.1,
                    ));
                }
                PreviewKind::Travel => {}
            }
        }
        if let Some((head, _)) = self.preview.head_at(time) {
            painter.circle_filled(to(head), 6.0, t.handle_fill);
            painter.circle_filled(to(head), 4.0, t.preview_head);
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
