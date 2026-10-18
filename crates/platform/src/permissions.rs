//! Permiso de grabación de pantalla. En macOS es TCC; en otros sistemas no hay permiso
//! que pedir y siempre se considera concedido.

use std::io;

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

/// Indica si la app ya puede capturar la pantalla. No muestra ningún diálogo.
#[cfg(target_os = "macos")]
pub fn screen_recording_granted() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

#[cfg(not(target_os = "macos"))]
pub fn screen_recording_granted() -> bool {
    true
}

/// Muestra el diálogo del sistema para pedir el permiso. macOS suele exigir reiniciar
/// la app después de concederlo.
#[cfg(target_os = "macos")]
pub fn request_screen_recording() {
    unsafe {
        CGRequestScreenCaptureAccess();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_screen_recording() {}

/// Abre el panel de Privacidad → Grabación de pantalla de Ajustes del Sistema.
#[cfg(target_os = "macos")]
pub fn open_screen_recording_settings() -> io::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
        .status()
        .map(|_| ())
}

#[cfg(not(target_os = "macos"))]
pub fn open_screen_recording_settings() -> io::Result<()> {
    Ok(())
}
