//! A JSON writer that keeps files short and diffable: arrays and objects that
//! fit on one line are written inline, larger ones one item per line.

use std::fmt::Write as _;

use serde_json::Value;

/// Longest inline value, in characters, before it is broken over lines.
const LINE: usize = 72;

/// Writes `value` with two-space indentation and a trailing newline.
#[must_use]
pub(crate) fn to_string(value: &Value) -> String {
    let mut out = String::new();
    write(&mut out, value, 0);
    out.push('\n');
    out
}

fn inline(value: &Value) -> String {
    match value {
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(inline).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Object(map) if map.is_empty() => "{}".to_owned(),
        Value::Object(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", Value::String(k.clone()), inline(v)))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
        scalar => scalar.to_string(),
    }
}

fn write(out: &mut String, value: &Value, depth: usize) {
    let flat = inline(value);
    let empty = matches!(value, Value::Array(a) if a.is_empty())
        || matches!(value, Value::Object(o) if o.is_empty());
    // The document itself is always broken over lines.
    if empty || (depth > 0 && flat.len() + depth * 2 <= LINE) {
        out.push_str(&flat);
        return;
    }
    let pad = "  ".repeat(depth + 1);
    match value {
        // Long runs of plain values are packed, several per line.
        Value::Array(items) if items.iter().all(|v| !v.is_array() && !v.is_object()) => {
            out.push_str("[\n");
            let mut line = String::new();
            for (i, item) in items.iter().enumerate() {
                let mut part = item.to_string();
                if i + 1 < items.len() {
                    part.push(',');
                }
                if !line.is_empty() && pad.len() + line.len() + 1 + part.len() > LINE {
                    let _ = writeln!(out, "{pad}{line}");
                    line.clear();
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(&part);
            }
            let _ = writeln!(out, "{pad}{line}");
            let _ = write!(out, "{}]", "  ".repeat(depth));
        }
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&pad);
                write(out, item, depth + 1);
                out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
            }
            let _ = write!(out, "{}]", "  ".repeat(depth));
        }
        Value::Object(map) => {
            out.push_str("{\n");
            for (i, (key, item)) in map.iter().enumerate() {
                let _ = write!(out, "{pad}{}: ", Value::String(key.clone()));
                write(out, item, depth + 1);
                out.push_str(if i + 1 < map.len() { ",\n" } else { "\n" });
            }
            let _ = write!(out, "{}}}", "  ".repeat(depth));
        }
        scalar => out.push_str(&scalar.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn short_values_stay_inline_and_long_ones_break() {
        let v = json!({
            "short": [1.0, 2.0],
            "point": { "x": 1, "y": 2 },
            "long": (0..40).collect::<Vec<i32>>(),
            "empty": [],
            "none": {}
        });
        let text = to_string(&v);
        assert!(text.contains("\"short\": [1.0, 2.0]"), "{text}");
        assert!(text.contains("\"point\": { \"x\": 1, \"y\": 2 }"), "{text}");
        assert!(text.contains("\"long\": [\n    0, 1, 2,"), "{text}");
        assert!(text.lines().all(|l| l.len() <= LINE + 4), "{text}");
        assert!(text.contains("\"empty\": []"), "{text}");
        assert!(text.contains("\"none\": {}"), "{text}");
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), v);
    }
}
