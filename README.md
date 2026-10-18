# screenrecorder

Grabador de pantalla minimalista, ligero y de código abierto para macOS, Linux y Windows.

**Objetivo principal:** velocidad y bajo consumo de RAM y VRAM.

## Estado

Fase 4 (audio, en curso). Graba la pantalla principal a MP4 H.264 desde una ventana Slint con bandeja y atajo global, y puede añadir la voz del micrófono. Aún no graba el audio del sistema. La interacción de la ventana, el atajo y la bandeja está pendiente de prueba manual.

## Desarrollo

Requiere Rust estable (1.80+). En macOS, la grabación requiere macOS 15+ y el permiso de grabación de pantalla para la terminal que ejecuta el comando.

```sh
cargo build --release
cargo run -p screenrecorder                  # ventana con bandeja y atajo
cargo run -p screenrecorder -- record salida.mp4
cargo run -p screenrecorder -- record salida.mp4 --fps 30 --max-width 1280
```

En la ventana: Record/Stop, micrófono (Mic on/off, solo antes de grabar), selector de FPS y tema (Sistema, Claro, Oscuro), y carpeta de salida (por defecto `~/Movies/screenrecorder`). Atajo global: Cmd+Shift+R.

Opciones de `record` (modo terminal):
- `--fps 30|60`: cuadros por segundo (por defecto 60).
- `--max-width <px>`: ancho máximo del video; la altura se ajusta al aspecto (por defecto 1920).
- `--no-mic`: graba sin la pista de micrófono (por defecto el micrófono está activo).

El micrófono necesita permiso de micrófono para la app desde la que se ejecuta el comando (Ajustes del Sistema → Privacidad y seguridad → Micrófono).

Para detener la grabación desde la terminal, presiona Enter o Ctrl+C.

Plan y guía de retoma: ver `docs/CONTINUAR.md`.
