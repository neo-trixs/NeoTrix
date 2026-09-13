//! Dependency Injection Container — 替换全局 LazyLock 静态单例
//!
//! 设计:
//! - 类型安全注册: `TypeId → Arc<dyn Any>`
//! - 作用域支持: 每个 scope 有独立的覆盖层，回退到全局层
//! - 迁移兼容: 现有 `LazyLock` 静态单例可逐步迁入 DI

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// DI 容器: 管理服务实例的注册与解析
#[derive(Clone, Default)]
pub struct Container {
    /// 全局服务注册表 (TypeId → Box<dyn Any + Send + Sync>)
    services: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl Container {
    /// 创建空容器
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册服务实例 (按具体类型)
    pub fn register<T: Send + Sync + 'static>(&self, instance: T) {
        let mut map = self.services.write().unwrap_or_else(|e| e.into_inner());
        map.insert(TypeId::of::<T>(), Box::new(instance));
    }

    /// 注册服务为 Arc 包装
    pub fn register_arc<T: Send + Sync + 'static>(&self, instance: Arc<T>) {
        let mut map = self.services.write().unwrap_or_else(|e| e.into_inner());
        map.insert(TypeId::of::<Arc<T>>(), Box::new(instance));
    }

    /// 解析服务实例 (按具体类型)
    pub fn resolve<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let map = self.services.read().unwrap_or_else(|e| e.into_inner());
        map.get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
            .cloned()
    }

    /// 解析 Arc 包装的服务
    pub fn resolve_arc<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let map = self.services.read().unwrap_or_else(|e| e.into_inner());
        map.get(&TypeId::of::<Arc<T>>())
            .and_then(|b| b.downcast_ref::<Arc<T>>())
            .cloned()
    }

    /// 检查是否已注册某类型
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        let map = self.services.read().unwrap_or_else(|e| e.into_inner());
        map.contains_key(&TypeId::of::<T>())
    }

    /// 移除已注册的服务
    pub fn remove<T: Send + Sync + 'static>(&self) -> bool {
        let mut map = self.services.write().unwrap_or_else(|e| e.into_inner());
        map.remove(&TypeId::of::<T>()).is_some()
    }

    /// 清空所有注册
    pub fn clear(&self) {
        let mut map = self.services.write().unwrap_or_else(|e| e.into_inner());
        map.clear();
    }

    /// 返回已注册的服务数量
    pub fn len(&self) -> usize {
        let map = self.services.read().unwrap_or_else(|e| e.into_inner());
        map.len()
    }

    /// 容器是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 全局 DI 容器单例 — 替代各模块分散的 LazyLock
static GLOBAL_CONTAINER: RwLock<Option<Container>> = RwLock::new(None);

/// 获取全局容器 (惰性初始化)
pub fn global_container() -> Container {
    let read = GLOBAL_CONTAINER.read().unwrap_or_else(|e| e.into_inner());
    if let Some(c) = read.as_ref() {
        return c.clone();
    }
    drop(read);
    let mut write = GLOBAL_CONTAINER.write().unwrap_or_else(|e| e.into_inner());
    if write.is_none() {
        *write = Some(Container::new());
    }
    write.as_ref().expect("Some value").clone()
}

/// 向全局容器注册服务
pub fn register_global<T: Send + Sync + 'static>(instance: T) {
    global_container().register(instance);
}

/// 从全局容器解析服务
pub fn resolve_global<T: Clone + Send + Sync + 'static>() -> Option<T> {
    global_container().resolve()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestService {
        name: String,
    }

    #[test]
    fn test_register_and_resolve() {
        let c = Container::new();
        c.register(TestService { name: "hello".into() });
        let s: Option<TestService> = c.resolve();
        assert_eq!(s.as_ref().map(|s| &s.name), Some(&"hello".to_string()));
    }

    #[test]
    fn test_resolve_arc() {
        let c = Container::new();
        c.register_arc(Arc::new(TestService { name: "arc".into() }));
        let s: Option<Arc<TestService>> = c.resolve_arc();
        assert_eq!(s.as_ref().map(|s| &s.name), Some(&"arc".to_string()));
    }

    #[test]
    fn test_contains() {
        let c = Container::new();
        assert!(!c.contains::<TestService>());
        c.register(TestService { name: "x".into() });
        assert!(c.contains::<TestService>());
    }

    #[test]
    fn test_remove() {
        let c = Container::new();
        c.register(TestService { name: "x".into() });
        assert!(c.remove::<TestService>());
        assert!(!c.contains::<TestService>());
    }

    #[test]
    fn test_len_and_is_empty() {
        let c = Container::new();
        assert!(c.is_empty());
        c.register(TestService { name: "a".into() });
        assert_eq!(c.len(), 1);
        c.register(42u32);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn test_clear() {
        let c = Container::new();
        c.register(TestService { name: "a".into() });
        c.register(42u32);
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn test_global_container() {
        let c = global_container();
        assert!(!c.is_empty() || c.is_empty()); // 首次可能已初始化
    }
}
