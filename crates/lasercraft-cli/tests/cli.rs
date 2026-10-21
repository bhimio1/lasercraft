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
fn m5_export_writes_the_golden_program_and_cli_export_matches() {
    let dir = temp_dir("export");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/m5-export.lcs");
    let out = Command::new(env!("CARGO_BIN_EXE_lasercraft-cli"))
        .args(["script", script])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    let gcode = std::fs::read_to_string(dir.join("m5-job.gcode")).unwrap();
    let golden_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/m5-export.gcode");
    if std::env::var_os("LASERCRAFT_UPDATE_GOLDEN").is_some() {
        std::fs::write(golden_path, &gcode).unwrap();
    }
    assert_eq!(
        gcode,
        std::fs::read_to_string(golden_path).unwrap_or_default()
    );
    // The export subcommand gives the same program.
    let again = dir.join("again.gcode");
    let job = dir.join("m5-job.lcproj");
    let out = cli(&[
        "export",
        path_str(&job),
        path_str(&again),
        "--s-max",
        "1000",
        "--finish",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).starts_with("exported "), "{}", stdout(&out));
    assert_eq!(std::fs::read_to_string(&again).unwrap(), gcode);
    // An off-bed job fails with exit code 1 and writes nothing.
    let out = cli(&["run", path_str(&job), "shape.add_rect 290 190 20 20 @1"]);
    assert!(out.status.success());
    let refused = dir.join("refused.gcode");
    let out = cli(&["export", path_str(&job), path_str(&refused)]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("does not fit"), "{}", stderr(&out));
    assert!(!refused.exists());
    let out = cli(&["export", path_str(&job), path_str(&refused), "--bogus"]);
    assert_eq!(out.status.code(), Some(2));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn bad_usage_exits_with_two() {
    let out = cli(&["export"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("usage:"));
}

/// Runs the CLI in `dir`.
fn cli_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lasercraft-cli"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("spawn lasercraft-cli")
}

#[test]
fn m4_preview_matches_golden_output_and_writes_svg() {
    let dir = temp_dir("preview");
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/m4-planning.lcs"
    );
    let out = cli_in(&dir, &["script", script]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = cli_in(&dir, &["preview", "m4-planning.lcproj", "--svg", "m4.svg"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let golden_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/m4-preview.out");
    if std::env::var_os("LASERCRAFT_UPDATE_GOLDEN").is_some() {
        std::fs::write(golden_path, stdout(&out)).unwrap();
    }
    assert_eq!(stdout(&out), std::fs::read_to_string(golden_path).unwrap());
    let svg = std::fs::read_to_string(dir.join("m4.svg")).unwrap();
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(
        svg.contains("stroke-dasharray=\"1 1\""),
        "travel drawn dashed"
    );
    assert!(svg.contains("stroke=\"#f80\""), "overscan and tabs drawn");
    assert!(svg.trim_end().ends_with("</svg>"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn preview_lists_moves_takes_machine_flags_and_reports_bounds() {
    let dir = temp_dir("preview-moves");
    let file = dir.join("edge.lcproj");
    let f = path_str(&file);
    assert!(cli(&["new", f, "100", "50"]).status.success());
    let out = cli(&[
        "run",
        f,
        "shape.add_rect 1 10 20 10 @4",
        "layer.set 4 mode fill",
        "layer.set 4 interval 2",
        "layer.set 4 speed 100",
        "layer.set 4 overscan on",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = cli(&["preview", f, "--moves"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("\nburn   (1, 19) -> (21, 19) @4 p1 #1 F100 S20\n"),
        "{text}"
    );
    assert!(text.contains("overscan 5 mm"), "{text}");
    assert!(
        text.contains("does not fit: shape 1 on layer 4 goes 4.000 mm past the left edge"),
        "{text}"
    );
    // A gentler acceleration needs more run-on; a lower speed cap less.
    let text = stdout(&cli(&["preview", f, "--accel", "500", "--max-speed", "50"]));
    assert!(text.contains("overscan 2.5 mm"), "{text}");
    assert!(text.contains("does not fit"), "{text}");
    let text = stdout(&cli(&["preview", f, "--max-speed", "40"]));
    assert!(text.contains("overscan 0.8 mm"), "{text}");
    assert!(text.contains("fits the work area (100 × 50 mm)"), "{text}");
    let out = cli(&["preview", f, "--accel", "0"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("acceleration"), "{}", stderr(&out));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn m6_simulate_runs_device_commands_on_the_simulated_grbl() {
    let dir = temp_dir("simulate");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/m5-export.lcs");
    let out = cli_in(&dir, &["script", script]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = cli_in(
        &dir,
        &[
            "simulate",
            "m5-job.lcproj",
            "--locked",
            "--user-origin",
            "200,150",
            "device.start",
            "device.home",
            "device.move_to 150 100 speed=200",
            "device.jog up 10",
            "device.frame",
            "device.frame outline from=current anchor=center",
            "device.start from=current anchor=center",
            "device.start from=user anchor=top-right",
            "device.send $32",
            "device.connect simulator",
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let golden_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/m6-simulate.out");
    if std::env::var_os("LASERCRAFT_UPDATE_GOLDEN").is_some() {
        std::fs::write(golden_path, stdout(&out)).unwrap();
    }
    assert_eq!(
        stdout(&out),
        std::fs::read_to_string(golden_path).unwrap_or_default()
    );
    // Refusals fail the run and say why.
    let out = cli_in(
        &dir,
        &["simulate", "m5-job.lcproj", "device.start from=user"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("set an origin"), "{}", stderr(&out));
    let out = cli_in(&dir, &["simulate", "m5-job.lcproj"]);
    assert_eq!(out.status.code(), Some(2));
    let out = cli_in(
        &dir,
        &["simulate", "m5-job.lcproj", "--fast", "device.home"],
    );
    assert_eq!(out.status.code(), Some(2));
    std::fs::remove_dir_all(&dir).unwrap();
}
