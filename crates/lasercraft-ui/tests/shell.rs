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
    /// Window height, px.
    height: f32,
}

impl Harness {
    fn new(lines: &[&str]) -> Self {
        Self::with_height(lines, 800.0)
    }

    fn with_height(lines: &[&str], height: f32) -> Self {
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
            height,
        };
        // Panels settle to their content's size on the second frame.
        h.frame(Vec::new(), Modifiers::NONE);
        h.frame(Vec::new(), Modifiers::NONE);
        h
    }

    /// Runs one frame with `events`, then applies the emitted actions.
    fn frame(&mut self, mut events: Vec<Event>, modifiers: Modifiers) {
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, self.height))),
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
                workshop: session.workshop(),
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
    assert!(h.log[1].starts_with("edit.paste "), "{:?}", h.log);
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
            workshop: h.session.workshop(),
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
            workshop: h.session.workshop(),
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
            workshop: h.session.workshop(),
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
        self.drawn_texts()
            .into_iter()
            .map(|(text, _)| text)
            .collect()
    }

    /// Where `want` was last drawn (the topmost copy).
    fn text_at(&mut self, want: &str) -> Option<Pos2> {
        self.drawn_texts()
            .into_iter()
            .rev()
            .find(|(text, _)| text == want)
            .map(|(_, rect)| rect.center())
    }

    fn click(&mut self, pos: Pos2) {
        let button = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(pos)], Modifiers::NONE);
        self.frame(vec![button(true)], Modifiers::NONE);
        self.frame(vec![button(false)], Modifiers::NONE);
    }

    fn drawn_texts(&mut self) -> Vec<(String, Rect)> {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, self.height))),
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
                workshop: session.workshop(),
            };
            let _ = shell.show(ui, &view);
        });
        output.textures_delta.clear();
        output
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(text) => Some((
                    text.galley.text().to_owned(),
                    Rect::from_min_size(text.pos, text.galley.size()),
                )),
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

fn connected(h: &mut Harness) {
    h.laser.port = Some("/dev/ttyUSB0".into());
    h.laser.baud = Some(115_200);
    h.laser.ready = true;
    h.laser.word = lasercraft_ui::MachineWord::Ready;
    // A new layer is laid out once before it is drawn.
    h.frame(Vec::new(), Modifiers::NONE);
}

#[test]
fn the_job_bar_shows_stop_and_pause_whenever_a_laser_is_connected() {
    let mut h = Harness::new(&[]);
    // F12 hides the dock and its Machine panel; the job bar stays.
    h.key(Key::F12, Modifiers::NONE);
    let texts = h.texts();
    assert!(!texts.iter().any(|t| t == "Stop"), "no laser, no job bar");
    connected(&mut h);
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "Stop") && texts.iter().any(|t| t == "Pause"));
    assert!(texts.iter().any(|t| t == "Ready"), "{texts:?}");
    // A job with constant power says Pause only holds motion.
    h.laser.busy = Some(lasercraft_ui::Busy {
        kind: "job".into(),
        acknowledged: 1,
        total: 4,
        elapsed: 1.0,
        paused: false,
        constant_power: true,
    });
    h.laser.word = lasercraft_ui::MachineWord::Running;
    let texts = h.texts();
    for want in ["Pause (motion only)", "The laser may stay on", "25% sent"] {
        assert!(texts.iter().any(|t| t == want), "{want}: {texts:?}");
    }
}

#[test]
fn stop_in_the_job_bar_works_with_windows_open() {
    let mut h = Harness::new(&["shape.add_rect 10 10 50 30", "select.all"]);
    connected(&mut h);
    // Open the project notes and the offset window over everything.
    h.key(Key::N, Modifiers::CTRL | Modifiers::ALT);
    h.key(Key::O, Modifiers::ALT);
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "Project notes") && texts.iter().any(|t| t == "Offset"));
    let stop = h.text_at("Stop").expect("Stop is drawn");
    h.log.clear();
    h.click(stop);
    assert!(h.log.iter().any(|l| l == "device.stop"), "{:?}", h.log);
}

#[test]
fn pause_and_stop_chords_work_while_typing() {
    let mut h = Harness::new(&[]);
    connected(&mut h);
    // Focus the command line and type.
    let field = h
        .text_at("Type a command, for example project.set_work_area 400 300")
        .unwrap();
    h.click(field);
    h.frame(vec![Event::Text("abc".into())], Modifiers::NONE);
    h.log.clear();
    h.key(Key::Pause, Modifiers::NONE);
    h.key(Key::Pause, Modifiers::CTRL);
    // Other chords stay with the text field.
    h.key(Key::H, Modifiers::CTRL);
    assert_eq!(h.log, ["device.pause", "device.stop"]);
}

#[test]
fn the_machine_jog_and_console_panels_show_their_parts() {
    let mut h = Harness::new(&[]);
    connected(&mut h);
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
    h.laser.word = lasercraft_ui::MachineWord::Alarm;
    h.laser.alarm = Some("Alarm 1: a limit switch was hit".into());
    let texts = h.texts();
    for want in [
        "Machine",
        "on /dev/ttyUSB0 at 115200 baud",
        "Alarm",
        "Alarm 1: a limit switch was hit",
        "Clear alarm",
        "Frame outline",
        "Place job at",
        "Layers",
        "Jog",
        "Console",
    ] {
        assert!(texts.iter().any(|t| t == want), "{want}: {texts:?}");
    }
    let jog = h.text_at("Jog").unwrap();
    h.click(jog);
    let texts = h.texts();
    for want in [
        "Head position",
        "X 12.50  Y 30.00",
        "Set user origin",
        "Jog distance (mm)",
    ] {
        assert!(texts.iter().any(|t| t == want), "{want}: {texts:?}");
    }
    h.shell.show_panel(lasercraft_ui::DockPanel::Console);
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "> $$"), "{texts:?}");
    assert!(
        !texts.iter().any(|t| t.starts_with("<Idle")),
        "routine lines are hidden by default"
    );
}

#[test]
fn the_menu_bar_titles_are_in_order() {
    let mut h = Harness::new(&[]);
    let texts = h.texts();
    let titles = [
        "File", "Edit", "Tools", "Arrange", "Machine", "View", "Window",
    ];
    let at: Vec<usize> = titles
        .iter()
        .map(|t| {
            texts
                .iter()
                .position(|x| x == t)
                .unwrap_or_else(|| panic!("{t}"))
        })
        .collect();
    assert!(at.windows(2).all(|w| w[0] < w[1]), "{texts:?}");
}

#[test]
fn images_are_drawn_as_textures_and_alt_i_opens_their_adjustments() {
    use lasercraft_commands::Invocation;
    use lasercraft_project::raster::Gray;
    use lasercraft_project::{Geometry, Image};
    let mut h = Harness::new(&[]);
    let img = Geometry::Image(Image::new(
        40.0,
        20.0,
        Gray::new(4, 2, vec![0, 80, 160, 255, 255, 160, 80, 0]).unwrap(),
    ));
    let shape = format!(
        r#"{{"id":7,"layer":2,"geometry":{},"transform":[1.0,0.0,0.0,1.0,50.0,50.0]}}"#,
        serde_json::to_string(&img).unwrap()
    );
    h.session
        .run(&Invocation::new("shape.insert", ["0".to_owned(), shape]))
        .unwrap();
    h.frame(Vec::new(), Modifiers::NONE);
    assert_eq!(h.shell.image_textures(), 1);
    // Nothing selected: Alt+I only explains.
    h.session.run_line("select.clear").unwrap();
    h.key(Key::I, Modifiers::ALT);
    assert!(!h.texts().iter().any(|t| t == "Image adjustments"));
    h.session.run_line("select.set 7").unwrap();
    h.key(Key::I, Modifiers::ALT);
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "Image adjustments"), "{texts:?}");
    // Deleting the image drops its texture.
    h.session.run_line("shape.delete 7").unwrap();
    h.frame(Vec::new(), Modifiers::NONE);
    assert_eq!(h.shell.image_textures(), 0);
}

#[test]
fn the_materials_tab_applies_an_entry_to_the_edited_layer() {
    let mut h = Harness::new(&["shape.add_rect 10 10 20 20 @4"]);
    let tab = h.text_at("Materials").expect("Materials tab");
    h.click(tab);
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "LaserCraft starter"), "{texts:?}");
    assert!(texts.iter().any(|t| t == "Library    "), "{texts:?}");
    let entry = h.text_at("Engrave").expect("an entry");
    h.click(entry);
    let apply = h.text_at("Apply to layer").expect("Apply to layer");
    h.click(apply);
    assert_eq!(h.log.len(), 1, "{:?}", h.log);
    assert!(
        h.log[0].starts_with("material.apply ") && h.log[0].ends_with(" 4"),
        "{:?}",
        h.log
    );
    // Following a material makes the editor read-only and shows the band.
    let entry = h.text_at("Engrave").expect("an entry");
    h.click(entry);
    let follow = h.text_at("Follow this entry").expect("Follow this entry");
    h.click(follow);
    assert!(h.session.project().layer(4).unwrap().follows.is_some());
    let layers = h.text_at("Layers").expect("Layers tab");
    h.click(layers);
    let texts = h.texts();
    assert!(
        texts
            .iter()
            .any(|t| t.starts_with("Follows material Engrave")),
        "{texts:?}"
    );
    let stop = h.text_at("Stop following").expect("Stop following");
    h.click(stop);
    assert!(h.session.project().layer(4).unwrap().follows.is_none());
}

fn menu(h: &mut Harness, menu: &str, row: &str) {
    // The menu bar's copy is the highest one.
    let at = h
        .drawn_texts()
        .into_iter()
        .filter(|(t, _)| t == menu)
        .map(|(_, r)| r.center())
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap_or_else(|| panic!("{menu} menu"));
    h.click(at);
    let at = h.text_at(row).unwrap_or_else(|| panic!("{row}"));
    h.click(at);
}

#[test]
fn device_profiles_are_added_from_the_dialog_and_chosen_in_the_machine_panel() {
    // A tall window shows the whole Machine panel.
    let mut h = Harness::with_height(&[], 1400.0);
    menu(&mut h, "Machine", "Device profiles…");
    let new = h.text_at("New profile…").expect("Device profiles dialog");
    h.click(new);
    assert!(h.texts().iter().any(|t| t == "New device profile"));
    let ok = h.text_at("OK").expect("OK");
    h.click(ok);
    assert!(
        h.log[0].starts_with("profile.add \"Laser 1\" width=400"),
        "{:?}",
        h.log
    );
    assert_eq!(h.session.workshop().profiles().len(), 1);
    let close = h.text_at("Close").expect("Close");
    h.click(close);
    // The Device dropdown lists it; choosing it names it in the project.
    let device = h.text_at("(none)").expect("Device dropdown");
    h.click(device);
    let row = h
        .drawn_texts()
        .into_iter()
        .rev()
        .find(|(t, _)| t.starts_with("Laser 1 "))
        .map(|(_, r)| r.center())
        .unwrap_or_else(|| panic!("profile row: {:?}", h.texts()));
    h.click(row);
    assert_eq!(h.session.project().device(), Some("Laser 1"));
}

#[test]
fn the_speed_and_power_window_starts_the_pattern_it_was_told_is_valid() {
    let mut h = Harness::new(&[]);
    menu(&mut h, "Machine", "Speed and power test…");
    let requests = h.shell.test_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].kind, lasercraft_ui::TestKind::SpeedPower);
    assert!(h.shell.test_requests().is_empty(), "nothing changed");
    // Until the app answers, the job buttons stay off.
    connected(&mut h);
    let start = h.text_at("Start").expect("Start");
    h.click(start);
    assert!(
        h.log.iter().all(|l| !l.starts_with("device.test")),
        "{:?}",
        h.log
    );
    let full = serde_json::json!({"rows": {"count": 3}});
    h.shell.set_test_info(
        lasercraft_ui::TestKind::SpeedPower,
        lasercraft_ui::TestInfo {
            revision: requests[0].revision,
            settings: Some(full.clone()),
            size: Some((40.0, 30.0)),
            ..lasercraft_ui::TestInfo::default()
        },
    );
    assert!(h.texts().iter().any(|t| t == "Pattern size 40.0 × 30.0 mm"));
    // The window's Start is the topmost copy.
    let start = h.text_at("Start").expect("Start");
    h.click(start);
    assert_eq!(
        h.log.last().unwrap(),
        &format!(
            "device.test speed_power {} from=absolute anchor=bottom-left",
            lasercraft_commands::Invocation::new("x", [full.to_string()])
                .to_string()
                .trim_start_matches("x ")
        )
    );
}

#[test]
fn the_rotary_dialog_sends_one_change_and_shows_as_an_active_setup() {
    let mut h = Harness::new(&["shape.add_rect 10 10 20 20"]);
    h.key(Key::R, Modifiers::COMMAND | Modifiers::SHIFT);
    assert!(h.texts().iter().any(|t| t == "Rotary attachment"));
    let use_rotary = h.text_at("Use rotary").expect("Use rotary");
    h.click(use_rotary);
    let ok = h.text_at("OK").expect("OK");
    h.click(ok);
    assert_eq!(h.log, ["rotary.set enabled on"]);
    assert!(h.session.project().rotary_on());
    assert!(
        h.texts().iter().any(|t| t == "Rotary"),
        "status bar names it"
    );
    // Taper correction opens from the Machine menu now that rotary is on.
    menu(&mut h, "Machine", "Taper correction…");
    assert!(h.texts().iter().any(|t| t == "Taper correction"));
    let ok = h.text_at("OK").expect("OK");
    h.click(ok);
    assert!(
        h.log
            .last()
            .unwrap()
            .starts_with("shape.taper top 80 100 64"),
        "{:?}",
        h.log
    );
}

#[test]
fn print_registration_lists_what_is_missing_and_shows_refusals() {
    let mut h = Harness::new(&["shape.add_rect 10 10 20 20"]);
    h.key(Key::Num0, Modifiers::ALT);
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "Print registration"), "{texts:?}");
    assert!(
        texts
            .iter()
            .any(|t| t.contains("Connect the machine first.")),
        "{texts:?}"
    );
    h.shell.command_failed(
        &lasercraft_commands::Invocation::parse("registration.set_target 2").unwrap(),
        "invalid argument for `registration.set_target`: Set target 1 first.",
    );
    // A new window is sized on its first frame and drawn on the next.
    h.frame(Vec::new(), Modifiers::NONE);
    let texts = h.texts();
    assert!(texts.iter().any(|t| t == "Registration failed"));
    assert!(
        texts.iter().any(|t| t == "Set target 1 first."),
        "{texts:?}"
    );
}
