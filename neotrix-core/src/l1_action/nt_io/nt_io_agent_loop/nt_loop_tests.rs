//! NT-IO AgentLoop tests — 循环单测（纯搬移，行为零变更）。
//!
//! 内容：`MockCalc`/`ScriptedLlm` 可编程 Mock + `turn`/`turn_stream`/
//! 治理/压缩/蒸馏 20 单测。缺失导入在此补显式 `use`（不依赖父文件私有 `use`）。

use super::nt_loop_step::distill_output;
use super::nt_loop_types::{AgentLoop, COMPACTION_MIN_MESSAGES, COMPACTION_THRESHOLD_RATIO};
use super::*;
use crate::l0_substrate::nt_core_traits::{NativeTool, ToolDef, ToolOutput};
use crate::l1_action::nt_io::nt_io_provider::LlmResponse;
use crate::l1_action::nt_io::nt_io_provider::context_budget::{estimate_messages_tokens, estimate_tokens};
use crate::l1_action::nt_io::nt_io_provider::types::{FinishReason, LlmError, LlmProvider, LlmRequest, Message, Role, ToolCallInfo};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, Mutex};

// ── Mock 工具 ─────────────────────────────────────────────────────

struct MockCalc {
    calls: Arc<Mutex<Vec<String>>>,
}

impl NativeTool for MockCalc {
    fn id(&self) -> &str {
        "calc"
    }
    fn description(&self) -> &str {
        "Mock calculator"
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({"type": "object", "properties": {"expr": {"type": "string"}}})
    }
    fn capability_tags(&self) -> Vec<&'static str> {
        vec!["compute"]
    }
    fn execute(&self, args: &Value) -> Result<ToolOutput, String> {
        let expr = args["expr"].as_str().unwrap_or("").to_string();
        self.calls
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(expr.clone());
        if expr == "1+1" {
            Ok(ToolOutput {
                success: true,
                content: "2".to_string(),
            })
        } else {
            Err(format!("cannot compute {}", expr))
        }
    }
}

fn tool_def(id: &str) -> ToolDef {
    ToolDef {
        name: id.to_string(),
        description: "tool".to_string(),
        input_schema: serde_json::json!({"type": "object"}),
    }
}

// ── 可编程 Mock LLM ───────────────────────────────────────────────

/// 按预设脚本返回响应序列；`tool_calls_seq` 控制每轮是否返回工具调用。
struct ScriptedLlm {
    /// (content, finish_reason, tool_calls) 序列，每次调用 pop 第一个。
    script: Arc<Mutex<Vec<(String, FinishReason, Vec<ToolCallInfo>)>>>,
    /// 记录每次请求携带的工具数量。
    seen_tools: Arc<Mutex<Vec<usize>>>,
}

#[async_trait]
impl LlmProvider for ScriptedLlm {
fn set_proxy(&mut self, _proxy_url: &str) {}
fn data_trust(&self) -> crate::l1_action::nt_core_llm::DataTrust {
    crate::l1_action::nt_core_llm::DataTrust::Trusted
}

    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.seen_tools
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(request.tools.len());
        let mut script = self.script.lock().unwrap_or_else(|e| e.into_inner());
        if script.is_empty() {
            return Ok(LlmResponse {
                content: "done".into(),
                model: "mock".into(),
                usage: Default::default(),
                finish_reason: FinishReason::Stop,
                tool_calls: None,
                reasoning: None,
            });
        }
        let (content, fr, calls) = script.remove(0);
        Ok(LlmResponse {
            content,
            model: "mock".into(),
            usage: Default::default(),
            finish_reason: fr,
            tool_calls: Some(calls),
         reasoning: None,})
    }

    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        use tokio::sync::mpsc;
        self.seen_tools
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(request.tools.len());
        let mut script = self.script.lock().unwrap_or_else(|e| e.into_inner());
        let (tx, rx) = mpsc::channel(16);
        if script.is_empty() {
            let _ = tx.try_send(Ok(LlmResponse {
                content: "done".into(),
                model: "mock".into(),
                usage: Default::default(),
                finish_reason: FinishReason::Stop,
                tool_calls: None,
                reasoning: None,
            }));
            return Ok(rx);
        }
        let (content, fr, calls) = script.remove(0);
        // 模拟逐 chunk 推送：按 1 字符切块，保留 finish_reason 与 tool_calls。
        for ch in content.chars() {
            let resp = LlmResponse {
                content: ch.to_string(),
                model: "mock".into(),
                usage: Default::default(),
                finish_reason: FinishReason::Stop,
                tool_calls: None,
             reasoning: None,};
            if tx.try_send(Ok(resp)).is_err() {
                break;
            }
        }
        let final_resp = LlmResponse {
            content: String::new(),
            model: "mock".into(),
            usage: Default::default(),
            finish_reason: fr,
            tool_calls: Some(calls),
         reasoning: None,};
        let _ = tx.try_send(Ok(final_resp));
        Ok(rx)
    }
}

fn tool_call(name: &str, id: &str, args: &str) -> ToolCallInfo {
    ToolCallInfo {
        id: id.to_string(),
        name: name.to_string(),
        arguments: args.to_string(),
        call_type: Some("function".to_string()),
        function: Some(super::super::nt_io_provider::types::ToolCallFunction {
            name: name.to_string(),
            arguments: args.to_string(),
        }),
    }
}

fn backend_with(
    script: Vec<(String, FinishReason, Vec<ToolCallInfo>)>,
) -> (Arc<ScriptedLlm>, Arc<Mutex<Vec<usize>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let llm = ScriptedLlm {
        script: Arc::new(Mutex::new(script)),
        seen_tools: seen.clone(),
    };
    (Arc::new(llm), seen)
}

// ── 测试 ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_turn_simple_stop() {
    let (llm, _seen) = backend_with(vec![("hello there".into(), FinishReason::Stop, vec![])]);
    let mut loop_ = AgentLoop::new(llm, "mock", "You are NeoTrix.");
    let out = loop_.turn("hi").await.expect("turn ok");
    assert_eq!(out, "hello there");
    assert_eq!(loop_.history_len(), 3); // System + User + Assistant
}

#[tokio::test]
async fn test_turn_without_system_prompt() {
    let (llm, _seen) = backend_with(vec![("ok".into(), FinishReason::Stop, vec![])]);
    let mut loop_ = AgentLoop::new(llm, "mock", "");
    let out = loop_.turn("q").await.expect("turn ok");
    assert_eq!(out, "ok");
    assert_eq!(loop_.history_len(), 2); // User + Assistant
}

#[tokio::test]
async fn test_turn_executes_tool_and_continues() {
    // 第 1 轮：模型请求 calc(1+1)；第 2 轮：模型给出最终答案。
    let (llm, seen) = backend_with(vec![
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("calc", "call_1", r#"{"expr":"1+1"}"#)],
        ),
        ("result is 2".into(), FinishReason::Stop, vec![]),
    ]);
    let calc_calls = Arc::new(Mutex::new(Vec::new()));
    let calc = MockCalc {
        calls: calc_calls.clone(),
    };
    let mut loop_ = AgentLoop::new(llm, "mock", "sys")
        .with_tools(vec![Box::new(calc)])
        .with_max_tool_rounds(4);

    let out = loop_.turn("compute 1+1").await.expect("turn ok");
    assert_eq!(out, "result is 2");

    // 工具确实被执行了。
    assert_eq!(
        calc_calls
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_slice(),
        &["1+1".to_string()]
    );
    // 工具日志有记录。
    assert_eq!(loop_.tool_log.len(), 1);
    assert_eq!(loop_.tool_log[0].name, "calc");
    assert!(loop_.tool_log[0].success);
    assert_eq!(loop_.tool_log[0].output, "2");
    // LLM 两次调用都拿到了工具定义。
    assert_eq!(
        seen.lock().unwrap_or_else(|e| e.into_inner()).as_slice(),
        &[1, 1]
    );
    // 历史：System + User + Assistant(tool_calls) + Tool + Assistant(final) = 5
    assert_eq!(loop_.history_len(), 5);
}

#[tokio::test]
async fn test_turn_tool_error_surfaces() {
    let (llm, _seen) = backend_with(vec![
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("calc", "call_1", r#"{"expr":"2+2"}"#)],
        ),
        ("cannot compute".into(), FinishReason::Stop, vec![]),
    ]);
    let calc = MockCalc {
        calls: Arc::new(Mutex::new(Vec::new())),
    };
    let mut loop_ = AgentLoop::new(llm, "mock", "").with_tools(vec![Box::new(calc)]);
    let out = loop_.turn("compute 2+2").await.expect("turn ok");
    assert_eq!(out, "cannot compute");
    assert_eq!(loop_.tool_log.len(), 1);
    assert!(!loop_.tool_log[0].success);
    assert!(loop_.tool_log[0].output.contains("TOOL_ERROR"));
}

#[tokio::test]
async fn test_turn_unknown_tool_surfaces_error() {
    let (llm, _seen) = backend_with(vec![
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("ghost", "call_9", "{}")],
        ),
        ("recovered".into(), FinishReason::Stop, vec![]),
    ]);
    let mut loop_ = AgentLoop::new(llm, "mock", "");
    let out = loop_.turn("use ghost").await.expect("turn ok");
    assert_eq!(out, "recovered");
    assert_eq!(loop_.tool_log.len(), 1);
    assert!(!loop_.tool_log[0].success);
    assert!(loop_.tool_log[0].output.contains("Unknown tool"));
}

#[tokio::test]
async fn test_turn_loop_cap_prevents_infinite() {
    // 模型永远请求工具 → 达到上限必须报错而非死循环。
    let (llm, _seen) = backend_with(vec![
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("calc", "c1", r#"{"expr":"1+1"}"#)],
        ),
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("calc", "c2", r#"{"expr":"1+1"}"#)],
        ),
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("calc", "c3", r#"{"expr":"1+1"}"#)],
        ),
    ]);
    let calc = MockCalc {
        calls: Arc::new(Mutex::new(Vec::new())),
    };
    let mut loop_ = AgentLoop::new(llm, "mock", "")
        .with_tools(vec![Box::new(calc)])
        .with_max_tool_rounds(3);
    let err = loop_.turn("loop").await.err().expect("must error");
    assert!(err.to_string().contains("max_tool_rounds"), "got: {}", err);
}

#[tokio::test]
async fn test_turn_empty_tool_calls_treated_as_stop() {
    let (llm, _seen) =
        backend_with(vec![("finished anyway".into(), FinishReason::Tool, vec![])]);
    let mut loop_ = AgentLoop::new(llm, "mock", "");
    let out = loop_.turn("q").await.expect("turn ok");
    assert_eq!(out, "finished anyway");
}

#[tokio::test]
async fn test_turn_request_carries_tools_and_history() {
    // 验证 build_request 把工具定义 + 全部历史传给 LLM。
    let (llm, seen) = backend_with(vec![("ok".into(), FinishReason::Stop, vec![])]);
    let calc = MockCalc {
        calls: Arc::new(Mutex::new(Vec::new())),
    };
    let mut loop_ = AgentLoop::new(llm, "mock", "sys").with_tools(vec![Box::new(calc)]);
    let _ = loop_.turn("first message").await;
    let _ = loop_.turn("second message").await;

    let seen = seen.lock().unwrap_or_else(|e| e.into_inner());
    // 两次调用都应携带 1 个工具。
    assert_eq!(seen.as_slice(), &[1, 1]);

    // 消息历史应累积（System+2×(User+Assistant) = 5）。
    let roles: Vec<Role> = loop_.history().iter().map(|m| m.role).collect();
    assert_eq!(roles.len(), 5);
    assert_eq!(roles[0], Role::System);
}

#[tokio::test]
async fn test_trim_history_keeps_system() {
    let (llm, _seen) = backend_with(vec![("ok".into(), FinishReason::Stop, vec![])]);
    let mut loop_ = AgentLoop::new(llm, "mock", "sys").with_max_history(4);
    for i in 0..5 {
        let _ = loop_.turn(&format!("msg {}", i)).await;
    }
    assert!(loop_.history_len() <= 4);
    assert_eq!(loop_.history()[0].role, Role::System);
}

#[tokio::test]
async fn test_turn_stream_simple_stop() {
    let (llm, seen) = backend_with(vec![("streamed hello".into(), FinishReason::Stop, vec![])]);
    let mut loop_ = AgentLoop::new(llm, "mock", "sys");
    let mut chunks: Vec<String> = Vec::new();
    let out = loop_
        .turn_stream(
            "hi",
            |c| {
                chunks.push(c.to_string());
                true
            },
            |_, _| {},
        )
        .await
        .expect("turn_stream ok");
    assert_eq!(out, "streamed hello");
    // 逐字符 chunk：11 chars → 11 chunks + final。
    assert!(
        chunks.len() >= 11,
        "expected >=11 chunks, got {}",
        chunks.len()
    );
    let joined: String = chunks.concat();
    assert_eq!(joined, "streamed hello");
    assert_eq!(
        seen.lock().unwrap_or_else(|e| e.into_inner()).as_slice(),
        &[0]
    );
}

#[tokio::test]
async fn test_turn_stream_executes_tool_and_continues() {
    let (llm, _seen) = backend_with(vec![
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("calc", "call_1", r#"{"expr":"1+1"}"#)],
        ),
        ("answer is 2".into(), FinishReason::Stop, vec![]),
    ]);
    let calc = MockCalc {
        calls: Arc::new(Mutex::new(Vec::new())),
    };
    let mut loop_ = AgentLoop::new(llm, "mock", "sys")
        .with_tools(vec![Box::new(calc)])
        .with_max_tool_rounds(4);

    let mut tool_seen: Vec<(String, String)> = Vec::new();
    let out = loop_
        .turn_stream(
            "compute",
            |_| true,
            |call, output| tool_seen.push((call.name.clone(), output.content.clone())),
        )
        .await
        .expect("turn_stream ok");
    assert_eq!(out, "answer is 2");
    assert_eq!(tool_seen.len(), 1);
    assert_eq!(tool_seen[0].0, "calc");
    assert_eq!(tool_seen[0].1, "2");
    assert_eq!(loop_.tool_log.len(), 1);
    assert!(loop_.tool_log[0].success);
}

#[tokio::test]
async fn test_turn_stream_cancel_stops_generation() {
    // 模型永远请求工具 → 若 on_token 立即返回 false，应取消并返回已累积文本。
    let (llm, _seen) = backend_with(vec![(
        "partial".into(),
        FinishReason::Tool,
        vec![tool_call("calc", "c1", r#"{"expr":"1+1"}"#)],
    )]);
    let calc = MockCalc {
        calls: Arc::new(Mutex::new(Vec::new())),
    };
    let mut loop_ = AgentLoop::new(llm, "mock", "sys").with_tools(vec![Box::new(calc)]);
    // on_token 第一次调用返回 false → 取消。
    let mut first = true;
    let out = loop_
        .turn_stream(
            "q",
            |_| {
                let keep = first;
                first = false;
                keep
            },
            |_, _| {},
        )
        .await
        .expect("cancel is not error");
    // 取消后返回累积内容（至少首 chunk）。
    assert!(out.chars().count() >= 1);
    // 工具不应被真正执行（取消发生在工具轮之前…但脚本第二项缺失，工具轮会因脚本空而 stop）。
    assert!(loop_.tool_log.len() <= 1);
}

#[test]
fn test_tool_def_helper() {
    let _ = tool_def("x");
}

// ── G27 输出纪律治理接线: 每条最终输出经 OutputGovernor 检查 ──────
#[tokio::test]
async fn test_emit_final_wires_governance() {
    
    let mut loop_ = AgentLoop::new(
        Arc::new(ScriptedLlm {
            script: Arc::new(Mutex::new(vec![(
                "def add(a,b): return a+b".to_string(),
                FinishReason::Stop,
                Vec::new(),
            )])),
            seen_tools: Arc::new(Mutex::new(Vec::new())),
        }),
        "mock",
        "",
    );
    let out = loop_.turn("计算 1+1").await.expect("turn ok");
    let report = loop_.last_governance().expect("governance attached");
    assert!(report.rule_results.len() >= 10, "all 10 rules run");
    assert_eq!(out.trim(), "def add(a,b): return a+b");
    assert!(!report.fixes_applied.is_empty() == false);
    assert!(report.overall_score > 0, "clean sample scores > 0");
}

#[tokio::test]
async fn test_governance_reports_violations_but_keeps_output() {
    let mut loop_ = AgentLoop::new(
        Arc::new(ScriptedLlm {
            script: Arc::new(Mutex::new(vec![(
                "答案：42。\n抱歉，这只是一个占位。TODO 补充细节。".to_string(),
                FinishReason::Stop,
                Vec::new(),
            )])),
            seen_tools: Arc::new(Mutex::new(Vec::new())),
        }),
        "mock",
        "",
    );
    let out = loop_.turn("question").await.expect("turn ok");
    let report = loop_.last_governance().expect("governance attached");
    assert!(!report.violations.is_empty(), "violations surfaced: {:?}", report.violations);
    assert_eq!(out, "答案：42。\n抱歉，这只是一个占位。TODO 补充细节。");
}

// ── P1-B2 双相 compaction ────────────────────────────────────────

#[tokio::test]
async fn test_compaction_replaces_old_turns_at_high_water() {
    // 摘要调用脚本: 第一个响应命中 compaction, 返回摘要文本。
    let (backend, _seen) = backend_with(vec![(
        "【压缩结果】goal: answer 1+1".to_string(),
        FinishReason::Stop,
        vec![],
    )]);
    let mut loop_ = AgentLoop::new(backend, "mock", "You are sys.");
    // 灌入 12 条中等消息 (超过 COMPACTION_MIN_MESSAGES=8)。
    for i in 0..12 {
        loop_.messages.push(Message::new(
            Role::User,
            &format!("turn {i}: {}", "x".repeat(300)),
        ));
    }
    let before = loop_.messages.len();
    assert!(before >= COMPACTION_MIN_MESSAGES + 2, "must be compactable");
    // 动态推导预算 (避免 tiktoken 离线/在线口径差异): 预算 = 当前 token,
    // 阈值 (90%) = 0.9× 当前 token, 必命中。
    let tokens_before = estimate_messages_tokens(&loop_.messages);
    loop_ = loop_.with_context_token_budget(tokens_before);
    assert!(
        estimate_messages_tokens(&loop_.messages)
            >= (loop_.context_token_budget as f64 * COMPACTION_THRESHOLD_RATIO) as usize,
        "precondition: over 90% budget ({} vs {})",
        tokens_before,
        loop_.context_token_budget
    );

    loop_.maybe_compact_context().await;

    assert!(loop_.messages.len() < before, "old turns collapsed");
    let has_summary = loop_.messages.iter().any(|m| {
        m.content.contains("【上下文摘要") && m.content.contains("【压缩结果】")
    });
    assert!(has_summary, "summary message present: {:?}", loop_.messages);
    // System 头仍在原位。
    assert_eq!(loop_.messages[0].role, Role::System);
}

#[tokio::test]
async fn test_compaction_skipped_below_threshold() {
    let (backend, _seen) = backend_with(vec![(
        "unused".to_string(),
        FinishReason::Stop,
        vec![],
    )]);
    let mut loop_ = AgentLoop::new(backend, "mock", "sys");
    // 预算拉满, 消息少, 不该触发压缩。
    loop_ = loop_.with_context_token_budget(10_000);
    for i in 0..5 {
        loop_.messages.push(Message::new(
            Role::User,
            &format!("turn {i}: {}", "x".repeat(50)),
        ));
    }
    let before = loop_.messages.len();
    loop_.maybe_compact_context().await;
    assert_eq!(loop_.messages.len(), before, "no compaction below threshold");
}

// ── P2 可逆输出蒸馏 (repowise absorbed 2026-08-19) ──────────
#[test]
fn test_distill_returns_original_when_under_budget() {
    let content = "ok\nall good\n";
    assert_eq!(distill_output(content, 500), content, "未超预算应原样返回");
}

#[test]
fn test_distill_errors_first_and_refs() {
    // 高熵超预算长输出 (tiktoken 压缩不友好): 错误行应前置,
    // body 中段标记 [ref#1], 尾部 exit code 保留。
    let mut content = String::from("line-ok-1\nline-ok-2\nline-ok-3\n");
    for i in 0..200 {
        content.push_str(&format!("body-line-{i}-x0q9z7w5v3r1t8m2\n"));
    }
    content.push_str("ERROR: build failed at module src/main.rs:42\n");
    content.push_str("exit code 1\n");
    let orig_tokens = estimate_tokens(&content);
    assert!(orig_tokens > 120, "原文必须超预算才触发蒸馏, got {orig_tokens} tokens");
    let out = distill_output(&content, 120);
    let out_tokens = estimate_tokens(&out);
    assert!(out_tokens <= 120, "蒸馏后必须不超预算, got {out_tokens} tokens");
    assert!(
        out_tokens < orig_tokens,
        "蒸馏必须显著小于原文 ({out_tokens} vs {orig_tokens})"
    );
    assert!(out.contains("ERROR"), "错误行必须前置 (errors-first)");
    assert!(out.contains("[ref#1]"), "省略区必须有 [ref#N] 标记");
    let err_pos = out.find("ERROR").unwrap();
    let ref_pos = out.find("[ref#1]").unwrap();
    assert!(err_pos < ref_pos, "错误段必须在 body 省略标记之前");
    assert!(out.contains("exit code 1"), "尾部 exit code 必须保留");
}

#[test]
fn test_distill_tail_retained_when_no_error() {
    // 高熵无错误长输出: body 中段标记 + 尾部摘要保留。
    let mut content = String::new();
    for i in 0..150 {
        content.push_str(&format!("plain-{i}-a1b2c3d4e5f6g7h8\n"));
    }
    content.push_str("SUMMARY: done in 3.2s\n");
    let orig_tokens = estimate_tokens(&content);
    assert!(orig_tokens > 80, "原文必须超预算才触发蒸馏, got {orig_tokens} tokens");
    let out = distill_output(&content, 80);
    let out_tokens = estimate_tokens(&out);
    assert!(out_tokens <= 80, "蒸馏后必须不超预算, got {out_tokens} tokens");
    assert!(
        out_tokens < orig_tokens,
        "蒸馏必须显著小于原文 ({out_tokens} vs {orig_tokens})"
    );
    assert!(out.contains("[ref#1]"), "body 中段应标记");
    assert!(out.contains("SUMMARY: done in 3.2s"), "尾部摘要应保留");
}
