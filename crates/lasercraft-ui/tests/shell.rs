//! Drives the real shell headlessly through egui with synthetic input, and
//! applies the actions it emits to a real command session.

use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, vec2};
use lasercraft_commands::{Registry, Session};
use lasercraft_geom::Point;
use lasercraft_project::Project;
use lasercraft_ui::interaction::Tool;
use lasercraft_ui::keymap::Keymap;
use lasercraft_ui::{Action, Shell, ShellView};

struct Harness {
    ctx: egui::Context,
    shell: Shell,
    session: Session,
    time: f64,
    log: Vec<String>,
}

impl Harness {
    fn new(lines: &[&str]) -> Self {
        let mut session = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            session.run_line(line).unwrap();
        }
        let mut h = Self {
            ctx: egui::Context::default(),
            shell: Shell::new(Keymap::default_keymap()),
            session,
            time: 0.0,
            log: Vec::new(),
        };
        h.frame(Vec::new(), Modifiers::NONE);
        h
    }

    /// Runs one frame with `events`, then applies the emitted actions.
    fn frame(&mut self, mut events: Vec<Event>, modifiers: Modifiers) {
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
            time: Some(self.time),
            events,
            ..RawInput::default()
        };
        self.time += 0.25;
        let undo = self.session.undo_label().cloned();
        let mut actions = Vec::new();
        let (shell, session) = (&mut self.shell, &self.session);
        let mut output = self.ctx.run_ui(input, |ui| {
            let view = ShellView {
                project: session.project(),
                file: None,
                modified: false,
                undo: undo.as_ref(),
                can_redo: session.redo_depth() > 0,
            };
            actions = shell.show(ui, &view);
        });
        // No renderer here to upload textures to.
        output.textures_delta.clear();
        for action in actions {
            if let Action::Run(invocation) = action {
                self.session.run(&invocation).unwrap();
                self.log.push(invocation.to_string());
            }
        }
    }

    fn screen(&self, x: f64, y: f64) -> Pos2 {
        let s = self
            .shell
            .canvas_view()
            .unwrap()
            .to_screen(Point::new(x, y));
        Pos2::new(s.x as f32, s.y as f32)
    }

    fn drag(&mut self, from: (f64, f64), to: (f64, f64), modifiers: Modifiers) {
        let (a, b) = (self.screen(from.0, from.1), self.screen(to.0, to.1));
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers,
        };
        self.frame(vec![Event::PointerMoved(a)], modifiers);
        self.frame(vec![button(a, true)], modifiers);
        for t in [0.25_f32, 0.5, 0.75, 1.0] {
            self.frame(vec![Event::PointerMoved(a + (b - a) * t)], modifiers);
        }
        self.frame(vec![button(b, false)], modifiers);
    }

    fn key(&mut self, key: Key, modifiers: Modifiers) {
        let event = Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        self.frame(vec![event], modifiers);
    }

    fn bounds(&self, id: u32) -> lasercraft_geom::Rect {
        self.session.project().shape(id).unwrap().bounds()
    }
}

#[test]
fn dragging_a_shape_on_the_canvas_moves_it() {
    let mut h = Harness::new(&["shape.add_rect 100 100 50 50", "select.clear"]);
    h.drag((125.0, 125.0), (225.0, 125.0), Modifiers::NONE);
    assert_eq!(h.log, ["select.set 1", "shape.move 100 0 1"]);
    assert_eq!(h.bounds(1).min, Point::new(200.0, 100.0));
    assert_eq!(h.session.undo_depth(), 2);
}

#[test]
fn rectangle_tool_draws_through_the_keymap() {
    let mut h = Harness::new(&[]);
    h.key(Key::R, Modifiers::COMMAND);
    assert_eq!(h.shell.tool(), Tool::Rect);
    h.drag((20.0, 30.0), (80.0, 70.0), Modifiers::NONE);
    assert_eq!(h.log, ["shape.add_rect 20 30 60 40"]);
    h.key(Key::Escape, Modifiers::NONE);
    assert_eq!(h.shell.tool(), Tool::Select);
}

#[test]
fn keyboard_edits_act_on_the_selection() {
    let mut h = Harness::new(&["shape.add_rect 0 0 10 10"]);
    h.key(Key::D, Modifiers::COMMAND);
    h.key(Key::ArrowRight, Modifiers::SHIFT);
    h.key(Key::Period, Modifiers::NONE);
    h.key(Key::Delete, Modifiers::NONE);
    h.key(Key::Z, Modifiers::COMMAND);
    assert_eq!(
        h.log,
        [
            "shape.duplicate",
            "shape.move 10 0",
            "shape.rotate 90",
            "shape.delete",
            "edit.undo"
        ]
    );
    assert_eq!(h.bounds(2).min, Point::new(10.0, 0.0));
}

#[test]
fn clipboard_events_map_to_the_clipboard_chords() {
    let mut h = Harness::new(&["shape.add_rect 0 0 10 10"]);
    h.frame(vec![Event::Copy], Modifiers::NONE);
    // The pointer is over the canvas, so paste lands there.
    let target = h.screen(200.0, 100.0);
    h.frame(vec![Event::PointerMoved(target)], Modifiers::NONE);
    h.frame(vec![Event::Paste("[]".to_owned())], Modifiers::NONE);
    assert_eq!(h.log[0], "edit.copy");
    assert!(h.log[1].starts_with("edit.paste 2"), "{:?}", h.log);
    let pasted = h.bounds(2).center();
    assert!(
        pasted.approx_eq(Point::new(200.0, 100.0), 0.01),
        "{pasted:?}"
    );
}

#[test]
fn zoom_keys_change_the_view_and_fit_restores_it() {
    let mut h = Harness::new(&[]);
    let fitted = h.shell.canvas_view().unwrap().scale();
    h.key(Key::Equals, Modifiers::COMMAND);
    h.frame(Vec::new(), Modifiers::NONE);
    let zoomed = h.shell.canvas_view().unwrap().scale();
    assert!(
        (zoomed / fitted - 1.25).abs() < 1e-9,
        "{fitted} -> {zoomed}"
    );
    h.key(Key::Num0, Modifiers::COMMAND);
    h.frame(Vec::new(), Modifiers::NONE);
    assert_eq!(h.shell.canvas_view().unwrap().scale(), fitted);
}

#[test]
fn box_selection_direction_picks_contained_or_touched() {
    let mut h = Harness::new(&[
        "shape.add_rect 0 0 10 10",
        "shape.add_rect 50 0 10 10",
        "select.clear",
    ]);
    h.drag((55.0, 20.0), (5.0, 5.0), Modifiers::NONE);
    assert_eq!(h.session.project().selection(), [1, 2]);
    h.drag((-5.0, -5.0), (30.0, 30.0), Modifiers::NONE);
    assert_eq!(h.session.project().selection(), [1]);
}
