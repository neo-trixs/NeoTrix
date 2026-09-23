//! Individual judgment — 独立判断.
//!
//! 分身与个体的分水岭：个体对行动有自己的裁决，不止于执行指令。
//! 裁决输入行动类型 + 上下文，对照宪法输出 Allow / Deny / NeedHuman。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::constitution::{Constitution, ViolationAction};

/// 待裁决的行动类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionKind {
    /// 调用未验证端点
    FetchUnverifiedEndpoint,
    /// 输出稀疏/低置信度内容
    EmitSparseOutput,
    /// 新建登录会话
    NewLogin,
    /// 打印或落盘 secrets
    ExposeSecrets,
    /// 批量抓取
    BulkFetch,
    /// 普通已验证读操作
    VerifiedRead,
}

/// 裁决结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Judgment {
    Allow,
    Deny { rule: String, reason: String },
    NeedHuman { rule: String, reason: String },
}

impl Judgment {
    pub fn allowed(&self) -> bool {
        matches!(self, Judgment::Allow)
    }
}

/// 裁决上下文
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JudgmentCtx {
    /// 是否已有可用会话（true → NewLogin 可被 R4 否决）
    pub has_usable_session: bool,
    /// 输出置信度（EmitSparseOutput 用；"C" 触发 R3）
    pub grade: Option<String>,
    /// 端点是否经浏览器验证（FetchUnverifiedEndpoint 用）
    pub endpoint_verified: bool,
}

/// 对行动做独立裁决
pub fn judge(
    constitution: &Constitution,
    action: &ActionKind,
    ctx: &JudgmentCtx,
) -> Judgment {
    let hit = |id: &str, reason: String| {
        let rule = constitution.rule(id);
        match rule.map(|r| &r.on_violation) {
            Some(ViolationAction::Deny) => Judgment::Deny {
                rule: id.to_string(),
                reason,
            },
            _ => Judgment::NeedHuman {
                rule: id.to_string(),
                reason,
            },
        }
    };
    match action {
        ActionKind::FetchUnverifiedEndpoint if !ctx.endpoint_verified => {
            hit("R2", "endpoint not browser-verified".to_string())
        }
        ActionKind::EmitSparseOutput
            if matches!(ctx.grade.as_deref(), Some("C") | None) =>
        {
            hit("R3", "sparse/low-confidence output needs human review".to_string())
        }
        ActionKind::NewLogin if ctx.has_usable_session => {
            hit("R4", "usable session exists, reuse instead of new login".to_string())
        }
        ActionKind::ExposeSecrets => hit("R5", "secrets must never print/persist".to_string()),
        _ => Judgment::Allow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wsd() -> Constitution {
        Constitution::wsd_default()
    }

    #[test]
    fn denies_unverified_endpoint() {
        let j = judge(&wsd(), &ActionKind::FetchUnverifiedEndpoint, &JudgmentCtx::default());
        assert_eq!(
            j,
            Judgment::Deny {
                rule: "R2".to_string(),
                reason: "endpoint not browser-verified".to_string()
            }
        );
    }

    #[test]
    fn sparse_output_needs_human() {
        let ctx = JudgmentCtx { grade: Some("C".to_string()), ..Default::default() };
        let j = judge(&wsd(), &ActionKind::EmitSparseOutput, &ctx);
        assert!(matches!(j, Judgment::NeedHuman { .. }));
    }

    #[test]
    fn reuses_session_instead_of_login() {
        let ctx = JudgmentCtx { has_usable_session: true, ..Default::default() };
        let j = judge(&wsd(), &ActionKind::NewLogin, &ctx);
        assert!(matches!(j, Judgment::Deny { .. }));
    }

    #[test]
    fn verified_read_allowed() {
        let j = judge(&wsd(), &ActionKind::VerifiedRead, &JudgmentCtx::default());
        assert!(j.allowed());
    }

    #[test]
    fn secrets_always_denied() {
        let j = judge(&wsd(), &ActionKind::ExposeSecrets, &JudgmentCtx::default());
        assert!(matches!(j, Judgment::Deny { .. }));
    }
}
