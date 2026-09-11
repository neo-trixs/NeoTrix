//! NT-WORLD Asset Map 统一能力接口实现

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// Asset Map能力实现
pub struct AssetMapCapability {
    meta: CapabilityMeta,
    health: CapabilityHealth,
}

impl AssetMapCapability {
    /// 创建新的Asset Map能力
    pub fn new() -> Self {
        Self {
            meta: CapabilityMeta {
                id: "nt-world-asset-map".into(),
                name: "NT-WORLD Asset Map".into(),
                layer: Layer::L2Perception,
                domain: Domain::NtWorld,
                version: "0.1.0".into(),
                description: "互联网资产测绘能力".into(),
                tags: vec!["asset".into(), "scan".into(), "fingerprint".into()],
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

impl UnifiedCapability for AssetMapCapability {
    fn meta(&self) -> CapabilityMeta {
        self.meta.clone()
    }

    fn health(&self) -> CapabilityHealth {
        self.health.clone()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Asset(asset) => {
                // 简单查询验证
                if asset.query.is_empty() {
                    return Err(CapabilityError::UnsupportedInput("查询不能为空".into()));
                }

                // 返回模拟结果
                Ok(CapabilityOutput::Asset(AssetOutput {
                    assets: vec![],
                    total: 0,
                }))
            }
            CapabilityInput::Network(net) => {
                // 端口扫描
                Ok(CapabilityOutput::Network(NetworkOutput {
                    target: net.target,
                    open_ports: vec![],
                    services: vec![],
                    latency_ms: 0,
                }))
            }
            _ => Err(CapabilityError::UnsupportedInput("不支持的输入类型".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Asset(_) | CapabilityInput::Network(_))
    }
}

/// 创建Asset Map能力实例
pub fn create_asset_map_capability() -> Arc<dyn UnifiedCapability> {
    Arc::new(AssetMapCapability::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_map_capability_meta() {
        let cap = AssetMapCapability::new();
        let meta = cap.meta();
        assert_eq!(meta.id, "nt-world-asset-map");
        assert_eq!(meta.layer, Layer::L2Perception);
        assert_eq!(meta.domain, Domain::NtWorld);
    }

    #[test]
    fn asset_map_query() {
        let cap = AssetMapCapability::new();
        let input = CapabilityInput::Asset(AssetInput {
            query: r#"port="80""#.into(),
            asset_type: None,
            limit: 10,
        });

        let output = cap.execute(input).unwrap();
        match output {
            CapabilityOutput::Asset(asset) => {
                assert_eq!(asset.total, 0);
            }
            _ => panic!("Expected asset output"),
        }
    }
}
