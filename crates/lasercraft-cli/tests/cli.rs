//! Spawns the real `lasercraft-cli` binary.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lasercraft-cli"))
        .args(args)
        .output()
        .expect("spawn lasercraft-cli")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lasercraft-cli-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn path_str(p: &Path) -> &str {
    p.to_str().unwrap()
}

#[test]
fn new_run_and_show() {
    let dir = temp_dir("run");
    let file = dir.join("job.lcproj");
    let f = path_str(&file);

    let out = cli(&["new", f, "500", "300"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let out = cli(&[
        "run",
        f,
        "project.rename Name tags",
        "project.set_origin rear-right",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "renamed to Name tags\norigin rear-right\n");

    let out = cli(&["show", f]);
    assert_eq!(
        stdout(&out),
        "project \"Name tags\": work area 500 × 300 mm, origin rear-right, 0 shape(s)\n"
    );
    let json = stdout(&cli(&["show", f, "--json"]));
    assert!(json.contains("\"origin\": \"rear_right\""));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn failed_run_does_not_touch_the_file() {
    let dir = temp_dir("fail");
    let file = dir.join("job.lcproj");
    let f = path_str(&file);
    assert!(cli(&["new", f]).status.success());
    let before = std::fs::read_to_string(&file).unwrap();

    let out = cli(&[
        "run",
        f,
        "project.rename Changed",
        "project.set_work_area 0 0",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("work area width"), "{}", stderr(&out));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn example_script_matches_golden_output() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/m0-basics.lcs");
    let out = cli(&["script", script]);
    assert!(out.status.success(), "{}", stderr(&out));
    let golden = include_str!("golden/m0-basics.out");
    assert_eq!(stdout(&out), golden);
}

#[test]
fn m1_shapes_script_matches_golden_output() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/m1-shapes.lcs");
    let out = cli(&["script", script]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), include_str!("golden/m1-shapes.out"));
}

#[test]
fn m2_layers_script_matches_golden_output() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/m2-layers.lcs");
    let out = cli(&["script", script]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), include_str!("golden/m2-layers.out"));
}

#[test]
fn m3_design_script_matches_golden_output() {
    // The script names its fixtures relative to the repository root.
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let out = Command::new(env!("CARGO_BIN_EXE_lasercraft-cli"))
        .args(["script", "examples/m3-design.lcs"])
        .current_dir(root)
        .output()
        .expect("spawn lasercraft-cli");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), include_str!("golden/m3-design.out"));
}

#[test]
fn imports_through_run_and_show_groups() {
    let dir = temp_dir("import");
    let file = dir.join("plate.lcproj");
    let f = path_str(&file);
    assert!(cli(&["new", f]).status.success());
    let dxf = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/fixtures/m3-plate.dxf"
    );
    let out = cli(&["run", f, &format!("project.import \"{dxf}\" @3")]);
    assert!(out.status.success(), "{}", stderr(&out));
    let shown = stdout(&cli(&["show", f]));
    assert!(shown.contains("4 shape(s)"), "{shown}");
    assert!(shown.contains("@3  path"), "{shown}");
    assert!(shown.contains("group 5"), "{shown}");
    let out = cli(&["run", f, "project.import /nonexistent/x.dxf"]);
    assert_eq!(out.status.code(), Some(1));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn script_errors_name_the_line() {
    let dir = temp_dir("script");
    let script = dir.join("bad.lcs");
    std::fs::write(
        &script,
        "# ok\nproject.rename A\nproject.set_origin middle\n",
    )
    .unwrap();
    let out = cli(&["script", path_str(&script)]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("bad.lcs:3:"), "{}", stderr(&out));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn script_can_save_and_load() {
    let dir = temp_dir("save");
    let file = dir.join("saved.lcproj");
    let script = dir.join("save.lcs");
    std::fs::write(
        &script,
        format!(
            "project.rename From script\nproject.save \"{}\"\n",
            path_str(&file)
        ),
    )
    .unwrap();
    assert!(cli(&["script", path_str(&script)]).status.success());
    let shown = stdout(&cli(&["show", path_str(&file)]));
    assert!(shown.contains("From script"));

    let other = dir.join("noop.lcs");
    std::fs::write(&other, "project.set_work_area 123 45\n").unwrap();
    let out = cli(&["script", path_str(&other), path_str(&file)]);
    assert!(stdout(&out).contains("\"From script\": work area 123 × 45 mm"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn bad_usage_exits_with_two() {
    let out = cli(&["export"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("usage:"));
}
