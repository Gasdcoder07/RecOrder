//! Barra de título de macOS. Slint crea la ventana de winit con fondo transparente, así
//! que la barra deja ver el escritorio y el título queda en negro sobre cualquier fondo.
//! Aquí la ventana vuelve opaca, se pinta del mismo color que el contenido y su apariencia
//! sigue al tema elegido en la app.

use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSColor, NSView,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Aplica la apariencia y el color de fondo a la barra de título. `dark: None` sigue al
/// sistema. `background` es el color RGB del contenido: la barra lo toma para verse plana.
/// No hace nada si winit todavía no creó la ventana nativa.
///
/// Debe llamarse desde el hilo principal, como el resto de la interfaz.
pub fn apply(window: &impl HasWindowHandle, dark: Option<bool>, background: [u8; 3]) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };

    // SAFETY: winit mantiene vivo el NSView mientras la ventana exista, y aquí solo se
    // lee para obtener su NSWindow.
    let view: &NSView = unsafe { handle.ns_view.cast().as_ref() };
    let Some(ns_window) = view.window() else {
        return;
    };

    unsafe {
        ns_window.setOpaque(true);
        let [red, green, blue] = background.map(|channel| f64::from(channel) / 255.0);
        let color = NSColor::colorWithSRGBRed_green_blue_alpha(red, green, blue, 1.0);
        ns_window.setBackgroundColor(Some(&color));
        let appearance = match dark {
            None => None,
            Some(false) => NSAppearance::appearanceNamed(NSAppearanceNameAqua),
            Some(true) => NSAppearance::appearanceNamed(NSAppearanceNameDarkAqua),
        };
        ns_window.setAppearance(appearance.as_deref());
    }
}
