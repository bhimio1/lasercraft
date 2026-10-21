//! The LaserCraft desktop app: wires the command session to the egui shell.
//!
//! `lasercraft [project.lcproj] [--keymap <file>]`

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use eframe::egui;
use lasercraft_commands::{Invocation, Registry, Session};
use lasercraft_project::Project;
use lasercraft_ui::keymap::Keymap;
use lasercraft_ui::{Action, Shell, ShellView};

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
}

impl App {
    fn new(keymap: Keymap, file: Option<PathBuf>) -> Self {
        let mut app = Self {
            session: Session::new(Project::default(), Registry::with_builtin()),
            shell: Shell::new(keymap),
            file: None,
            clean: Project::default(),
            pending: None,
            closing: false,
            previewed: None,
        };
        if let Some(path) = file {
            app.open(path);
        }
        app
    }

    fn modified(&self) -> bool {
        self.session.project() != &self.clean
    }

    /// Runs an invocation, logging the outcome. Returns whether it succeeded.
    fn run(&mut self, invocation: &Invocation) -> bool {
        match self.session.run(invocation) {
            Ok(outcome) => {
                self.shell
                    .log(format!("> {invocation}\n{}", outcome.message));
                true
            }
            Err(err) => {
                self.shell.log(format!("> {invocation}\nerror: {err}"));
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
            Action::Save | Action::SaveAs(_) | Action::Preview => false,
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
            let preview = preview::build(self.session.project());
            self.shell.set_preview(preview);
            self.previewed = Some(revision);
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
        format!("{marker}{} — LaserCraft", self.session.project().name())
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested()) && self.modified() && !self.closing {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Action::Quit);
        }
        let undo = self.session.undo_label().cloned();
        let view = ShellView {
            project: self.session.project(),
            file: self.file.as_deref(),
            modified: self.modified(),
            undo: undo.as_ref(),
            can_redo: self.session.redo_depth() > 0,
        };
        let actions = self.shell.show(ui, &view);
        for action in actions {
            self.apply(action, &ctx);
        }
        self.refresh_preview(false);
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
            Box::new(move |_cc| Ok(Box::new(App::new(keymap, options.file)))),
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
