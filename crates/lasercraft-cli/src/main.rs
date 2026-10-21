//! `lasercraft-cli`: see `lasercraft_cli::USAGE`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match lasercraft_cli::run(&args) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("lasercraft-cli: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}
