//! Pipeline principal: CaptureSource -> FrameQueue -> Encoder -> Muxer.
//!
//! Las colas son acotadas para aplicar backpressure: si el encoder se atrasa,
//! se descartan frames en lugar de crecer la memoria.
