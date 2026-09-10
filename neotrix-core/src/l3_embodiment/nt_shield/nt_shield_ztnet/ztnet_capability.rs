//! NT-SHIELD ZT-Net 统一能力接口实现

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// ZT-Net能力实现
pub struct ZtnetCapability {
    meta: CapabilityMeta,
    health: CapabilityHealth,
}

impl ZtnetCapability {
    /// 创建新的ZT-Net能力
    pub fn new() -> Self {
        Self {
            meta: CapabilityMeta {
                id: "nt-shield-ztnet".into(),
                name: "NT-SHIELD ZT-Net".into(),
                layer: Layer::L3Embodiment,
                domain: Domain::NtShield,
                version: "0.1.0".into(),
                description: "零信任网络连接能力".into(),
                tags: vec!["ztnet".into(), "wireguard".into(), "ice".into(), "stun".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
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

impl UnifiedCapability for ZtnetCapability {
    fn meta(&self) -> CapabilityMeta {
        self.meta.clone()
    }

    fn health(&self) -> CapabilityHealth {
        self.health.clone()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Network(net) => {
                // 网络连通性检查
                Ok(CapabilityOutput::Network(NetworkOutput {
                    target: net.target,
                    open_ports: vec![],
                    services: vec![],
                    latency_ms: 0,
                }))
            }
            CapabilityInput::Security(_sec) => {
                // 安全策略检查
                Ok(CapabilityOutput::Security(SecurityOutput {
                    decision: SecurityDecision::Allow,
                    reason: "Default allow for testing".into(),
                    details: std::collections::HashMap::new(),
                }))
            }
            _ => Err(CapabilityError::UnsupportedInput("不支持的输入类型".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Network(_) | CapabilityInput::Security(_))
    }
}

/// 创建ZT-Net能力实例
pub fn create_ztnet_capability() -> Arc<dyn UnifiedCapability> {
    Arc::new(ZtnetCapability::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ztnet_capability_meta() {
        let cap = ZtnetCapability::new();
        let meta = cap.meta();
        assert_eq!(meta.id, "nt-shield-ztnet");
        assert_eq!(meta.layer, Layer::L3Embodiment);
        assert_eq!(meta.domain, Domain::NtShield);
    }

    #[test]
    fn ztnet_network_check() {
        let cap = ZtnetCapability::new();
        let input = CapabilityInput::Network(NetworkInput {
            target: "192.168.1.1".into(),
            ports: vec![80, 443],
            timeout_ms: 1000,
        });

        let output = cap.execute(input).unwrap();
        match output {
            CapabilityOutput::Network(net) => {
                assert_eq!(net.target, "192.168.1.1");
            }
            _ => panic!("Expected network output"),
        }
    }
}
