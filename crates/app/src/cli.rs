//! Grabación desde la terminal: `screenrecorder record <salida.mp4> [--fps 30|60] [--max-width <px>] [--no-mic]`.
//! Se detiene con Enter o Ctrl+C; Ctrl+C también cierra el MP4 antes de salir.

use std::process::ExitCode;

use crate::{DEFAULT_FPS, DEFAULT_MAX_WIDTH};

const USAGE: &str =
    "usage: screenrecorder record <output.mp4> [--fps 30|60] [--max-width <px>] [--no-mic]";

struct RecordOptions {
    fps: u32,
    max_width: u32,
    microphone: bool,
}

pub fn run(args: &[String]) -> ExitCode {
    match args {
        [output, flags @ ..] => match parse_flags(flags) {
            Ok(options) => record(output.into(), options),
            Err(err) => {
                eprintln!("error: {err}");
                eprintln!("{USAGE}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn parse_flags(flags: &[String]) -> Result<RecordOptions, String> {
    let mut options = RecordOptions {
        fps: DEFAULT_FPS,
        max_width: DEFAULT_MAX_WIDTH,
        microphone: true,
    };
    let mut iter = flags.iter();
    while let Some(flag) = iter.next() {
        if flag == "--no-mic" {
            options.microphone = false;
            continue;
        }
        let value = iter
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--fps" => {
                options.fps = value
                    .parse()
                    .map_err(|_| format!("invalid --fps: {value}"))?;
                if !matches!(options.fps, 30 | 60) {
                    return Err("--fps must be 30 or 60".into());
                }
            }
            "--max-width" => {
                options.max_width = value
                    .parse()
                    .map_err(|_| format!("invalid --max-width: {value}"))?;
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }
    Ok(options)
}

#[cfg(target_os = "macos")]
fn record(output: std::path::PathBuf, options: RecordOptions) -> ExitCode {
    match record_until_stopped(&output, &options) {
        Ok(()) => {
            println!("Saved {}", output.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "macos")]
fn record_until_stopped(
    output: &std::path::Path,
    options: &RecordOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io;
    use std::sync::mpsc;
    use std::thread;

    // Enter (en otra hebra, porque read_line bloquea) o Ctrl+C piden detener.
    // El manejador se instala antes de empezar para no dejar un MP4 sin cerrar.
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let ctrlc_tx = stop_tx.clone();
    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(());
    })?;
    thread::spawn(move || {
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
        let _ = stop_tx.send(());
    });

    let recording = sr_capture::macos::start_main_display(
        output,
        options.fps,
        options.max_width,
        options.microphone,
    )?;
    println!("Recording to {}. Press Enter or Ctrl+C to stop.", output.display());
    let _ = stop_rx.recv();
    recording.stop()
}

#[cfg(not(target_os = "macos"))]
fn record(_output: std::path::PathBuf, _options: RecordOptions) -> ExitCode {
    eprintln!("error: recording is only implemented for macOS in phase 2");
    ExitCode::FAILURE
}
