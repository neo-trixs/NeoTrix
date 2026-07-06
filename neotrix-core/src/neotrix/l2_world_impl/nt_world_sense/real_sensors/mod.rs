//! Real sensor implementations (macOS native via CLI tools).
//! Falls back to file-simulated on non-macOS platforms.

pub mod screen;
pub mod mic;

pub use screen::ScreenCapture;
pub use mic::MicCapture;
