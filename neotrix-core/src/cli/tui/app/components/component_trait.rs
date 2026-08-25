//! Component & Widget Traits - 组件系统核心

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
#[allow(unused_imports)]
use ratatui::Frame;
#[allow(unused_imports)]
use ratatui::layout::Rect;
#[allow(unused_imports)]
use crate::cli::tui::app::state::AppState;
#[allow(unused_imports)]
use crate::cli::tui::app::actions::Action;

/// 组件 ID
pub type ComponentId = String;

/// 组件 trait - 所有 UI 组件的基础 trait
#[async_trait]
pub trait Component: Send + Sync + Debug {
    /// 组件唯一 ID
    fn id(&self) -> ComponentId;

    /// 组件名称（用于调试）
    fn name(&self) -> &'static str;

    /// 渲染组件
    fn view(&self, state: &crate::cli::tui::app::state::AppState, frame: &mut ratatui::Frame, area: ratatui::layout::Rect);

    /// 处理动作，返回副作用
    async fn update(&mut self, state: &mut crate::cli::tui::app::state::AppState, action: crate::cli::tui::app::actions::Action) -> Vec<crate::cli::tui::app::effects::Effect>;

    /// 是否可聚焦
    fn focusable(&self) -> bool {
        false
    }

    /// 获取/设置聚焦状态
    fn focused(&mut self, _focused: bool) {}

    /// 获取子组件 ID
    fn children(&self) -> Vec<ComponentId> {
        Vec::new()
    }

    /// 生命周期：挂载
    async fn on_mount(&mut self, _state: &mut crate::cli::tui::app::state::AppState) {}

    /// 生命周期：卸载
    async fn on_unmount(&mut self, _state: &mut crate::cli::tui::app::state::AppState) {}

    /// 转为 Any 以支持向下转型
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Widget trait - 纯渲染组件，无状态
pub trait Widget: Send + Sync + Debug {
    fn render(&self, state: &crate::cli::tui::app::state::AppState, frame: &mut ratatui::Frame, area: ratatui::layout::Rect);
}

/// 组件注册表
pub struct ComponentRegistry {
    components: std::collections::HashMap<String, Arc<dyn Component>>,
    mount_order: Vec<String>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            mount_order: Vec::new(),
        }
    }

    pub fn register(&mut self, component: Arc<dyn Component>) {
        let id = component.id();
        self.components.insert(id.clone(), component);
        self.mount_order.push(id);
    }

    pub fn unregister(&mut self, id: &str) -> Option<Arc<dyn Component>> {
        self.mount_order.retain(|id| id != &id);
        self.components.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn Component>> {
        self.components.get(id).cloned()
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Arc<dyn Component>> {
        self.components.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn Component>> {
        self.mount_order.iter().filter_map(|id| self.components.get(id))
    }

    pub fn iter_mut(&mut self) -> Vec<&mut Arc<dyn Component>> {
        self.mount_order
            .iter()
            .filter_map(|id| self.components.get_mut(id))
            .collect()
    }

    pub async fn mount_all(&mut self, state: &mut crate::cli::tui::app::state::AppState) {
        for id in &self.mount_order {
            if let Some(comp) = self.components.get(id) {
                comp.on_mount(state).await;
            }
        }
    }

    pub async fn unmount_all(&mut self, state: &mut crate::cli::tui::app::state::AppState) {
        for id in self.mount_order.iter().rev() {
            if let Some(comp) = self.components.get(id) {
                comp.on_unmount(state).await;
            }
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件工厂 trait
pub trait ComponentFactory: Send + Sync {
    fn create(&self, config: &crate::cli::tui::app::state::config::RuntimeConfig) -> Arc<dyn Component>;
}

/// 组件工厂注册表
pub struct ComponentFactoryRegistry {
    factories: HashMap<String, Arc<dyn ComponentFactory>>,
}

impl ComponentFactoryRegistry {
    pub fn new() -> Self {
        Self { factories: HashMap::new() }
    }

    pub fn register(&mut self, name: String, factory: Arc<dyn ComponentFactory>) {
        self.factories.insert(name, factory);
    }

    pub fn create(&self, name: &str, config: &crate::cli::tui::app::state::config::RuntimeConfig) -> Option<Arc<dyn Component>> {
        self.factories.get(name).map(|f| f.create(config))
    }

    pub fn list(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_registry() {
        let registry = ComponentRegistry::new();
        assert!(registry.mount_order.is_empty());
    }
}
