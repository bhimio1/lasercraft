//! The LaserCraft desktop app: wires the command session to the egui shell.
//!
//! `lasercraft [project.lcproj] [--keymap <file>]`

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use eframe::egui;
use lasercraft_commands::{Invocation, Registry, Session, with_context};
use lasercraft_project::Project;
use lasercraft_ui::keymap::Keymap;
use lasercraft_ui::{Action, Shell, ShellView};
use lasercraft_workshop::Workshop;

mod camera;
mod laser;
mod preview;
mod scan;
mod trace;

struct Options {
    file: Option<PathBuf>,
    keymap: Option<PathBuf>,
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        file: None,
        keymap: None,
    };
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--keymap" {
            let path = args.next().ok_or("--keymap needs a file")?;
            options.keymap = Some(path.into());
        } else if options.file.is_none() {
            options.file = Some(arg.into());
        } else {
            return Err("usage: lasercraft [project.lcproj] [--keymap <file>]".to_owned());
        }
    }
    Ok(options)
}

fn load_keymap(path: Option<&Path>) -> Result<Keymap, String> {
    let Some(path) = path else {
        return Ok(Keymap::default_keymap());
    };
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Keymap::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

struct App {
    session: Session,
    shell: Shell,
    file: Option<PathBuf>,
    /// The document at the last save or load. Selection and clipboard do not
    /// count: project equality ignores them.
    clean: Project,
    /// An action waiting for "discard unsaved changes?".
    pending: Option<Action>,
    /// The user confirmed closing with unsaved changes.
    closing: bool,
    /// Session revision the open preview was planned from.
    previewed: Option<u64>,
    /// The laser connection (M6).
    laser: laser::Laser,
    /// Camera pictures and the overlay (M9).
    cameras: camera::Cameras,
    /// A bed scan with a head camera, while it runs.
    scan: Option<scan::Scan>,
}

impl App {
    fn new(keymap: Keymap, file: Option<PathBuf>, ctx: &egui::Context) -> Self {
        let (workshop, messages) = match lasercraft_workshop::config_dir() {
            Some(dir) => Workshop::load(&dir),
            None => (
                Workshop::in_memory(),
                vec!["No configuration folder was found: device profiles and libraries are not kept.".to_owned()],
            ),
        };
        let mut app = Self {
            session: Session::with_workshop(Project::default(), Registry::with_builtin(), workshop),
            shell: Shell::new(keymap),
            file: None,
            clean: Project::default(),
            pending: None,
            closing: false,
            previewed: None,
            laser: laser::Laser::new(ctx),
            cameras: camera::Cameras::new(ctx),
            scan: None,
        };
        for line in messages {
            app.shell.log(line);
        }
        // The first project takes the default device profile's work area.
        if app.run(&Invocation::bare("project.new")) {
            app.clean = app.session.project().clone();
        }
        if let Some(path) = file {
            app.open(path);
        }
        app
    }

    fn modified(&self) -> bool {
        self.session.project() != &self.clean
    }

    /// Runs an invocation, logging the outcome. Returns whether it succeeded.
    /// `device.*` commands get what the app knows about the machine filled
    /// in, and their request goes to the device thread.
    fn run(&mut self, invocation: &Invocation) -> bool {
        let invocation = &with_context(invocation, &self.laser.context());
        // Stop means stop: no further scan moves either.
        if matches!(
            invocation.name.as_str(),
            "device.stop" | "device.disconnect"
        ) && self.scan.take().is_some()
        {
            self.shell.log("Scan stopped.");
        }
        match self.session.run(invocation) {
            Ok(outcome) => {
                self.shell
                    .log(format!("> {invocation}\n{}", outcome.message));
                if let Some(warning) = self.session.take_workshop_warning() {
                    self.shell.log(format!("Warning: {warning}"));
                }
                if matches!(invocation.name.as_str(), "device.start" | "device.frame") {
                    self.laser.act(&lasercraft_ui::LaserAction::DismissRefused);
                }
                if let Some(request) = outcome.device {
                    self.watch_head(&request);
                    self.laser.send(request);
                }
                true
            }
            Err(err) => {
                self.shell.log(format!("> {invocation}\nerror: {err}"));
                self.shell.command_failed(invocation, &err.to_string());
                if matches!(invocation.name.as_str(), "device.start" | "device.frame") {
                    self.laser.refuse(format!("Cannot run: {err}"));
                }
                false
            }
        }
    }

    fn open(&mut self, path: PathBuf) {
        let invocation = Invocation::new("project.load", [path.display().to_string()]);
        if self.run(&invocation) {
            self.file = Some(path);
            self.clean = self.session.project().clone();
        }
    }

    fn save_to(&mut self, path: PathBuf) {
        let invocation = Invocation::new("project.save", [path.display().to_string()]);
        if self.run(&invocation) {
            self.file = Some(path);
            self.clean = self.session.project().clone();
        }
    }

    fn apply(&mut self, action: Action, ctx: &egui::Context) {
        let discards = match &action {
            Action::New | Action::Open(_) | Action::Quit => true,
            Action::Run(inv) => inv.name == "project.new" || inv.name == "project.load",
            Action::Save
            | Action::SaveAs(_)
            | Action::Preview
            | Action::Laser(_)
            | Action::Camera(_) => false,
        };
        if discards && self.modified() && self.pending.is_none() {
            self.pending = Some(action);
            return;
        }
        match action {
            Action::Run(invocation) => {
                // Console `project.new`/`project.load` replace the document too.
                if invocation.name == "project.load" && invocation.args.len() == 1 {
                    self.open(PathBuf::from(&invocation.args[0]));
                } else if self.run(&invocation) {
                    match invocation.name.as_str() {
                        "project.new" => {
                            self.file = None;
                            self.clean = self.session.project().clone();
                        }
                        "edit.copy" | "edit.cut" => self.share_clipboard(ctx),
                        _ => {}
                    }
                }
            }
            Action::New => {
                if self.run(&Invocation::bare("project.new")) {
                    self.file = None;
                    self.clean = self.session.project().clone();
                }
            }
            Action::Open(path) => self.open(path),
            Action::Save => match self.file.clone() {
                Some(path) => self.save_to(path),
                None => self.shell.prompt_save_as(),
            },
            Action::SaveAs(path) => self.save_to(path),
            Action::Quit => {
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Action::Preview => self.refresh_preview(true),
            Action::Laser(action) => {
                if let Some(line) = self.laser.act(&action) {
                    self.shell.log(line);
                }
            }
            Action::Camera(action) => self.camera_action(action),
        }
    }

    /// A program or move for the laser: the camera picture may go stale,
    /// and the simulated camera sees what the simulated laser burns.
    fn watch_head(&mut self, request: &lasercraft_device::Request) {
        use lasercraft_device::{Request, SIMULATOR_PORT, StreamKind};
        match request {
            Request::Stream { kind, gcode } => {
                self.cameras.head_moved();
                if *kind == StreamKind::Job
                    && self.laser.view().port.as_deref() == Some(SIMULATOR_PORT)
                {
                    self.cameras.simulated_burn(gcode);
                }
            }
            Request::Jog { .. } | Request::Home => self.cameras.head_moved(),
            _ => {}
        }
    }

    /// The project's device profile's cameras.
    fn profile_cameras(&self) -> Vec<lasercraft_workshop::Camera> {
        self.session
            .project()
            .device()
            .and_then(|d| self.session.workshop().cameras(d).ok())
            .map(<[_]>::to_vec)
            .unwrap_or_default()
    }

    fn camera_action(&mut self, action: lasercraft_ui::CameraAction) {
        use lasercraft_ui::CameraAction as A;
        let area = *self.session.project().work_area();
        let cams = self.profile_cameras();
        match action {
            A::Capture(region) => {
                if let Err(e) = self.cameras.capture(&cams, &area, region) {
                    self.shell.log(e);
                }
            }
            A::Rate(r) => self.cameras.set_rate(r),
            A::RefreshUsb => self.cameras.refresh_usb(),
            A::KeepPicture(name) => self.keep_picture(&name),
            A::DropPicture(i) => {
                if i < self.cameras.captures.len() {
                    self.cameras.captures.remove(i);
                }
            }
            A::ClearPictures => {
                self.cameras.captures.clear();
                self.cameras.fits.clear();
            }
            A::FitLens(name) => self.fit_lens(&name),
            A::FindMarks(name) => {
                self.cameras.marks = Some(self.find_marks(&cams, &name));
            }
            A::SimulatedSheet(place) => self.cameras.simulated_sheet(place),
            A::InsertPicture { camera, corrected } => {
                self.insert_picture(&cams, &area, &camera, corrected)
            }
            A::Trace { settings, color } => self.trace_camera(&settings, color),
            A::Scan(true) => self.start_scan(&cams, &area),
            A::Scan(false) => {
                if self.scan.take().is_some() {
                    self.shell.log("Scan stopped; the move under way finishes.");
                }
            }
        }
    }

    /// Starts a bed scan with the first used, aligned head camera.
    fn start_scan(
        &mut self,
        cams: &[lasercraft_workshop::Camera],
        area: &lasercraft_project::WorkArea,
    ) {
        use lasercraft_ui::MachineWord;
        let camera = cams.iter().find(|c| {
            c.used && c.mount == lasercraft_workshop::Mount::Head && c.alignment().is_some()
        });
        let Some(camera) = camera else {
            self.shell
                .log("Scan bed needs an aligned camera on the head.");
            return;
        };
        let laser = self.laser.view();
        if laser.port.is_none() || laser.word != MachineWord::Ready {
            self.shell
                .log("Connect the laser and wait until it is ready to scan.");
            return;
        }
        let Some(picture) = self.cameras.picture(&camera.name) else {
            self.shell
                .log("No picture yet: wait until the camera shows one.");
            return;
        };
        let lens = camera.lens_or_plain(picture.width(), picture.height());
        let stops = camera
            .alignment()
            .and_then(|a| lasercraft_camera::scan::footprint(a, &lens))
            .and_then(|f| lasercraft_camera::scan::stops(f, area.width_mm, area.height_mm));
        let Some(stops) = stops else {
            self.shell
                .log("The head camera sees too little of the bed to scan it.");
            return;
        };
        let scan = scan::Scan::new(
            camera.name.clone(),
            stops,
            camera.skip,
            std::time::Instant::now(),
        );
        let first = scan.first();
        self.scan = Some(scan);
        if let Some(p) = first {
            self.move_head(area, p);
        }
    }

    /// Moves the head, laser off, to a document point.
    fn move_head(&mut self, area: &lasercraft_project::WorkArea, p: lasercraft_geom::Point) {
        let m = area.to_machine(p);
        let n = |v: f64| lasercraft_plan::num((v * 1000.0).round() / 1000.0);
        if !self.run(&Invocation::new("device.move_to", [n(m.x), n(m.y)])) {
            self.scan = None;
        }
    }

    /// Advances a running scan.
    fn step_scan(
        &mut self,
        cams: &[lasercraft_workshop::Camera],
        area: &lasercraft_project::WorkArea,
        ctx: &egui::Context,
    ) {
        let Some(scan) = &mut self.scan else {
            return;
        };
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        let now = std::time::Instant::now();
        let head = self.cameras.head();
        let pictures = self.cameras.revision(&scan.camera);
        let mut step = scan.step(self.laser.view(), head, pictures, now);
        if step == scan::Step::Capture {
            let name = scan.camera.clone();
            let painted = cams
                .iter()
                .find(|c| c.name == name)
                .map(|c| self.cameras.capture_one(c, area));
            step = match painted {
                Some(Ok(())) => self
                    .scan
                    .as_mut()
                    .map_or(scan::Step::Wait, |s| s.captured(now)),
                Some(Err(e)) => scan::Step::Failed(format!("Scan stopped: {e}")),
                None => scan::Step::Failed("Scan stopped: the camera was removed.".to_owned()),
            };
        }
        match step {
            scan::Step::Wait | scan::Step::Capture => {}
            scan::Step::MoveTo(p) => self.move_head(area, p),
            scan::Step::Done(message) | scan::Step::Failed(message) => {
                self.scan = None;
                self.shell.log(message);
            }
        }
    }

    /// Traces the overlay picture onto the bed through
    /// `image.trace_picture`.
    fn trace_camera(&mut self, settings: &str, color: u8) {
        let Some(gray) = self
            .cameras
            .overlay()
            .as_deref()
            .and_then(|o| trace::overlay_gray(o).map(|g| (g, o.mm_per_px)))
        else {
            self.shell.log("Capture a camera picture first.");
            return;
        };
        let (gray, mm) = gray;
        let path = scratch("trace.png");
        if let Err(e) = std::fs::write(&path, lasercraft_project::raster::encode_png(&gray)) {
            self.shell
                .log(format!("error: cannot write the picture: {e}"));
            return;
        }
        let n = |v: f64| lasercraft_plan::num(v);
        self.run(&Invocation::new(
            "image.trace_picture",
            [
                path.display().to_string(),
                "0".into(),
                "0".into(),
                n(f64::from(gray.width()) * mm),
                n(f64::from(gray.height()) * mm),
                settings.to_owned(),
                format!("@{color}"),
            ],
        ));
    }

    /// Saves a camera's latest picture for the lens correction and looks
    /// for the sheet in it.
    fn keep_picture(&mut self, name: &str) {
        let Some(picture) = self.cameras.picture(name) else {
            self.shell
                .log("No picture yet: wait until the camera shows one.");
            return;
        };
        let n = self.cameras.captures.len() + 1;
        let path = scratch(&format!("lens-{n}.png"));
        if let Err(e) = std::fs::write(&path, picture.encode_png()) {
            self.shell
                .log(format!("error: cannot keep the picture: {e}"));
            return;
        }
        let found = lasercraft_camera::dots::find_sheet(&picture);
        self.cameras.captures.push(lasercraft_ui::LensCapture {
            path: path.display().to_string(),
            dots: found
                .as_ref()
                .map(|d| d.iter().map(|(_, p)| *p).collect())
                .unwrap_or_default(),
            error: found.err().map(|e| e.0),
        });
    }

    /// Fits every model to the kept pictures.
    fn fit_lens(&mut self, name: &str) {
        let mut views = Vec::new();
        let mut size = (1, 1);
        for c in self.cameras.captures.iter().filter(|c| c.error.is_none()) {
            let Ok(picture) = lasercraft_camera::Picture::load(std::path::Path::new(&c.path))
            else {
                continue;
            };
            size = (picture.width(), picture.height());
            if let Ok(dots) = lasercraft_camera::dots::find_sheet(&picture) {
                views.push(
                    dots.into_iter()
                        .map(|((i, j), p)| (lasercraft_camera::sheet::point(i, j), p))
                        .collect(),
                );
            }
        }
        self.cameras.fits = lasercraft_camera::lens::Model::ALL
            .iter()
            .map(|m| {
                lasercraft_camera::calibrate::calibrate(*m, size.0, size.1, &views)
                    .map(|mut lens| {
                        lens.created = lasercraft_camera::timestamp_now();
                        lens
                    })
                    .map_err(|e| e.0)
            })
            .collect();
        self.shell
            .log(format!("fitted the lens models for \"{name}\""));
    }

    fn find_marks(
        &self,
        cams: &[lasercraft_workshop::Camera],
        name: &str,
    ) -> Result<[lasercraft_geom::Point; 4], String> {
        let picture = self
            .cameras
            .picture(name)
            .ok_or("No picture yet: wait until the camera shows one.")?;
        let cam = cams
            .iter()
            .find(|c| c.name == name)
            .ok_or("the camera was removed")?;
        let lens = cam.lens_or_plain(picture.width(), picture.height());
        let scaled = picture.resized(lens.width, lens.height);
        lasercraft_camera::marks::find_marks(&scaled).map_err(|e| e.0)
    }

    /// Puts a camera picture on the canvas through `image.insert_picture`.
    fn insert_picture(
        &mut self,
        cams: &[lasercraft_workshop::Camera],
        area: &lasercraft_project::WorkArea,
        name: &str,
        corrected: bool,
    ) {
        let (Some(picture), Some(cam)) = (
            self.cameras.picture(name),
            cams.iter().find(|c| c.name == name),
        ) else {
            self.shell
                .log("No picture yet: wait until the camera shows one.");
            return;
        };
        let path = scratch("insert.png");
        let (bytes, w, h) = match (corrected, cam.alignment()) {
            (true, Some(alignment)) => {
                let lens = cam.lens_or_plain(picture.width(), picture.height());
                let mut o =
                    lasercraft_camera::overlay::Overlay::empty(area.width_mm, area.height_mm);
                o.paint(&picture, &lens, alignment, &cam.fit, None);
                let (w, h) = (
                    f64::from(o.width) * o.mm_per_px,
                    f64::from(o.height) * o.mm_per_px,
                );
                (o.encode_png(), w, h)
            }
            _ => (
                picture.encode_png(),
                f64::from(picture.width()) * 0.5,
                f64::from(picture.height()) * 0.5,
            ),
        };
        if let Err(e) = std::fs::write(&path, bytes) {
            self.shell
                .log(format!("error: cannot write the picture: {e}"));
            return;
        }
        let n = |v: f64| lasercraft_plan::num(v);
        self.run(&Invocation::new(
            "image.insert_picture",
            [
                path.display().to_string(),
                "0".into(),
                "0".into(),
                n(w),
                n(h),
            ],
        ));
    }

    /// Plans the job for the open preview window when it is new or the
    /// project changed since (`docs/spec/preview.md`).
    fn refresh_preview(&mut self, force: bool) {
        if !self.shell.preview_open() {
            self.previewed = None;
            return;
        }
        let revision = self.session.revision();
        if force || self.previewed != Some(revision) {
            let preview = preview::build(self.session.project(), self.session.workshop());
            self.shell.set_preview(preview);
            self.previewed = Some(revision);
        }
    }

    /// Answers the test windows: checks and plans their settings, and
    /// remembers settled valid ones.
    fn check_tests(&mut self) {
        for request in self.shell.test_requests() {
            let info = preview::pattern(&request, self.session.project(), self.session.workshop());
            if request.settled
                && let Some(full) = &info.settings
                && self.session.workshop().last_settings(request.kind.arg()) != Some(full)
            {
                let remember = Invocation::new(
                    "test.remember",
                    [request.kind.arg().to_owned(), full.to_string()],
                );
                if let Err(e) = self.session.run(&remember) {
                    self.shell.log(format!("error: {e}"));
                }
            }
            self.shell.set_test_info(request.kind, info);
        }
    }

    /// Puts copied shapes on the system clipboard as JSON. Besides being
    /// useful elsewhere, this makes the platform's paste chord reach us.
    fn share_clipboard(&self, ctx: &egui::Context) {
        if let Ok(json) = serde_json::to_string(self.session.project().clipboard()) {
            ctx.copy_text(json);
        }
    }

    fn confirm_discard(&mut self, ctx: &egui::Context) {
        let Some(action) = self.pending.clone() else {
            return;
        };
        let mut choice = None;
        egui::Modal::new(egui::Id::unique("discard")).show(ctx, |ui| {
            ui.heading("Unsaved changes");
            ui.label("The project has changes that are not saved. Discard them?");
            ui.horizontal(|ui| {
                if ui.button("Discard").clicked() {
                    choice = Some(true);
                }
                if ui.button("Cancel").clicked() {
                    choice = Some(false);
                }
            });
        });
        match choice {
            Some(true) => {
                self.apply(action, ctx);
                self.pending = None;
            }
            Some(false) => self.pending = None,
            None => {}
        }
    }

    fn title(&self) -> String {
        let marker = if self.modified() { "• " } else { "" };
        let project = self.session.project();
        match project.device() {
            Some(device) => format!("{marker}{} — {device} — LaserCraft", project.name()),
            None => format!("{marker}{} — LaserCraft", project.name()),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested()) && self.modified() && !self.closing {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Action::Quit);
        }
        for line in self.laser.poll() {
            self.shell.log(line);
        }
        if self.laser.connected() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        let cams = self.profile_cameras();
        let area = *self.session.project().work_area();
        let head = self
            .laser
            .view()
            .position
            .map(|[x, y]| area.to_document(lasercraft_geom::Point::new(x, y)));
        self.cameras.set_head(head);
        self.cameras.sync(
            &cams,
            &area,
            self.shell.cameras_live() || self.scan.is_some(),
        );
        for notice in self.cameras.poll() {
            self.shell.log(notice);
        }
        self.step_scan(&cams, &area, &ctx);
        let mut camera_view = self.cameras.view(&cams);
        camera_view.scan = self.scan.as_ref().map(scan::Scan::progress);
        let undo = self.session.undo_label().cloned();
        let view = ShellView {
            project: self.session.project(),
            file: self.file.as_deref(),
            modified: self.modified(),
            undo: undo.as_ref(),
            can_redo: self.session.redo_depth() > 0,
            laser: self.laser.view(),
            workshop: self.session.workshop(),
            camera: &camera_view,
        };
        let actions = self.shell.show(ui, &view);
        for action in actions {
            self.apply(action, &ctx);
        }
        self.refresh_preview(false);
        self.check_tests();
        if let Some(request) = self.shell.trace_request() {
            let overlay = self.cameras.overlay();
            let info = trace::answer(&request, self.session.project(), overlay.as_deref());
            self.shell.set_trace_info(info);
        }
        self.confirm_discard(&ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title()));
    }
}

fn main() -> ExitCode {
    let result = parse_options().and_then(|options| {
        let keymap = load_keymap(options.keymap.as_deref())?;
        let native = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("LaserCraft")
                .with_inner_size([1200.0, 800.0])
                .with_app_id("io.github.bhimio1.LaserCraft"),
            ..Default::default()
        };
        eframe::run_native(
            "LaserCraft",
            native,
            Box::new(move |cc| Ok(Box::new(App::new(keymap, options.file, &cc.egui_ctx)))),
        )
        .map_err(|e| e.to_string())
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("lasercraft: {err}");
            ExitCode::FAILURE
        }
    }
}

/// A file in this run's scratch folder (camera pictures for commands).
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lasercraft-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir.join(name)
}
