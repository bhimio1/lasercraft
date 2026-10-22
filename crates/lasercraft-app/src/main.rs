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

mod laser;
mod preview;

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
                    self.laser.send(request);
                }
                true
            }
            Err(err) => {
                self.shell.log(format!("> {invocation}\nerror: {err}"));
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
            Action::Save | Action::SaveAs(_) | Action::Preview | Action::Laser(_) => false,
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
        }
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
        let undo = self.session.undo_label().cloned();
        let view = ShellView {
            project: self.session.project(),
            file: self.file.as_deref(),
            modified: self.modified(),
            undo: undo.as_ref(),
            can_redo: self.session.redo_depth() > 0,
            laser: self.laser.view(),
            workshop: self.session.workshop(),
        };
        let actions = self.shell.show(ui, &view);
        for action in actions {
            self.apply(action, &ctx);
        }
        self.refresh_preview(false);
        self.check_tests();
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
