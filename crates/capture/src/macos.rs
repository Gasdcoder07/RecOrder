//! Grabación de pantalla en macOS con ScreenCaptureKit (`SCRecordingOutput`, macOS 15+).
//!
//! El sistema codifica H.264 en hardware y escribe el MP4 directamente, así que
//! los frames nunca se copian a memoria de la CPU.
//!
//! `SCStream` no implementa `Send`: una grabación debe iniciarse y detenerse desde
//! el mismo hilo (en la app, el hilo de la UI).

use std::error::Error;
use std::path::Path;

use screencapturekit::prelude::*;
use screencapturekit::recording_output::{
    SCRecordingOutput, SCRecordingOutputCodec, SCRecordingOutputConfiguration,
    SCRecordingOutputFileType,
};

/// Una grabación en curso. Si se descarta sin llamar a [`Recording::stop`], el MP4
/// queda sin finalizar.
pub struct Recording {
    stream: SCStream,
    recording: SCRecordingOutput,
}

/// Empieza a grabar la pantalla principal en `output`.
///
/// `max_width` limita el ancho del video; la altura se ajusta al aspecto original.
/// Bajarlo reduce la RAM y la VRAM que usa el codificador.
///
/// `microphone` añade la voz del micrófono como pista de audio del MP4. macOS pide
/// permiso de micrófono la primera vez.
pub fn start_main_display(
    output: &Path,
    fps: u32,
    max_width: u32,
    microphone: bool,
) -> Result<Recording, Box<dyn Error>> {
    if output.exists() {
        return Err(format!("{} already exists", output.display()).into());
    }

    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("no display found")?;

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let (width, height) = scaled_size(display.width(), display.height(), max_width);
    let config = SCStreamConfiguration::new()
        .with_width(width)
        .with_height(height)
        .with_shows_cursor(true)
        .with_minimum_frame_interval(&CMTime::new(1, fps as i32))
        .with_captures_microphone(microphone);

    let recording_config = SCRecordingOutputConfiguration::new()
        .with_output_url(output)
        .with_video_codec(SCRecordingOutputCodec::H264)
        .with_output_file_type(SCRecordingOutputFileType::MP4);
    let recording = SCRecordingOutput::new(&recording_config)
        .ok_or("screen recording requires macOS 15 or later")?;

    let stream = SCStream::new(&filter, &config);
    stream.add_recording_output(&recording)?;
    stream.start_capture()?;

    Ok(Recording { stream, recording })
}

impl Recording {
    /// Detiene la captura y cierra el MP4. Bloquea hasta que el archivo está completo.
    pub fn stop(self) -> Result<(), Box<dyn Error>> {
        // Quitar la salida detiene la captura y cierra el archivo MP4 correctamente.
        self.stream.remove_recording_output(&self.recording)?;
        Ok(())
    }
}

/// Ancho acotado a `max_width` y alto proporcional. Ambas dimensiones se redondean
/// a par, porque H.264 no admite dimensiones impares.
fn scaled_size(width: u32, height: u32, max_width: u32) -> (u32, u32) {
    if width <= max_width {
        return (width, height);
    }
    let w = max_width & !1;
    let h = u64::from(height) * u64::from(w) / u64::from(width);
    (w, (h as u32) & !1)
}

#[cfg(test)]
mod tests {
    use super::scaled_size;

    #[test]
    fn scales_5k_retina_down_to_full_hd() {
        assert_eq!(scaled_size(5120, 2880, 1920), (1920, 1080));
    }

    #[test]
    fn keeps_size_when_already_below_limit() {
        assert_eq!(scaled_size(1280, 800, 1920), (1280, 800));
    }

    #[test]
    fn rounds_scaled_dimensions_to_even() {
        assert_eq!(scaled_size(1000, 999, 500), (500, 498));
        assert_eq!(scaled_size(5120, 2880, 1921), (1920, 1080));
    }
}
