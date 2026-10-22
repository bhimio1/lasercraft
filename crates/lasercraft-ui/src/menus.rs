//! Menu bar rows as data (`docs/spec/style/menus.md`): their order,
//! separators, and when each row can run. The shell draws them; the
//! Machine menu, whose labels change with the machine state, is drawn by
//! the shell directly.

use lasercraft_project::{Geometry, Project};

/// What a row needs before it can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Need {
    /// Always available.
    Always,
    /// The project has shapes.
    Shapes,
    /// Something is selected.
    Selection,
    /// At least this many objects (a group counts as one) are selected.
    Objects(usize),
    /// Exactly two objects are selected.
    Pair,
    /// A selected object is a group.
    Group,
    /// The clipboard holds shapes.
    Clipboard,
    /// A selected shape is not a path yet.
    NotPath,
    /// At least this many open paths are selected.
    OpenPaths(usize),
    /// An image is selected.
    Image,
}

/// What the rows' needs are checked against, read from the project once
/// per frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Context {
    shapes: bool,
    selected: usize,
    objects: usize,
    group: bool,
    clipboard: bool,
    not_path: bool,
    open_paths: usize,
    image: bool,
}

impl Context {
    pub(crate) fn of(project: &Project) -> Self {
        let selected: Vec<_> = project
            .selection()
            .into_iter()
            .filter_map(|id| project.shape(id))
            .collect();
        let mut tops: Vec<(bool, u32)> = selected
            .iter()
            .map(|s| s.top_group().map_or((false, s.id()), |g| (true, g)))
            .collect();
        tops.sort_unstable();
        tops.dedup();
        let open = |g: &Geometry| match g {
            Geometry::Polyline { closed, .. } => !closed,
            Geometry::Path(path) => !path.is_closed(),
            _ => false,
        };
        Self {
            shapes: !project.shapes().is_empty(),
            selected: selected.len(),
            objects: tops.len(),
            group: tops.iter().any(|t| t.0),
            clipboard: !project.clipboard().is_empty(),
            not_path: selected.iter().any(|s| {
                matches!(
                    s.geometry(),
                    Geometry::Rect { .. } | Geometry::Ellipse { .. } | Geometry::Polyline { .. }
                )
            }),
            open_paths: selected.iter().filter(|s| open(s.geometry())).count(),
            image: selected.iter().any(|s| s.geometry().is_image()),
        }
    }
}

impl Need {
    pub(crate) fn met(self, cx: &Context) -> bool {
        match self {
            Self::Always => true,
            Self::Shapes => cx.shapes,
            Self::Selection => cx.selected > 0,
            Self::Objects(n) => cx.objects >= n,
            Self::Pair => cx.objects == 2,
            Self::Group => cx.group,
            Self::Clipboard => cx.clipboard,
            Self::NotPath => cx.not_path,
            Self::OpenPaths(n) => cx.open_paths >= n,
            Self::Image => cx.image,
        }
    }

    /// The tooltip of a disabled row: what is missing.
    pub(crate) fn reason(self) -> &'static str {
        match self {
            Self::Always => "",
            Self::Shapes => "Draw or import a shape first.",
            Self::Selection => "Select something first.",
            Self::Objects(2) => "Select two or more objects.",
            Self::Objects(_) => "Select three or more objects.",
            Self::Pair => "Select exactly two objects.",
            Self::Group => "Select a group.",
            Self::Clipboard => "Cut or copy something first.",
            Self::NotPath => "Select a shape that is not a path yet.",
            Self::OpenPaths(1) => "Select an open path.",
            Self::OpenPaths(_) => "Select two or more open paths.",
            Self::Image => "Select an image.",
        }
    }
}

/// One row of a menu.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Row {
    /// Runs `line` (a command or `ui.*` invocation); shows its bound chord.
    Item(&'static str, &'static str, Need),
    /// A drawing tool (`ui.tool ‹name›`), marked while active.
    Tool(&'static str, &'static str),
    /// A submenu.
    Sub(&'static str, Need, &'static [Row]),
    /// A separator.
    Sep,
}

use Need::{Always, Clipboard, Image, NotPath, Objects, OpenPaths, Pair, Selection, Shapes};
use Row::{Item, Sep, Sub, Tool};

/// File.
pub(crate) const FILE: &[Row] = &[
    Item("New", "ui.new", Always),
    Sep,
    Item("Open…", "ui.open", Always),
    Item("Import…", "ui.import", Always),
    Item("Project notes…", "ui.notes", Always),
    Sep,
    Item("Save", "ui.save", Always),
    Item("Save as…", "ui.save_as", Always),
    Item("Save G-code…", "ui.export_gcode", Always),
    Sep,
    Item("Quit", "ui.quit", Always),
];

/// Edit, below Undo and Redo (whose labels change).
pub(crate) const EDIT: &[Row] = &[
    Sep,
    Item("Select all", "select.all", Shapes),
    Item("Invert selection", "select.invert", Shapes),
    Item("Cut", "edit.cut", Selection),
    Item("Copy", "edit.copy", Selection),
    Item("Duplicate", "shape.duplicate", Selection),
    Item("Paste", "ui.paste", Clipboard),
    Item("Paste in place", "edit.paste_in_place", Clipboard),
    Item("Delete", "shape.delete", Selection),
    Sep,
    Item("Convert to path", "shape.to_path", NotPath),
    Item("Close path", "path.close", OpenPaths(1)),
    Item("Join paths", "path.join", OpenPaths(2)),
];

/// Tools.
pub(crate) const TOOLS: &[Row] = &[
    Tool("Select", "select"),
    Tool("Line", "line"),
    Sub("Shape tools", Always, SHAPE_TOOLS),
    Tool("Edit nodes", "nodes"),
    Tool("Text", "text"),
    Sep,
    Item("Offset…", "ui.offset", Selection),
    Item("Weld", "path.weld", Selection),
    Item("Union", "path.union", Pair),
    Item("Subtract", "path.subtract", Pair),
    Item("Intersect", "path.intersect", Pair),
    Sep,
    Item("Image adjustments…", "ui.adjust_image", Image),
];

const SHAPE_TOOLS: &[Row] = &[Tool("Rectangle", "rect"), Tool("Ellipse", "ellipse")];

/// Arrange.
pub(crate) const ARRANGE: &[Row] = &[
    Item("Group", "group.create", Objects(2)),
    Item("Ungroup", "group.ungroup", Need::Group),
    Sep,
    Item("Mirror horizontally", "shape.flip horizontal", Selection),
    Item("Mirror vertically", "shape.flip vertical", Selection),
    Item("Rotate 90° clockwise", "shape.rotate 90", Selection),
    Item(
        "Rotate 90° counter-clockwise",
        "shape.rotate -90",
        Selection,
    ),
    Sep,
    Sub("Align", Objects(2), ALIGN),
    Sub("Distribute", Objects(2), DISTRIBUTE),
    Sep,
    Sub("Move on bed", Selection, MOVE_ON_BED),
    Sep,
    Item("Grid array…", "ui.grid_array", Selection),
    Item("Circular array…", "ui.circular_array", Selection),
    Sep,
    Item("Break apart", "path.break_apart", Selection),
    Sep,
    Item("Bring forward", "shape.raise", Selection),
    Item("Send backward", "shape.lower", Selection),
    Item("Bring to front", "shape.to_front", Selection),
    Item("Send to back", "shape.to_back", Selection),
    Sep,
    Item("Select next", "select.next", Shapes),
    Item("Select previous", "select.previous", Shapes),
];

const ALIGN: &[Row] = &[
    Item("Left edges", "arrange.align left", Objects(2)),
    Item("Right edges", "arrange.align right", Objects(2)),
    Item("Top edges", "arrange.align top", Objects(2)),
    Item("Bottom edges", "arrange.align bottom", Objects(2)),
    Item("Vertical centers", "arrange.align center-x", Objects(2)),
    Item("Horizontal centers", "arrange.align center-y", Objects(2)),
    Item("Centers", "arrange.align center", Objects(2)),
];

const DISTRIBUTE: &[Row] = &[
    Item(
        "Horizontal spacing",
        "arrange.distribute h-spaced",
        Objects(3),
    ),
    Item(
        "Horizontal centers",
        "arrange.distribute h-centered",
        Objects(3),
    ),
    Item(
        "Vertical spacing",
        "arrange.distribute v-spaced",
        Objects(3),
    ),
    Item(
        "Vertical centers",
        "arrange.distribute v-centered",
        Objects(3),
    ),
    Sep,
    Item(
        "Pack horizontally",
        "arrange.move_together horizontal",
        Objects(2),
    ),
    Item(
        "Pack vertically",
        "arrange.move_together vertical",
        Objects(2),
    ),
];

const MOVE_ON_BED: &[Row] = &[
    Item("Center", "arrange.move_to_bed center", Selection),
    Item("Top left", "arrange.move_to_bed top-left", Selection),
    Item("Top right", "arrange.move_to_bed top-right", Selection),
    Item("Bottom left", "arrange.move_to_bed bottom-left", Selection),
    Item(
        "Bottom right",
        "arrange.move_to_bed bottom-right",
        Selection,
    ),
];

/// View.
pub(crate) const VIEW: &[Row] = &[
    Item("Job preview…", "ui.preview", Always),
    Item("Fit bed", "view.zoom_fit", Always),
    Item("Zoom in", "view.zoom_in", Always),
    Item("Zoom out", "view.zoom_out", Always),
];

/// Every invocation the menu tables run, submenus included.
#[cfg(test)]
fn lines(rows: &[Row], out: &mut Vec<String>) {
    for row in rows {
        match *row {
            Item(_, line, _) => out.push(line.to_owned()),
            Tool(_, name) => out.push(format!("ui.tool {name}")),
            Sub(_, _, rows) => lines(rows, out),
            Sep => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_parses_and_ui_names_are_known() {
        let mut all = Vec::new();
        for rows in [FILE, EDIT, TOOLS, ARRANGE, VIEW] {
            lines(rows, &mut all);
        }
        for line in &all {
            let inv = lasercraft_commands::Invocation::parse(line).expect(line);
            if inv.name.starts_with("ui.") {
                assert!(crate::UI_ACTIONS.contains(&inv.name.as_str()), "{line}");
            }
        }
    }

    #[test]
    fn separators_never_start_end_or_double() {
        for rows in [FILE, EDIT, TOOLS, ARRANGE, VIEW, DISTRIBUTE] {
            let seps: Vec<bool> = rows.iter().map(|r| matches!(r, Sep)).collect();
            assert!(!seps.last().unwrap());
            assert!(!seps.windows(2).any(|w| w[0] && w[1]));
        }
        for rows in [FILE, TOOLS, ARRANGE, VIEW] {
            assert!(!matches!(rows[0], Sep));
        }
    }

    #[test]
    fn labels_are_sentence_case() {
        let mut stack: Vec<&[Row]> = vec![FILE, EDIT, TOOLS, ARRANGE, VIEW];
        while let Some(rows) = stack.pop() {
            for row in rows {
                let label = match *row {
                    Item(l, ..) | Tool(l, _) => l,
                    Sub(l, _, rows) => {
                        stack.push(rows);
                        l
                    }
                    Sep => continue,
                };
                let rest = label.split_whitespace().skip(1);
                for word in rest {
                    let proper = ["G-code"].contains(&word.trim_end_matches('…'));
                    let first = word.chars().next().unwrap();
                    assert!(proper || !first.is_uppercase(), "{label}");
                }
            }
        }
    }

    #[test]
    fn needs_follow_the_selection() {
        let none = Context::default();
        assert!(Always.met(&none) && !Selection.met(&none) && !Shapes.met(&none));
        let two = Context {
            shapes: true,
            selected: 3,
            objects: 2,
            group: true,
            ..Context::default()
        };
        assert!(Pair.met(&two) && Objects(2).met(&two) && !Objects(3).met(&two));
        assert!(Need::Group.met(&two) && !Image.met(&two));
        assert_eq!(Objects(3).reason(), "Select three or more objects.");
    }
}
