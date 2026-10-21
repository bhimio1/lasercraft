//! Golden tests: fixture project + machine → expected plan dump.
//!
//! Fixtures are made from the `.lcs` scripts beside them. To accept new
//! output after an intended change, run with `LASERCRAFT_UPDATE_GOLDEN=1`
//! and review the diff.

use std::path::PathBuf;

use lasercraft_plan::{Machine, plan};
use lasercraft_project::Project;

fn dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(sub)
}

fn check(fixture: &str, golden: &str, machine: &Machine) {
    let project = Project::load(&dir("fixtures").join(format!("{fixture}.lcproj"))).unwrap();
    let dump = plan(&project, machine).unwrap().dump();
    let path = dir("golden").join(format!("{golden}.plan"));
    if std::env::var_os("LASERCRAFT_UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &dump).unwrap();
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (set LASERCRAFT_UPDATE_GOLDEN=1)", path.display()));
    assert_eq!(
        dump, expected,
        "{golden}: plan differs from the golden file"
    );
}

#[test]
fn line_order_inner_first_nearest_neighbour_and_passes() {
    check("line-order", "line-order", &Machine::default());
}

#[test]
fn kerf_and_tabs() {
    check("kerf-tabs", "kerf-tabs", &Machine::default());
}

#[test]
fn fill_even_odd_overscan_one_way_and_crosshatch() {
    check("fill", "fill", &Machine::default());
}

#[test]
fn slow_machine_changes_times_and_overscan_only() {
    let slow = Machine {
        max_speed: 30.0,
        acceleration: 200.0,
        max_power: 50.0,
    };
    check("fill", "fill-slow-machine", &slow);
}
