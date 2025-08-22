//! LinHT-GUI - Minimal radio control interface
//!
//! Features:
//! - Direct framebuffer access
//! - M17 protocol parsing
//! - Flowgraph management
//! - GPIO-ready input handling

pub mod framebuffer;
pub mod display;
pub mod m17;
pub mod input;

// Re-export main types
pub use framebuffer::{Framebuffer, Color, Point, FontId, FramebufferError};
pub use display::Display;
pub use m17::parse_m17_line;
pub use input::{InputEvent, InputHandler};
