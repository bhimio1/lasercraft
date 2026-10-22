//! Plans the job and turns the plan into what the preview window draws
//! (`docs/spec/preview.md`). The UI never depends on the planner.

use lasercraft_commands::job_settings;
use lasercraft_plan::patterns::{self, TestSettings};
use lasercraft_plan::{MoveKind, Plan, num};
use lasercraft_project::Project;
use lasercraft_ui::{Preview, PreviewKind, PreviewMove, TestInfo, TestRequest, clock as duration};
use lasercraft_workshop::Workshop;

/// The preview of `project` with its device profile's limits.
pub fn build(project: &Project, workshop: &Workshop) -> Preview {
    let (machine, _, _) = job_settings(project, workshop);
    match lasercraft_plan::plan(project, &machine) {
        Ok(plan) => {
            let mut preview = from_plan(project, &plan);
            if project.rotary_on() {
                let height = plan.stats.bounds.map_or(0.0, |b| b.height());
                preview.lines.insert(
                    2,
                    format!(
                        "Rotary: Y turns {}° for the design's height",
                        num((project.rotary().degrees(height) * 10.0).round() / 10.0)
                    ),
                );
                // Y is a rotation: only X is checked (docs/spec/rotary.md).
                preview.error = plan
                    .check_bounds_x(project.work_area())
                    .err()
                    .map(|e| e.to_string());
            }
            if project.registration_transform().is_some() {
                preview.lines.insert(
                    2,
                    "Print registration: the job is fitted to its targets".to_owned(),
                );
            }
            preview
        }
        Err(e) => Preview {
            error: Some(e.to_string()),
            ..Preview::default()
        },
    }
}

/// Checks and plans a test window's settings on `project`'s bed
/// (`docs/spec/test-patterns.md`, "Windows (app)").
pub fn pattern(request: &TestRequest, project: &Project, workshop: &Workshop) -> TestInfo {
    let mut info = TestInfo {
        revision: request.revision,
        ..TestInfo::default()
    };
    let parsed = match TestSettings::parse(request.kind.arg(), Some(&request.settings)) {
        Ok(parsed) => parsed,
        Err(e) => {
            info.error = Some(e.0);
            return info;
        }
    };
    info.settings = Some(parsed.to_value());
    let (machine, _, _) = job_settings(project, workshop);
    match patterns::build(&parsed, project.work_area(), &machine) {
        Ok(p) => {
            info.size = Some(p.size);
            info.error = p
                .plan
                .check_bounds(project.work_area())
                .err()
                .map(|e| e.to_string());
            info.warnings = p.plan.warnings.iter().map(|w| w.message.clone()).collect();
            if request.preview {
                info.preview = Some(from_plan(&p.project, &p.plan));
            }
        }
        Err(e) => info.error = Some(e.0),
    }
    info
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

    fn build(project: &Project) -> Preview {
        super::build(project, &Workshop::in_memory())
    }

    #[test]
    fn test_settings_are_checked_completed_and_planned() {
        let request = |settings: serde_json::Value, preview: bool| TestRequest {
            kind: lasercraft_ui::TestKind::Interval,
            settings,
            revision: 7,
            settled: true,
            preview,
        };
        let ws = Workshop::in_memory();
        let ok = pattern(
            &request(serde_json::json!({"patches": 3}), true),
            &Project::default(),
            &ws,
        );
        assert_eq!(ok.revision, 7);
        assert!(ok.error.is_none(), "{:?}", ok.error);
        assert_eq!(ok.settings.as_ref().unwrap()["patches"], 3);
        assert!(ok.size.unwrap().0 > 0.0);
        assert!(!ok.preview.unwrap().moves.is_empty());
        let bad = pattern(
            &request(serde_json::json!({"patches": 0}), false),
            &Project::default(),
            &ws,
        );
        assert!(bad.error.is_some() && bad.settings.is_none());
        let small = project(&["project.set_work_area 20 20"]);
        let outside = pattern(&request(serde_json::json!({}), false), &small, &ws);
        assert!(outside.error.is_some() && outside.settings.is_some());
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
