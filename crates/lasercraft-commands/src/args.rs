//! Argument parsing shared by the built-in commands.

use std::str::FromStr;

use crate::CommandError;

/// Positional arguments of one invocation, with helpers that produce consistent
/// errors.
pub(crate) struct Args<'a> {
    name: &'static str,
    usage: &'static str,
    items: &'a [String],
}

impl<'a> Args<'a> {
    pub fn new(name: &'static str, usage: &'static str, items: &'a [String]) -> Self {
        Self { name, usage, items }
    }

    /// A usage error for this command.
    pub fn usage_error(&self) -> CommandError {
        CommandError::Usage {
            command: self.name.to_owned(),
            usage: self.usage.to_owned(),
        }
    }

    /// An invalid-argument error for this command.
    pub fn invalid(&self, reason: impl Into<String>) -> CommandError {
        CommandError::InvalidArgument {
            command: self.name.to_owned(),
            reason: reason.into(),
        }
    }

    /// Requires between `min` and `max` arguments (inclusive).
    pub fn expect(&self, min: usize, max: usize) -> Result<(), CommandError> {
        if (min..=max).contains(&self.items.len()) {
            Ok(())
        } else {
            Err(self.usage_error())
        }
    }

    /// Number of arguments.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Argument `i`, which must exist.
    pub fn str(&self, i: usize) -> Result<&'a str, CommandError> {
        self.items
            .get(i)
            .map(String::as_str)
            .ok_or_else(|| self.usage_error())
    }

    /// Arguments from `i` on, joined by spaces (for names typed without quotes).
    pub fn rest(&self, i: usize) -> Result<String, CommandError> {
        let rest = self.items.get(i..).unwrap_or_default();
        if rest.is_empty() {
            return Err(self.usage_error());
        }
        Ok(rest.join(" "))
    }

    /// Parses argument `i`.
    pub fn parse<T: FromStr>(&self, i: usize, what: &str) -> Result<T, CommandError> {
        let raw = self.str(i)?;
        raw.parse()
            .map_err(|_| self.invalid(format!("`{raw}` is not a valid {what}")))
    }

    /// Parses a finite number.
    pub fn f64(&self, i: usize, what: &str) -> Result<f64, CommandError> {
        let value: f64 = self.parse(i, what)?;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(self.invalid(format!("{what} must be a finite number")))
        }
    }
}
