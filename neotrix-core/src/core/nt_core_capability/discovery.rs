//! 分布式能力发现
//!
//! 支持本地和远程能力发现、注册、同步

use crate::core::nt_core_capability::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 发现协议
#[derive(Debug, Clone)]
pub enum DiscoveryProtocol {
    /// 本地发现
    Local,
    /// UDP广播
    UdpBroadcast,
    /// HTTP发现
    Http,
    /// gRPC发现
    Grpc,
    /// WebSocket发现
    WebSocket,
}

/// 发现配置
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// 发现协议
    pub protocol: DiscoveryProtocol,
    /// 发现间隔
    pub interval: Duration,
    /// 超时时间
    pub timeout: Duration,
    /// 最大节点数
    pub max_nodes: usize,
    /// 启用加密
    pub enable_encryption: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            protocol: DiscoveryProtocol::Local,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_nodes: 100,
            enable_encryption: true,
        }
    }
}

/// 能力节点
#[derive(Debug, Clone)]
pub struct CapabilityNode {
    /// 节点ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 节点地址
    pub address: String,
    /// 节点端口
    pub port: u16,
    /// 节点能力
    pub capabilities: Vec<CapabilityMeta>,
    /// 节点状态
    pub status: NodeStatus,
    /// 最后心跳时间
    pub last_heartbeat: Instant,
    /// 节点延迟
    pub latency_ms: u64,
}

/// 节点状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    Online,
    Offline,
    Degraded,
    Maintenance,
}

/// 发现结果
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// 发现的节点
    pub nodes: Vec<CapabilityNode>,
    /// 发现时间
    pub discovery_time: Instant,
    /// 发现耗时
    pub duration_ms: u64,
    /// 发现的总能力数
    pub total_capabilities: usize,
}

/// 分布式发现器
pub struct DistributedDiscovery {
    /// 配置
    config: DiscoveryConfig,
    /// 已知节点
    nodes: HashMap<String, CapabilityNode>,
    /// 本地注册表
    local_registry: Arc<CapabilityRegistry>,
    /// 发现历史
    history: Vec<DiscoveryResult>,
    /// 最大历史记录
    max_history: usize,
}

impl DistributedDiscovery {
    /// 创建新的发现器
    pub fn new(config: DiscoveryConfig, local_registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            config,
            nodes: HashMap::new(),
            local_registry,
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// 执行发现
    pub fn discover(&mut self) -> DiscoveryResult {
        let start = Instant::now();

        // 根据协议执行发现
        let nodes = match self.config.protocol {
            DiscoveryProtocol::Local => self.discover_local(),
            DiscoveryProtocol::UdpBroadcast => self.discover_udp(),
            DiscoveryProtocol::Http => self.discover_http(),
            DiscoveryProtocol::Grpc => self.discover_grpc(),
            DiscoveryProtocol::WebSocket => self.discover_websocket(),
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let total_capabilities = nodes.iter().map(|n| n.capabilities.len()).sum();

        let result = DiscoveryResult {
            nodes: nodes.clone(),
            discovery_time: start,
            duration_ms,
            total_capabilities,
        };

        // 更新节点
        for node in nodes {
            self.nodes.insert(node.id.clone(), node);
        }

        // 记录历史
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(result.clone());

        result
    }

    /// 本地发现
    fn discover_local(&self) -> Vec<CapabilityNode> {
        // 从本地注册表生成节点
        vec![CapabilityNode {
            id: "local".into(),
            name: "本地节点".into(),
            address: "127.0.0.1".into(),
            port: 8080,
            capabilities: self.local_registry.list_all(),
            status: NodeStatus::Online,
            last_heartbeat: Instant::now(),
            latency_ms: 0,
        }]
    }

    /// UDP发现
    fn discover_udp(&self) -> Vec<CapabilityNode> {
        // 模拟UDP广播发现
        vec![CapabilityNode {
            id: "udp_1".into(),
            name: "UDP节点1".into(),
            address: "192.168.1.100".into(),
            port: 9000,
            capabilities: vec![],
            status: NodeStatus::Online,
            last_heartbeat: Instant::now(),
            latency_ms: 5,
        }]
    }

    /// HTTP发现
    fn discover_http(&self) -> Vec<CapabilityNode> {
        // 模拟HTTP发现
        vec![CapabilityNode {
            id: "http_1".into(),
            name: "HTTP节点1".into(),
            address: "api.example.com".into(),
            port: 443,
            capabilities: vec![],
            status: NodeStatus::Online,
            last_heartbeat: Instant::now(),
            latency_ms: 50,
        }]
    }

    /// gRPC发现
    fn discover_grpc(&self) -> Vec<CapabilityNode> {
        // 模拟gRPC发现
        vec![CapabilityNode {
            id: "grpc_1".into(),
            name: "gRPC节点1".into(),
            address: "grpc.example.com".into(),
            port: 50051,
            capabilities: vec![],
            status: NodeStatus::Online,
            last_heartbeat: Instant::now(),
            latency_ms: 30,
        }]
    }

    /// WebSocket发现
    fn discover_websocket(&self) -> Vec<CapabilityNode> {
        // 模拟WebSocket发现
        vec![CapabilityNode {
            id: "ws_1".into(),
            name: "WebSocket节点1".into(),
            address: "ws.example.com".into(),
            port: 8443,
            capabilities: vec![],
            status: NodeStatus::Online,
            last_heartbeat: Instant::now(),
            latency_ms: 20,
        }]
    }

    /// 获取所有节点
    pub fn get_nodes(&self) -> Vec<&CapabilityNode> {
        self.nodes.values().collect()
    }

    /// 获取在线节点
    pub fn get_online_nodes(&self) -> Vec<&CapabilityNode> {
        self.nodes
            .values()
            .filter(|n| n.status == NodeStatus::Online)
            .collect()
    }

    /// 获取能力所在的节点
    pub fn find_capability_nodes(&self, capability_id: &str) -> Vec<&CapabilityNode> {
        self.nodes
            .values()
            .filter(|n| n.capabilities.iter().any(|c| c.id == capability_id))
            .collect()
    }

    /// 获取最近发现历史
    pub fn recent_history(&self, count: usize) -> Vec<&DiscoveryResult> {
        self.history.iter().rev().take(count).collect()
    }

    /// 清理离线节点
    pub fn cleanup_offline(&mut self, max_age: Duration) {
        let cutoff = Instant::now() - max_age;
        self.nodes.retain(|_, node| node.last_heartbeat > cutoff);
    }

    /// 更新节点状态
    pub fn update_node_status(&mut self, node_id: &str, status: NodeStatus) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.status = status;
        }
    }

    /// 计算节点延迟
    pub fn calculate_latency(&self, node_id: &str) -> Option<u64> {
        self.nodes.get(node_id).map(|n| n.latency_ms)
    }
}

/// 发现管理器
pub struct DiscoveryManager {
    /// 发现器
    pub discovery: DistributedDiscovery,
    /// 发现任务
    tasks: Vec<DiscoveryTask>,
    /// 最大并发任务
    #[allow(dead_code)]
    max_concurrent_tasks: usize,
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// 发现任务
#[derive(Debug, Clone)]
pub struct DiscoveryTask {
    /// 任务ID
    pub id: String,
    /// 任务状态
    pub status: TaskStatus,
    /// 发现的节点
    pub nodes: Vec<CapabilityNode>,
    /// 创建时间
    pub created_at: Instant,
}

impl DiscoveryManager {
    /// 创建新的管理器
    pub fn new(discovery: DistributedDiscovery) -> Self {
        Self {
            discovery,
            tasks: Vec::new(),
            max_concurrent_tasks: 10,
        }
    }

    /// 启动发现任务
    pub fn start_discovery(&mut self) -> String {
        let task_id = format!("discovery_{}", chrono::Utc::now().timestamp());
        let task = DiscoveryTask {
            id: task_id.clone(),
            status: TaskStatus::Running,
            nodes: Vec::new(),
            created_at: Instant::now(),
        };
        self.tasks.push(task);
        task_id
    }

    /// 完成发现任务
    pub fn complete_discovery(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Completed;
            task.nodes = self
                .discovery
                .get_online_nodes()
                .into_iter()
                .cloned()
                .collect();
        }
    }

    /// 获取任务状态
    pub fn get_task_status(&self, task_id: &str) -> Option<&DiscoveryTask> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<&DiscoveryTask> {
        self.tasks.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_discovery() {
        let config = DiscoveryConfig::default();
        let registry = Arc::new(CapabilityRegistry::new());
        let mut discovery = DistributedDiscovery::new(config, registry);

        let result = discovery.discover();
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, "local");
    }

    #[test]
    fn find_capability_nodes() {
        let config = DiscoveryConfig::default();
        let registry = Arc::new(CapabilityRegistry::new());
        let mut discovery = DistributedDiscovery::new(config, registry);

        discovery.discover();
        let nodes = discovery.find_capability_nodes("test");
        assert!(nodes.is_empty());
    }

    #[test]
    fn discovery_manager() {
        let config = DiscoveryConfig::default();
        let registry = Arc::new(CapabilityRegistry::new());
        let discovery = DistributedDiscovery::new(config, registry);
        let mut manager = DiscoveryManager::new(discovery);

        let task_id = manager.start_discovery();
        assert!(!task_id.is_empty());
    }
}
