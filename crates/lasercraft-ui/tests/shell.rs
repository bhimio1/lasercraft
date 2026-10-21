//! Drives the real shell headlessly through egui with synthetic input, and
//! applies the actions it emits to a real command session.

use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, vec2};
use lasercraft_commands::{Registry, Session};
use lasercraft_geom::Point;
use lasercraft_project::Project;
use lasercraft_ui::interaction::Tool;
use lasercraft_ui::keymap::Keymap;
use lasercraft_ui::{Action, LaserView, Shell, ShellView};

struct Harness {
    ctx: egui::Context,
    shell: Shell,
    session: Session,
    time: f64,
    log: Vec<String>,
    laser: LaserView,
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
            laser: LaserView::default(),
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
        let (shell, session, laser) = (&mut self.shell, &self.session, &self.laser);
        let mut output = self.ctx.run_ui(input, |ui| {
            let view = ShellView {
                project: session.project(),
                file: None,
                modified: false,
                undo: undo.as_ref(),
                can_redo: session.redo_depth() > 0,
                laser,
            };
            actions = shell.show(ui, &view);
        });
        // No renderer here to upload textures to.
        output.textures_delta.clear();
        for action in actions {
            if let Action::Run(invocation) = action {
                self.session.run(&invocation).unwrap();
                self.log.push(invocation.to_string());
            } else {
                self.log.push(format!("{action:?}"));
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

    /// Presses and releases a key (an unreleased key would make the next
    /// press of it a repeat, which shortcuts ignore).
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        let event = |pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        };
        self.frame(vec![event(true)], modifiers);
        self.frame(vec![event(false)], modifiers);
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
    assert_eq!(h.log, ["shape.add_rect 20 30 60 40 @0"]);
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

#[test]
fn f12_hides_the_side_panels_and_the_bed_fits_larger() {
    // A wide bed, so its fit depends on the canvas width.
    let mut h = Harness::new(&["project.set_work_area 1000 200"]);
    h.key(Key::Num0, Modifiers::COMMAND);
    h.frame(Vec::new(), Modifiers::NONE);
    let with_panels = h.shell.canvas_view().unwrap().scale();
    h.key(Key::F12, Modifiers::NONE);
    h.key(Key::Num0, Modifiers::COMMAND);
    h.frame(Vec::new(), Modifiers::NONE);
    let without = h.shell.canvas_view().unwrap().scale();
    assert!(without > with_panels, "{with_panels} -> {without}");
}

#[test]
fn layers_panel_lists_used_layers() {
    let mut h = Harness::new(&["shape.add_rect 0 0 10 10 @4", "layer.set 4 name Engrave"]);
    h.frame(Vec::new(), Modifiers::NONE);
    // The panel's text is in the frame's shapes; look for the layer name.
    let mut found = false;
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
        ..RawInput::default()
    };
    let undo = None;
    let mut output = h.ctx.run_ui(input, |ui| {
        let view = ShellView {
            project: h.session.project(),
            file: None,
            modified: false,
            undo,
            can_redo: false,
            laser: &LaserView::default(),
        };
        let _ = h.shell.show(ui, &view);
    });
    output.textures_delta.clear();
    for clipped in &output.shapes {
        if let egui::Shape::Text(text) = &clipped.shape {
            found |= text.galley.text() == "Engrave";
        }
    }
    assert!(found, "layer name not drawn");
}

#[test]
fn alt_p_opens_the_preview_and_draws_what_it_is_given() {
    use lasercraft_ui::{Preview, PreviewKind, PreviewMove};
    let mut h = Harness::new(&["shape.add_rect 10 10 20 20"]);
    assert!(!h.shell.preview_open());
    h.key(Key::P, Modifiers::ALT);
    assert_eq!(h.log, ["Preview"]);
    assert!(h.shell.preview_open());
    h.shell.set_preview(Preview {
        moves: vec![PreviewMove {
            from: Point::new(10.0, 10.0),
            to: Point::new(30.0, 10.0),
            kind: PreviewKind::Burn(0),
            start: 0.0,
            end: 2.0,
        }],
        total_time: 2.0,
        lines: vec!["Estimated time 2.0 s".to_owned()],
        error: Some("shape 1 goes past the right edge".to_owned()),
        warnings: vec!["layer 0: a warning".to_owned()],
    });
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
        ..RawInput::default()
    };
    let mut output = h.ctx.run_ui(input, |ui| {
        let view = ShellView {
            project: h.session.project(),
            file: None,
            modified: false,
            undo: None,
            can_redo: false,
            laser: &LaserView::default(),
        };
        let _ = h.shell.show(ui, &view);
    });
    output.textures_delta.clear();
    let texts: Vec<String> = output
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
            _ => None,
        })
        .collect();
    for wanted in [
        "Job preview",
        "Estimated time 2.0 s",
        "Cannot run: shape 1 goes past the right edge",
        "Warning: layer 0: a warning",
    ] {
        assert!(
            texts.iter().any(|t| t == wanted),
            "{wanted} not in {texts:?}"
        );
    }
}

#[test]
fn m3_shortcuts_group_align_and_weld() {
    let mut h = Harness::new(&[
        "shape.add_rect 0 0 20 20",
        "shape.add_rect 10 30 20 20",
        "select.all",
    ]);
    h.key(Key::ArrowLeft, Modifiers::ALT);
    assert_eq!(h.log, ["arrange.align left"]);
    assert_eq!(h.bounds(2).min.x, 0.0);
    h.key(Key::G, Modifiers::COMMAND);
    assert_eq!(h.session.project().shape(2).unwrap().groups(), [3]);
    h.key(Key::U, Modifiers::COMMAND);
    h.key(Key::W, Modifiers::COMMAND);
    assert_eq!(h.log[1..], ["group.create", "group.ungroup", "path.weld"]);
    assert_eq!(h.session.project().order(), [4]);
}

#[test]
fn node_tool_drags_a_node_on_the_canvas() {
    let mut h = Harness::new(&["shape.add_rect 100 100 50 50", "shape.to_path"]);
    h.key(Key::Backtick, Modifiers::COMMAND);
    assert_eq!(h.shell.tool(), Tool::Nodes);
    // Node 2 is the bottom-right corner; drag it with snapping off.
    h.drag((150.0, 150.0), (163.0, 171.0), Modifiers::COMMAND);
    assert_eq!(h.log.len(), 1);
    assert!(h.log[0].starts_with("node.move 1 0 2 "), "{:?}", h.log);
    let b = h.bounds(1);
    assert!(
        (b.max.x - 163.0).abs() < 0.5 && (b.max.y - 171.0).abs() < 0.5,
        "{b:?}"
    );
}

#[test]
fn text_tool_click_opens_the_text_window_without_running_anything() {
    let mut h = Harness::new(&[]);
    h.key(Key::T, Modifiers::COMMAND);
    assert_eq!(h.shell.tool(), Tool::Text);
    h.drag((50.0, 50.0), (50.0, 50.0), Modifiers::NONE);
    h.frame(Vec::new(), Modifiers::NONE);
    assert!(h.log.is_empty());
    assert!(h.session.project().shapes().is_empty());
}

#[test]
fn alt_shift_l_asks_where_to_save_gcode() {
    let mut h = Harness::new(&["shape.add_rect 10 10 20 20"]);
    h.key(Key::L, Modifiers::ALT | Modifiers::SHIFT);
    assert!(
        h.log.is_empty(),
        "the prompt alone runs nothing: {:?}",
        h.log
    );
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
        ..RawInput::default()
    };
    let mut output = h.ctx.run_ui(input, |ui| {
        let view = ShellView {
            project: h.session.project(),
            file: None,
            modified: false,
            undo: None,
            can_redo: false,
            laser: &LaserView::default(),
        };
        let _ = h.shell.show(ui, &view);
    });
    output.textures_delta.clear();
    let found = output.shapes.iter().any(|c| match &c.shape {
        egui::Shape::Text(t) => t.galley.text() == "Path to the .gcode file to write:",
        _ => false,
    });
    assert!(found, "G-code prompt not shown");
}

impl Harness {
    /// Every piece of text drawn in one frame.
    fn texts(&mut self) -> Vec<String> {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
            time: Some(self.time),
            ..RawInput::default()
        };
        let (shell, session, laser) = (&mut self.shell, &self.session, &self.laser);
        let mut output = self.ctx.run_ui(input, |ui| {
            let view = ShellView {
                project: session.project(),
                file: None,
                modified: false,
                undo: None,
                can_redo: false,
                laser,
            };
            let _ = shell.show(ui, &view);
        });
        output.textures_delta.clear();
        output
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }
}

#[test]
fn laser_shortcuts_run_device_commands_with_the_panel_settings() {
    let mut h = Harness::new(&["shape.add_rect 10 10 20 20"]);
    let ctrl_shift = Modifiers::COMMAND | Modifiers::SHIFT;
    let ctrl_alt = Modifiers::COMMAND | Modifiers::ALT;
    h.key(Key::S, Modifiers::ALT);
    h.key(Key::R, Modifiers::ALT);
    h.key(Key::R, Modifiers::ALT | Modifiers::SHIFT);
    h.key(Key::CloseBracket, ctrl_shift);
    h.key(Key::CloseCurlyBracket, ctrl_shift);
    h.key(Key::OpenBracket, ctrl_shift);
    h.key(Key::OpenBracket, ctrl_alt);
    h.key(Key::CloseBracket, ctrl_alt);
    h.key(Key::H, Modifiers::COMMAND);
    h.key(Key::Pause, Modifiers::NONE);
    h.key(Key::Pause, Modifiers::COMMAND);
    assert_eq!(
        h.log,
        [
            "device.start from=absolute anchor=bottom-left",
            "device.frame from=absolute anchor=bottom-left speed=50",
            "device.frame outline from=absolute anchor=bottom-left speed=50",
            "device.jog up 10 speed=50",
            "device.jog up 10 speed=50",
            "device.jog down 10 speed=50",
            "device.jog left 10 speed=50",
            "device.jog right 10 speed=50",
            "device.home",
            "device.pause",
            "device.stop",
        ]
    );
    // While paused, the same key resumes.
    h.laser.port = Some("simulator".into());
    h.laser.busy = Some(lasercraft_ui::Busy {
        kind: "job".into(),
        acknowledged: 3,
        total: 10,
        elapsed: 2.0,
        paused: true,
        constant_power: false,
    });
    h.log.clear();
    h.key(Key::Pause, Modifiers::NONE);
    assert_eq!(h.log, ["device.resume"]);
}

#[test]
fn stop_and_pause_show_whenever_a_laser_is_connected() {
    let mut h = Harness::new(&[]);
    let texts = h.texts();
    assert!(!texts.iter().any(|t| t == "Stop"), "no laser, no Stop");
    h.laser.port = Some("/dev/ttyUSB0".into());
    h.laser.ready = true;
    h.laser.state = "Idle".into();
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "Stop") && texts.iter().any(|t| t == "Pause"));
    assert!(
        texts.iter().any(|t| t == "Laser Idle on /dev/ttyUSB0"),
        "{texts:?}"
    );
    // A job with constant power says Pause only holds motion.
    h.laser.busy = Some(lasercraft_ui::Busy {
        kind: "job".into(),
        acknowledged: 1,
        total: 4,
        elapsed: 1.0,
        paused: false,
        constant_power: true,
    });
    assert!(h.texts().iter().any(|t| t == "Pause (motion)"));
}

#[test]
fn the_laser_tab_shows_status_console_and_the_alarm_window() {
    let mut h = Harness::new(&[]);
    h.shell.show_laser_tab();
    h.laser.port = Some("simulator".into());
    h.laser.ready = true;
    h.laser.state = "Idle".into();
    h.laser.position = Some([12.5, 30.0]);
    h.laser.console = vec![
        lasercraft_ui::ConsoleLine {
            sent: true,
            text: "$$".into(),
            routine: false,
        },
        lasercraft_ui::ConsoleLine {
            sent: false,
            text: "<Idle|MPos:0,0,0>".into(),
            routine: true,
        },
    ];
    h.laser.alarm = Some("Alarm 1: a limit switch was hit".into());
    let texts = h.texts();
    for want in [
        "Ready (Idle) at X12.5 Y30 on simulator",
        "> $$",
        "Laser alarm",
        "Alarm 1: a limit switch was hit",
        "Unlock",
        "Set origin",
        "Frame outline",
    ] {
        assert!(texts.iter().any(|t| t == want), "{want}: {texts:?}");
    }
    assert!(
        !texts.iter().any(|t| t.starts_with("<Idle")),
        "routine lines are hidden by default"
    );
}
