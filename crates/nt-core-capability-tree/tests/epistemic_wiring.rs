//! 4.2 集成验证: `Epistemic` 确实接进了 `maturity_audit` 的**真实路径**。
//!
//! 单元测试 (`src/epistemic.rs`) 只证明枚举逻辑; 本文件证明
//! `CapabilityRegistry::maturity_audit()` 产出的每个 finding 都真的携带
//! 该字段, 且判据来自节点 metadata 而非硬编码。

use nt_core_capability_tree::node::{CapabilityNode, ConstellationLevel, Domain};
use nt_core_capability_tree::registry::CapabilityRegistry;
use nt_core_capability_tree::Epistemic;

/// 声称 C4 但**零证据**登记的节点 —— 门禁必须报它, 且标为「未解析」。
///
/// 这是 TODO 4.2 的核心场景: 复算得出 supported=C0, 但 C0 是
/// 「我们没有证据」, 不是「已查清它只到 C0」。
#[test]
fn finding_carries_lower_bound_when_no_evidence_exhaustiveness_claimed() {
    let mut reg = CapabilityRegistry::new();
    let mut node = CapabilityNode::new_primitive(
        "test::unproven_claim".into(),
        Domain::Core,
        vec!["does_something".into()],
    );
    // 声称高等级, 但 metadata 不含 evidence_exhaustive
    node.constellation = ConstellationLevel::C4MainPipeline;
    reg.register(node).expect("register");

    let findings = reg.maturity_audit();
    assert_eq!(findings.len(), 1, "零证据却声称 C4 ⇒ 必须被审计出来");

    let f = &findings[0];
    assert_eq!(f.id, "test::unproven_claim");
    assert_eq!(f.claimed, ConstellationLevel::C4MainPipeline);
    // provides 非空 ⇒ supported 至少 C1, 但**不是** Exact
    assert_eq!(
        f.epistemic,
        Epistemic::LowerBound,
        "未声明证据穷尽 ⇒ 未解析, 不得被当成已查清"
    );
    assert!(f.epistemic.is_unresolved());
    assert!(!f.epistemic.is_exact());
}

/// 同一节点显式认领 `evidence_exhaustive` 后, 判据必须**跟着 metadata 变** ——
/// 证明这不是硬编码常量, 而是真正接在节点证据上。
#[test]
fn epistemic_tracks_node_metadata() {
    let mut reg = CapabilityRegistry::new();
    let mut node = CapabilityNode::new_primitive(
        "test::declared_exhaustive".into(),
        Domain::Core,
        vec!["does_something".into()],
    );
    node.constellation = ConstellationLevel::C4MainPipeline;
    node.metadata.insert(
        "evidence_exhaustive".into(),
        serde_json::Value::String("true".into()),
    );
    reg.register(node).expect("register");

    let findings = reg.maturity_audit();
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].epistemic,
        Epistemic::Exact,
        "节点认领证据穷尽 ⇒ Exact"
    );
    assert!(findings[0].epistemic.is_exact());
}

/// 反向: 没有任何 finding 时, 门禁输出走 `OK:` 分支, 不应产生任何
/// epistemic 标签 —— 空标签会污染 CI 日志。
#[test]
fn clean_registry_produces_no_findings() {
    let mut reg = CapabilityRegistry::new();
    // 声称 C1 且 provides 非空 ⇒ supported 恰为 C1, 无虚标
    reg.register(CapabilityNode::new_primitive(
        "test::honest".into(),
        Domain::Core,
        vec!["does_something".into()],
    ))
    .expect("register");

    assert!(
        reg.maturity_audit().is_empty(),
        "声称值被证据支撑 ⇒ 无 finding (CI 打印 OK 分支)"
    );
}
