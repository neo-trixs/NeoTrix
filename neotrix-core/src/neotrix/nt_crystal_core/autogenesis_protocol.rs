//! AutogenesisProtocol — 资源版本化协议 + 回滚机制
//!
//! 基于 Autogenesis Protocol (arXiv 2604.15034):
//! - 协议注册资源 (Prompt/Agent/Tool/Environment/Memory)
//! - 显式状态 + 生命周期 + 版本化接口
//! - 闭环操作: Propose → Assess → Commit → Audit → Rollback

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 资源类型
// ═══════════════════════════════════════════════════════════════

/// 资源 ID
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceId(pub String);

/// 版本号
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn next_major(&self) -> Self {
        Self { major: self.major + 1, minor: 0, patch: 0 }
    }

    pub fn next_minor(&self) -> Self {
        Self { major: self.major, minor: self.minor + 1, patch: 0 }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 资源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Prompt,
    Agent,
    Tool,
    Environment,
    Memory,
}

/// 资源状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceState {
    Init,
    Active,
    Suspended,
    Terminated,
}

/// 生命周期
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub state: ResourceState,
    pub created_at: u64,
    pub activated_at: Option<u64>,
    pub suspended_at: Option<u64>,
    pub terminated_at: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════
// 协议资源
// ═══════════════════════════════════════════════════════════════

/// 协议注册资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolResource {
    pub id: ResourceId,
    pub resource_type: ResourceType,
    pub state: ResourceState,
    pub lifecycle: Lifecycle,
    pub version: Version,
    pub content: ResourceContent,
    pub metadata: HashMap<String, String>,
}

/// 资源内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub name: String,
    pub description: String,
    pub data: String,
    pub embedding: Vec<f64>,
}

// ═══════════════════════════════════════════════════════════════
// 版本化存储
// ═══════════════════════════════════════════════════════════════

/// 版本化条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedEntry {
    pub version: Version,
    pub resource: ProtocolResource,
    pub committed_at: u64,
    pub commit_message: String,
}

/// 快照 ID
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

/// 资源快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub id: SnapshotId,
    pub resources: HashMap<ResourceId, ProtocolResource>,
    pub created_at: u64,
    pub description: String,
}

/// 版本化资源存储
pub struct VersionedResourceStore {
    /// 当前版本
    pub current: HashMap<ResourceId, ProtocolResource>,
    /// 版本历史
    pub history: HashMap<ResourceId, Vec<VersionedEntry>>,
    /// 快照
    pub snapshots: HashMap<SnapshotId, ResourceSnapshot>,
}

impl VersionedResourceStore {
    pub fn new() -> Self {
        Self {
            current: HashMap::new(),
            history: HashMap::new(),
            snapshots: HashMap::new(),
        }
    }

    /// 注册新资源
    pub fn register(&mut self, resource: ProtocolResource) {
        let id = resource.id.clone();
        let entry = VersionedEntry {
            version: resource.version.clone(),
            resource: resource.clone(),
            committed_at: now_ms(),
            commit_message: "Initial registration".to_string(),
        };
        self.history.entry(id.clone()).or_default().push(entry);
        self.current.insert(id, resource);
    }

    /// 更新资源 (创建新版本)
    pub fn update(
        &mut self,
        id: &ResourceId,
        new_content: ResourceContent,
        commit_message: &str,
    ) -> Result<Version, UpdateError> {
        let resource = self.current.get_mut(id).ok_or(UpdateError::NotFound)?;
        let new_version = resource.version.next_minor();
        resource.version = new_version.clone();
        resource.content = new_content;
        resource.lifecycle.state = ResourceState::Active;

        let entry = VersionedEntry {
            version: new_version.clone(),
            resource: resource.clone(),
            committed_at: now_ms(),
            commit_message: commit_message.to_string(),
        };
        self.history.entry(id.clone()).or_default().push(entry);

        Ok(new_version)
    }

    /// 回滚到指定版本
    pub fn rollback(
        &mut self,
        id: &ResourceId,
        target_version: &Version,
    ) -> Result<ProtocolResource, RollbackError> {
        let history = self.history.get(id).ok_or(RollbackError::NoHistory)?;
        let target = history
            .iter()
            .find(|e| &e.version == target_version)
            .ok_or(RollbackError::VersionNotFound)?;

        let restored = target.resource.clone();
        self.current.insert(id.clone(), restored.clone());
        Ok(restored)
    }

    /// 创建快照
    pub fn snapshot(&mut self, description: &str) -> SnapshotId {
        let id = SnapshotId(uuid::Uuid::new_v4().to_string());
        let snapshot = ResourceSnapshot {
            id: id.clone(),
            resources: self.current.clone(),
            created_at: now_ms(),
            description: description.to_string(),
        };
        self.snapshots.insert(id.clone(), snapshot);
        id
    }

    /// 从快照恢复
    pub fn restore_snapshot(&mut self, snapshot_id: &SnapshotId) -> Result<(), RollbackError> {
        let snapshot = self.snapshots.get(snapshot_id).ok_or(RollbackError::NoHistory)?;
        self.current = snapshot.resources.clone();
        Ok(())
    }

    /// 获取版本历史
    pub fn history(&self, id: &ResourceId) -> Vec<&VersionedEntry> {
        self.history.get(id).map(|h| h.iter().collect()).unwrap_or_default()
    }
}

// ═══════════════════════════════════════════════════════════════
// 审计账本
// ═══════════════════════════════════════════════════════════════

/// 审计操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOpType {
    Propose,
    Assess,
    Commit,
    Rollback,
    Snapshot,
    Restore,
}

/// 审计条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub op_type: AuditOpType,
    pub resource_id: ResourceId,
    pub version: Version,
    pub timestamp: u64,
    pub details: String,
    pub success: bool,
}

/// 审计账本
pub struct AuditLedger {
    pub entries: Vec<AuditEntry>,
}

impl AuditLedger {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn record(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    pub fn for_resource(&self, id: &ResourceId) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.resource_id == *id).collect()
    }
}

// ═══════════════════════════════════════════════════════════════
// Autogenesis Protocol 主引擎
// ═══════════════════════════════════════════════════════════════

/// Autogenesis Protocol — 资源版本化协议
pub struct AutogenesisProtocol {
    pub store: VersionedResourceStore,
    pub audit: AuditLedger,
}

impl AutogenesisProtocol {
    pub fn new() -> Self {
        Self {
            store: VersionedResourceStore::new(),
            audit: AuditLedger::new(),
        }
    }

    /// Propose — 提交改进建议
    pub fn propose(
        &mut self,
        resource: ProtocolResource,
    ) -> ResourceId {
        let id = resource.id.clone();
        self.store.register(resource.clone());
        self.audit.record(AuditEntry {
            op_type: AuditOpType::Propose,
            resource_id: id.clone(),
            version: resource.version,
            timestamp: now_ms(),
            details: format!("Proposed resource: {}", resource.content.name),
            success: true,
        });
        id
    }

    /// Assess — 影响评估
    pub fn assess(
        &mut self,
        id: &ResourceId,
        impact_score: f64,
    ) -> bool {
        let approved = impact_score > 0.5;
        if let Some(resource) = self.store.current.get(id) {
            self.audit.record(AuditEntry {
                op_type: AuditOpType::Assess,
                resource_id: id.clone(),
                version: resource.version.clone(),
                timestamp: now_ms(),
                details: format!("Impact score: {:.2}, approved: {}", impact_score, approved),
                success: approved,
            });
        }
        approved
    }

    /// Commit — 提交变更
    pub fn commit(
        &mut self,
        id: &ResourceId,
        new_content: ResourceContent,
        message: &str,
    ) -> Result<Version, UpdateError> {
        let version = self.store.update(id, new_content, message)?;
        self.audit.record(AuditEntry {
            op_type: AuditOpType::Commit,
            resource_id: id.clone(),
            version: version.clone(),
            timestamp: now_ms(),
            details: message.to_string(),
            success: true,
        });
        Ok(version)
    }

    /// Rollback — 回滚
    pub fn rollback(
        &mut self,
        id: &ResourceId,
        target_version: &Version,
    ) -> Result<ProtocolResource, RollbackError> {
        let resource = self.store.rollback(id, target_version)?;
        self.audit.record(AuditEntry {
            op_type: AuditOpType::Rollback,
            resource_id: id.clone(),
            version: target_version.clone(),
            timestamp: now_ms(),
            details: format!("Rolled back to {}", target_version),
            success: true,
        });
        Ok(resource)
    }
}

// ═══════════════════════════════════════════════════════════════
// 错误类型
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum UpdateError {
    NotFound,
}

#[derive(Debug, Clone)]
pub enum RollbackError {
    NoHistory,
    VersionNotFound,
}

// ═══════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_resource(name: &str) -> ProtocolResource {
        ProtocolResource {
            id: ResourceId(uuid::Uuid::new_v4().to_string()),
            resource_type: ResourceType::Tool,
            state: ResourceState::Init,
            lifecycle: Lifecycle {
                state: ResourceState::Init,
                created_at: now_ms(),
                activated_at: None,
                suspended_at: None,
                terminated_at: None,
            },
            version: Version::new(1, 0, 0),
            content: ResourceContent {
                name: name.to_string(),
                description: "test".to_string(),
                data: "data".to_string(),
                embedding: vec![0.1, 0.2],
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_propose_and_commit() {
        let mut ap = AutogenesisProtocol::new();
        let resource = make_resource("test_tool");
        let id = ap.propose(resource);
        assert!(ap.assess(&id, 0.8));

        let new_content = ResourceContent {
            name: "test_tool_v2".to_string(),
            description: "updated".to_string(),
            data: "data_v2".to_string(),
            embedding: vec![0.3, 0.4],
        };
        let version = ap.commit(&id, new_content, "v2 update").unwrap();
        assert_eq!(version.minor, 1);
    }

    #[test]
    fn test_rollback() {
        let mut ap = AutogenesisProtocol::new();
        let resource = make_resource("test_tool");
        let id = ap.propose(resource);
        let v1 = Version::new(1, 0, 0);

        let v2_content = ResourceContent {
            name: "v2".to_string(),
            description: "v2".to_string(),
            data: "v2".to_string(),
            embedding: vec![],
        };
        ap.commit(&id, v2_content, "v2").unwrap();

        let restored = ap.rollback(&id, &v1).unwrap();
        assert_eq!(restored.version, v1);
    }
}
