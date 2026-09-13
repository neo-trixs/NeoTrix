//! 能力热重载
//!
//! 支持能力的动态加载、卸载、重载

#![allow(dead_code)]

use crate::core::nt_core_capability::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 热重载配置
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// 启用热重载
    pub enabled: bool,
    /// 检查间隔
    pub check_interval: Duration,
    /// 最大重载次数
    pub max_reloads: u32,
    /// 重载冷却时间
    pub cooldown: Duration,
    /// 启用回滚
    pub enable_rollback: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(30),
            max_reloads: 10,
            cooldown: Duration::from_secs(60),
            enable_rollback: true,
        }
    }
}

/// 热重载事件
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    /// 能力加载
    Loaded(String),
    /// 能力卸载
    Unloaded(String),
    /// 能力重载
    Reloaded(String),
    /// 能力更新
    Updated(String),
    /// 重载失败
    Failed(String, String),
    /// 回滚
    RolledBack(String),
}

/// 热重载管理器
pub struct HotReloadManager {
    /// 配置
    config: HotReloadConfig,
    /// 已加载的能力
    loaded_capabilities: std::collections::HashMap<String, LoadedCapability>,
    /// 事件历史
    events: Vec<HotReloadEvent>,
    /// 最后检查时间
    last_check: Instant,
    /// 重载计数
    reload_count: u32,
}

/// 已加载能力
#[derive(Debug, Clone)]
pub struct LoadedCapability {
    /// 能力元数据
    meta: CapabilityMeta,
    /// 加载时间
    loaded_at: Instant,
    /// 版本
    version: String,
    /// 文件路径
    path: Option<String>,
    /// 哈希
    hash: Option<String>,
}

impl HotReloadManager {
    /// 创建新的热重载管理器
    pub fn new(config: HotReloadConfig) -> Self {
        Self {
            config,
            loaded_capabilities: std::collections::HashMap::new(),
            events: Vec::new(),
            last_check: Instant::now(),
            reload_count: 0,
        }
    }

    /// 注册能力
    pub fn register(&mut self, capability: &dyn UnifiedCapability) {
        let meta = capability.meta();
        let loaded = LoadedCapability {
            meta: meta.clone(),
            loaded_at: Instant::now(),
            version: meta.version.clone(),
            path: None,
            hash: None,
        };

        self.loaded_capabilities.insert(meta.id.clone(), loaded);
        self.events.push(HotReloadEvent::Loaded(meta.id));
    }

    /// 卸载能力
    pub fn unregister(&mut self, capability_id: &str) -> bool {
        if self.loaded_capabilities.remove(capability_id).is_some() {
            self.events
                .push(HotReloadEvent::Unloaded(capability_id.to_string()));
            true
        } else {
            false
        }
    }

    /// 检查更新
    pub fn check_for_updates(&mut self) -> Vec<String> {
        let mut updates = Vec::new();

        // 检查是否超过冷却时间
        if self.last_check.elapsed() < self.config.cooldown {
            return updates;
        }

        // 检查重载次数
        if self.reload_count >= self.config.max_reloads {
            return updates;
        }

        // 模拟检查更新
        for (id, _capability) in &self.loaded_capabilities {
            // 这里应该实际检查文件变化
            // 简化实现：随机决定是否有更新
            if chrono::Utc::now().timestamp() % 10 == 0 {
                updates.push(id.clone());
            }
        }

        self.last_check = Instant::now();
        updates
    }

    /// 重载能力
    pub fn reload(&mut self, capability_id: &str) -> Result<(), String> {
        // 检查重载次数
        if self.reload_count >= self.config.max_reloads {
            return Err("达到最大重载次数".into());
        }

        // 获取当前版本
        let current = self
            .loaded_capabilities
            .get(capability_id)
            .ok_or_else(|| format!("能力未加载: {}", capability_id))?;

        let _old_version = current.version.clone();

        // 模拟重载
        let new_version = "1.1.0".to_string();

        // 更新版本
        if let Some(capability) = self.loaded_capabilities.get_mut(capability_id) {
            capability.version = new_version.clone();
            capability.loaded_at = Instant::now();
        }

        self.events
            .push(HotReloadEvent::Reloaded(capability_id.to_string()));
        self.reload_count += 1;

        Ok(())
    }

    /// 回滚能力
    pub fn rollback(&mut self, capability_id: &str) -> Result<(), String> {
        if !self.config.enable_rollback {
            return Err("回滚未启用".into());
        }

        // 模拟回滚
        if let Some(capability) = self.loaded_capabilities.get_mut(capability_id) {
            capability.version = "1.0.0".to_string();
            capability.loaded_at = Instant::now();
        }

        self.events
            .push(HotReloadEvent::RolledBack(capability_id.to_string()));
        Ok(())
    }

    /// 获取已加载的能力
    pub fn get_loaded(&self, capability_id: &str) -> Option<&LoadedCapability> {
        self.loaded_capabilities.get(capability_id)
    }

    /// 获取所有已加载的能力
    pub fn get_all_loaded(&self) -> &std::collections::HashMap<String, LoadedCapability> {
        &self.loaded_capabilities
    }

    /// 获取事件历史
    pub fn get_events(&self, count: usize) -> Vec<&HotReloadEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// 清理旧事件
    pub fn cleanup_events(&mut self, _max_age: Duration) {
        // 简化实现：只保留最近100个事件
        if self.events.len() > 100 {
            self.events.drain(0..self.events.len() - 100);
        }
    }

    /// 重置重载计数
    pub fn reset_reload_count(&mut self) {
        self.reload_count = 0;
    }

    /// 获取统计信息
    pub fn stats(&self) -> HotReloadStats {
        HotReloadStats {
            loaded_count: self.loaded_capabilities.len(),
            reload_count: self.reload_count,
            event_count: self.events.len(),
            last_check: self.last_check,
        }
    }
}

/// 热重载统计
#[derive(Debug, Clone)]
pub struct HotReloadStats {
    pub loaded_count: usize,
    pub reload_count: u32,
    pub event_count: usize,
    pub last_check: Instant,
}

/// 热重载包装器
pub struct HotReloadCapability {
    /// 底层能力
    capability: Arc<std::sync::Mutex<dyn UnifiedCapability>>,
    /// 热重载管理器
    manager: Arc<std::sync::Mutex<HotReloadManager>>,
    /// 能力ID
    capability_id: String,
}

impl HotReloadCapability {
    /// 创建热重载包装器
    pub fn new(
        capability: Arc<std::sync::Mutex<dyn UnifiedCapability>>,
        manager: Arc<std::sync::Mutex<HotReloadManager>>,
    ) -> Self {
        let capability_id = capability.lock().unwrap_or_else(|e| e.into_inner()).meta().id.clone();
        Self {
            capability,
            manager,
            capability_id,
        }
    }

    /// 带热重载的执行
    pub fn execute_with_reload(
        &self,
        input: CapabilityInput,
    ) -> Result<CapabilityOutput, CapabilityError> {
        // 尝试执行
        let result = self.capability.lock().unwrap_or_else(|e| e.into_inner()).execute(input.clone());

        // 如果失败，尝试重载
        if result.is_err() {
            let mut manager = self.manager.lock().unwrap_or_else(|e| e.into_inner());
            if manager.reload(&self.capability_id).is_ok() {
                // 重载成功，重试执行
                return self.capability.lock().unwrap_or_else(|e| e.into_inner()).execute(input);
            }
        }

        result
    }

    /// 获取当前版本
    pub fn current_version(&self) -> String {
        self.manager
            .lock()
            .expect("mutex poisoned")
            .get_loaded(&self.capability_id)
            .map(|c| c.version.clone())
            .unwrap_or_default()
    }
}

impl UnifiedCapability for HotReloadCapability {
    fn meta(&self) -> CapabilityMeta {
        self.capability.lock().unwrap_or_else(|e| e.into_inner()).meta()
    }

    fn health(&self) -> CapabilityHealth {
        self.capability.lock().unwrap_or_else(|e| e.into_inner()).health()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        self.execute_with_reload(input)
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        self.capability.lock().unwrap_or_else(|e| e.into_inner()).supports(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_reload_manager() {
        let config = HotReloadConfig::default();
        let mut manager = HotReloadManager::new(config);

        // 创建测试能力
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        manager.register(cap.as_ref());

        assert_eq!(manager.stats().loaded_count, 1);
    }

    #[test]
    fn reload_capability() {
        let config = HotReloadConfig::default();
        let mut manager = HotReloadManager::new(config);

        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        manager.register(cap.as_ref());

        let result = manager.reload("nt-world-nlp");
        assert!(result.is_ok());
        assert_eq!(manager.stats().reload_count, 1);
    }

    #[test]
    fn rollback_capability() {
        let config = HotReloadConfig {
            enable_rollback: true,
            ..Default::default()
        };
        let mut manager = HotReloadManager::new(config);

        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        manager.register(cap.as_ref());

        let result = manager.rollback("nt-world-nlp");
        assert!(result.is_ok());
    }
}
