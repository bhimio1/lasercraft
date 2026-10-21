//! Argument parsing shared by the built-in commands.

use std::str::FromStr;

use lasercraft_geom::Point;
use lasercraft_project::Project;

use crate::CommandError;

/// Positional arguments of one invocation, with helpers that produce consistent
/// errors.
pub(crate) struct Args<'a> {
    name: &'static str,
    usage: &'static str,
    items: &'a [String],
    /// Trailing `@<color>` selector, if the usage allows one and it was given.
    color: Option<&'a str>,
}

impl<'a> Args<'a> {
    pub fn new(name: &'static str, usage: &'static str, items: &'a [String]) -> Self {
        let (items, color) = match items.split_last() {
            Some((last, rest)) if usage.contains("[@color]") && last.starts_with('@') => {
                (rest, Some(&last[1..]))
            }
            _ => (items, None),
        };
        Self {
            name,
            usage,
            items,
            color,
        }
    }

    /// The `@<color>` argument, or color 0 when none was given.
    pub fn color_or_default(&self) -> Result<u8, CommandError> {
        self.color
            .map_or(Ok(0), |c| Ok(lasercraft_project::layer::parse_color(c)?))
    }

    /// Argument `i` as a palette color.
    pub fn color_at(&self, i: usize) -> Result<u8, CommandError> {
        Ok(lasercraft_project::layer::parse_color(self.str(i)?)?)
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

impl Args<'_> {
    /// Shape ids from argument `from` on, or the selection when there are
    /// none. Ids are checked, deduplicated and returned in drawing order.
    pub fn ids(&self, from: usize, project: &Project) -> Result<Vec<u32>, CommandError> {
        let given = self.items.get(from..).unwrap_or_default();
        if given.is_empty() {
            let selection = project.selection();
            if selection.is_empty() {
                return Err(self.invalid("nothing selected"));
            }
            return Ok(selection);
        }
        self.parse_ids(given, project)
    }

    /// Shape ids from argument `from` on, at least one, all existing.
    pub fn required_ids(&self, from: usize, project: &Project) -> Result<Vec<u32>, CommandError> {
        let given = self.items.get(from..).unwrap_or_default();
        if given.is_empty() {
            return Err(self.usage_error());
        }
        self.parse_ids(given, project)
    }

    fn parse_ids(&self, given: &[String], project: &Project) -> Result<Vec<u32>, CommandError> {
        let mut wanted = std::collections::HashSet::with_capacity(given.len());
        for raw in given {
            let id: u32 = raw
                .parse()
                .map_err(|_| self.invalid(format!("`{raw}` is not a shape id")))?;
            if project.shape(id).is_none() {
                return Err(self.invalid(format!("shape {id} does not exist")));
            }
            wanted.insert(id);
        }
        Ok(project
            .order()
            .into_iter()
            .filter(|id| wanted.contains(id))
            .collect())
    }

    /// Parses a point from arguments `i` and `i + 1`.
    pub fn point(&self, i: usize) -> Result<Point, CommandError> {
        Ok(Point::new(self.f64(i, "x")?, self.f64(i + 1, "y")?))
    }
}
