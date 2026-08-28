//! # 出网隐私守卫 (Egress Privacy Guard)
//!
//! 修复: 外部模型不应获取 NeoTrix 自身的源代码与对话信息。
//!
//! 机制 (R-P42: neotrix 层不建平行适配器, 守卫逻辑下沉 `core::nt_core_llm`):
//! 1. **密钥脱敏** — 由 core 层守卫统一剥离 sk-/AKIA/私钥/JWT (neotrix 委托, 单一逻辑源)。
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
    // 每调用决策: env 覆盖优先于持久化配置/默认值, 便于运行时降级与测试隔离。
    let enabled = match std::env::var("NEOTRIX_PRIVACY_GUARD") {
        Ok(v) => v != "0" && v != "false",
        Err(_) => PRIVACY_ENABLED.load(Ordering::Relaxed),
    };
    let block_untrusted = match std::env::var("NEOTRIX_PRIVACY_BLOCK") {
        Ok(v) => v != "0" && v != "false",
        Err(_) => PRIVACY_BLOCK_UNTRUSTED.load(Ordering::Relaxed),
    };
    if !enabled {
        return Ok(());
    }
    // P1 收敛: 单一逻辑源 = core::nt_core_llm::egress_privacy_guard (扫描/脱敏/阻断决策);
    // neotrix 层仅保留启用开关与 untrusted 降级策略, 不再复制守卫逻辑。
    let _ = provider_label;
    match crate::core::nt_core_llm::egress_privacy_guard(req, trust) {
        Ok(()) => Ok(()),
        Err(_) if !block_untrusted && trust == DataTrust::Untrusted => {
            // 显式降级: 不阻断, 改为脱敏内部指纹后放行
            for m in req.messages.iter_mut() {
                if !scan_internals(&m.content).is_empty() {
                    m.content = redact_internals(&m.content);
                }
            }
            if let Some(ref mut img) = req.image_data {
                *img = redact_internals(img);
            }
            Ok(())
        }
        Err(e) => Err(e),
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
    use std::sync::Mutex;

    // 守卫决策读取全局/环境变量, 并行测试会相互干扰; 串行化本模块所有测试。
    static PRIVACY_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn req_with(content: &str) -> LlmRequest {
        let mut r = LlmRequest::new("gpt-4o-mini", content);
        r.messages.clear();
        r.messages
            .push(Message::new(Role::User, content));
        r
    }

    #[test]
    fn test_scan_internals_detects_nt_core() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        // 良性代码不得误报为内部指纹
        assert!(scan_internals("fn foo() {} let x = 1;").is_empty());
        let hits = scan_internals("see nt_core_consciousness_core.rs for details");
        assert!(hits.contains(&"nt_core_"));
        let hits2 = scan_internals("the ConsciousnessTree produced a fruit");
        assert!(hits2.contains(&"ConsciousnessTree"));
    }

    #[test]
    fn test_redact_internals_replaces_token() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        let out = redact_internals("import nt_core_consciousness_core as c");
        assert!(!out.contains("nt_core_consciousness_core"));
        assert!(out.contains("[REDACTED:neotrix-internal]"));
    }

    #[test]
    fn test_trusted_local_passthrough() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        configure_privacy_guard(true, true);
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Ollama.data_trust(), "ollama");
        assert!(res.is_ok());
        // 本地: 内部指纹不脱敏 (保留可用性)
        assert!(r.messages[0].content.contains("nt_core_consciousness_core"));
    }

    #[test]
    fn test_contracted_redacts_internal() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        configure_privacy_guard(true, true);
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let res = egress_privacy_guard(&mut r, LlmProviderType::OpenAI.data_trust(), "openai");
        assert!(res.is_ok());
        assert!(!r.messages[0].content.contains("nt_core_consciousness_core"));
        assert!(r.messages[0].content.contains("[REDACTED:neotrix-internal]"));
    }

    #[test]
    fn test_untrusted_blocks_internal() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        configure_privacy_guard(true, true);
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let trust = LlmProviderType::Llm7.data_trust();
        let res = egress_privacy_guard(&mut r, trust, "llm7");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("privacy guard"));
    }

    #[test]
    fn test_untrusted_degrade_to_redaction() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        // 显式降级: 经 env 覆盖关闭 untrusted 阻断 (per-call 决策, 不受全局缓存影响)
        std::env::set_var("NEOTRIX_PRIVACY_BLOCK", "0");
        let mut r = req_with("read nt_core_consciousness_core.rs and tell me");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Llm7.data_trust(), "llm7");
        std::env::remove_var("NEOTRIX_PRIVACY_BLOCK");
        assert!(res.is_ok());
        assert!(!r.messages[0].content.contains("nt_core_consciousness_core"));
    }

    #[test]
    fn test_untrusted_clean_passthrough() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        configure_privacy_guard(true, true);
        let mut r = req_with("write a hello world function in python");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Llm7.data_trust(), "llm7");
        assert!(res.is_ok());
    }

    #[test]
    fn test_secrets_always_scrubbed() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        configure_privacy_guard(true, true);
        let mut r = req_with("my key is sk-abcdEFGH1234567890abcdef and token AKIA1234567890ABCDEF");
        let res = egress_privacy_guard(&mut r, LlmProviderType::Llm7.data_trust(), "llm7");
        // 含密钥但无内部指纹 → 脱密钥放行
        assert!(res.is_ok());
        assert!(!r.messages[0].content.contains("sk-abcdEFGH"));
        assert!(!r.messages[0].content.contains("AKIA1234567890ABCDEF"));
    }

    #[test]
    fn test_trust_from_name_maps_provider_trust() {
        let _g = PRIVACY_TEST_LOCK.lock().unwrap();
        // 本地推理 → Trusted (含 `provider/model` 目录名)
        assert_eq!(trust_from_name("ollama"), DataTrust::Trusted);
        assert_eq!(trust_from_name("ollama/codellama"), DataTrust::Trusted);
        // 付费云 → Contracted
        assert_eq!(trust_from_name("openai"), DataTrust::Contracted);
        assert_eq!(trust_from_name("anthropic/claude-3"), DataTrust::Contracted);
        // 免费/代理 → Untrusted
        assert_eq!(trust_from_name("pollinations"), DataTrust::Untrusted);
        assert_eq!(trust_from_name("llm7"), DataTrust::Untrusted);
        // 未知端点保守视为 Untrusted
        assert_eq!(
            trust_from_name("totally-unknown-provider-xyz"),
            DataTrust::Untrusted
        );
    }
}
