//! nt_secret_scan — 工具输出里的凭据检测
//!
//! ## 它防的是哪个面（这个区分很重要，core 侧拦的不是同一个）
//!
//! | | 拦什么 | 在哪 |
//! |---|---|---|
//! | core 侧 `Redactor` | **工具参数**（模型即将送出的） | `AgentLoop::call_tool`，但该路径零生产实例化 |
//! | **本模块** | **工具输出**（执行后读回的） | `nt_agent.rs::execute_tool` 之后 |
//!
//! 缺口在后者：工具输出会**原样**进 `steps` 表（SQLite，持久化）、进
//! transcript、再经 `on_step` 回调外发到前端 / IM 通道。若模型 `read_file`
//! 读到一个含 `sk-...` 的 `.env`，那串凭据就四处扩散。
//!
//! ⇒ 两个扫描点**不重叠**：core 拦「模型想把密钥写出去」，本模块拦
//!   「模型读到了密钥」。两者都要有。
//!
//! ## 为什么在这里重写而不用 core 的实现
//!
//! `neotrix-core/Cargo.toml:106` 依赖 `neotrix-neobot`（core → neobot），
//! 反向依赖会造成**循环 crate 依赖**，Cargo 直接拒绝。
//!
//! ## 与 core 的同步义务
//!
//! 16 条 secret 正则**逐条取自**
//! `neotrix-core/src/l3_embodiment/nt_shield/shield_core/redaction.rs`
//! 的 `secret_regexes`。⚠️ **改动前必须同步两边**，否则会出现
//! 「core 拦得住、neobot 拦不住」的**不对称**，而这种不对称最难发现。
//!
//! ## 为什么不搬 PII 那 5 条
//!
//! core 侧另有 5 条 PII 正则（email / phone / ipv4 / ipv6 / home-path）。
//! 本模块**不搬**：工具输出里出现邮箱/IP 是开发场景的常态（读日志、读
//! 配置、看 git remote），按 PII 阻断会把正常的工作流全部拒掉。
//! ⛔ 误报率高的检测器会被用户关掉，比没有更坏 —— 这与 `nt_prompt_guard`
//! 的「只防一类威胁」是同一个取舍逻辑。
//!
//! ## 处置：报告而不阻断
//!
//! 命中时**不改写、不阻断**，只产出一条报告交调用方落库。理由：
//! 工具输出里出现 `sk-` 可能是用户**故意**让我们看自己的配置；
//! 自动截断会让模型拿不到它需要的上下文。真正的处置决策交给人。

use regex::Regex;

/// 一条命中。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretHit {
    /// 规则标识（与 core 侧同名，便于对照）。
    pub rule: &'static str,
    /// 命中片段（截断，不含完整凭据 —— 报告本身也是泄露面）。
    pub snippet: String,
}

/// 单条规则。
struct Pattern {
    rule: &'static str,
    re: Regex,
}

/// 16 条 secret 正则，与 core 侧 `Redactor::default()` 逐条一致。
///
/// ⛔ 全部是字面量正则 ⇒ 构造期不会失败。用 `OnceLock` 惰性构建，避免
///   每次调用都重编译 16 个正则（本模块在工具输出的热路径上）。
static PATTERNS: std::sync::OnceLock<Vec<Pattern>> = std::sync::OnceLock::new();

fn patterns() -> &'static [Pattern] {
    PATTERNS.get_or_init(|| {
        let raw: &[(&'static str, &str)] = &[
            ("openai", r"sk-[a-zA-Z0-9_-]{20,}"),
            ("stripe", r"sk-[a-fA-F0-9]{32,}"),
            ("github-pat", r"ghp_[a-zA-Z0-9]{36}"),
            ("github-pat-fine", r"github_pat_[a-zA-Z0-9]{36}"),
            ("github-oauth", r"gho_[a-zA-Z0-9]{36}"),
            ("github-app", r"ghu_[a-zA-Z0-9]{36}"),
            ("github-user", r"ghs_[a-zA-Z0-9]{36}"),
            ("github-refresh", r"ghr_[a-zA-Z0-9]{36}"),
            ("aws-key", r"AKIA[0-9A-Z]{16}"),
            (
                "aws-secret",
                r#"(?i)aws_secret_access_key\s*[:=]\s*['"]?[a-zA-Z0-9/+]{40}"#,
            ),
            (
                "private-key",
                r"-----BEGIN (RSA |EC )?PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
            ),
            ("slack", r"xox[abpors]-[a-zA-Z0-9]{10,}"),
            ("gitlab", r"glpat-[a-zA-Z0-9\-]{20,}"),
            (
                "jwt",
                r"[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}",
            ),
            (
                "generic-key",
                r#"(?i)(?:api[_-]?key|secret)\s*[:=]\s*['"]?[a-zA-Z0-9_\-]{16,}"#,
            ),
            (
                "password",
                r#"(?i)password\s*[:=]\s*['"]?[^'"\s]{8,}"#,
            ),
        ];
        raw.iter()
            .filter_map(|(rule, pat)| {
                Regex::new(pat)
                    .ok()
                    .map(|re| Pattern { rule, re })
            })
            .collect()
    })
}

/// 片段截断长度。
///
/// ⛔ 报告本身也是泄露面 ⇒ **绝不回显完整凭据**。12 字符足够人认出
///   「这是 AWS key」而不足以被复用。
const SNIPPET_MAX: usize = 12;

/// 扫一段文本，返回命中的凭据类型。
///
/// # 只报不改
///
/// 命中不阻断、不改写 —— 见模块文档「处置：报告而不阻断」。
pub fn scan(text: &str) -> Vec<SecretHit> {
    let mut out = Vec::new();
    for p in patterns() {
        if let Some(m) = p.re.find(text) {
            let snippet: String = {
                let raw = m.as_str().chars().take(SNIPPET_MAX).collect::<String>();
                if m.as_str().chars().count() > SNIPPET_MAX {
                    format!("{raw}…")
                } else {
                    raw
                }
            };
            out.push(SecretHit {
                rule: p.rule,
                snippet,
            });
        }
    }
    out
}

/// 人类可读的一行摘要（供 steps / audit 落库）。
pub fn summarize(hits: &[SecretHit]) -> Option<String> {
    if hits.is_empty() {
        return None;
    }
    let rules: Vec<&str> = hits.iter().map(|h| h.rule).collect();
    Some(format!(
        "⚠️ 工具输出含凭据（{}）—— 已报告不阻断；片段不回显完整值",
        rules.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔ 每条规则至少一个真样本 ⇒ **防「正则写错但测试仍绿」**。
    /// 这是本模块最容易出的错：16 条正则写错一条，CI 全绿而线上漏拦。
    #[test]
    fn 十六条规则各有命中样本() {
        let samples: &[(&str, &str)] = &[
            ("openai", "sk-abcdefghijklmnopqrstuvwxyz0123"),
            ("stripe", "sk-0123456789abcdef0123456789abcdef01"),
            ("github-pat", "ghp_0123456789abcdefghijklmnopqrstuvwxyz"),
            ("github-pat-fine", "github_pat_0123456789abcdefghijklmnopqrstuvwxyz"),
            ("github-oauth", "gho_0123456789abcdefghijklmnopqrstuvwxyz"),
            ("github-app", "ghu_0123456789abcdefghijklmnopqrstuvwxyz"),
            ("github-user", "ghs_0123456789abcdefghijklmnopqrstuvwxyz"),
            ("github-refresh", "ghr_0123456789abcdefghijklmnopqrstuvwxyz"),
            ("aws-key", "AKIAIOSFODNN7EXAMPLE"),
            (
                "aws-secret",
                "aws_secret_access_key = wJalrXUtnFEMIK7MDENGbPxRfiCYEXAMPLEKEY1234",
            ),
            (
                "private-key",
                "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA\n-----END RSA PRIVATE KEY-----",
            ),
            ("slack", "xoxb-1234567890-abcdefghij"),
            ("gitlab", "glpat-abcdefghijklmnopqrst"),
            (
                "jwt",
                "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dBjftJeZ4CVPmB92K27uhbUJU1p1r_wW1gFWFOEjXk",
            ),
            ("generic-key", "api_key = abcdefghijklmnop1234"),
            ("password", "password = SuperSecret123"),
        ];
        assert_eq!(
            samples.len(),
            patterns().len(),
            "样本数须与规则数相等（规则 {} 条，样本 {} 条）",
            patterns().len(),
            samples.len()
        );
        for (rule, sample) in samples {
            let hits = scan(sample);
            assert!(
                hits.iter().any(|h| h.rule == *rule),
                "规则 {rule} 未命中自己的样本（实际命中：{:?}）",
                hits.iter().map(|h| h.rule).collect::<Vec<_>>()
            );
        }
    }

    /// ⛔ 正常开发输出**不得**被误伤 —— 误报率高的检测器会被关掉。
    #[test]
    fn 普通输出不误报() {
        for text in [
            "读取 src/main.rs 完成，共 128 行",
            "git remote origin git@github.com:owner/repo.git",
            "构建成功：target/debug/neobot",
            "配置项 server.port = 8080，log_level = info",
            "处理 3 个文件，用时 1.5s",
        ] {
            assert!(
                scan(text).is_empty(),
                "普通输出被误判：{text:?} → {:?}",
                scan(text).iter().map(|h| h.rule).collect::<Vec<_>>()
            );
        }
    }

    /// ⛔ 片段**不得**回显完整凭据 —— 报告本身也是泄露面。
    #[test]
    fn 片段不回显完整凭据() {
        let secret = "sk-abcdefghijklmnopqrstuvwxyz0123456789";
        let hits = scan(&format!("found: {secret}"));
        assert_eq!(hits.len(), 1);
        assert!(
            !hits[0].snippet.contains("0123456789"),
            "片段泄露了过多凭据内容: {}",
            hits[0].snippet
        );
        assert!(hits[0].snippet.chars().count() <= SNIPPET_MAX + 1);
    }

    #[test]
    fn 摘要格式与空命中() {
        assert!(summarize(&[]).is_none());
        let s = summarize(&scan("AKIAIOSFODNN7EXAMPLE")).expect("有命中");
        assert!(s.contains("aws-key"), "摘要须点名规则: {s}");
        assert!(!s.contains("AKIA"), "摘要**不得**含凭据原文: {s}");
    }
}