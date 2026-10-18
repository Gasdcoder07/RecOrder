//! Ventana Slint: Record/Stop, temporizador, tamaño en vivo, tema y carpeta de salida.
//! Todo corre en el hilo principal porque `SCStream` no es `Send`.
//!
//! El estado vive en un `RefCell` compartido. Ningún préstamo debe seguir activo
//! mientras se abre un diálogo: su bucle anidado dispara los timers.

use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use sr_capture::macos::{self, Recording};
use sr_platform::hotkey::{self, Hotkey};
use sr_platform::permissions;
use sr_platform::titlebar;
use sr_platform::tray::{Tray, TrayCommand};
use slint::{ComponentHandle, SharedString, Timer, TimerMode};

use crate::DEFAULT_MAX_WIDTH;

slint::include_modules!();

const FPS_OPTIONS: [u32; 2] = [30, 60];

struct Session {
    recording: Recording,
    path: PathBuf,
    started: Instant,
}

struct App {
    window: slint::Weak<MainWindow>,
    tray: Tray,
    hotkey: Hotkey,
    folder: PathBuf,
    session: Option<Session>,
}

impl App {
    fn ui(&self) -> MainWindow {
        self.window
            .upgrade()
            .expect("the window lives as long as the event loop")
    }
}

type Shared = Rc<RefCell<App>>;

pub fn run() -> Result<(), Box<dyn Error>> {
    let window = MainWindow::new()?;
    let folder = default_folder();
    window.set_folder_text(folder.display().to_string().into());
    window.set_status_text(format!("Ready · {} to record", hotkey::LABEL).into());

    let app: Shared = Rc::new(RefCell::new(App {
        window: window.as_weak(),
        tray: Tray::new()?,
        hotkey: Hotkey::new()?,
        folder,
        session: None,
    }));

    let shared = app.clone();
    window.on_toggle_recording(move || toggle(&mut shared.borrow_mut()));

    let shared = app.clone();
    window.on_change_folder(move || change_folder(&shared));

    window.on_open_settings(|| {
        if let Err(err) = permissions::open_screen_recording_settings() {
            eprintln!("error: could not open System Settings: {err}");
        }
    });

    let weak = window.as_weak();
    window.global::<Theme>().on_appearance_changed(move || {
        if let Some(ui) = weak.upgrade() {
            style_titlebar(&ui);
        }
    });

    // Temporizador y tamaño en vivo.
    let stats_timer = Timer::default();
    let shared = app.clone();
    stats_timer.start(TimerMode::Repeated, Duration::from_millis(250), move || {
        refresh_stats(&shared.borrow());
    });

    // Bandeja y atajo global entregan eventos por canales; se consultan aquí.
    let poll_timer = Timer::default();
    let shared = app.clone();
    poll_timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        poll_external(&shared);
    });

    // La app sigue viva sin ventana visible (bandeja + atajo); cerrar la ventana la oculta.
    window.show()?;
    style_titlebar(&window);
    slint::run_event_loop_until_quit()?;
    Ok(())
}

/// Alinea la barra de título con el tema elegido (Sistema, Claro u Oscuro).
fn style_titlebar(ui: &MainWindow) {
    let dark = match ui.global::<Theme>().get_mode() {
        1 => Some(false),
        2 => Some(true),
        _ => None,
    };
    let background = ui.global::<Theme>().get_background();
    let rgb = [background.red(), background.green(), background.blue()];
    titlebar::apply(&ui.window().window_handle(), dark, rgb);
}

fn poll_external(shared: &Shared) {
    let pressed = shared.borrow().hotkey.pressed();
    if pressed {
        toggle(&mut shared.borrow_mut());
    }

    let command = shared.borrow().tray.poll();
    match command {
        Some(TrayCommand::ShowWindow) => {
            let _ = shared.borrow().ui().show();
        }
        Some(TrayCommand::ToggleRecording) => toggle(&mut shared.borrow_mut()),
        Some(TrayCommand::Quit) => quit(&mut shared.borrow_mut()),
        None => {}
    }
}

fn toggle(app: &mut App) {
    if app.session.is_some() {
        stop(app);
    } else {
        start(app);
    }
}

fn start(app: &mut App) {
    let ui = app.ui();

    if !permissions::screen_recording_granted() {
        permissions::request_screen_recording();
        ui.set_show_settings_button(true);
        set_status(
            &ui,
            "Allow screen recording for screenrecorder, then restart it.",
            true,
        );
        return;
    }

    if let Err(err) = fs::create_dir_all(&app.folder) {
        set_status(&ui, format!("Cannot create {}: {err}", app.folder.display()), true);
        return;
    }

    let fps = FPS_OPTIONS[ui.get_fps_index().clamp(0, 1) as usize];
    let name = format!(
        "screenrecording-{}.mp4",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let path = app.folder.join(name);

    let microphone = ui.get_microphone();
    match macos::start_main_display(&path, fps, DEFAULT_MAX_WIDTH, microphone) {
        Ok(recording) => {
            app.session = Some(Session {
                recording,
                path,
                started: Instant::now(),
            });
            ui.set_show_settings_button(false);
            ui.set_recording(true);
            app.tray.set_recording(true);
            set_status(&ui, format!("Recording at {fps} fps"), false);
        }
        Err(err) => set_status(&ui, err.to_string(), true),
    }
}

fn stop(app: &mut App) {
    let Some(session) = app.session.take() else {
        return;
    };
    let ui = app.ui();
    let elapsed = session.started.elapsed();
    let name = file_name(&session.path);

    ui.set_recording(false);
    app.tray.set_recording(false);

    match session.recording.stop() {
        Ok(()) => {
            show_stats(&ui, elapsed, &session.path);
            set_status(&ui, format!("Saved {name}"), false);
        }
        Err(err) => set_status(&ui, format!("Could not finish {name}: {err}"), true),
    }
}

fn quit(app: &mut App) {
    if app.session.is_some() {
        stop(app);
    }
    let _ = slint::quit_event_loop();
}

fn change_folder(shared: &Shared) {
    let current = shared.borrow().folder.clone();
    let Some(folder) = rfd::FileDialog::new()
        .set_directory(&current)
        .pick_folder()
    else {
        return;
    };

    let mut app = shared.borrow_mut();
    app.ui().set_folder_text(folder.display().to_string().into());
    app.folder = folder;
}

fn refresh_stats(app: &App) {
    if let Some(session) = &app.session {
        show_stats(&app.ui(), session.started.elapsed(), &session.path);
    }
}

fn show_stats(ui: &MainWindow, elapsed: Duration, path: &Path) {
    let bytes = fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    ui.set_timer_text(format_elapsed(elapsed).into());
    ui.set_size_text(format_size(bytes).into());
}

fn set_status(ui: &MainWindow, text: impl Into<SharedString>, error: bool) {
    ui.set_status_text(text.into());
    ui.set_status_error(error);
}

fn default_folder() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("Movies").join("screenrecorder")
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn format_elapsed(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    format!("{:02}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
}

fn format_size(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
}
