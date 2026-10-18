//! Punto de entrada. Sin argumentos abre la ventana con bandeja y atajo global;
//! `record <salida.mp4>` graba desde la terminal.

mod cli;
#[cfg(target_os = "macos")]
mod gui;

use std::process::ExitCode;

const USAGE: &str =
    "usage: screenrecorder [record <output.mp4> [--fps 30|60] [--max-width <px>] [--no-mic]]";
const DEFAULT_FPS: u32 = 60;
const DEFAULT_MAX_WIDTH: u32 = 1920;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => run_gui(),
        [cmd, rest @ ..] if cmd == "record" => cli::run(rest),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "macos")]
fn run_gui() -> ExitCode {
    match gui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn run_gui() -> ExitCode {
    eprintln!("error: the window is only implemented for macOS in phase 3");
    ExitCode::FAILURE
}
