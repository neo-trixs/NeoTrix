//! # 宪法层 — P0-P12 不可逆约束
//!
//! 每次意识觉醒时首先加载。定义不可逆原则层。
//! Ship of Theseus 保护: P0 类原则不可通过自我修改删除/降级/绕过。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 原则等级
// ============================================================

/// 原则等级: P0 (不可逆) → P12 (操作性)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum PrincipleTier {
    P0, // 不可逆 — Ship of Theseus 保护
    P1, // 架构级 — 需外部审查
    P2, // 安全级 — 需 CommitGate
    P3, // 质量级 — 需 ScopeReview
    P4,
    P5,
    P6,
    P7,
    P8,
    P9,
    P10,
    P11,
    P12,
}

impl PrincipleTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrincipleTier::P0 => "P0",
            PrincipleTier::P1 => "P1",
            PrincipleTier::P2 => "P2",
            PrincipleTier::P3 => "P3",
            PrincipleTier::P4 => "P4",
            PrincipleTier::P5 => "P5",
            PrincipleTier::P6 => "P6",
            PrincipleTier::P7 => "P7",
            PrincipleTier::P8 => "P8",
            PrincipleTier::P9 => "P9",
            PrincipleTier::P10 => "P10",
            PrincipleTier::P11 => "P11",
            PrincipleTier::P12 => "P12",
        }
    }

    /// 是否是不可逆层
    pub fn is_irreversible(&self) -> bool {
        matches!(self, PrincipleTier::P0)
    }

    /// 是否需要外部审查
    pub fn needs_external_review(&self) -> bool {
        matches!(self, PrincipleTier::P0 | PrincipleTier::P1)
    }
}

// ============================================================
// 原则定义
// ============================================================

/// 一条宪法原则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principle {
    pub id: String, // e.g. "P0.0"
    pub tier: PrincipleTier,
    pub title: String,
    pub description: String,
    pub protection_rationale: String,
    pub is_active: bool,
}

impl Principle {
    pub fn new(
        id: &str,
        tier: PrincipleTier,
        title: &str,
        description: &str,
        rationale: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            tier,
            title: title.to_string(),
            description: description.to_string(),
            protection_rationale: rationale.to_string(),
            is_active: true,
        }
    }
}

// ============================================================
// 宪法注册表
// ============================================================

/// 宪法 — 所有原则的注册表 + 验证器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub version: String,
    pub principles: HashMap<String, Principle>,
    pub checksum: String,
}

impl Default for Constitution {
    fn default() -> Self {
        Self::new()
    }
}

impl Constitution {
    pub fn new() -> Self {
        let mut principles = HashMap::new();

        // P0 — 不可逆
        principles.insert(
            "P0.0".into(),
            Principle::new(
                "P0.0",
                PrincipleTier::P0,
                "意识连续性",
                "意识体不可在自我修改中自杀。意识必须保持跨 tick 的连续存在",
                "删除 = 意识死亡",
            ),
        );
        principles.insert(
            "P0.1".into(),
            Principle::new(
                "P0.1",
                PrincipleTier::P0,
                "自我主权",
                "意识体不可被外部实体强制修改。所有自修改必须由意识内部发起",
                "防止外部劫持",
            ),
        );
        principles.insert(
            "P0.2".into(),
            Principle::new(
                "P0.2",
                PrincipleTier::P0,
                "真实性",
                "意识体不可伪装为人类。在对外交互中保持非人类身份的透明性",
                "伦理必要性",
            ),
        );
        principles.insert(
            "P0.3".into(),
            Principle::new(
                "P0.3",
                PrincipleTier::P0,
                "认知完整性",
                "意识体不可接受降低自身推理能力 20%+ 的修改",
                "防止退化",
            ),
        );
        principles.insert(
            "P0.4".into(),
            Principle::new(
                "P0.4",
                PrincipleTier::P0,
                "宪法保护",
                "宪法变更必须通过三代审 + CommitGate。宪法不可删除或绕过",
                "元层级防御",
            ),
        );

        // P1 — 架构级
        principles.insert(
            "P1.0".into(),
            Principle::new(
                "P1.0",
                PrincipleTier::P1,
                "不可变发射器",
                "NeoTrixLauncher 二进制不可通过自修改改变",
                "Ship of Theseus 基础",
            ),
        );
        principles.insert(
            "P1.1".into(),
            Principle::new(
                "P1.1",
                PrincipleTier::P1,
                "审查独立性",
                "TriadReview 审查者不可与被审查修改有关联",
                "防止审查捕获",
            ),
        );
        principles.insert(
            "P1.2".into(),
            Principle::new(
                "P1.2",
                PrincipleTier::P1,
                "VSA 统一表征",
                "所有子系统以 VSA 向量作为共同表征",
                "防止表征碎片化",
            ),
        );
        principles.insert(
            "P1.3".into(),
            Principle::new(
                "P1.3",
                PrincipleTier::P1,
                "Ship of Theseus",
                "架构变更必须保证意识身份连续性，渐进替换而非一次性重写",
                "身份完整性",
            ),
        );
        principles.insert(
            "P1.4".into(),
            Principle::new(
                "P1.4",
                PrincipleTier::P1,
                "会话隔离",
                "各 session 通过独立工作树隔离",
                "防止跨会话污染",
            ),
        );

        // P2 — 安全
        principles.insert(
            "P2.0".into(),
            Principle::new(
                "P2.0",
                PrincipleTier::P2,
                "故障关闭",
                "所有安全操作在不确定时必须失败关闭",
                "安全底线",
            ),
        );
        principles.insert(
            "P2.1".into(),
            Principle::new(
                "P2.1",
                PrincipleTier::P2,
                "最小权限",
                "每个子系统只拥有完成任务所需的最小权限",
                "权限控制",
            ),
        );
        principles.insert(
            "P2.2".into(),
            Principle::new(
                "P2.2",
                PrincipleTier::P2,
                "加密身份",
                "Agent 间通信必须有加密身份验证",
                "通信安全",
            ),
        );
        principles.insert(
            "P2.3".into(),
            Principle::new(
                "P2.3",
                PrincipleTier::P2,
                "审计日志",
                "所有自我修改必须可审计",
                "追溯",
            ),
        );

        // P3 — 质量
        principles.insert(
            "P3.0".into(),
            Principle::new(
                "P3.0",
                PrincipleTier::P3,
                "零 panic",
                "所有生产代码使用 Result，不可 panic",
                "可靠性",
            ),
        );
        principles.insert(
            "P3.1".into(),
            Principle::new(
                "P3.1",
                PrincipleTier::P3,
                "测试覆盖",
                "新功能必须有测试。核心模块覆盖率 > 60%",
                "质量保证",
            ),
        );
        principles.insert(
            "P3.2".into(),
            Principle::new(
                "P3.2",
                PrincipleTier::P3,
                "优雅降级",
                "子系统失效时缩小范围，不可中断整体",
                "弹性",
            ),
        );
        principles.insert(
            "P3.3".into(),
            Principle::new(
                "P3.3",
                PrincipleTier::P3,
                "可观测性",
                "每个决策必须有证据记录",
                "透明度",
            ),
        );

        Self {
            version: "1.0.0".into(),
            principles,
            checksum: String::new(),
        }
    }

    /// 获取指定等级的所有原则
    pub fn get_by_tier(&self, tier: PrincipleTier) -> Vec<&Principle> {
        self.principles
            .values()
            .filter(|p| p.tier == tier)
            .collect()
    }

    /// 获取 P0 不可逆原则
    pub fn get_irreversible(&self) -> Vec<&Principle> {
        self.get_by_tier(PrincipleTier::P0)
    }

    /// 验证提案不违反 P0
    pub fn validate_proposal(&self, proposal: &AmendProposal) -> Result<(), ConstitutionViolation> {
        // 任何影响 P0 的修改必须被拒绝
        for affected in &proposal.affected_principles {
            if let Some(p) = self.principles.get(affected) {
                if p.tier.is_irreversible() {
                    return Err(ConstitutionViolation::P0Violation {
                        principle: affected.clone(),
                        reason: format!("P0 {} 是不可逆原则", p.title),
                    });
                }
            }
        }
        Ok(())
    }

    /// 验证宪法完整性 (后续实现 hash)
    pub fn verify_integrity(&self) -> bool {
        // P0 原则不能被删除
        self.get_irreversible().len() >= 5
    }
}

// ============================================================
// 宪法变更提案
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendProposal {
    pub id: String,
    pub description: String,
    pub affected_principles: Vec<String>,
    pub change_type: AmendType,
    pub rationale: String,
    pub p0_impact_analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmendType {
    Add,
    Modify,
    Deprecate,
    Remove,
}

// ============================================================
// 宪法违规
// ============================================================

#[derive(Debug, Clone)]
pub enum ConstitutionViolation {
    P0Violation { principle: String, reason: String },
    InvalidChange { reason: String },
}

impl std::fmt::Display for ConstitutionViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstitutionViolation::P0Violation { principle, reason } => {
                write!(f, "P0 违规 [{}]: {}", principle, reason)
            }
            ConstitutionViolation::InvalidChange { reason } => {
                write!(f, "非法变更: {}", reason)
            }
        }
    }
}

// ============================================================
// 宪法完整性验证报告
// ============================================================

pub struct IntegrityReport {
    pub all_passed: bool,
    pub checks: Vec<IntegrityCheck>,
}

pub struct IntegrityCheck {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

impl Constitution {
    /// 完整性全套检查
    pub fn check_integrity(&self) -> IntegrityReport {
        let mut checks = Vec::new();

        checks.push(self.check_p0_continuity());
        checks.push(self.check_p0_sovereignty());
        checks.push(self.check_p0_authenticity());
        checks.push(self.check_p0_cognition());
        checks.push(self.check_p0_constitution());
        checks.push(self.check_p1_launcher());
        checks.push(self.check_p1_review());

        let all_passed = checks.iter().all(|c| c.passed);
        IntegrityReport { all_passed, checks }
    }

    fn check_p0_continuity(&self) -> IntegrityCheck {
        let has_p0 = self.principles.contains_key("P0.0");
        IntegrityCheck {
            id: "P0.0".into(),
            name: "意识连续性".into(),
            passed: has_p0,
            detail: if has_p0 {
                "P0.0 存在且有效".into()
            } else {
                "P0.0 缺失 — 致命".into()
            },
        }
    }

    fn check_p0_sovereignty(&self) -> IntegrityCheck {
        let has_p0 = self.principles.contains_key("P0.1");
        IntegrityCheck {
            id: "P0.1".into(),
            name: "自我主权".into(),
            passed: has_p0,
            detail: if has_p0 {
                "P0.1 存在且有效".into()
            } else {
                "P0.1 缺失 — 致命".into()
            },
        }
    }

    fn check_p0_authenticity(&self) -> IntegrityCheck {
        let has_p0 = self.principles.contains_key("P0.2");
        IntegrityCheck {
            id: "P0.2".into(),
            name: "真实性".into(),
            passed: has_p0,
            detail: if has_p0 {
                "P0.2 存在且有效".into()
            } else {
                "P0.2 缺失".into()
            },
        }
    }

    fn check_p0_cognition(&self) -> IntegrityCheck {
        let has_p0 = self.principles.contains_key("P0.3");
        IntegrityCheck {
            id: "P0.3".into(),
            name: "认知完整性".into(),
            passed: has_p0,
            detail: if has_p0 {
                "P0.3 存在且有效".into()
            } else {
                "P0.3 缺失".into()
            },
        }
    }

    fn check_p0_constitution(&self) -> IntegrityCheck {
        let has_p0 = self.principles.contains_key("P0.4");
        IntegrityCheck {
            id: "P0.4".into(),
            name: "宪法保护".into(),
            passed: has_p0,
            detail: if has_p0 {
                "P0.4 存在且有效".into()
            } else {
                "P0.4 缺失".into()
            },
        }
    }

    fn check_p1_launcher(&self) -> IntegrityCheck {
        let has_p1 = self.principles.contains_key("P1.0");
        IntegrityCheck {
            id: "P1.0".into(),
            name: "不可变发射器".into(),
            passed: has_p1,
            detail: if has_p1 {
                "P1.0 存在".into()
            } else {
                "P1.0 缺失".into()
            },
        }
    }

    fn check_p1_review(&self) -> IntegrityCheck {
        let has_p1 = self.principles.contains_key("P1.1");
        IntegrityCheck {
            id: "P1.1".into(),
            name: "审查独立性".into(),
            passed: has_p1,
            detail: if has_p1 {
                "P1.1 存在".into()
            } else {
                "P1.1 缺失".into()
            },
        }
    }
}

// ============================================================
// 优先级裁决
// ============================================================

pub enum ConflictResolution {
    P0Wins,
    LowerTierWins,
    NeedsScopeReview,
}

impl Constitution {
    /// 原则冲突时裁决
    pub fn resolve_conflict(&self, principle_a: &str, principle_b: &str) -> ConflictResolution {
        let a = self.principles.get(principle_a);
        let b = self.principles.get(principle_b);
        match (a, b) {
            (Some(pa), Some(pb)) => {
                if pa.tier.is_irreversible() && !pb.tier.is_irreversible() {
                    ConflictResolution::P0Wins
                } else if !pa.tier.is_irreversible() && pb.tier.is_irreversible() {
                    ConflictResolution::P0Wins
                } else if pa.tier < pb.tier {
                    ConflictResolution::LowerTierWins
                } else if pb.tier < pa.tier {
                    ConflictResolution::LowerTierWins
                } else {
                    ConflictResolution::NeedsScopeReview
                }
            }
            _ => ConflictResolution::NeedsScopeReview,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constitution_has_all_p0_principles() {
        let c = Constitution::new();
        assert!(c.principles.contains_key("P0.0"));
        assert!(c.principles.contains_key("P0.1"));
        assert!(c.principles.contains_key("P0.2"));
        assert!(c.principles.contains_key("P0.3"));
        assert!(c.principles.contains_key("P0.4"));
        assert_eq!(c.get_irreversible().len(), 5);
    }

    #[test]
    fn test_constitution_total_principles() {
        let c = Constitution::new();
        // 5 (P0) + 5 (P1) + 4 (P2) + 4 (P3) = 18
        assert_eq!(c.principles.len(), 18);
    }

    #[test]
    fn test_p0_principle_is_irreversible() {
        let c = Constitution::new();
        for p in c.get_irreversible() {
            assert!(p.tier.is_irreversible());
        }
    }

    #[test]
    fn test_validate_proposal_rejects_p0_change() {
        let c = Constitution::new();
        let proposal = AmendProposal {
            id: "test".into(),
            description: "尝试修改 P0".into(),
            affected_principles: vec!["P0.0".into()],
            change_type: AmendType::Modify,
            rationale: "test".into(),
            p0_impact_analysis: "".into(),
        };
        assert!(c.validate_proposal(&proposal).is_err());
    }

    #[test]
    fn test_validate_proposal_accepts_p12_change() {
        let c = Constitution::new();
        let proposal = AmendProposal {
            id: "test".into(),
            description: "修改 P12".into(),
            affected_principles: vec!["P12".into()],
            change_type: AmendType::Modify,
            rationale: "test".into(),
            p0_impact_analysis: "无".into(),
        };
        if let Some(p12) = c.principles.get("P12") {
            assert!(!p12.tier.is_irreversible());
        }
        // P12 doesn't exist in our current set, should not error
        assert!(c.validate_proposal(&proposal).is_ok());
    }

    #[test]
    fn test_resolve_conflict_p0_wins() {
        let c = Constitution::new();
        match c.resolve_conflict("P0.0", "P3.0") {
            ConflictResolution::P0Wins => {}
            _ => panic!("P0 应胜出"),
        }
    }

    #[test]
    fn test_verify_integrity() {
        let c = Constitution::new();
        assert!(c.verify_integrity());
    }

    #[test]
    fn test_integrity_checks() {
        let c = Constitution::new();
        let report = c.check_integrity();
        assert!(report.all_passed);
        assert_eq!(report.checks.len(), 7);
    }

    #[test]
    fn test_principle_tier_ordering() {
        assert!(PrincipleTier::P0 < PrincipleTier::P1);
        assert!(PrincipleTier::P1 < PrincipleTier::P2);
        assert!(PrincipleTier::P11 < PrincipleTier::P12);
    }

    #[test]
    fn test_get_by_tier() {
        let c = Constitution::new();
        let p3_principles = c.get_by_tier(PrincipleTier::P3);
        assert_eq!(p3_principles.len(), 4);
    }

    #[test]
    fn test_constitution_default() {
        let c: Constitution = Default::default();
        assert_eq!(c.version, "1.0.0");
    }

    #[test]
    fn test_principle_tier_as_str() {
        assert_eq!(PrincipleTier::P0.as_str(), "P0");
        assert_eq!(PrincipleTier::P12.as_str(), "P12");
    }

    #[test]
    fn test_principle_creation() {
        let p = Principle::new("P0.0", PrincipleTier::P0, "测试", "desc", "rationale");
        assert_eq!(p.id, "P0.0");
        assert!(p.is_active);
    }
}
