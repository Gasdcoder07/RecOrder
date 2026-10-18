//! Atajo global para iniciar o detener la grabación desde cualquier app.

use std::error::Error;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

/// Atajo tal como se muestra al usuario.
#[cfg(target_os = "macos")]
pub const LABEL: &str = "Cmd+Shift+R";
#[cfg(not(target_os = "macos"))]
pub const LABEL: &str = "Ctrl+Shift+R";

pub struct Hotkey {
    // El manager debe vivir mientras el atajo esté registrado.
    _manager: GlobalHotKeyManager,
    id: u32,
}

impl Hotkey {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        #[cfg(target_os = "macos")]
        let modifiers = Modifiers::META | Modifiers::SHIFT;
        #[cfg(not(target_os = "macos"))]
        let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;

        let manager = GlobalHotKeyManager::new()?;
        let hotkey = HotKey::new(Some(modifiers), Code::KeyR);
        manager.register(hotkey)?;
        Ok(Self {
            _manager: manager,
            id: hotkey.id(),
        })
    }

    /// Indica si el atajo se presionó desde la última consulta.
    pub fn pressed(&self) -> bool {
        let mut pressed = false;
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            pressed |= event.id == self.id && event.state == HotKeyState::Pressed;
        }
        pressed
    }
}
