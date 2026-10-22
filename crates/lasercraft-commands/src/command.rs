use lasercraft_device::Request;
use lasercraft_project::Project;
use lasercraft_workshop::Workshop;

use crate::{CommandError, Invocation, args::Args};

/// The result of a successful command.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// Human-readable description of what happened.
    pub message: String,
    /// An invocation that reverses this command, or `None` if the command is not
    /// part of undo history.
    pub inverse: Option<Invocation>,
    /// The project was replaced wholesale (new/load); undo history must be cleared.
    pub reset_history: bool,
    /// A request for the device thread (`device.*` commands); the front end
    /// delivers it (`docs/spec/laser.md`).
    pub device: Option<Request>,
}

impl Outcome {
    /// A successful outcome that is not recorded in undo history.
    pub fn done(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            inverse: None,
            reset_history: false,
            device: None,
        }
    }

    /// A successful outcome that asks the front end to send `request` to the
    /// device. Not undoable: the project does not change.
    pub fn device(message: impl Into<String>, request: Request) -> Self {
        Self {
            device: Some(request),
            ..Self::done(message)
        }
    }

    /// A successful outcome that can be undone by running `inverse`.
    pub fn undoable(message: impl Into<String>, inverse: Invocation) -> Self {
        Self {
            message: message.into(),
            inverse: Some(inverse),
            reset_history: false,
            device: None,
        }
    }

    /// A successful outcome after which undo history no longer applies.
    pub fn reset(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            inverse: None,
            reset_history: true,
            device: None,
        }
    }
}

/// What a command runs with: the project, and the user's workshop of
/// device profiles, material libraries and test presets
/// (`docs/spec/commands.md`, "User data commands").
pub struct Context<'a> {
    /// The project.
    pub project: &'a mut Project,
    /// The user's data outside the project.
    pub workshop: &'a mut Workshop,
}

/// A named action that changes a [`Project`] or the user's workshop.
pub trait Command: Send + Sync {
    /// Unique name in `area.action` form.
    fn name(&self) -> &'static str;

    /// One-line description.
    fn summary(&self) -> &'static str;

    /// Usage string, starting with the command name.
    fn usage(&self) -> &'static str;

    /// Runs the command.
    ///
    /// # Errors
    ///
    /// Returns a [`CommandError`] if the arguments are wrong or the project rejects
    /// the change. On error the project and the workshop must be left
    /// unchanged.
    fn execute(&self, cx: &mut Context<'_>, args: &[String]) -> Result<Outcome, CommandError>;
}

/// Signature of a table-driven command body that needs only the project.
pub(crate) type RunFn = fn(&mut Project, &Args<'_>) -> Result<Outcome, CommandError>;

/// Signature of a table-driven command body that needs the workshop too.
pub(crate) type ContextFn = fn(&mut Context<'_>, &Args<'_>) -> Result<Outcome, CommandError>;

/// A command body.
pub(crate) enum Body {
    /// Project only.
    Project(RunFn),
    /// Project and workshop.
    Context(ContextFn),
}

/// A command defined by a name, help text and a function.
pub(crate) struct FnCommand {
    pub name: &'static str,
    pub summary: &'static str,
    pub usage: &'static str,
    pub run: Body,
}

impl Command for FnCommand {
    fn name(&self) -> &'static str {
        self.name
    }

    fn summary(&self) -> &'static str {
        self.summary
    }

    fn usage(&self) -> &'static str {
        self.usage
    }

    fn execute(&self, cx: &mut Context<'_>, args: &[String]) -> Result<Outcome, CommandError> {
        let args = Args::new(self.name, self.usage, args);
        match self.run {
            Body::Project(run) => run(cx.project, &args),
            Body::Context(run) => run(cx, &args),
        }
    }
}

/// Builds a boxed [`FnCommand`].
pub(crate) fn cmd(
    name: &'static str,
    usage: &'static str,
    summary: &'static str,
    run: RunFn,
) -> Box<dyn Command> {
    Box::new(FnCommand {
        name,
        summary,
        usage,
        run: Body::Project(run),
    })
}

/// Builds a boxed [`FnCommand`] whose body needs the workshop.
pub(crate) fn ccmd(
    name: &'static str,
    usage: &'static str,
    summary: &'static str,
    run: ContextFn,
) -> Box<dyn Command> {
    Box::new(FnCommand {
        name,
        summary,
        usage,
        run: Body::Context(run),
    })
}
