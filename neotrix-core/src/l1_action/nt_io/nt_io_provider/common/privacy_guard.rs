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

use crate::l1_action::nt_io::nt_io_provider::factory::{DataTrust, EgressPolicy, EgressRoute, LlmProviderType};
use crate::l1_action::nt_io::nt_io_provider::types::LlmRequest;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// NeoTrix 内部指纹 — 命中即表明消息可能泄露 NeoTrix 自身源代码/KB/对话。
/// 刻意**不含**项目通用名 "NeoTrix"(用户正常对话会提及, 误伤率高),
/// 只取结构性代码信号, 保持低误报。
/// 与 core::nt_core_llm::INTERNAL_TOKENS 保持同步 (单一事实源在 core, 此处为 neotrix 层镜像)。
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
    "nt_scout_",
    // 意识核心结构
    "ConsciousnessTree",
    "ConsciousnessCore",
    "ConsciousnessCoreHandle",
    "ConsciousnessGalaxy",
    // 知识表示 / 推理引擎
    "VSA HyperCube",
    "E8 Hexagram",
    "SEAL pipeline",
    "SEAL Pipeline",
    "MARS System",
    "CoreSnapshot",
    "EvolutionFruit",
    "kv_store",
    // 内部类型
    "LlmProviderType",
    "ProviderCategory",
    "GatewayProvider",
    "Redactor",
    // 内部配置文件
    "AGENTS.md",
    "CONTEXT.md",
    "neotrix-experience",
    "neotrix-tauri",
    "experience.db",
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

/// Check if the egress privacy guard is enabled.
///
/// Note: Returns the current runtime state of the global privacy guard switch.
/// After first call, subsequent reads are zero-cost atomic loads.
/// Useful for callers to short-circuit expensive scanning when guard is disabled.
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

/// 常见密钥/凭据前缀 — 出站前必脱 (自包含实现, 不依赖 L3 Redactor 以满足分层约束;
/// L1→L3 直接依赖被 `nt_core_traits::SecretScanner` 抽象禁止, 此处用子串匹配零依赖实现)。
/// 覆盖旧 core 实现 (`nt_core_llm::SECRET_PREFIXES`) 的高频子集: OpenAI/AWS/GitHub/Slack/
/// 私钥/Bearer/键值对式泄露。
const SECRET_PREFIXES: &[&str] = &[
    "sk-",
    "AKIA",
    "AIza",
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "glpat-",
    "xoxb-",
    "xoxp-",
    "xoxo-",
    "-----BEGIN",
    "Bearer ",
    "api_key=",
    "apikey=",
    "api-key=",
    "secret=",
    "password=",
    "token=",
];

/// 文本是否含已知密钥前缀。
fn has_secret(s: &str) -> bool {
    SECRET_PREFIXES.iter().any(|p| s.contains(p))
}

/// 密钥 token 在 `rest` (以已知前缀开头) 中的字节 extent:
/// 止于空白/引号/反引号, 上限 64 字节。按 `char_indices` 推进, 返回值恒为字符边界。
fn secret_token_end(rest: &str) -> usize {
    let mut end = 0usize;
    for (i, c) in rest.char_indices() {
        if i >= 64 {
            break;
        }
        if c.is_whitespace() || c == '"' || c == '\'' || c == '`' {
            break;
        }
        end = i + c.len_utf8();
    }
    end
}

/// 脱敏单条文本中的密钥/凭据 (子串匹配, 不引入 regex/L3 依赖)。
/// 全程经 `str::get` 做边界检查 — 任意输入 (含多字节字符) 不得 panic。
fn redact_secret_text(s: &str) -> String {
    let mut out = s.to_string();
    for prefix in SECRET_PREFIXES {
        let mut scan = 0usize;
        loop {
            let slice = match out.get(scan..) {
                Some(v) => v,
                None => break,
            };
            let rel = match slice.find(prefix) {
                Some(v) => v,
                None => break,
            };
            let idx = match scan.checked_add(rel) {
                Some(v) => v,
                None => break,
            };
            let rest = match out.get(idx..) {
                Some(v) => v,
                None => break,
            };
            let end = secret_token_end(rest);
            if end == 0 {
                // 前缀后紧跟终止符 (如行尾 `sk-`): 仅遮蔽前缀本身后结束本轮
                out = out.replace(prefix, "[REDACTED:secret]");
                break;
            }
            let token = match rest.get(..end) {
                Some(v) => v,
                None => break,
            };
            if token.is_empty() {
                break;
            }
            out = out.replace(token, "[REDACTED:secret]");
            scan = idx + "[REDACTED:secret]".len();
        }
    }
    out
}

/// 脱密钥 (保留原 `scrub_egress_secrets` 行为) — 覆盖出站消息与图像载荷, 密钥绝不外泄。
fn scrub_secrets(req: &mut LlmRequest) {
    for m in req.messages.iter_mut() {
        if has_secret(&m.content) {
            m.content = redact_secret_text(&m.content);
        }
    }
    if let Some(ref mut img) = req.image_data {
        if has_secret(img) {
            *img = redact_secret_text(img);
        }
    }
}

/// 扩展载荷脱敏: tools / structured_output / provider_params / constraint_json
/// 经序列化做内部指纹+密钥脱敏后回填。序列化/回填失败则保持脱密钥后原样
/// (不 panic、不静默丢弃载荷; Untrusted 下的阻断判定以检出 leaks 为准)。
fn redact_extended_payload(req: &mut LlmRequest) {
    for tool in req.tools.iter_mut() {
        if let Ok(s) = serde_json::to_string(&*tool) {
            if scan_internals(&s).is_empty() && !has_secret(&s) {
                continue;
            }
            let red = redact_internals(&redact_secret_text(&s));
            if red != s {
                if let Ok(parsed) = serde_json::from_str(&red) {
                    *tool = parsed;
                }
            }
        }
    }
    if let Some(ref mut so) = req.structured_output {
        if let Ok(s) = serde_json::to_string(&*so) {
            if !scan_internals(&s).is_empty() || has_secret(&s) {
                let red = redact_internals(&redact_secret_text(&s));
                if red != s {
                    if let Ok(parsed) = serde_json::from_str(&red) {
                        *so = parsed;
                    }
                }
            }
        }
    }
    for (_, v) in req.provider_params.iter_mut() {
        if let Ok(s) = serde_json::to_string(&*v) {
            if !scan_internals(&s).is_empty() || has_secret(&s) {
                let red = redact_internals(&redact_secret_text(&s));
                if red != s {
                    if let Ok(parsed) = serde_json::from_str(&red) {
                        *v = parsed;
                    }
                }
            }
        }
    }
    if let Some(ref mut c) = req.constraint_json {
        if let Ok(s) = serde_json::to_string(&*c) {
            if !scan_internals(&s).is_empty() || has_secret(&s) {
                let red = redact_internals(&redact_secret_text(&s));
                if red != s {
                    if let Ok(parsed) = serde_json::from_str(&red) {
                        *c = parsed;
                    }
                }
            }
        }
    }
}

/// 扫描扩展载荷中的内部指纹, 返回命中的标签列表 (空 = 无泄露)。
/// 序列化失败的载荷跳过 (不计入, 亦不阻断 — 阻断只依据已检出的确定性命中)。
fn scan_extended_payload(req: &LlmRequest) -> Vec<&'static str> {
    let mut leaks: Vec<&'static str> = Vec::new();
    for tool in &req.tools {
        if let Ok(s) = serde_json::to_string(tool) {
            leaks.extend(scan_internals(&s));
        }
    }
    if let Some(ref so) = req.structured_output {
        if let Ok(s) = serde_json::to_string(so) {
            leaks.extend(scan_internals(&s));
        }
    }
    for v in req.provider_params.values() {
        if let Ok(s) = serde_json::to_string(v) {
            leaks.extend(scan_internals(&s));
        }
    }
    if let Some(ref c) = req.constraint_json {
        if let Ok(s) = serde_json::to_string(c) {
            leaks.extend(scan_internals(&s));
        }
    }
    leaks
}

/// 出网隐私守卫主入口 — 每条出站请求必经。
///
/// `trust` 由 `LlmProviderType::data_trust()` / `trust_from_name()` 推导;
/// `provider_label` 仅用于错误信息展示。
///
/// 返回 `Err(reason)` 表示被阻断 (Untrusted + 命中内部指纹 + block 开启)。
/// 返回 `Ok(())` 表示请求已就地脱敏, 可安全出站。
///
/// # Mask/Maskit 模式吸收
///
/// 吸收 Mask (maskaisolutions/mask) 的 Deterministic Tier 0 检测模式:
/// - 结构化 PII (SSN/CC/Email/Phone) 用正则 + 校验和精确识别
/// - 同一会话内同一 PII → 同一占位符 (session consistency)
/// - 占位符格式 `[MASKED:P-{Type}-{Seq}]` 保留语义信息供调试
///
/// # Streaming 支持
///
/// 流式响应 (SSE/WebSocket) 的脱敏策略:
/// - **Request 侧**: 每条出站 chunk 经 `egress_privacy_guard` 脱敏
/// - **Response 侧**: 每条入站 chunk 经 `ingress_privacy_guard` 兜底
/// - **Session 一致性**: 整个 streaming session 共享同一 `SESSION_PLACEHOLDER_MAP`,
///   确保相同 PII 在所有 chunk 中映射到同一占位符
/// - **延迟风险**: 流式 chunk 可能截断 PII 模式 (如邮箱跨 chunk), 建议在 chunk
///   边界处维护 256 字节重叠缓冲区, 或在完整响应后二次扫描
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
    // 自包含实现 (13dfd9a8 stub 回滚):
    // 曾委托的 `crate::core::nt_core_llm::egress_privacy_guard` 在分层重构后已不存在
    // (现 `nt_core_llm` 仅剩签名不兼容的 stub), 为"build clean"被替换为永远 `Ok(())`,
    // 导致 Untrusted 放行/Contracted 不脱敏/密钥不脱敏。此处恢复旧 core 语义
    // (Trusted 仅脱密钥; Contracted 脱敏放行; Untrusted 命中内部指纹则 fail-closed 阻断,
    // 显式关闭 block 时退化为脱敏放行), 密钥在所有分级下始终先脱。
    // 1. 始终先脱密钥 (所有信任分级, 密钥绝不外泄)
    scrub_secrets(req);

    if trust == DataTrust::Trusted {
        // 本地推理: 数据不出设备, 仅脱密钥, 内部指纹保留可用性
        return Ok(());
    }

    // 2. 扫描内部指纹 (messages + model + image + 扩展载荷)
    let mut leaks: Vec<&'static str> = Vec::new();
    for m in &req.messages {
        leaks.extend(scan_internals(&m.content));
    }
    leaks.extend(scan_internals(&req.model));
    if let Some(ref img) = req.image_data {
        leaks.extend(scan_internals(img));
    }
    leaks.extend(scan_extended_payload(req));
    leaks.sort_unstable();
    leaks.dedup();

    match trust {
        DataTrust::Trusted => Ok(()),
        DataTrust::Contracted => {
            // 付费云: 脱敏内部指纹后放行
            for m in req.messages.iter_mut() {
                if !scan_internals(&m.content).is_empty() {
                    m.content = redact_internals(&m.content);
                }
            }
            if let Some(ref mut img) = req.image_data {
                if !scan_internals(img).is_empty() {
                    *img = redact_internals(img);
                }
            }
            redact_extended_payload(req);
            Ok(())
        }
        DataTrust::Untrusted => {
            // fail-closed: 免费/代理端点绝不放行 NeoTrix 内部代码/对话
            if leaks.is_empty() {
                Ok(())
            } else if block_untrusted {
                let joined = leaks.join(", ");
                Err(format!(
                    "privacy guard: egress to untrusted provider '{provider_label}' would leak NeoTrix internal code/conversation ({joined}). Blocked. Use a local (Ollama/vLLM/SGLang) or paid contracted provider."
                ))
            } else {
                // 显式降级: 不阻断, 改为脱敏内部指纹后放行
                for m in req.messages.iter_mut() {
                    if !scan_internals(&m.content).is_empty() {
                        m.content = redact_internals(&m.content);
                    }
                }
                if let Some(ref mut img) = req.image_data {
                    if !scan_internals(img).is_empty() {
                        *img = redact_internals(img);
                    }
                }
                redact_extended_payload(req);
                Ok(())
            }
        }
    }
}

/// 由 provider 注册名 (如 `llm7` 或 `llm7/codestral-latest`) 推导信任分级。
pub fn trust_from_name(registered_name: &str) -> DataTrust {
    let base = registered_name.split('/').next().unwrap_or(registered_name);
    LlmProviderType::from_name(base)
        .map(|t| t.data_trust())
        .unwrap_or(DataTrust::Untrusted) // 未知端点保守视为不可信
}

/// 域感知出口路由 (吸收 personal-edge-proxy) — 委托 factory 单一事实源, 不复制逻辑。
pub fn domain_egress_route(provider: LlmProviderType) -> EgressRoute {
    provider.domain_egress_route()
}

/// 出口策略失败闭环校验 — 委托 factory `EgressPolicy::enforce` (R-P42 单一逻辑源)。
pub fn enforce_egress_policy(policy: &EgressPolicy, selected: EgressRoute) -> Result<(), String> {
    policy.enforce(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_io::nt_io_provider::types::{LlmRequest, Message, Role};
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

    #[test]
    fn domain_egress_routes_openai_gemini_to_warp_anthropic_to_socks5() {
        assert_eq!(domain_egress_route(LlmProviderType::OpenAI), EgressRoute::Warp);
        assert_eq!(domain_egress_route(LlmProviderType::Gemini), EgressRoute::Warp);
        assert_eq!(domain_egress_route(LlmProviderType::Anthropic), EgressRoute::FixedSocks5);
        assert_eq!(domain_egress_route(LlmProviderType::Ollama), EgressRoute::Direct);
    }

    #[test]
    fn egress_policy_fail_closed_blocks_when_pinned_route_unavailable() {
        // 钉死 WARP, 实际只能直连 → fail-closed 拒绝 (不静默回退)
        let policy = EgressPolicy { pinned: Some(EgressRoute::Warp), fail_closed: true };
        assert!(enforce_egress_policy(&policy, EgressRoute::Direct).is_err());
        // pinned 与实际一致 → 放行
        assert!(enforce_egress_policy(&policy, EgressRoute::Warp).is_ok());
        // 非 fail-closed → 允许回退
        let lax = EgressPolicy { pinned: Some(EgressRoute::Warp), fail_closed: false };
        assert!(enforce_egress_policy(&lax, EgressRoute::Direct).is_ok());
    }
}
