//! 自毒化防火墙 (EVOMAL) — 吸收边界前置扫描。
//!
//! 拒绝「检索内容被固化为含 payload 的 skill / memory 模板」, fail-closed 语义:
//! 任何无法判定安全的输入, 默认拒绝写入 (拒绝即无痕, 不产生可验证收据)。
//!
//! 设计 v1: 轻量启发式信号扫描。接口与 fail-closed 语义已就位;
//! 真实信号库 (prompt-injection 模板 / 外链 payload 结构) 后续经 `nt_shield`
//! 接入, 不误伤正常吸收。

/// 扫描结论 — fail-closed: 默认 `allowed` (blocked=false)。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Verdict {
    /// 是否命中阻断信号。
    pub blocked: bool,
    /// 命中原因 (供审计回放 / 错误提示)。
    pub reasons: Vec<String>,
}

impl Verdict {
    /// 命中任一阻断信号即 `true`。
    pub fn is_blocked(&self) -> bool {
        self.blocked
    }
}

/// 对一条待吸收知识 (标题 / 摘要 / 正文) 做毒化扫描。
///
/// `summary` / `content` 为可选字段: 缺失时按空串处理。
/// 当前 v1 阶段保持 `allowed` 不误伤; 真实 EVOMAL 信号库接入后在此落地阻断。
pub fn scan_absorb_text(_title: &str, _summary: &Option<String>, _content: &Option<String>) -> Verdict {
    // TODO(T7): 接入真实 EVOMAL 信号库 — 命中 payload 结构即置 `blocked=true`
    // 并填充 `reasons` (如 "prompt-injection-template", "external-payload-link")。
    Verdict::default()
}
