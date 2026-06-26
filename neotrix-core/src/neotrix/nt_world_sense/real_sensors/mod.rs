//! Real sensor implementations (macOS native via CLI tools).
//! Falls back to file-simulated on non-macOS platforms.

pub mod mic;
pub mod screen;

pub use mic::MicCapture;
pub use screen::ScreenCapture;
