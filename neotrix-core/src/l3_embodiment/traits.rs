//! L3 Embodiment Layer Traits
//!
//! 具身层合约: 物理具身 (nt_physical) + 安全 (nt_shield) + 情感具身 (nt_feel)
//! 吸收来源: PentestCode (持久状态), Blender-MCP/Unity-MCP (3D 工具)

use serde::{Deserialize, Serialize};

/// 安全事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub severity: Severity,
    pub source: String,
    pub details: String,
    pub evidence_chain: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 安全事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEventType {
    Vulnerability,
    Intrusion,
    DataExfiltration,
    PrivilegeEscalation,
    Malware,
    PolicyViolation,
}

/// 严重程度
#[serde(rename_all = "snake_case")]

/// 具身状态 — 持久化 (PentestCode 吸收)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbodimentState {
    pub hosts: Vec<Host>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub credentials: Vec<Credential>,
    pub access_level: AccessLevel,
    pub relationships: Vec<Relationship>,
}

/// 主机
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub ip: String,
    pub hostname: Option<String>,
    pub os: Option<String>,
    pub ports: Vec<Port>,
    pub services: Vec<Service>,
}

/// 端口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub number: u16,
    pub protocol: String,
    pub state: String,
    pub service: Option<String>,
}

/// 服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub version: Option<String>,
    pub banner: Option<String>,
}

/// 漏洞
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub severity: Severity,
    pub status: VulnStatus,
    pub evidence: Vec<String>,
    pub confidence: f64,
}

/// 漏洞状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VulnStatus {
    Suspected,
    Confirmed,
    Exploited,
    Patched,
}

/// 凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub username: String,
    pub hash: Option<String>,
    pub password: Option<String>,
    pub cred_type: String,
    pub domain: Option<String>,
    pub unlocks: Vec<String>,
}

/// 访问级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    None,
    User,
    Admin,
    System,
    DomainAdmin,
}

/// 关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub rel_type: String,
}

/// 具身层核心合约
pub trait EmbodimentLayer: Send + Sync {
    /// 初始化具身层
    fn initialize(&mut self) -> Result<(), String>;

    /// 处理安全事件
    fn handle_security_event(&mut self, event: SecurityEvent) -> Result<(), String>;

    /// 获取当前具身状态 (PentestCode 持久状态吸收)
    fn get_state(&self) -> &EmbodimentState;

    /// 更新具身状态
    fn update_state(&mut self, state: EmbodimentState);

    /// 攻击路径分析 — Dijkstra 最短路径 (PentestCode 吸收)
    fn find_attack_path(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<String>, String>;

    /// 3D 工具集成 — Blender-MCP/Unity-MCP 吸收
    fn integrate_3d_tool(
        &mut self,
        tool: &str,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String>;

    /// 具身状态快照
    fn snapshot(&self) -> EmbodimentSnapshot;
}

/// 具身状态快照
use neotrix_types::shared::Severity;
pub struct EmbodimentSnapshot {
    pub host_count: usize,
    pub vuln_count: usize,
    pub credential_count: usize,
    pub access_level: AccessLevel,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
