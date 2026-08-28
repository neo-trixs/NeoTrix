//! Keyless free-model routing (P0, Cycle 160).
//!
//! 匿名免费端点 (如 llm7/codestral-latest) 无需 API key 即可调用, 但常有速率
//! 限制 (RateLimit, 带 retry_after)。本模块提供:
//!   - `call_provider_backoff`: 单 provider 在 RateLimit 时按 retry_after 退避重试;
//!   - `route_keyless`:         跨 keyless 候选路由, 命中限流/失败则换下一个。
//! 目标: 与 opencode 同等 — 不配 key 也能调通免费模型。

use std::time::Duration;

use super::GatewayV2;
use crate::core::nt_core_llm::{LlmError, LlmRequest, LlmResponse};

impl GatewayV2 {
    /// 有序 keyless 候选 (匿名, 无需 API key, 实测可返回文本)。
    /// 新增匿名端点时在此追加, 即可被 `route_keyless` 自动纳入路由。
    pub fn keyless_candidates() -> &'static [&'static str] {
        &["llm7/codestral-latest"]
    }

    /// 单 provider RateLimit 退避重试: 遇 `RateLimit` 按 `retry_after` (默认 1s)
    /// 退避, 最多重试 `MAX_ATTEMPTS` 次; 其它错误立即透传。
    pub(super) async fn call_provider_backoff(
        &self,
        name: &str,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempt = 0u32;
        loop {
            match self.call_provider(name, request).await {
                Ok(resp) => return Ok(resp),
                Err(LlmError::RateLimit(msg)) => {
                    if attempt >= MAX_ATTEMPTS {
                        return Err(LlmError::RateLimit(msg));
                    }
                    let backoff = parse_retry_after(&msg).unwrap_or(1.0);
                    tokio::time::sleep(Duration::from_secs_f32(backoff)).await;
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 跨 keyless 候选路由: 依次尝试 `keyless_candidates`, 每个候选经
    /// `call_provider_backoff` 退避重试; 全部失败返回最后一个错误。
    pub async fn route_keyless(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let candidates = Self::keyless_candidates();
        let mut last_err = LlmError::Unknown("no keyless candidates configured".into());
        for cand in candidates {
            match self.call_provider_backoff(cand, request).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    last_err = e;
                    continue;
                }
            }
        }
        Err(last_err)
    }
}

/// 从 RateLimit 错误 JSON 提取 `retry_after` (秒)。解析失败回退 1.0s。
fn parse_retry_after(msg: &str) -> Option<f32> {
    let v: serde_json::Value = serde_json::from_str(msg).ok()?;
    v.get("error")?
        .get("retry_after")?
        .as_f64()
        .map(|x| x as f32)
}
