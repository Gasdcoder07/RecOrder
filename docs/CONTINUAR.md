# Cómo continuar el proyecto

Guía de retoma. Léela de arriba abajo antes de tocar código.

Plan completo aprobado: `/Users/gasdcoder07/.claude/plans/hazme-un-plan-por-nifty-kernighan.md`

## 1. Estado actual

| Fase | Estado |
|---|---|
| 1. Setup del workspace | Hecha |
| 2. MVP macOS (captura → MP4 H.264) | **Grabación funcional.** Falta revisión visual en QuickTime y medición de recursos |
| 3. UI Slint, temas, bandeja, hotkey | **Código escrito y compila.** Falta probar clics, atajo y bandeja de extremo a extremo (ver sección 2b) |
| 4. Audio (micrófono y sistema) | **En curso.** Micrófono implementado, sin verificar (ver sección 2c). Audio del sistema pendiente |
| 5. Windows | Pendiente |
| 6. Linux | Pendiente |
| 7. Ventana / región / multi-monitor / cursor | Pendiente |
| 8. Optimización y empaquetado | Pendiente |

Hecho:
- Rust 1.99.1 instalado con rustup (perfil minimal). Antes de usar `cargo` en una terminal nueva: `source "$HOME/.cargo/env"`.
- Workspace con 6 crates: `core`, `capture`, `encode`, `audio`, `platform`, `app`.
- `cargo build --workspace` y `cargo build --release -p screenrecorder` compilan sin warnings.
- `crates/capture/src/macos.rs`: `record_main_display(output, fps, max_width)` graba la pantalla principal con `SCRecordingOutput` (H.264 por hardware, MP4). Se detiene con Enter o Ctrl+C.
- `crates/app/src/main.rs`: CLI `screenrecorder record <salida.mp4> [--fps 30|60] [--max-width <px>]`.
- Ctrl+C finaliza el MP4 (antes dejaba el archivo inválido).
- Salida escalada: por defecto ancho máximo 1920 px, alto proporcional y par.
- FPS configurable: 30 o 60 (por defecto 60).
- `.cargo/config.toml`: agrega `-Wl,-rpath,/usr/lib/swift` para macOS. Sin esto el binario no arranca (`Library not loaded: @rpath/libswift_Concurrency.dylib`).
- Pruebas unitarias del escalado (`cargo test -p sr-capture`): 3 pasan.
- Fase 3 (código):
  - `ui/main.slint` y `ui/theme.slint`: ventana de 360×250 con Record/Stop, temporizador, tamaño en vivo, selector de FPS (30/60), tema (Sistema/Claro/Oscuro) y carpeta de salida (`Change…`). Tokens de color en el global `Theme`.
  - `crates/app/src/gui.rs`: estado en el hilo de la UI; temporizador cada 250 ms; sondeo de bandeja y atajo cada 100 ms. Los archivos se guardan en `~/Movies/screenrecorder/screenrecording-AAAAMMDD-HHMMSS.mp4`.
  - `crates/platform`: `tray.rs` (icono de bandeja con Show / Start-Stop / Quit; el icono se vuelve rojo al grabar), `hotkey.rs` (Cmd+Shift+R en macOS, Ctrl+Shift+R en otros), `permissions.rs` (`CGPreflightScreenCaptureAccess`, abrir Ajustes del Sistema).
  - Cerrar la ventana la oculta; la app sigue en la bandeja. Quit de la bandeja detiene una grabación activa antes de salir.
  - `crates/capture`: la API ahora es `start_main_display` → `Recording::stop`. La CLI `record` sigue funcionando sobre esa API.
  - Pruebas de humo: la ventana arranca sin errores en log; captura de la ventana en claro y en oscuro (con Sistema siguiendo la apariencia de macOS).
  - Verificación de la CLI tras el refactor: Enter y Ctrl+C producen MP4 válidos (3.8 s para 4 s grabados).
- Binario release: 2.36 MB (`cargo build --release -p screenrecorder`).
- Barra de título (`crates/platform/src/titlebar.rs`): Slint crea la ventana winit con `with_transparent(true)`, así que la barra de título era transparente y su texto quedaba negro sobre el escritorio. Ahora la ventana es opaca (`setOpaque(true)`) y su apariencia sigue al tema (Sistema, Claro u Oscuro) mediante el callback `Theme.mode-changed`. Verificado: al arrancar, la ventana nativa existía con `isOpaque = false`; captura en claro correcta.
- Micrófono: `start_main_display(..., microphone)` activa `SCStreamConfiguration::with_captures_microphone` (SCK 10.0.3, macOS 15+) y `SCRecordingOutput` escribe la pista en el MP4, así que no hace falta `cpal` ni sincronizar A/V a mano. Interruptor "Mic on/off" en la ventana y `--no-mic` en la CLI. Activo por defecto.
- `crates/app/Info.plist` se embebe en el binario (`build.rs`, sección `__info_plist`) con `NSMicrophoneUsageDescription` y `NSScreenCaptureUsageDescription`. Sin `CFBundleIdentifier`: con él, TCC trata el binario como app propia y deja de usar el permiso de la terminal.

## 2. Verificación de la grabación (última corrida)

- El error de TCC que bloqueaba la prueba ya no aparece: la grabación se completa y guarda el archivo.
- Enter tras 5 s: MP4 de 4.1 s, 1280×832 (escala de una pantalla 16:10 a 1280 px de ancho).
- Ctrl+C tras 5 s: el proceso sale con código 0, imprime `Saved` y el MP4 de 4.8 s queda finalizado.
- Medido con AVFoundation (ver script de sonda abajo). La tasa de frames medida (17–20 fps con fps=30 pedidos) es baja, pero ScreenCaptureKit solo entrega frames al cambiar la pantalla; no es concluyente. Se medirá en la sección 4.

Si el permiso de TCC se pierde (por ejemplo, al cambiar de terminal), la acción del usuario es:
1. Ajustes del Sistema → Privacidad y seguridad → Grabación de pantalla y audio del sistema.
2. Activar la app desde la que se ejecuta el comando (Terminal, iTerm, Warp, VS Code, etc.).
3. Reiniciar esa app.

## 2b. Qué no está probado en la fase 3

Estas pruebas requieren interacción que no se pudo automatizar desde la terminal: `osascript` no tiene permiso de Accesibilidad para enviar teclas ni clics.

- Botón Record/Stop, atajo global Cmd+Shift+R y menú de la bandeja: el código compila y la app arranca, pero no se han ejecutado de extremo a extremo.
- Cerrar la ventana deja la app viva en la bandeja: no probado.
- Botón "Open Settings" cuando falta el permiso: no probado.
- Cambio de carpeta con `Change…`: no probado.

Para probarlos a mano: `cargo run --release -p screenrecorder` y seguir la sección 3.

**Memoria en reposo:** `footprint` da 56 MB con la ventana visible. El presupuesto del plan (< 25 MB) es para el estado solo-bandeja, que aún no se midió. Hay que medirlo con la ventana oculta y, si sigue alto, perfilar el renderer por software y las fuentes.

**Detener la grabación bloquea la UI** brevemente: `remove_recording_output` espera a que el MP4 se cierre. `SCStream` no es `Send`, así que no se puede mover a otro hilo sin rediseñar.

## 2c. Micrófono: bloqueado por permiso (última corrida)

- Con `--no-mic` la grabación funciona (MP4 de 4 s guardado).
- Con micrófono, SCK falla con "El usuario rechazó la configuración de TCC". `AVCaptureDevice.authorizationStatus(for: .audio)` devuelve **2 (denegado)** para el proceso que ejecuta el comando.
- Acción del usuario: Ajustes del Sistema → Privacidad y seguridad → Micrófono → activar la app desde la que se ejecuta el comando (Terminal, iTerm, VS Code, etc.). Reiniciar esa app y volver a probar.
- Pendiente tras conceder el permiso: confirmar que el MP4 tiene una pista de audio con señal (sonda Swift con `AVAssetReader` en el scratchpad), y que el audio del sistema no se cuela (reproducir un sonido durante la prueba).
- Pendiente: si el permiso está denegado, la ventana muestra el error genérico de TCC. Convendría un mensaje claro en la barra de estado.

## 3. Cómo probar

```sh
cd /Volumes/PortableSSD/Projects/screenrecorder
source "$HOME/.cargo/env"
cargo run --release -p screenrecorder -- record prueba.mp4 --fps 30 --max-width 1280
# Grabar unos segundos y presionar Enter o Ctrl+C para detener.
```

Notas:
- El archivo de salida no debe existir; el programa se niega a sobrescribirlo.
- `*.mp4` está en `.gitignore`, así que las pruebas en la raíz no ensucian el repo.
- Para ejecutar sin stdin interactivo, `< /dev/null` detiene la grabación de inmediato (útil solo para probar que el archivo se cierra).
- Para una grabación de duración fija sin interacción: `( sleep 5; echo ) | cargo run ...`.
- No hay `timeout` en este shell (zsh de macOS sin coreutils).

Verificación del MP4:
- Abrirlo en QuickTime: debe reproducirse, con duración correcta y cursor visible. **Pendiente**: solo se verificó con AVFoundation.
- `afinfo` no sirve para MP4 con video. Sonda usada: `swift <script>.swift prueba.mp4` con `AVURLAsset` (duración, `naturalSize` y `nominalFrameRate` de la pista de video). El script vive en el scratchpad de la sesión, no en el repo.
- Alternativa por consola: `brew install ffmpeg` y luego `ffprobe prueba.mp4` (instalar requiere permiso del usuario).

## 4. Pasos siguientes, en orden

1. **Revisar el MP4 en QuickTime** (sección 3). Criterio de éxito: se reproduce, duración ≈ tiempo grabado, cursor visible.
2. **Medir el presupuesto** del plan (son objetivos, no mediciones todavía):
   - Idle: no aplica aún; medir el proceso grabando.
   - Grabando a pantalla completa: `ps -o rss,%cpu -p <pid>` y `footprint <pid>`; o Activity Monitor.
   - Comparar `--max-width 1920` contra `--max-width 99999` (nativo) y `--fps 30` contra `--fps 60`.
   - Comparar con OBS como referencia.
   - Revisar la tasa de frames con contenido en movimiento (un escritorio estático no la representa).
3. ~~Manejar Ctrl+C.~~ **Hecho.** `ctrlc` en `crates/capture`; el stop se coordina con un canal mpsc.
4. ~~Resolución de salida.~~ **Hecho.** `--max-width` (defecto 1920). Hace falta medir el efecto en RAM/VRAM (paso 2).
5. ~~FPS configurable.~~ **Hecho.** `--fps 30|60` (defecto 60).
6. **Fase 3.** Código hecho (ver sección 1). Pendiente: probar botón, atajo, bandeja y ventana oculta (sección 2b); medir la memoria en modo bandeja.
7. **Fase 4.** Audio. Micrófono: hecho en código, falta verificar con permiso (sección 2c). Audio del sistema: pendiente (ScreenCaptureKit `with_captures_audio`). Ya no se usará `cpal`: SCK entrega el micrófono dentro del mismo `SCRecordingOutput`, con la sincronía A/V incluida.

## 5. Decisiones pendientes del usuario

- **Licencia:** el plan dice MIT o Apache-2.0. El `Cargo.toml` ya declara `MIT OR Apache-2.0`, pero falta el archivo `LICENSE`.
- **Repositorio:** `Cargo.toml` tiene el placeholder `https://github.com/OWNER/screenrecorder`.
- **Git:** el directorio aún no es repo. Si se inicializa, el primer commit debe usar el trailer `Co-Authored-By` indicado en la instrucción de atribución vigente.
- **Mínimo de macOS:** el plan decía 13+ (por audio). La grabación nativa `SCRecordingOutput` exige **macOS 15+**. Hay que decidir si se acepta, o si se implementa un encoder con VideoToolbox para macOS 13–14 (más trabajo).
- **MSRV:** `rust-version = "1.80"` en el workspace; la crate `screencapturekit` pide 1.76. Compatible, pero conviene revisarlo al añadir dependencias.
- **Slint fijado a la serie 1.9 (`~1.9.2`) por MSRV.** Slint 1.10 pide Rust 1.82, 1.13 pide 1.85 y 1.18 pide 1.92. Para usar una versión nueva hay que subir `rust-version` del workspace. Decisión pendiente: subir el MSRV o quedarse en 1.9.
- **tray-icon fijado a 0.24.2:** la 0.26 pide Rust 1.90. Mismo motivo.
- **Ancho máximo por defecto (1920):** es una elección técnica tomada en esta fase. Si se prefiere la resolución nativa por defecto, cambiar `DEFAULT_MAX_WIDTH` en `crates/app/src/main.rs`.

## 6. Decisiones técnicas ya tomadas (y por qué)

- **`SCRecordingOutput` en lugar de VideoToolbox propio:** el sistema codifica en hardware y escribe el MP4; cero copias de frames a CPU. Mucho menos código que mantener.
- **Versión de crate `screencapturekit = "10"` con feature `macos_15_0`:** la feature es obligatoria para `SCRecordingOutput`.
- **Firmas de API:** la documentación de Context7 para esta crate no coincide con la versión 10.0.3 (por ejemplo, `SCStream::new` devuelve `Self`, no `Result`). Como referencia manda el código fuente en `~/.cargo/registry/src/index.crates.io-*/screencapturekit-10.0.3/src/`.
- **`remove_recording_output` finaliza el archivo:** por eso el stop usa esa llamada y no solo `stop_capture`.
- **Sin dependencia de CLI pesada:** se parsea `std::env::args` a mano para mantener el binario pequeño.
- **Crate `ctrlc` para Ctrl+C:** la std no ofrece manejo de SIGINT. Context7 no indexa esta crate; la API (`set_handler(FnMut() + Send + 'static)`) se verificó en el código fuente de la 3.5.2.
- **Estado de la UI solo en el hilo principal:** `SCStream` no es `Send`. Los timers y los sondeos de bandeja y atajo corren en el mismo hilo, así que no hay sincronización entre hilos.
- **No mantener `borrow()` abierto durante un diálogo:** `rfd` abre un bucle anidado que dispara los timers; si hay un préstamo activo, `RefCell` entra en pánico.
- **Icono de la bandeja en negro con `template`:** macOS lo adapta a la barra clara u oscura. El rojo se usa solo mientras se graba.
- **Atajo en texto plano:** la fuente de Slint no tiene los glifos ⌘ y ⇧, así que se muestra "Cmd+Shift+R".
- **Enter y Ctrl+C por el mismo canal:** un hilo lee stdin y el manejador de Ctrl+C envía al mismo `mpsc`, así el hilo principal no queda bloqueado en `read_line`.

## 7. Verificación rápida al retomar

```sh
source "$HOME/.cargo/env"
cd /Volumes/PortableSSD/Projects/screenrecorder
cargo build --workspace                    # debe compilar sin errores
cargo test -p sr-capture                   # 3 pruebas de escalado
cargo build --release -p screenrecorder    # binario en target/release/screenrecorder
```

Si algo falla en el enlazado con Swift, revisar `.cargo/config.toml`.
