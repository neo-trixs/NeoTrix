//! Components - 可组合的 UI 组件

pub mod component_trait;
pub mod chat;
pub mod sidebar;
pub mod status;
pub mod dialogs;
pub mod layout;

pub use component_trait::{Component, Widget, ComponentRegistry, ComponentFactory, ComponentFactoryRegistry};
// pub use component_trait;
