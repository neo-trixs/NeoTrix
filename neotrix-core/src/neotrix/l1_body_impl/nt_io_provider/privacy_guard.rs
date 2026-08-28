//! # 出网隐私守卫 (Egress Privacy Guard)
//!
//! 修复: 外部模型不应获取 NeoTrix 自身的源代码与对话信息。
//!
//! 机制 (R-P42 强化现有 `scrub_egress_secrets` 节点, 不建平行适配器):
//! 1. **密钥脱敏** — 复用 `nt_shield::redaction::Redactor` 剥离 sk-/AKIA/私钥/JWT。
//! 2. **内部指纹检测** — 扫描出站消息中是否含 NeoTrix 自身源码/KB/对话指纹
//!    (`nt_core_*` 模块、`neotrix-core/` 路径、`ConsciousnessTree`、`VSA HyperCube`、
//!    `kv_store` 等), 这些一旦送出去训练即泄露 NeoTrix 知识产权。
//! 3. **信任分级门控** — 由 `LlmProviderType::data_trust()` 决定处置:
//!    - `Trusted`   (本地): 数据不出设备, 仅脱密钥, 不动内部指纹 (本地可用)。
//!    - `Contracted`(付费云): 脱密钥 + 脱内部指纹, 放行。
//!    - `Untrusted` (免费/代理): 检测到内部指纹 → 默认**阻断** (fail-closed),
//!      除非显式关闭 `block_untrusted_leak` 则退化为脱敏。
//!
//! 调用点: `gateway/execution.rs::call_provider` / `call_provider_stream`,
//! 每一条出网请求必经此门。

use crate::neotrix::l1_body_impl::nt_shield::redaction::Redactor;
use crate::neotrix::l1_body_impl::nt_io_provider::factory::{DataTrust, LlmProviderType};
use crate::neotrix::l1_body_impl::nt_io_provider::types::LlmRequest;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// NeoTrix 内部指纹 — 命中即表明消息可能泄露 NeoTrix 自身源代码/KB/对话。
/// 刻意**不含**项目通用名 "NeoTrix"(用户正常对话会提及, 误伤率高),
/// 只取结构性代码信号, 保持低误报。
const INTERNAL_TOKENS: &[&str] = &[
    // 模块路径 / 源码树
    "neotrix-core/",
    "neotrix_core",
    "crates/neotrix",
    "neotrix_knowledge.db",
    ".neotrix/knowledge.db",
    // 11 域 Rust 模块前缀 (NT-* 域的 nt_* 子系统)
    "nt_core_",
    "nt_mind_",
    "nt_world_",
    "nt_io_",
    "nt_shield_",
    "nt_memory_",
    "nt_act_",
    "nt_meta_",
    "nt_nexus_",
    "nt_repair_",
    "nt_governance_",
    // 意识核心结构
    "ConsciousnessTree",
    "ConsciousnessCore",
    "ConsciousnessCoreHandle",
    "ConsciousnessGalaxy",
    // 知识表示 / 推理引擎
    "VSA HyperCube",
    "E8 Hexagram",
    "SEAL pipeline",
    "MARS System",
    "CoreSnapshot",
    "EvolutionFruit",
    "kv_store",
    // 内部配置文件
    "AGENTS.md",
    "CONTEXT.md",
];

/// 模块级开关 — 默认开启 (safe-by-default), 经 env 或 config 可降级。
/// `PRIVACY_ENABLED`: 总开关 (关则退化为仅脱密钥)。
/// `PRIVACY_BLOCK_UNTRUSTED`: Untrusted 命中内部指纹时是否阻断 (否→脱敏放行)。
static PRIVACY_ENABLED: AtomicBool = AtomicBool::new(true);
static PRIVACY_BLOCK_UNTRUSTED: AtomicBool = AtomicBool::new(true);
static CONFIGURED: OnceLock<()> = OnceLock::new();

/// 运行时配置入口 (main 初始化 / 测试可调用)。
pub fn configure_privacy_guard(enabled: bool, block_untrusted: bool) {
    PRIVACY_ENABLED.store(enabled, Ordering::Relaxed);
    PRIVACY_BLOCK_UNTRUSTED.store(block_untrusted, Ordering::Relaxed);
    let _ = CONFIGURED.set(());
}

/// 懒初始化: 首次调用时合并 env 覆盖 + `NeoTrixConfig` 持久化配置。
/// 仅执行一次, 后续读原子开关零成本。
fn ensure_configured() {
    if CONFIGURED.get().is_some() {
        return;
    }
    // env 覆盖优先 (便于测试/临时降级)
    if let Ok(v) = std::env::var("NEOTRIX_PRIVACY_GUARD") {
        PRIVACY_ENABLED.store(v != "0" && v != "false", Ordering::Relaxed);
    }
    if let Ok(v) = std::env::var("NEOTRIX_PRIVACY_BLOCK") {
        PRIVACY_BLOCK_UNTRUSTED.store(v != "0" && v != "false", Ordering::Relaxed);
    }
    // 持久化配置 (字段缺失 → 维持默认 true)
    let cfg = crate::config::NeoTrixConfig::load();
    if let Some(false) = cfg.privacy_guard {
        PRIVACY_ENABLED.store(false, Ordering::Relaxed);
    }
    if let Some(false) = cfg.privacy_block_untrusted {
        PRIVACY_BLOCK_UNTRUSTED.store(false, Ordering::Relaxed);
    }
    let _ = CONFIGURED.set(());
}

pub fn privacy_guard_enabled() -> bool {
    ensure_configured();
    PRIVACY_ENABLED.load(Ordering::Relaxed)
}

/// 扫描文本中的 NeoTrix 内部指纹, 返回命中的标签列表 (空 = 无泄露)。
pub fn scan_internals(content: &str) -> Vec<&'static str> {
    INTERNAL_TOKENS
        .iter()
        .copied()
        .filter(|tok| content.contains(tok))
        .collect()
}

/// 将内部指纹替换为占位符, 防止 NeoTrix 源码/KB 泄露给外部模型。
pub fn redact_internals(content: &str) -> String {
    let mut out = content.to_string();
    for tok in INTERNAL_TOKENS {
        if out.contains(tok) {
            out = out.replace(tok, "[REDACTED:neotrix-internal]");
        }
    }
    out
}

/// 仅脱密钥 (保留原 `scrub_egress_secrets` 行为, 既有测试继续通过)。
pub fn scrub_egress_secrets(req: &mut LlmRequest) {
    let redactor = Redactor::new();
    for msg in req.messages.iter_mut() {
        if !redactor.find_secrets(&msg.content).is_empty() {
            msg.content = redactor.redact_secrets_only(&msg.content);
        }
    }
}

/// 出网隐私守卫主入口 — 每条出站请求必经。
///
/// `trust` 由 `LlmProviderType::data_trust()` / `trust_from_name()` 推导;
/// `provider_label` 仅用于错误信息展示。
///
/// 返回 `Err(reason)` 表示被阻断 (Untrusted + 命中内部指纹 + block 开启)。
/// 返回 `Ok(())` 表示请求已就地脱敏, 可安全出站。
pub fn egress_privacy_guard(
    req: &mut LlmRequest,
    trust: DataTrust,
    provider_label: &str,
) -> Result<(), String> {
    ensure_configured();
    // 1. 始终脱密钥 (即使总开关关, 密钥也绝不外泄)
    scrub_egress_secrets(req);

    if !PRIVACY_ENABLED.load(Ordering::Relaxed) {
        return Ok(());
    }

    if trust == DataTrust::Trusted {
        // 本地推理: 数据不出设备, 仅脱密钥即可。
        return Ok(());
    }

    // 2. 扫描所有消息 (含 System/User/Tool/Assistant) 与图像数据中的内部指纹
    let mut leaks: Vec<&'static str> = Vec::new();
    for msg in req.messages.iter() {
        leaks.extend(scan_internals(&msg.content));
    }
    if let Some(ref img) = req.image_data {
        leaks.extend(scan_internals(img));
    }
    if let Some(ref c) = req.constraint_json {
        if let Ok(s) = serde_json::to_string(c) {
            leaks.extend(scan_internals(&s));
        }
    }
    leaks.sort_unstable();
    leaks.dedup();

    if leaks.is_empty() {
        return Ok(());
    }

    match trust {
        DataTrust::Contracted => {
            // 付费云: 脱敏内部指纹后放行
            for msg in req.messages.iter_mut() {
                if scan_internals(&msg.content).is_empty() {
                    continue;
                }
                msg.content = redact_internals(&msg.content);
            }
            if let Some(ref mut img) = req.image_data {
                *img = redact_internals(img);
            }
            Ok(())
        }
        DataTrust::Untrusted => {
            if PRIVACY_BLOCK_UNTRUSTED.load(Ordering::Relaxed) {
                // fail-closed: 免费/代理端点绝不放行 NeoTrix 内部代码/对话
                let joined = leaks.join(", ");
                Err(format!(
                    "privacy guard: egress to untrusted provider '{}' would leak NeoTrix internal code/conversation ({}). \
                     Blocked. Use a local (Ollama/vLLM) or paid contracted provider, or set NEOTRIX_PRIVACY_BLOCK=0 to degrade to redaction.",
                    provider_label, joined
                ))
            } else {
                // 显式降级: 退化为脱敏放行
                for msg in req.messages.iter_mut() {
                    if scan_internals(&msg.content).is_empty() {
                        continue;
                    }
                    msg.content = redact_internals(&msg.content);
                }
                if let Some(ref mut img) = req.image_data {
                    *img = redact_internals(img);
                }
                Ok(())
            }
        }
        DataTrust::Trusted => Ok(()),
    }
}

/// 由 provider 注册名 (如 `llm7` 或 `llm7/codestral-latest`) 推导信任分级。
pub fn trust_from_name(registered_name: &str) -> DataTrust {
    let base = registered_name.split('/').next().unwrap_or(registered_name);
    LlmProviderType::from_name(base)
        .map(|t| t.data_trust())
        .unwrap_or(DataTrust::Untrusted) // 未知端点保守视为不可信
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l1_body_impl::nt_io_provider::types::{LlmRequest, Message, Role};

    fn req_with(content: &str) -> LlmRequest {
        let mut r = LlmRequest::new("gpt-4o-mini", content);
        r.messages.clear();
        r.messages
            .push(Message::new(Role::User, content));
        r
    }

    #[test]
    fn test_scan_internals_detects_nt_core() {
        // 良性代码不得误报为内部指纹
        assert!(scan_internals("fn foo() {} let x = 1;").is_empty());
        let hits = scan_internals("see nt_core_consciousness_core.rs for details");
        assert!(hits.contains(&"nt_core_"));
        let hits2 = scan_internals("the ConsciousnessTree produced a fruit");
        assert!(hits2.contains(&"ConsciousnessTree"));
    }

    #[test]
    fn test_redact_internals_replaces_token() {
        let out = redact_internals("import nt_core_consciousness_core as c");
        assert!(!out.contains("nt_core_consciousness_core"));
        assert!(out.contains("[REDACTED:neotrix-internal]"));
    }

    #[test]
    fn test_trusted_local_passthrough() {
        configure_privacy_guard(true, true);
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Ollama.data_trust(), "ollama");
        assert!(res.is_ok());
        // 本地: 内部指纹不脱敏 (保留可用性)
        assert!(r.messages[0].content.contains("nt_core_consciousness_core"));
    }

    #[test]
    fn test_contracted_redacts_internal() {
        configure_privacy_guard(true, true);
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let res = egress_privacy_guard(&mut r, LlmProviderType::OpenAI.data_trust(), "openai");
        assert!(res.is_ok());
        assert!(!r.messages[0].content.contains("nt_core_consciousness_core"));
        assert!(r.messages[0].content.contains("[REDACTED:neotrix-internal]"));
    }

    #[test]
    fn test_untrusted_blocks_internal() {
        configure_privacy_guard(true, true);
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Llm7.data_trust(), "llm7");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("privacy guard"));
    }

    #[test]
    fn test_untrusted_degrade_to_redaction() {
        configure_privacy_guard(true, false);
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Llm7.data_trust(), "llm7");
        assert!(res.is_ok());
        assert!(!r.messages[0].content.contains("nt_core_consciousness_core"));
    }

    #[test]
    fn test_untrusted_clean_passthrough() {
        configure_privacy_guard(true, true);
        let mut r = req_with("write a hello world function in python");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Llm7.data_trust(), "llm7");
        assert!(res.is_ok());
    }

    #[test]
    fn test_secrets_always_scrubbed() {
        configure_privacy_guard(true, true);
        let mut r = req_with("my key is sk-abcdEFGH1234567890abcdef and token AKIA1234567890ABCDEF");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Llm7.data_trust(), "llm7");
        // 含密钥但无内部指纹 → 脱密钥放行
        assert!(res.is_ok());
        assert!(!r.messages[0].content.contains("sk-abcdEFGH"));
        assert!(!r.messages[0].content.contains("AKIA1234567890ABCDEF"));
    }
}
