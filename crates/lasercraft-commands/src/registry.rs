use std::collections::BTreeMap;

use lasercraft_project::Project;

use crate::{Command, CommandError, Invocation, Outcome, cmds};

/// The set of commands that can change a project.
#[derive(Default)]
pub struct Registry {
    commands: BTreeMap<&'static str, Box<dyn Command>>,
}

impl Registry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry holding every built-in command.
    #[must_use]
    pub fn with_builtin() -> Self {
        let mut registry = Self::new();
        for command in cmds::all() {
            registry.commands.insert(command.name(), command);
        }
        registry
    }

    /// Adds a command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::Duplicate`] if the name is already registered
    /// or reserved by [`crate::Session`].
    pub fn register(&mut self, command: Box<dyn Command>) -> Result<(), CommandError> {
        let name = command.name();
        let reserved = crate::SESSION_COMMANDS.iter().any(|(n, ..)| *n == name);
        if reserved || self.commands.contains_key(name) {
            return Err(CommandError::Duplicate(name.to_owned()));
        }
        self.commands.insert(name, command);
        Ok(())
    }

    /// Looks a command up by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands.get(name).map(|command| &**command)
    }

    /// Iterates over commands in name order.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Command> {
        self.commands.values().map(|command| &**command)
    }

    /// Number of registered commands.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether no commands are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Runs an invocation against a project.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::Unknown`] for unregistered names, otherwise any
    /// error from the command itself.
    pub fn execute(
        &self,
        project: &mut Project,
        invocation: &Invocation,
    ) -> Result<Outcome, CommandError> {
        let command = self
            .get(&invocation.name)
            .ok_or_else(|| CommandError::Unknown(invocation.name.clone()))?;
        command.execute(project, &invocation.args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_names_are_namespaced() {
        let registry = Registry::with_builtin();
        assert!(!registry.is_empty());
        for command in registry.iter() {
            let (area, action) = command.name().split_once('.').expect("area.action");
            assert!(!area.is_empty() && !action.is_empty());
            assert!(
                command.usage().starts_with(command.name()),
                "{} usage `{}`",
                command.name(),
                command.usage()
            );
            assert!(!command.summary().is_empty());
        }
    }

    #[test]
    fn unknown_command_is_reported() {
        let registry = Registry::with_builtin();
        let mut project = Project::default();
        let err = registry
            .execute(&mut project, &Invocation::bare("nope.nothing"))
            .unwrap_err();
        assert_eq!(err, CommandError::Unknown("nope.nothing".to_owned()));
    }

    #[test]
    fn duplicate_and_reserved_registrations_are_rejected() {
        let mut registry = Registry::with_builtin();
        let duplicate = cmds::all().remove(0);
        assert!(matches!(
            registry.register(duplicate),
            Err(CommandError::Duplicate(_))
        ));
        let reserved =
            crate::command::cmd("edit.undo", "edit.undo", "x", |_, _| Ok(Outcome::done("")));
        assert!(matches!(
            registry.register(reserved),
            Err(CommandError::Duplicate(_))
        ));
    }
}
