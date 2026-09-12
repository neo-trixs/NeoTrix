//! Domain Registry — 插件注册表 + 事件总线
//!
//! 参考 DeepSeek Harness 的 Cordis context 模式：
//! 所有插件注册到共享 context，通过事件通信。

use super::{
    err, ok, DomainCall, DomainError, DomainEvent, DomainInfo, DomainPlugin, DomainResponse,
};
use std::collections::HashMap;
use tokio::sync::broadcast;

/// 事件总线容量
const EVENT_BUS_CAPACITY: usize = 256;

/// 域注册表
pub struct DomainRegistry {
    plugins: Vec<Box<dyn DomainPlugin>>,
    plugin_index: HashMap<String, usize>,
    event_tx: broadcast::Sender<DomainEvent>,
}

impl DomainRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_BUS_CAPACITY);
        Self {
            plugins: Vec::new(),
            plugin_index: HashMap::new(),
            event_tx,
        }
    }

    /// 注册域插件
    pub fn register(&mut self, plugin: Box<dyn DomainPlugin>) -> Result<(), DomainError> {
        let name = plugin.name().to_string();
        if self.plugin_index.contains_key(&name) {
            return Err(DomainError {
                code: "DOMAIN_DUPLICATE".into(),
                message: format!("Domain '{}' already registered", name),
                recoverable: false,
            });
        }
        let idx = self.plugins.len();
        self.plugins.push(plugin);
        self.plugin_index.insert(name, idx);
        Ok(())
    }

    /// 注册域插件 (带初始化)
    pub fn register_with_init(
        &mut self,
        mut plugin: Box<dyn DomainPlugin>,
    ) -> Result<(), DomainError> {
        plugin.init()?;
        self.register(plugin)
    }

    /// 执行域调用
    pub fn call(&self, request: DomainCall) -> DomainResponse {
        let DomainCall {
            domain,
            action,
            args,
        } = request;

        // 查找插件
        let idx = match self.plugin_index.get(&domain) {
            Some(idx) => *idx,
            None => {
                return err(DomainError {
                    code: "DOMAIN_NOT_FOUND".into(),
                    message: format!("Domain '{}' not found", domain),
                    recoverable: true,
                });
            }
        };

        // 执行 action
        let result = self.plugins[idx].call(&action, args);

        match result {
            Ok(data) => ok(data),
            Err(e) => err(e),
        }
    }

    /// 列出所有已注册域
    pub fn list(&self) -> Vec<DomainInfo> {
        self.plugins
            .iter()
            .map(|p| DomainInfo {
                name: p.name().to_string(),
                description: p.description().to_string(),
                actions: p.actions(),
            })
            .collect()
    }

    /// 发布事件
    pub fn emit(&self, event: DomainEvent) {
        let _ = self.event_tx.send(event);
    }

    /// 订阅事件
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.event_tx.subscribe()
    }

    /// 关闭所有插件
    pub fn shutdown_all(&mut self) {
        for plugin in &mut self.plugins {
            let _ = plugin.shutdown();
        }
    }

    /// 获取插件数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// 检查域是否已注册
    pub fn has_domain(&self, domain: &str) -> bool {
        self.plugin_index.contains_key(domain)
    }

    /// 异步域调用 — 支持异步插件
    pub async fn call_async(
        &self,
        domain: &str,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        let idx = self.plugin_index.get(domain).ok_or_else(|| DomainError {
            code: "DOMAIN_NOT_FOUND".into(),
            message: format!("Domain '{}' not found", domain),
            recoverable: true,
        })?;
        self.plugins[*idx].call(action, args)
    }
}

impl Default for DomainRegistry {
    fn default() -> Self {
        Self::new()
    }
}
