//! # Desktop Abstraction Layer
//!
//! 统一桌面功能抽象层，隔离平台特定实现与业务逻辑。
//! 基于 Domain Plugin 架构，通过 trait 定义能力契约。

pub mod capabilities;
pub mod model_manager;
pub mod session_manager;
pub mod tauri_impl;
pub mod universal_ui;

pub use capabilities::{DesktopCapabilities, UpdateInfo};
pub use model_manager::{DownloadProgress, DownloadTask, ModelManager, ModelMetadata, ModelFormat, ModelSource};
pub use session_manager::{Message, Session, SessionManager};
pub use tauri_impl::TauriDesktopCapabilities;
pub use universal_ui::{UniversalModelManager, UniversalUIState, UniversalModel, UniversalProvider};
