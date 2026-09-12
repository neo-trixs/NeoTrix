//! GatewayV2 适配器 — 将 GatewayV2 桥接到 ProviderRegistry
//!
//! Adapter 模式: 包装 GatewayV2, 使其满足 LlmProvider trait

use std::sync::Arc;

use async_trait::async_trait;

use crate::l1_action::nt_io::nt_io_provider::common::types::*;
use crate::l1_action::nt_io::nt_io_provider::gateway::GatewayV2;

/// GatewayV2 适配器 — 将 GatewayV2 包装为 LlmProvider
///
/// 允许 ProviderRegistry 接收 GatewayV2 作为统一 provider 后端，
/// 复用其完整的执行管道（熔断器/限流/缓存/故障转移）。
pub struct GatewayV2Adapter {
    gateway: Arc<GatewayV2>,
    name: String,
}

impl GatewayV2Adapter {
    /// 创建适配器
    pub fn new(gateway: Arc<GatewayV2>, name: String) -> Self {
        Self { gateway, name }
    }

    /// 获取 Gateway 引用
    pub fn gateway(&self) -> &GatewayV2 {
        &self.gateway
    }

    /// 获取 provider 名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[async_trait]
impl LlmProvider for GatewayV2Adapter {
    fn data_trust(&self) -> crate::core::nt_core_llm::DataTrust {
        crate::core::nt_core_llm::DataTrust::Trusted
    }

    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.gateway.complete_with_selection(request).await.map(|s| s.response)
    }

    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        self.gateway.stream_complete_with_selection(request).await
    }
}

impl std::fmt::Debug for GatewayV2Adapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayV2Adapter")
            .field("name", &self.name)
            .finish()
    }
}

impl std::fmt::Display for GatewayV2Adapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GatewayV2Adapter({})", self.name)
    }
}
