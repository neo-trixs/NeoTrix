//! # nt_crystal_llm_bridge — L1 异步 Provider → 晶体同步问答
//!
//! 把 `TaskDecomposerDispatcher` 持有的异步 `LlmProvider` 桥成晶体闭环要的
//! 同步 `NtLlmAsk`，让 dispatcher 能一行直调晶体任务闭环：
//!
//! ```ignore
//! dispatcher.crystal_loop_report(goal, &core, "gpt-4o")
//! // goal → 晶体智能拆解 → 本 provider 问答 → JEV 融合 → 后续任务
//! ```
//!
//! ## 依赖方向（DIP）
//! L1 只依赖晶体定义的 **trait 抽象**（`NtLlmAsk`），不依赖晶体引擎具体类型；
//! 与既有 `ReasoningEngineProvider`（L1 定义 trait、L5 实现）镜像对称。
//!
//! ## 运行时桥接
//! - tokio multi-thread 运行时内：`block_in_place + handle.block_on`
//!  （出让 worker，不饿死反应器）。
//! - 无 runtime 的同步上下文（含单元测试）：现场建一次性
//!   `current_thread` 运行时。
//! - 约束：不要在 `current_thread` 运行时的异步任务体内调 `ask`
//!  （`block_in_place` 在该场景会 panic）；dispatcher 的生产路径是
//!   multi-thread，不受影响。
//!
//! ## 置信度启发式
//! LLM 回复不自带置信度，`confidence_for_text` 给结构分：拒答/空/过短降权，
//! 与晶体 J2（Noul `< 0.7` 自动 `needs_review`）联动，拒答自动进复核。
//!
//! # Safety
//! - 无 unsafe (R-P1)；生产代码无 `unwrap/expect/panic`。

use crate::l1_action::nt_core_llm::{LlmProvider, LlmRequest};
use crate::l5_cognition::nt_crystal_core::{NtLlmAsk, NtLlmReply, NtTaskFusionError};
use std::sync::Arc;

/// L1 Provider → 晶体问答桥。
pub struct NtLlmProviderBridge {
    provider: Arc<dyn LlmProvider>,
    model: String,
    max_tokens: u32,
    base_confidence: f64,
}

impl NtLlmProviderBridge {
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            max_tokens: 4096,
            base_confidence: 0.7,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens.max(1);
        self
    }

    pub fn with_base_confidence(mut self, confidence: f64) -> Self {
        self.base_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }
}

/// 回复文本自置信度：空 0；命中拒答 markers −0.3；过短（<8字）−0.2。
pub fn confidence_for_text(text: &str, base: f64) -> f64 {
    let t = text.trim();
    if t.is_empty() {
        return 0.0;
    }
    let lower = t.to_lowercase();
    let refusal = [
        "不知道",
        "无法回答",
        "抱歉",
        "不能",
        "无法提供",
        "as an ai",
        "i don't know",
        "i cannot",
        "unable to",
    ];
    let mut c = base.clamp(0.0, 1.0);
    if refusal.iter().any(|m| lower.contains(m)) {
        c = (c - 0.3).max(0.0);
    }
    if t.chars().count() < 8 {
        c = (c - 0.2).max(0.0);
    }
    c.clamp(0.0, 1.0)
}

/// 在同步上下文驱动异步 future：运行时内出让 worker，无 runtime 则建一次性运行时。
fn block_on_isolated<F, T>(fut: F) -> Result<T, String>
where
    F: std::future::Future<Output = T>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        Ok(tokio::task::block_in_place(|| handle.block_on(fut)))
    } else {
        match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => Ok(rt.block_on(fut)),
            Err(e) => Err(format!("isolated runtime build failed: {e}")),
        }
    }
}

impl NtLlmAsk for NtLlmProviderBridge {
    fn ask(&self, prompt: &str) -> Result<NtLlmReply, NtTaskFusionError> {
        let req = LlmRequest::new(self.model.clone(), prompt).with_max_tokens(self.max_tokens);
        let resp = block_on_isolated(self.provider.complete(&req))
            .map_err(NtTaskFusionError::Llm)?
            .map_err(|e| NtTaskFusionError::Llm(e.to_string()))?;
        let confidence = confidence_for_text(&resp.content, self.base_confidence);
        Ok(NtLlmReply {
            text: resp.content,
            confidence,
            model: resp.model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_core_llm::{
        DataTrust, FinishReason, LlmError, LlmResponse, Usage,
    };

    struct StubProvider {
        text: String,
    }

    #[async_trait::async_trait]
    impl LlmProvider for StubProvider {
        fn set_proxy(&mut self, _proxy_url: &str) {}
        async fn complete_raw(
            &self,
            _request: &LlmRequest,
        ) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                content: self.text.clone(),
                model: "stub-model".to_string(),
                usage: Usage::default(),
                finish_reason: FinishReason::Stop,
                tool_calls: None,
                reasoning: None,
            })
        }
        async fn stream_complete_raw(
            &self,
            _request: &LlmRequest,
        ) -> Result<
            tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>,
            LlmError,
        > {
            Err(LlmError::UnsupportedOperation("no stream in stub".to_string()))
        }
        fn data_trust(&self) -> DataTrust {
            DataTrust::Trusted
        }
    }

    #[test]
    fn test_confidence_normal_keeps_base() {
        assert!((confidence_for_text("这是一个足够长的正常回答内容", 0.7) - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_confidence_empty_is_zero() {
        assert_eq!(confidence_for_text("   ", 0.7), 0.0);
    }

    #[test]
    fn test_confidence_refusal_downshifted() {
        let c = confidence_for_text("抱歉，我不知道这个问题的答案是什么", 0.7);
        assert!((c - 0.4).abs() < 1e-9);
    }

    #[test]
    fn test_confidence_short_downshifted() {
        let c = confidence_for_text("好的", 0.7);
        assert!((c - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_bridge_ask_maps_response() {
        // 无 runtime 上下文：走一次性 current_thread 运行时分支
        let provider: Arc<dyn LlmProvider> = Arc::new(StubProvider {
            text: "晶体桥接成功的完整回答内容".to_string(),
        });
        let bridge = NtLlmProviderBridge::new(provider, "stub-model");
        let reply = bridge.ask("支付如何接入？").unwrap();
        assert_eq!(reply.model, "stub-model");
        assert!(reply.text.contains("桥接成功"));
        assert!((reply.confidence - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_bridge_ask_refusal_triggers_review_band() {
        let provider: Arc<dyn LlmProvider> = Arc::new(StubProvider {
            text: "抱歉，我无法回答这个问题".to_string(),
        });
        let bridge = NtLlmProviderBridge::new(provider, "stub-model");
        let reply = bridge.ask("问一个答不出的问题").unwrap();
        assert!(reply.confidence < 0.7);
    }
}
