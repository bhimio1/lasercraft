//! The parity checklist (`docs/spec/parity.md`): reading it, checking its
//! claims against the code, and the report.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

/// Where the documentation pages live.
pub const SITE: &str = "https://docs.lightburnsoftware.com/";

const STATUSES: [&str; 5] = ["done", "partial", "missing", "out_of_scope", "guide"];

/// The checklist file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checklist {
    format: String,
    version: u32,
    /// One entry per documentation page.
    pub pages: Vec<Page>,
}

/// One documentation page and what LaserCraft does about it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// Path under [`SITE`].
    pub page: String,
    /// What the page is about, in our words.
    pub feature: String,
    /// `done`, `partial`, `missing`, `out_of_scope` or `guide`.
    pub status: String,
    /// Our spec for it.
    #[serde(default)]
    pub spec: Option<String>,
    /// Commands that implement it.
    #[serde(default)]
    pub commands: Vec<String>,
    /// `ui.*` actions that open it.
    #[serde(default)]
    pub actions: Vec<String>,
    /// Default chords that must be bound.
    #[serde(default)]
    pub keys: Vec<String>,
    /// What is missing, or why it is out of scope.
    #[serde(default)]
    pub note: String,
}

/// What the code offers, for checking claims.
pub struct Facts {
    commands: BTreeSet<String>,
    actions: BTreeSet<String>,
    keys: BTreeSet<String>,
}

impl Facts {
    /// The built-in registry, the UI's actions and the default keymap.
    pub fn gather(root: &Path) -> Self {
        let registry = lasercraft_commands::Registry::with_builtin();
        let mut commands: BTreeSet<String> = registry.iter().map(|c| c.name().to_owned()).collect();
        commands.extend(
            lasercraft_commands::SESSION_COMMANDS
                .iter()
                .map(|(name, _, _)| (*name).to_owned()),
        );
        let keymap =
            std::fs::read_to_string(root.join("crates/lasercraft-ui/keymap/default.keymap"))
                .unwrap_or_default();
        Self {
            commands,
            actions: lasercraft_ui::UI_ACTIONS
                .iter()
                .map(|a| (*a).to_owned())
                .collect(),
            keys: chords(&keymap),
        }
    }
}

/// The chords a keymap binds, as written.
fn chords(keymap: &str) -> BTreeSet<String> {
    keymap
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('=').map(|(chord, _)| chord.trim().to_owned()))
        .collect()
}

/// Documentation pages linked from markdown text, as paths under [`SITE`];
/// site roots are not pages.
pub fn linked_pages(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (at, _) in text.match_indices(SITE) {
        let rest = &text[at + SITE.len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || "/._-".contains(c)))
            .unwrap_or(rest.len());
        // A root ("", "2.1") has no slash once trailing ones are gone.
        let page = rest[..end].trim_end_matches(['/', '.']);
        if page.contains('/') {
            out.insert(page.to_owned());
        }
    }
    out
}

impl Checklist {
    /// Reads the checklist.
    ///
    /// # Errors
    ///
    /// A missing file, bad JSON or an unknown format.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let list: Self =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if list.format != "lasercraft-parity" || list.version != 1 {
            return Err(format!(
                "{}: not a version 1 parity checklist",
                path.display()
            ));
        }
        Ok(list)
    }

    /// Problems with the entries' claims.
    pub fn check(&self, root: &Path, facts: &Facts) -> Vec<String> {
        let mut problems = Vec::new();
        let mut seen = BTreeSet::new();
        for p in &self.pages {
            let at = &p.page;
            if !seen.insert(at.clone()) {
                problems.push(format!("{at}: listed twice"));
            }
            if !STATUSES.contains(&p.status.as_str()) {
                problems.push(format!("{at}: unknown status `{}`", p.status));
            }
            if p.feature.trim().is_empty() {
                problems.push(format!("{at}: no feature description"));
            }
            match &p.spec {
                Some(spec) if !root.join(spec).is_file() => {
                    problems.push(format!("{at}: spec {spec} does not exist"));
                }
                None if matches!(p.status.as_str(), "done" | "partial") => {
                    problems.push(format!("{at}: `{}` needs a spec", p.status));
                }
                _ => {}
            }
            if matches!(p.status.as_str(), "partial" | "out_of_scope") && p.note.trim().is_empty() {
                problems.push(format!("{at}: `{}` needs a note", p.status));
            }
            for c in &p.commands {
                if !facts.commands.contains(c) {
                    problems.push(format!("{at}: no command `{c}`"));
                }
            }
            for a in &p.actions {
                if !facts.actions.contains(a) && !facts.commands.contains(a) {
                    problems.push(format!("{at}: no action `{a}`"));
                }
            }
            for k in &p.keys {
                if !facts.keys.contains(k) {
                    problems.push(format!("{at}: `{k}` is not bound in the default keymap"));
                }
            }
        }
        problems
    }

    /// Pages our specs link to that the checklist does not list.
    pub fn coverage(&self, root: &Path) -> Vec<String> {
        let listed: BTreeSet<&str> = self.pages.iter().map(|p| p.page.as_str()).collect();
        let mut cited: BTreeMap<String, String> = BTreeMap::new();
        let mut stack = vec![root.join("docs/spec")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();
            for path in paths {
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "md") {
                    let text = std::fs::read_to_string(&path).unwrap_or_default();
                    for page in linked_pages(&text) {
                        let shown = path.strip_prefix(root).unwrap_or(&path);
                        cited
                            .entry(page)
                            .or_insert_with(|| shown.display().to_string());
                    }
                }
            }
        }
        cited
            .into_iter()
            .filter(|(page, _)| !listed.contains(page.as_str()))
            .map(|(page, file)| {
                format!("{file} cites {SITE}{page}, which the checklist does not list")
            })
            .collect()
    }

    fn counts(&self) -> BTreeMap<&str, usize> {
        let mut counts: BTreeMap<&str, usize> = STATUSES.iter().map(|s| (*s, 0)).collect();
        for p in &self.pages {
            if let Some(n) = counts.get_mut(p.status.as_str()) {
                *n += 1;
            }
        }
        counts
    }

    /// `done` plus half of `partial`, over every page that is a feature in
    /// scope, as a percentage.
    pub fn share_done(&self) -> f64 {
        let c = self.counts();
        let scope = c["done"] + c["partial"] + c["missing"];
        if scope == 0 {
            return 100.0;
        }
        (c["done"] as f64 + c["partial"] as f64 / 2.0) * 100.0 / scope as f64
    }

    /// One line per status and the share done.
    pub fn summary(&self) -> String {
        let c = self.counts();
        format!(
            "{} pages: {} done, {} partial, {} missing, {} out of scope, {} guides; {:.0}% done",
            self.pages.len(),
            c["done"],
            c["partial"],
            c["missing"],
            c["out_of_scope"],
            c["guide"],
            self.share_done()
        )
    }

    /// The report: summary and a table.
    pub fn markdown(&self) -> String {
        let mut out = String::from(
            "# Parity report\n\nGenerated by `cargo xtask parity --markdown docs/parity.md` from \
             `docs/parity/checklist.json` (`docs/spec/parity.md`). Do not edit by hand.\n\n",
        );
        let _ = writeln!(out, "{}\n", self.summary());
        out.push_str("| Feature | Page | Status | Note |\n| --- | --- | --- | --- |\n");
        let mut pages: Vec<&Page> = self.pages.iter().collect();
        pages.sort_by(|a, b| a.page.cmp(&b.page));
        for p in pages {
            let cell = |s: &str| s.replace('|', "\\|");
            let _ = writeln!(
                out,
                "| {} | [{}]({SITE}{}{}) | {} | {} |",
                cell(&p.feature),
                p.page,
                p.page,
                // Pages end in a slash; files (the shortcut PDFs) do not.
                if p.page.ends_with(".pdf") { "" } else { "/" },
                p.status.replace('_', " "),
                cell(&p.note)
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_become_page_paths_and_roots_are_skipped() {
        let text = format!(
            "see {SITE}2.1/Reference/TraceImage/, {SITE}2.1/Reference/Cameras/Alignment/). \
             Only pages under {SITE}2.1/: and {SITE}legacy/Hotkeys or `{SITE}`."
        );
        let pages: Vec<String> = linked_pages(&text).into_iter().collect();
        assert_eq!(
            pages,
            [
                "2.1/Reference/Cameras/Alignment",
                "2.1/Reference/TraceImage",
                "legacy/Hotkeys"
            ]
        );
    }

    #[test]
    fn claims_are_checked() {
        let list: Checklist = serde_json::from_str(
            r#"{"format": "lasercraft-parity", "version": 1, "pages": [
              {"page": "2.1/A", "feature": "a", "status": "done", "spec": "docs/spec/parity.md",
               "commands": ["image.trace", "nope.nope"], "actions": ["ui.trace_image"], "keys": ["Alt+P"]},
              {"page": "2.1/A", "feature": "b", "status": "partial", "spec": "docs/spec/nope.md"},
              {"page": "2.1/C", "feature": "c", "status": "soon"}
            ]}"#,
        )
        .unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        let problems = list.check(root, &Facts::gather(root));
        let text = problems.join("\n");
        assert!(text.contains("no command `nope.nope`"), "{text}");
        assert!(!text.contains("image.trace`"), "{text}");
        assert!(text.contains("2.1/A: listed twice"), "{text}");
        assert!(
            text.contains("spec docs/spec/nope.md does not exist"),
            "{text}"
        );
        assert!(text.contains("`partial` needs a note"), "{text}");
        assert!(text.contains("unknown status `soon`"), "{text}");
        assert_eq!(problems.len(), 5, "{text}");
        assert!(list.markdown().contains("| a | [2.1/A]("));
    }
}
