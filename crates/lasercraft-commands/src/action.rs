//! Readable names of what a command did, for "Undo ‹action›"
//! (`docs/spec/commands.md`, "Action names"). Lower case; the menu adds
//! "Undo ".

use crate::Invocation;

/// What `invocation` did, in words: "move", "rotate 90° clockwise",
/// "change layer speed". Never a command id; unknown commands read
/// "changes".
#[must_use]
pub fn action_name(invocation: &Invocation) -> String {
    let arg = |i: usize| invocation.args.get(i).map_or("", String::as_str);
    let fixed = match invocation.name.as_str() {
        "shape.add_rect" => "add rectangle",
        "shape.add_ellipse" => "add ellipse",
        "shape.add_line" | "shape.add_polyline" => "add line",
        "shape.add_text" => "add text",
        "shape.insert" => "add shapes",
        "shape.delete" => "delete",
        "shape.duplicate" => "duplicate",
        "shape.move" => "move",
        "shape.scale" | "shape.scale_about" => "scale",
        "shape.rotate" => return rotate(arg(0)),
        "shape.rotate_about" => "rotate",
        "shape.flip" if arg(0) == "vertical" => "mirror vertically",
        "shape.flip" => "mirror horizontally",
        "shape.set_transform" => "transform",
        "shape.set_geometry" => "edit shape",
        "shape.set_layer" => "move to layer",
        "shape.raise" => "bring forward",
        "shape.lower" => "send backward",
        "shape.to_front" => "bring to front",
        "shape.to_back" => "send to back",
        "shape.set_order" => "change drawing order",
        "shape.to_path" => "convert to path",
        "path.weld" => "weld",
        "path.union" => "union",
        "path.subtract" => "subtract",
        "path.intersect" => "intersect",
        "path.offset" => "offset",
        "path.close" => "close path",
        "path.join" => "join paths",
        "path.break_apart" => "break apart",
        "node.move" => "move node",
        "node.set_handle" => "move curve handle",
        "node.insert" => "add node",
        "node.delete" => "delete node",
        "node.delete_segment" => "delete segment",
        "node.break" => "break at node",
        "node.to_line" => "make straight",
        "node.to_curve" => "make curve",
        "group.create" | "group.set" => "group",
        "group.ungroup" => "ungroup",
        "arrange.align" => return align(arg(0)),
        "arrange.distribute" => "distribute",
        "arrange.move_together" if arg(0) == "vertical" => "pack vertically",
        "arrange.move_together" => "pack horizontally",
        "arrange.move_to_bed" => "move on bed",
        "arrange.grid" => "grid array",
        "arrange.circular" => "circular array",
        "edit.cut" => "cut",
        "edit.paste" => "paste",
        "edit.paste_in_place" => "paste in place",
        "layer.set" => return layer_setting(arg(1)),
        "layer.set_all" => "layer settings",
        "layer.copy_settings" => "paste settings",
        "layer.move" => match arg(1) {
            "down" => "move layer down",
            "top" => "move layer to top",
            "bottom" => "move layer to bottom",
            _ => "move layer up",
        },
        "layer.set_order" => "reorder layers",
        "layer.delete" => "delete shapes",
        "image.adjust" => "image adjustments",
        "project.import" => "import",
        "project.rename" => "rename project",
        "project.set_work_area" => "change work area",
        "project.set_origin" => "change origin corner",
        "project.set_notes" => "edit project notes",
        "project.set_device" => "change device profile",
        "material.apply" => "apply material",
        "material.follow" => "follow material",
        "material.unfollow" => "stop following material",
        "material.update" | "material.set" | "material.describe" | "material.delete" => {
            "material change"
        }
        "edit.batch" => {
            return invocation
                .args
                .first()
                .and_then(|line| Invocation::parse(line).ok())
                .map_or_else(|| "changes".to_owned(), |first| action_name(&first));
        }
        _ => "changes",
    };
    fixed.to_owned()
}

fn rotate(degrees: &str) -> String {
    match degrees.parse::<f64>() {
        Ok(90.0) => "rotate 90° clockwise".to_owned(),
        Ok(-90.0) => "rotate 90° counter-clockwise".to_owned(),
        _ => "rotate".to_owned(),
    }
}

fn align(edge: &str) -> String {
    let what = match edge {
        "left" => "left edges",
        "right" => "right edges",
        "top" => "top edges",
        "bottom" => "bottom edges",
        "center-x" => "vertical centers",
        "center-y" => "horizontal centers",
        "center" => "centers",
        _ => return "align".to_owned(),
    };
    format!("align {what}")
}

fn layer_setting(key: &str) -> String {
    let what = match key {
        "name" => "name",
        "mode" => "mode",
        "speed" => "speed",
        "max_power" => "max power",
        "min_power" => "min power",
        "passes" => "passes",
        "interval" => "line interval",
        "angle" => "scan angle",
        "bidirectional" => "bidirectional fill",
        "overscan" => "overscan",
        "crosshatch" => "crosshatch",
        "output" => "output",
        "show" => "show",
        "air" => "air assist",
        "image.dither" => "dither",
        "image.interval" => "image line interval",
        "image.negative" => "negative image",
        "image.halftone_cells" => "halftone frequency",
        "image.halftone_angle" => "halftone angle",
        "image.dot_correction" => "dot width correction",
        "kerf" => "beam compensation",
        "tabs" => "tabs",
        "tab_width" => "tab width",
        "constant_power" => "constant power",
        _ => return "layer settings".to_owned(),
    };
    format!("change layer {what}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(line: &str) -> String {
        action_name(&Invocation::parse(line).unwrap())
    }

    #[test]
    fn names_read_like_the_menus() {
        assert_eq!(name("shape.move 1 2"), "move");
        assert_eq!(name("shape.rotate 90"), "rotate 90° clockwise");
        assert_eq!(name("shape.rotate -90"), "rotate 90° counter-clockwise");
        assert_eq!(name("shape.rotate 30"), "rotate");
        assert_eq!(name("shape.flip vertical"), "mirror vertically");
        assert_eq!(name("arrange.align center-x"), "align vertical centers");
        assert_eq!(name("layer.set 3 max_power 40"), "change layer max power");
        assert_eq!(name("layer.move 3 down"), "move layer down");
        assert_eq!(
            name("edit.batch \"shape.delete\" \"select.clear\""),
            "delete"
        );
        assert_eq!(name("edit.batch"), "changes");
        assert_eq!(name("made.up"), "changes");
    }
}
