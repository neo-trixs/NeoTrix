//! 认知状态 —— 「这条成熟度结论有多确定」
//!
//! TODO 4.2: 把「未解析」建模为一等状态, 而不是把一个复算出来的数字
//! 当成事实印在 CI 门禁输出里。
//!
//! `CapabilityNode::evidence_supported_constellation` 只读**已登记的证据信号**
//! (`provides` / `metadata.wiring_evidence` / `metadata.evidence_gated`)。
//! 证据缺失时, 它无法区分两种截然不同的真相:
//!
//! 1. 这个能力**真的**只到 C1;
//! 2. 这个能力**确实是 C4**, 只是证据从没被登记 (= **未解析**)。
//!
//! 两者算出的 `supported` 数字**完全相同**。把这种数字当作「C1 就是 C1」
//! 陈述, 是在**用一个记录冒充事实**。故引入本枚举。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 认知状态 —— 与 `MaturityFinding.supported` 正交的**第二个轴**。
///
/// **不要把它塞进 `supported` 数字里** —— 数字表示「我们能证明的地板」,
/// 本枚举表示「我们对这个地板之上的真相是否已解析」。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Epistemic {
    /// 精确: 节点显式声明 `metadata.evidence_exhaustive == "true"`,
    /// 即「我的证据已穷尽」, 故 `supported` 就是真实上限。
    ///
    /// 这是**需要节点自己认领**的状态, 不是系统能推断出来的。
    Exact,
    /// 下界: 默认且 fail-closed —— 我们只能断言「已证明的地板」,
    /// 无法区分「真的不支持」与「证据尚未登记」(**未解析**)。
    ///
    /// 选它作默认, 是因为**默认值必须说真话**: 宁可说「未解析」,
    /// 也不可把「没登记证据」粉饰成「已查清不够格」。
    LowerBound,
}

impl Default for Epistemic {
    fn default() -> Self {
        Epistemic::LowerBound
    }
}

impl Epistemic {
    /// 唯一判据: 节点是否显式声明证据已穷尽。
    ///
    /// **不做任何推断** —— 系统无法证明「没有更多证据」, 只能采信节点的认领。
    ///
    /// `metadata` 是 `HashMap<String, serde_json::Value>`, 调用方可能写入
    /// 字符串 `"true"` 或真布尔 `true`, 两者都算认领。
    pub fn of(metadata: &HashMap<String, serde_json::Value>) -> Self {
        let exhaustive = metadata
            .get("evidence_exhaustive")
            .map(|v| match v {
                serde_json::Value::Bool(b) => *b,
                serde_json::Value::String(s) => s == "true",
                _ => false,
            })
            .unwrap_or(false);
        if exhaustive {
            Epistemic::Exact
        } else {
            Epistemic::LowerBound
        }
    }

    /// `supported` 能否被当作该节点真实成熟度的**陈述**(而非仅地板)。
    pub fn is_exact(&self) -> bool {
        matches!(self, Epistemic::Exact)
    }

    /// 是否处于「未解析」状态 —— 对应 TODO 4.2 的核心诉求。
    pub fn is_unresolved(&self) -> bool {
        !self.is_exact()
    }

    /// 门禁/报告用短标签。
    pub fn label(&self) -> &'static str {
        match self {
            Epistemic::Exact => "exact",
            Epistemic::LowerBound => "lower-bound(未解析)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(pairs: &[(&str, &str)]) -> HashMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
            .collect()
    }

    /// 核心不变量: **默认 fail-closed**。没声明证据穷尽 = 未解析。
    /// 若改成 Exact 作默认, 门禁输出会把「没查」说成「查清了」—— 正是本模块要消灭的谎。
    #[test]
    fn default_is_lower_bound_honest_not_optimistic() {
        assert_eq!(Epistemic::default(), Epistemic::LowerBound);
        assert!(Epistemic::default().is_unresolved());
        assert!(!Epistemic::default().is_exact());
    }

    /// 空 metadata(最常见: 什么都没登记)必须是**未解析**, 不是「精确的 C0」。
    #[test]
    fn empty_metadata_is_unresolved_not_exact_zero() {
        let e = Epistemic::of(&HashMap::new());
        assert_eq!(e, Epistemic::LowerBound);
        assert!(e.is_unresolved());
    }

    /// 只有节点**显式认领** `evidence_exhaustive` 才是 Exact。
    #[test]
    fn only_explicit_exhaustive_claim_yields_exact() {
        assert_eq!(Epistemic::of(&meta(&[("evidence_exhaustive", "true")])), Epistemic::Exact);
        // 近似值一律不得当成认领
        for v in ["false", "yes", "1", "TRUE", "maybe", ""] {
            assert_eq!(
                Epistemic::of(&meta(&[("evidence_exhaustive", v)])),
                Epistemic::LowerBound,
                "{v:?} 不得被当成认领"
            );
        }
    }

    /// 布尔型 true(而非字符串 "true")也算认领 —— metadata 是
    /// `HashMap<String, serde_json::Value>`, 调用方可能写入真布尔。
    #[test]
    fn boolean_true_also_counts_as_claim() {
        let m: HashMap<String, serde_json::Value> =
            [("evidence_exhaustive".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect();
        assert_eq!(Epistemic::of(&m), Epistemic::Exact);
    }

    /// 认领之外的其它 metadata 键**不得**影响判定(避免「碰巧有别的键就变 Exact」)。
    #[test]
    fn unrelated_metadata_does_not_grant_exactness() {
        assert_eq!(
            Epistemic::of(&meta(&[("wiring_evidence", "neotrix-core/src/x.rs:1"), ("evidence_gated", "passed")])),
            Epistemic::LowerBound,
            "证据齐 ≠ 已穷尽; 有证据只说明有地板, 不说明查清了上限"
        );
    }

    /// 标签不得为空 —— CI 门禁输出不允许出现裸的 `[]`。
    #[test]
    fn every_variant_has_a_non_empty_label() {
        for e in [Epistemic::Exact, Epistemic::LowerBound] {
            assert!(!e.label().is_empty());
            assert!(!e.label().trim().is_empty());
        }
        assert!(Epistemic::LowerBound.label().contains("未解析"), "下界标签须点明未解析");
    }
}
