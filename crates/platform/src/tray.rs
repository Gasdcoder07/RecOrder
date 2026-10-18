//! Icono de bandeja con menú: mostrar la ventana, iniciar o detener la grabación, salir.
//! El icono cambia a rojo mientras se graba.

use std::error::Error;

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub enum TrayCommand {
    ShowWindow,
    ToggleRecording,
    Quit,
}

pub struct Tray {
    icon: TrayIcon,
    idle_icon: Icon,
    recording_icon: Icon,
    toggle_item: MenuItem,
    show_id: MenuId,
    toggle_id: MenuId,
    quit_id: MenuId,
}

impl Tray {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let show_item = MenuItem::new("Show window", true, None);
        let toggle_item = MenuItem::new("Start recording", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let menu = Menu::new();
        menu.append_items(&[&show_item, &toggle_item, &quit_item])?;

        // El icono en reposo es negro: macOS lo adapta a la barra de menús clara u oscura.
        let idle_icon = dot_icon([0, 0, 0])?;
        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("screenrecorder")
            .with_icon(idle_icon.clone())
            .with_icon_as_template(true)
            .build()?;

        Ok(Self {
            icon,
            idle_icon,
            recording_icon: dot_icon([255, 59, 48])?,
            show_id: show_item.id().clone(),
            toggle_id: toggle_item.id().clone(),
            quit_id: quit_item.id().clone(),
            toggle_item,
        })
    }

    /// Refleja el estado de grabación en el icono y en el texto del menú.
    pub fn set_recording(&self, recording: bool) {
        if recording {
            let _ = self.icon.set_icon(Some(self.recording_icon.clone()));
            self.icon.set_icon_as_template(false);
            self.toggle_item.set_text("Stop recording");
        } else {
            let _ = self.icon.set_icon(Some(self.idle_icon.clone()));
            self.icon.set_icon_as_template(true);
            self.toggle_item.set_text("Start recording");
        }
    }

    /// Devuelve la siguiente acción elegida en el menú, si hay alguna.
    pub fn poll(&self) -> Option<TrayCommand> {
        loop {
            let event = MenuEvent::receiver().try_recv().ok()?;
            if event.id == self.show_id {
                return Some(TrayCommand::ShowWindow);
            } else if event.id == self.toggle_id {
                return Some(TrayCommand::ToggleRecording);
            } else if event.id == self.quit_id {
                return Some(TrayCommand::Quit);
            }
        }
    }
}

/// Círculo sólido de 32×32 px.
fn dot_icon(rgb: [u8; 3]) -> Result<Icon, Box<dyn Error>> {
    const SIZE: u32 = 32;
    const RADIUS: f32 = 9.0;
    let center = SIZE as f32 / 2.0;

    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let alpha = if dx * dx + dy * dy <= RADIUS * RADIUS { 255 } else { 0 };
            rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], alpha]);
        }
    }
    Ok(Icon::from_rgba(rgba, SIZE, SIZE)?)
}
