//! Built-in commands, one module per area.

mod clipboard;
mod layer;
mod project;
mod select;
mod shape;

use crate::{BATCH, Command, Invocation};

/// Every built-in command.
pub(crate) fn all() -> Vec<Box<dyn Command>> {
    let mut all = project::commands();
    all.extend(shape::commands());
    all.extend(select::commands());
    all.extend(clipboard::commands());
    all.extend(layer::commands());
    all
}

/// One invocation that runs `inverses` in order: the single inverse itself,
/// or a batch.
pub(crate) fn combine(mut inverses: Vec<Invocation>) -> Invocation {
    if inverses.len() == 1 {
        inverses.remove(0)
    } else {
        Invocation::new(BATCH, inverses.iter().map(ToString::to_string))
    }
}

/// Formats ids as invocation arguments.
pub(crate) fn id_args(ids: &[u32]) -> impl Iterator<Item = String> + '_ {
    ids.iter().map(ToString::to_string)
}
