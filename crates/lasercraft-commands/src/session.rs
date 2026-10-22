use lasercraft_project::Project;

use lasercraft_workshop::Workshop;

use crate::{CommandError, Context, Invocation, Outcome, Registry};

/// Reserved command name handled by [`Session`]: undo the last undoable command.
pub const UNDO: &str = "edit.undo";
/// Reserved command name handled by [`Session`]: redo the last undone command.
pub const REDO: &str = "edit.redo";
/// Reserved command name handled by [`Session`]: run each argument as a command,
/// all or nothing, as one undo step.
pub const BATCH: &str = "edit.batch";

/// Reserved names with their help text, for listings.
pub const SESSION_COMMANDS: [(&str, &str, &str); 3] = [
    (UNDO, "edit.undo", "Undo the last undoable command."),
    (REDO, "edit.redo", "Redo the last undone command."),
    (
        BATCH,
        "edit.batch <command> <command> ...",
        "Run several quoted commands as one undo step; all or nothing.",
    ),
];

#[derive(Debug, Clone)]
struct HistoryEntry {
    command: Invocation,
    inverse: Invocation,
    coalesce: Option<u64>,
    /// Identifier allocator position before the command first ran, so that a
    /// redo recreates the same identifiers that later history entries refer to.
    next_id: u32,
}

/// A project together with its command registry and undo/redo history.
pub struct Session {
    project: Project,
    workshop: Workshop,
    registry: Registry,
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    revision: u64,
}

impl Session {
    /// Creates a session with empty history.
    #[must_use]
    pub fn new(project: Project, registry: Registry) -> Self {
        Self::with_workshop(project, registry, Workshop::in_memory())
    }

    /// Creates a session with empty history and the given user data.
    #[must_use]
    pub fn with_workshop(project: Project, registry: Registry, workshop: Workshop) -> Self {
        Self {
            project,
            workshop,
            registry,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            revision: 0,
        }
    }

    /// The current project state.
    #[must_use]
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// The user's device profiles, libraries and presets.
    #[must_use]
    pub fn workshop(&self) -> &Workshop {
        &self.workshop
    }

    /// Takes the message about user data that could not be written, if the
    /// last change could not be saved.
    pub fn take_workshop_warning(&mut self) -> Option<String> {
        self.workshop.take_warning()
    }

    /// The registry in use.
    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Mutable access to the registry, for front ends that add commands.
    pub fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }

    /// Number of steps that can be undone.
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of steps that can be redone.
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.redo_stack.len()
    }

    /// The command that `edit.undo` would reverse.
    #[must_use]
    pub fn undo_label(&self) -> Option<&Invocation> {
        self.undo_stack.last().map(|e| &e.command)
    }

    /// Counter that increases whenever a command succeeds. Front ends use it to
    /// notice that the project may have changed.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Runs an invocation, recording undo history when the command is undoable.
    /// The reserved names [`UNDO`], [`REDO`] and [`BATCH`] are handled here.
    ///
    /// # Errors
    ///
    /// Returns any [`CommandError`] from the command; on error the project and
    /// history are unchanged.
    pub fn run(&mut self, invocation: &Invocation) -> Result<Outcome, CommandError> {
        self.run_inner(invocation, None)
    }

    /// Like [`Session::run`], but consecutive runs with the same `key` and the
    /// same command name merge into one undo step. Use one key per continuous
    /// gesture on one control (for example a fader drag).
    ///
    /// # Errors
    ///
    /// As [`Session::run`].
    pub fn run_coalesced(
        &mut self,
        invocation: &Invocation,
        key: u64,
    ) -> Result<Outcome, CommandError> {
        self.run_inner(invocation, Some(key))
    }

    /// Parses and runs one line.
    ///
    /// # Errors
    ///
    /// Returns parse errors and any error from [`Session::run`].
    pub fn run_line(&mut self, line: &str) -> Result<Outcome, CommandError> {
        self.run(&Invocation::parse(line)?)
    }

    fn run_inner(
        &mut self,
        invocation: &Invocation,
        coalesce: Option<u64>,
    ) -> Result<Outcome, CommandError> {
        let outcome = match invocation.name.as_str() {
            UNDO => self.undo()?,
            REDO => self.redo()?,
            _ => {
                let next_id = self.project.next_id();
                let outcome = self.execute(invocation)?;
                if outcome.reset_history {
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                } else if let Some(inverse) = &outcome.inverse {
                    self.record(invocation, inverse, coalesce, next_id);
                }
                outcome
            }
        };
        self.revision += 1;
        Ok(outcome)
    }

    fn record(
        &mut self,
        invocation: &Invocation,
        inverse: &Invocation,
        coalesce: Option<u64>,
        next_id: u32,
    ) {
        self.redo_stack.clear();
        if let (Some(key), Some(top)) = (coalesce, self.undo_stack.last_mut()) {
            // The gesture key identifies the control being dragged, so the
            // command name is enough to tell the gesture continues.
            if top.coalesce == Some(key) && top.command.name == invocation.name {
                top.command = invocation.clone();
                return;
            }
        }
        self.undo_stack.push(HistoryEntry {
            command: invocation.clone(),
            inverse: inverse.clone(),
            coalesce,
            next_id,
        });
    }

    /// Executes without touching history. Handles nested batches.
    fn execute(&mut self, invocation: &Invocation) -> Result<Outcome, CommandError> {
        if invocation.name == BATCH {
            self.execute_batch(&invocation.args)
        } else {
            let mut cx = Context {
                project: &mut self.project,
                workshop: &mut self.workshop,
            };
            self.registry.execute(&mut cx, invocation)
        }
    }

    fn execute_batch(&mut self, lines: &[String]) -> Result<Outcome, CommandError> {
        let mut inverses = Vec::new();
        let mut messages = Vec::new();
        for line in lines {
            let result = Invocation::parse(line).and_then(|inv| {
                if inv.name == UNDO || inv.name == REDO {
                    Err(CommandError::InvalidArgument {
                        command: BATCH.to_owned(),
                        reason: "undo and redo cannot be batched".to_owned(),
                    })
                } else {
                    self.execute(&inv)
                }
            });
            match result {
                Ok(outcome) if outcome.reset_history => {
                    return Err(CommandError::InvalidArgument {
                        command: BATCH.to_owned(),
                        reason: "commands that replace the project cannot be batched".to_owned(),
                    });
                }
                Ok(outcome) => {
                    messages.push(outcome.message);
                    inverses.extend(outcome.inverse);
                }
                Err(err) => {
                    // Roll back what already ran, newest first.
                    for inverse in inverses.iter().rev() {
                        let _ = self.execute(inverse);
                    }
                    return Err(err);
                }
            }
        }
        let message = messages.join("; ");
        if inverses.is_empty() {
            return Ok(Outcome::done(message));
        }
        let inverse = if inverses.len() == 1 {
            inverses.remove(0)
        } else {
            Invocation::new(BATCH, inverses.iter().rev().map(ToString::to_string))
        };
        Ok(Outcome::undoable(message, inverse))
    }

    /// Undoes the most recent undoable command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::NothingToUndo`] if history is empty.
    pub fn undo(&mut self) -> Result<Outcome, CommandError> {
        let entry = self.undo_stack.pop().ok_or(CommandError::NothingToUndo)?;
        if let Err(err) = self.execute(&entry.inverse) {
            self.undo_stack.push(entry);
            return Err(err);
        }
        // Ids handed out by the undone command become free again, so the
        // project is exactly as it was before the command.
        self.project.set_next_id(entry.next_id);
        let message = format!("undid `{}`", entry.command);
        self.redo_stack.push(HistoryEntry {
            coalesce: None,
            ..entry
        });
        Ok(Outcome::done(message))
    }

    /// Redoes the most recently undone command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::NothingToRedo`] if there is nothing to redo.
    pub fn redo(&mut self) -> Result<Outcome, CommandError> {
        let entry = self.redo_stack.pop().ok_or(CommandError::NothingToRedo)?;
        let current_next_id = self.project.next_id();
        self.project.set_next_id(entry.next_id);
        if let Err(err) = self.execute(&entry.command) {
            self.project.set_next_id(current_next_id);
            self.redo_stack.push(entry);
            return Err(err);
        }
        let message = format!("redid `{}`", entry.command);
        self.undo_stack.push(entry);
        Ok(Outcome::done(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        Session::new(Project::default(), Registry::with_builtin())
    }

    fn run(session: &mut Session, line: &str) -> Result<Outcome, CommandError> {
        session.run(&Invocation::parse(line).unwrap())
    }

    fn width(session: &Session) -> f64 {
        session.project().work_area().width_mm
    }

    #[test]
    fn undo_and_redo_walk_history() {
        let mut s = session();
        run(&mut s, "project.set_work_area 500 400").unwrap();
        run(&mut s, "project.set_work_area 600 400").unwrap();
        assert_eq!(s.undo_depth(), 2);
        assert_eq!(s.undo_label().unwrap().args[0], "600");

        run(&mut s, "edit.undo").unwrap();
        assert_eq!(width(&s), 500.0);
        run(&mut s, "edit.undo").unwrap();
        assert_eq!(width(&s), 400.0);
        assert_eq!(run(&mut s, "edit.undo"), Err(CommandError::NothingToUndo));

        run(&mut s, "edit.redo").unwrap();
        assert_eq!(width(&s), 500.0);
        assert_eq!(s.redo_depth(), 1);
    }

    #[test]
    fn new_command_clears_redo_history() {
        let mut s = session();
        run(&mut s, "project.set_work_area 500 400").unwrap();
        run(&mut s, "edit.undo").unwrap();
        assert_eq!(s.redo_depth(), 1);
        run(&mut s, "project.set_work_area 300 400").unwrap();
        assert_eq!(s.redo_depth(), 0);
        assert_eq!(run(&mut s, "edit.redo"), Err(CommandError::NothingToRedo));
    }

    #[test]
    fn failed_command_leaves_history_untouched() {
        let mut s = session();
        run(&mut s, "project.set_work_area 500 400").unwrap();
        let revision = s.revision();
        assert!(run(&mut s, "project.set_work_area 50000 400").is_err());
        assert_eq!(s.undo_depth(), 1);
        assert_eq!(s.revision(), revision);
        assert_eq!(width(&s), 500.0);
    }

    #[test]
    fn batch_is_one_undo_step() {
        let mut s = session();
        run(
            &mut s,
            r#"edit.batch "project.set_work_area 300 200" "project.rename Demo""#,
        )
        .unwrap();
        assert_eq!(s.undo_depth(), 1);
        assert_eq!(s.project().name(), "Demo");
        run(&mut s, "edit.undo").unwrap();
        assert_eq!(s.project().name(), "Untitled");
        assert_eq!(width(&s), 400.0);
        run(&mut s, "edit.redo").unwrap();
        assert_eq!(width(&s), 300.0);
        assert_eq!(s.project().name(), "Demo");
    }

    #[test]
    fn failing_batch_rolls_back() {
        let mut s = session();
        let err = run(
            &mut s,
            r#"edit.batch "project.set_work_area 300 200" "project.set_work_area 0 0""#,
        );
        assert!(err.is_err());
        assert_eq!(width(&s), 400.0);
        assert_eq!(s.undo_depth(), 0);
        assert!(run(&mut s, r#"edit.batch "edit.undo""#).is_err());
        assert!(run(&mut s, r#"edit.batch "project.new""#).is_err());
        assert_eq!(s.undo_depth(), 0);
    }

    #[test]
    fn nested_batches_undo_as_one_step() {
        let mut s = session();
        run(
            &mut s,
            r#"edit.batch "edit.batch \"project.rename A\" \"project.rename B\"" "project.set_origin rear-left""#,
        )
        .unwrap();
        assert_eq!(s.project().name(), "B");
        run(&mut s, "edit.undo").unwrap();
        assert_eq!(s.project(), &Project::default());
    }

    #[test]
    fn coalesced_runs_merge_into_one_step() {
        let mut s = session();
        for w in [410, 420, 430] {
            s.run_coalesced(
                &Invocation::new("project.set_work_area", [w.to_string(), "400".to_owned()]),
                7,
            )
            .unwrap();
        }
        assert_eq!(s.undo_depth(), 1);
        s.run_coalesced(&Invocation::new("project.set_work_area", ["500", "400"]), 8)
            .unwrap();
        assert_eq!(s.undo_depth(), 2);
        run(&mut s, "edit.undo").unwrap();
        run(&mut s, "edit.undo").unwrap();
        assert_eq!(width(&s), 400.0);
        run(&mut s, "edit.redo").unwrap();
        assert_eq!(width(&s), 430.0);
    }

    #[test]
    fn project_new_clears_history() {
        let mut s = session();
        run(&mut s, "project.set_work_area 500 400").unwrap();
        run(&mut s, "project.new").unwrap();
        assert_eq!(s.undo_depth(), 0);
        assert_eq!(width(&s), 400.0);
    }

    /// Every undoable command, in an order where each line is valid.
    fn round_trip_script() -> Vec<String> {
        let mut settings = lasercraft_project::Layer::new(9);
        settings.passes = 3;
        let layer_json = Invocation::new(
            "layer.set_all",
            ["9".to_owned(), serde_json::to_string(&settings).unwrap()],
        );
        let order = Invocation::new("layer.set_order", (0..30).rev().map(|c: u8| c.to_string()));
        let inserted = Invocation::new(
            "shape.insert",
            [
                "0",
                r#"{"id":40,"geometry":{"kind":"rect","width":1.0,"height":2.0},"transform":[1.0,0.0,0.0,1.0,0.0,0.0]}"#,
            ],
        );
        let mut script = [
            "project.rename Round trip",
            r#"project.set_notes "cut at 80%""#,
            "project.set_work_area 610 305",
            "project.set_origin rear-right",
            "project.set_device Shop keep_area",
            // M8: the starter library is open in a fresh session.
            "material.apply 2 8",
            "material.follow 1 9",
            "material.unfollow 9",
            "material.follow 3 9",
            r#"edit.batch "project.rename Batched" "project.set_work_area 300 200""#,
            "shape.add_rect 0 0 20 10",
            "shape.add_ellipse 50 50 10 5",
            "shape.add_line 0 0 10 10",
            "shape.add_polyline closed 0 0 10 0 5 8",
            "shape.move 3 4 1 2",
            "shape.scale 2 0.5 1",
            "shape.scale_about 1.5 1.5 10 10 2 3",
            "shape.rotate 33 1 2",
            "shape.rotate_about -12 5 5 4",
            "shape.flip vertical 1 4",
            "shape.set_transform 3 1 0 0 1 7 7",
            "shape.duplicate 2",
            "shape.raise 1",
            "shape.lower 5",
            "shape.to_front 1",
            "shape.to_back 3",
            "shape.set_order 5 4 3 2 1",
            "shape.delete 2 4",
            &inserted.to_string(),
            "edit.cut 1",
            "edit.paste 100 100",
            "edit.paste_in_place",
            "shape.add_rect 10 10 5 5 @3",
            "shape.set_layer 7 3 5",
            "layer.set 7 mode fill",
            "layer.set 7 interval 300lpi",
            r#"layer.set 3 name "Score lines""#,
            "layer.copy_settings 7 3",
            &layer_json.to_string(),
            "layer.move 7 top",
            &order.to_string(),
            "layer.delete 3",
        ]
        .map(str::to_owned)
        .to_vec();
        script.extend(m3_round_trip_lines());
        // M7: an image, its adjustments and the image layer settings.
        let image = lasercraft_project::Geometry::Image(lasercraft_project::Image::new(
            3.0,
            2.0,
            lasercraft_project::raster::Gray::new(3, 2, vec![0, 60, 120, 180, 240, 255]).unwrap(),
        ));
        let image_shape = format!(
            r#"{{"id":500,"layer":4,"geometry":{},"transform":[1.0,0.0,0.0,1.0,5.0,5.0]}}"#,
            serde_json::to_string(&image).unwrap()
        );
        script.extend([
            Invocation::new("shape.insert", ["0".to_owned(), image_shape]).to_string(),
            "image.adjust 500 brightness=5 contrast=-10 gamma=1.25".to_owned(),
            "layer.set 4 image.dither halftone".to_owned(),
            "layer.set 4 image.halftone_cells 40".to_owned(),
            "layer.set 4 image.dot_correction 0.02".to_owned(),
            "shape.rotate 30 500".to_owned(),
        ]);
        // M9: rotary and print registration.
        let rect = |id: u32, x: f64| {
            format!(
                r#"{{"id":{id},"geometry":{{"kind":"rect","width":4.0,"height":4.0}},"transform":[1.0,0.0,0.0,1.0,{x},8.0]}}"#
            )
        };
        script.extend([
            Invocation::new("shape.insert", ["0".to_owned(), rect(501, 8.0)]).to_string(),
            Invocation::new("shape.insert", ["0".to_owned(), rect(502, 108.0)]).to_string(),
            "rotary.set enabled on kind=chuck object_diameter=70".to_owned(),
            "rotary.set mirror on".to_owned(),
            "rotary.clear".to_owned(),
            "registration.set_target 1 501 head=10,390".to_owned(),
            "registration.set_target 2 502 head=112,389".to_owned(),
            "registration.fit rigid".to_owned(),
            "registration.fit scaled".to_owned(),
            "registration.clear".to_owned(),
            "registration.restore 1 501 10,10 11,390".to_owned(),
            "shape.taper top 80 100 60 501 502".to_owned(),
        ]);
        script
    }

    /// M3 commands: groups, arrangement, arrays, nodes, booleans, import
    /// and text. Shape ids continue from the lines above (next id 44).
    fn m3_round_trip_lines() -> Vec<String> {
        let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/fixtures");
        let geometry = Invocation::new(
            "shape.set_geometry",
            ["45", r#"{"kind":"rect","width":5.0,"height":5.0}"#],
        );
        let import = Invocation::new("project.import", [format!("{fixtures}/m3-art.svg")]);
        let text = Invocation::new(
            "shape.add_text",
            [
                "10".to_owned(),
                "10".to_owned(),
                "5".to_owned(),
                format!("{fixtures}/lctest.ttf"),
                "HOLA".to_owned(),
            ],
        );
        let mut lines: Vec<String> = [
            "shape.add_rect 200 0 20 20",
            "shape.add_rect 210 10 20 20",
            "shape.add_ellipse 250 50 10 10",
            "shape.add_line 300 0 310 0",
            "shape.add_line 310 0 310 10",
            "group.create 44 45",
            "arrange.align left 44 46",
            "group.ungroup 44",
            "group.set 46 49",
            "arrange.distribute h-centered 44 45 46",
            "arrange.move_together vertical 44 45",
            "arrange.move_to_bed center 47 48",
            "arrange.grid 2 1 5 0 46",
            "arrange.circular 3 0 0 90 rotate 47",
            "shape.to_path 44",
        ]
        .map(str::to_owned)
        .to_vec();
        lines.push(geometry.to_string());
        lines.extend(
            [
                "node.move 44 0 0 1 1",
                "node.insert 44 0 0",
                "node.to_curve 44 0 0",
                "node.set_handle 44 0 0 1 3 3",
                "node.to_line 44 0 0",
                "node.delete 44 0 1",
                "node.break 44 0 0",
                "node.delete_segment 44 0 0",
                "path.join 47 48",
                "shape.add_polyline 0 0 10 0 10 10 0 0.2",
                "path.close 55",
                "path.break_apart 55",
                "shape.add_rect 0 300 10 10",
                "shape.add_rect 5 305 10 10",
                "path.union 60 61",
                "shape.add_rect 0 300 10 10",
                "shape.add_rect 5 305 10 10",
                "path.subtract 63 64",
                "shape.add_rect 0 300 10 10",
                "shape.add_rect 5 305 10 10",
                "path.intersect 66 67",
                "path.weld 62 65",
                "path.offset 1 round 69",
            ]
            .map(str::to_owned),
        );
        lines.push(import.to_string());
        lines.push(text.to_string());
        lines
    }

    /// Runs every undoable command, then undoes, redoes and undoes again,
    /// comparing the project with snapshots at every step.
    #[test]
    fn every_undoable_command_round_trips() {
        let script = round_trip_script();
        let mut s = session();
        let mut snapshots = vec![s.project().clone()];
        for line in &script {
            run(&mut s, line).unwrap_or_else(|e| panic!("{line}: {e}"));
            snapshots.push(s.project().clone());
        }
        assert_eq!(s.undo_depth(), script.len());
        for (expected, line) in snapshots.iter().rev().skip(1).zip(script.iter().rev()) {
            run(&mut s, "edit.undo").unwrap();
            assert_eq!(s.project(), expected, "undo of `{line}`");
        }
        for (expected, line) in snapshots.iter().skip(1).zip(&script) {
            run(&mut s, "edit.redo").unwrap();
            assert_eq!(s.project(), expected, "redo of `{line}`");
        }
        for expected in snapshots.iter().rev().skip(1) {
            run(&mut s, "edit.undo").unwrap();
            assert_eq!(s.project(), expected);
        }
    }

    #[test]
    fn every_undoable_command_has_an_action_name() {
        for line in round_trip_script() {
            let inv = Invocation::parse(&line).unwrap();
            let name = crate::action_name(&inv);
            assert!(
                name != "changes" && !name.contains('.') && !name.contains('_'),
                "`{line}` reads \"{name}\""
            );
        }
    }

    #[test]
    fn every_builtin_undoable_command_is_in_the_round_trip() {
        // Keep `round_trip_script` complete: every command is either covered
        // there or listed as not undoable here.
        let script = round_trip_script();
        let not_undoable = [
            "project.new",
            "project.load",
            "project.save",
            "edit.copy",
            "project.export_gcode",
            "layer.select",
            // A machine move (M9).
            "registration.expected",
        ];
        // Device commands never change the project (`docs/spec/laser.md`).
        for command in Registry::with_builtin().iter() {
            let name = command.name();
            let covered = script
                .iter()
                .any(|line| line.split_whitespace().next() == Some(name));
            assert!(
                covered
                    || not_undoable.contains(&name)
                    || name.starts_with("select.")
                    || name.starts_with("device.")
                    // User data (M8) is not undoable (`docs/spec/commands.md`).
                    || name.starts_with("profile.")
                    || name.starts_with("library.")
                    || name.starts_with("test.")
                    // Library edits; their project part is tested in
                    // `cmds::material`.
                    || matches!(
                        name,
                        "material.add"
                            | "material.update"
                            | "material.set"
                            | "material.describe"
                            | "material.duplicate"
                            | "material.delete"
                            | "material.list"
                    ),
                "add `{name}` to the round-trip script or list it as not undoable"
            );
        }
    }
}
