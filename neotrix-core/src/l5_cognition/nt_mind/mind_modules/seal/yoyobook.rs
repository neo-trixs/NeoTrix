//! NT-MIND — yoyobook.yolog.dev 吸收 (C0 文献节点).
//!
//! yoyobook: 《自进化编码智能体》叙事书 — 以故事/经验形态记录自进化 agent
//! 的演化路径 (identity 觉醒 → skill 习得 → memory 沉淀 → lineage 传承)。
//! 本模块为 C0 文献节点: 提取可复用 self-evolution lesson 模式 (枚举 stub) +
//! 叙事 pattern 提取 trait (存在级 + 结构占位, 无完整实现)。作为 SEAL pipeline
//! / ConsciousnessTree 的叙事侧对标参考。

use crate::core::nt_core_self_test::SelfTest;

/// 从 yoyobook 叙事中蒸馏出的自进化 lesson 类型 (C0 stub 枚举)。
///
/// 每个变体对应书中一个反复出现的演化主题, 后续 C1+ 阶段可扩展为具体策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _SelfEvolutionLesson {
    /// identity 先于 capability: 智能体须先确立"我是谁"再演化技能。
    IdentityBeforeCapability,
    /// skill 通过失败而非成功习得 (failure-driven crystallization)。
    SkillViaFailure,
    /// memory 是 lineage 的载体: 跨代传承靠经验而非代码。
    MemoryIsLineage,
    /// journal 提供演化可读性: 无叙事则无法审计自我。
    JournalForAuditability,
    /// patch→eval→decision→promote 闭环不可跳过任一阶段。
    ClosedLoopNonNull,
}

impl _SelfEvolutionLesson {
    /// 返回该 lesson 在 yoyobook 中的一句话陈述。
    pub fn statement(&self) -> &'static str {
        match self {
            _SelfEvolutionLesson::IdentityBeforeCapability =>
                "identity precedes capability: an agent must know who it is before evolving skills",
            _SelfEvolutionLesson::SkillViaFailure =>
                "skills crystallize through failure, not success",
            _SelfEvolutionLesson::MemoryIsLineage =>
                "memory is the carrier of lineage; transmission crosses generations via experience",
            _SelfEvolutionLesson::JournalForAuditability =>
                "without journaling, self-evolution cannot be audited",
            _SelfEvolutionLesson::ClosedLoopNonNull =>
                "the patch→eval→decision→promote loop tolerates no skipped stage",
        }
    }

    /// 全部 lesson 枚举 (迭代/注册用)。
    pub fn all() -> &'static [_SelfEvolutionLesson] {
        &[
            _SelfEvolutionLesson::IdentityBeforeCapability,
            _SelfEvolutionLesson::SkillViaFailure,
            _SelfEvolutionLesson::MemoryIsLineage,
            _SelfEvolutionLesson::JournalForAuditability,
            _SelfEvolutionLesson::ClosedLoopNonNull,
        ]
    }
}

/// 叙事 pattern 提取 trait — 从一段自进化叙事文本中抽取可复用 lesson。
///
/// C0 占位: 结构化提取逻辑留待 C1 实现, 此处仅定义契约。
pub trait _NarrativePatternExtractor {
    /// 从叙事片段识别命中的 lesson 集合 (C0: 返回全部声明式 lesson)。
    fn extract_lessons(&self, _narrative: &str) -> Vec<_SelfEvolutionLesson> {
        _SelfEvolutionLesson::all().to_vec()
    }
    /// 已收录的 lesson 总数。
    fn lesson_count(&self) -> usize {
        _SelfEvolutionLesson::all().len()
    }
}

/// yoyobook 文献节点默认提取器。
pub struct _YoyoBookExtractor;

impl _NarrativePatternExtractor for _YoyoBookExtractor {}

impl SelfTest for _YoyoBookExtractor {
    fn name(&self) -> &'static str {
        "_YoyoBookExtractor"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        if _SelfEvolutionLesson::all().is_empty() {
            errs.push("lesson set must not be empty".into());
        }
        for lesson in _SelfEvolutionLesson::all() {
            if lesson.statement().is_empty() {
                errs.push(format!("lesson {:?} has empty statement", lesson));
            }
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lesson_statement_nonempty() {
        let e = _YoyoBookExtractor;
        assert!(e.self_test().is_ok());
        for lesson in _SelfEvolutionLesson::all() {
            assert!(!lesson.statement().is_empty());
        }
    }

    #[test]
    fn test_extract_returns_all_declared_lessons() {
        let e = _YoyoBookExtractor;
        let got = e.extract_lessons("a self-evolving agent story");
        assert_eq!(got.len(), 5);
        assert!(got.contains(&_SelfEvolutionLesson::ClosedLoopNonNull));
    }

    #[test]
    fn test_lesson_count_matches_enum() {
        let e = _YoyoBookExtractor;
        assert_eq!(e.lesson_count(), 5);
    }
}
