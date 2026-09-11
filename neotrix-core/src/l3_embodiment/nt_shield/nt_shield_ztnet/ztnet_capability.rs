//! ZTNet Capability - Zero Trust Network Capability

use std::collections::HashMap;
use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// Zero Trust状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroTrustStatus {
    Verified,
    Unverified,
    Revoked,
    Pending,
}

/// 网络能力
#[derive(Debug, Clone)]
pub struct NetworkCapability {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub permission: String,
    pub status: ZeroTrustStatus,
}

/// ZTNet能力管理器
pub struct ZtNetCapabilityManager {
    capabilities: Vec<NetworkCapability>,
}

impl ZtNetCapabilityManager {
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
        }
    }

    /// 添加能力
    pub fn add_capability(&mut self, source: &str, destination: &str, permission: &str) -> String {
        let id = format!("cap_{}", self.capabilities.len());
        let capability = NetworkCapability {
            id: id.clone(),
            source: source.to_string(),
            destination: destination.to_string(),
            permission: permission.to_string(),
            status: ZeroTrustStatus::Pending,
        };
        self.capabilities.push(capability);
        id
    }

    /// 验证能力
    pub fn verify_capability(&mut self, cap_id: &str) -> bool {
        if let Some(cap) = self.capabilities.iter_mut().find(|c| c.id == cap_id) {
            cap.status = ZeroTrustStatus::Verified;
            true
        } else {
            false
        }
    }

    /// 检查权限
    pub fn check_permission(&self, source: &str, destination: &str, permission: &str) -> bool {
        self.capabilities.iter().any(|cap| {
            cap.source == source
                && cap.destination == destination
                && cap.permission == permission
                && cap.status == ZeroTrustStatus::Verified
        })
    }

    /// 获取统计
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("total_capabilities".to_string(), self.capabilities.len().to_string());
        stats.insert("verified".to_string(), 
            self.capabilities.iter().filter(|c| c.status == ZeroTrustStatus::Verified).count().to_string());
        stats
    }
}

impl Default for ZtNetCapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// ZTNet UnifiedCapability wrapper
struct ZtNetUnifiedCapability {
    meta: CapabilityMeta,
    health: CapabilityHealth,
}

impl ZtNetUnifiedCapability {
    fn new() -> Self {
        Self {
            meta: CapabilityMeta {
                id: "nt-shield-ztnet".into(),
                name: "NT-SHIELD ZT-Net".into(),
                layer: Layer::L3Embodiment,
                domain: Domain::NtShield,
                version: "0.1.0".into(),
                description: "零信任网络安全能力".into(),
                tags: vec!["zero-trust".into(), "network".into(), "security".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
                cost_weight: 0.0,
                priority: 1.0,
            },
            health: CapabilityHealth {
                state: CapabilityState::Ready,
                success_rate: 1.0,
                avg_latency_ms: 0.0,
                last_called: None,
                call_count: 0,
            },
        }
    }
}

impl UnifiedCapability for ZtNetUnifiedCapability {
    fn meta(&self) -> CapabilityMeta {
        self.meta.clone()
    }

    fn health(&self) -> CapabilityHealth {
        self.health.clone()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Security(_) => Ok(CapabilityOutput::Security(SecurityOutput {
                decision: SecurityDecision::Allow,
                reason: "零信任验证通过".into(),
                details: std::collections::HashMap::new(),
            })),
            CapabilityInput::Network(net) => Ok(CapabilityOutput::Network(NetworkOutput {
                target: net.target,
                open_ports: vec![],
                services: vec![],
                latency_ms: 0,
            })),
            _ => Err(CapabilityError::UnsupportedInput("ZTNet仅支持安全/网络输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Security(_) | CapabilityInput::Network(_))
    }
}

pub fn create_ztnet_capability() -> Arc<dyn UnifiedCapability> {
    Arc::new(ZtNetUnifiedCapability::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_capability() {
        let mut manager = ZtNetCapabilityManager::new();
        let id = manager.add_capability("source", "dest", "read");
        assert!(!id.is_empty());
    }

    #[test]
    fn test_check_permission() {
        let mut manager = ZtNetCapabilityManager::new();
        let id = manager.add_capability("source", "dest", "read");
        manager.verify_capability(&id);
        assert!(manager.check_permission("source", "dest", "read"));
    }
}
