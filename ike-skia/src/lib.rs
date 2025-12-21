#![warn(clippy::unwrap_used)]

mod canvas;
mod painter;

#[cfg(feature = "vulkan")]
pub mod vulkan;

pub use canvas::SkiaCanvas;
pub use painter::SkiaPainter;
