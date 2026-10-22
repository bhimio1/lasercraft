//! Built-in commands, one module per area.

mod clipboard;
mod layer;
mod project;
mod select;
mod shape;
// M3: import, text, groups, nodes, booleans, arrangement.
mod arrange;
mod group;
mod import;
mod node;
mod objects;
mod path;
// M5: G-code export.
mod export;
// M6: the laser.
pub(crate) mod device;
// M7: images.
mod image;
// M8: device profiles, material libraries, test patterns.
mod material;
mod profile;
mod test;

use crate::{BATCH, Command, Invocation};

/// Every built-in command.
pub(crate) fn all() -> Vec<Box<dyn Command>> {
    let mut all = project::commands();
    all.extend(shape::commands());
    all.extend(select::commands());
    all.extend(clipboard::commands());
    all.extend(layer::commands());
    // M3
    all.extend(import::commands());
    all.extend(group::commands());
    all.extend(node::commands());
    all.extend(path::commands());
    all.extend(arrange::commands());
    // M5
    all.extend(export::commands());
    // M6
    all.extend(device::commands());
    // M7
    all.extend(image::commands());
    // M8
    all.extend(profile::commands());
    all.extend(material::commands());
    all.extend(test::commands());
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
