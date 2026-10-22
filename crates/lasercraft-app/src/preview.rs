//! Plans the job and turns the plan into what the preview window draws
//! (`docs/spec/preview.md`). The UI never depends on the planner.

use lasercraft_plan::{Machine, MoveKind, Plan, num};
use lasercraft_project::Project;
use lasercraft_ui::{Preview, PreviewKind, PreviewMove, clock as duration};

/// The preview of `project` on the default machine (device profiles: M8).
pub fn build(project: &Project) -> Preview {
    match lasercraft_plan::plan(project, &Machine::default()) {
        Ok(plan) => from_plan(project, &plan),
        Err(e) => Preview {
            error: Some(e.to_string()),
            ..Preview::default()
        },
    }
}

fn from_plan(project: &Project, plan: &Plan) -> Preview {
    let mut elapsed = 0.0;
    let moves = plan
        .moves
        .iter()
        .map(|m| {
            let start = elapsed;
            elapsed += m.time;
            PreviewMove {
                from: m.from,
                to: m.to,
                kind: match m.kind {
                    MoveKind::Burn => PreviewKind::Burn(m.layer),
                    MoveKind::Unlit => PreviewKind::Unlit,
                    MoveKind::Travel => PreviewKind::Travel,
                },
                start,
                end: elapsed,
            }
        })
        .collect();
    let s = &plan.stats;
    let mut lines = vec![
        format!(
            "Burn {} mm ({}), unlit {} mm ({}), travel {} mm ({})",
            num(s.burn_length),
            duration(s.burn_time),
            num(s.unlit_length),
            duration(s.unlit_time),
            num(s.travel_length),
            duration(s.travel_time)
        ),
        format!(
            "Estimated time {} for {} moves",
            duration(s.total_time),
            plan.moves.len()
        ),
    ];
    if let Some(b) = s.bounds {
        lines.push(format!(
            "Job bounds ({}, {}) – ({}, {}) mm, {} × {} mm with overscan",
            num(b.min.x),
            num(b.min.y),
            num(b.max.x),
            num(b.max.y),
            num(b.width()),
            num(b.height())
        ));
    } else {
        lines.push("Nothing to cut: no output layer has shapes.".to_owned());
    }
    Preview {
        moves,
        total_time: s.total_time,
        lines,
        error: plan
            .check_bounds(project.work_area())
            .err()
            .map(|e| e.to_string()),
        warnings: plan.warnings.iter().map(ToString::to_string).collect(),
    }
}

#[cfg(test)]
mod tests {
    use lasercraft_commands::{Registry, Session};

    use super::*;

    fn project(lines: &[&str]) -> Project {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            s.run_line(line).unwrap();
        }
        s.project().clone()
    }

    #[test]
    fn moves_get_a_timeline_and_kinds() {
        let p = build(&project(&["shape.add_rect 10 10 20 20 @3"]));
        assert_eq!(p.moves.len(), 5);
        assert_eq!(p.moves[0].kind, PreviewKind::Travel);
        assert_eq!(p.moves[1].kind, PreviewKind::Burn(3));
        for pair in p.moves.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
        assert!((p.moves[4].end - p.total_time).abs() < 1e-9);
        assert!(p.error.is_none());
        assert!(p.lines[1].starts_with("Estimated time"));
    }

    #[test]
    fn out_of_bounds_jobs_carry_the_error() {
        let p = build(&project(&["shape.add_rect 390 10 20 20"]));
        let error = p.error.unwrap();
        assert!(error.contains("right edge"), "{error}");
        let empty = build(&Project::default());
        assert!(empty.moves.is_empty());
        assert!(empty.lines[2].starts_with("Nothing to cut"));
    }
}
