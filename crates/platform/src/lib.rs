//! Integración con el sistema operativo: atajo global, bandeja y permisos.

pub mod hotkey;
pub mod permissions;
#[cfg(target_os = "macos")]
pub mod titlebar;
pub mod tray;
