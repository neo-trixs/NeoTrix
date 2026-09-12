//! NT-MIND — RSI-Exam 吸收 (rsi-exam.ai).
//!
//! RSI-Exam: 递归自改进 (Recursive Self-Improvement) 可执行基准。智能体继承
//! 可运行工件, 持续进行 experiment / keep / discard 循环, 并在 hidden set 上
//! 重跑评分, 以衡量"改进可迁移性"。
//!
//! 本模块为 C0 级结构吸收: 仅定义 RSI 评测概念结构与 NeoTrix 自进化成熟度的
//! 映射注释, 编译通过即可, 后续 C1-C4 迭代填充真实评测接线。
//!
//! 映射锚点 (RSI-Exam ↔ NeoTrix SEAL pipeline):
//! - `inherited_artifact`  ↔ SEAL pipeline 上一轮蒸馏/吸收产出的可运行工件
//! - `visible_set` / `hidden_set` ↔ ConsciousnessTree 自进化闭环的可见训练态 / 隐藏验证态
//! - `rollout`             ↔ SEAL 迭代 cycle (explore → distill → self_test → absorb)
//! - `version_history`     ↔ 经验落盘 KB `experience` 命名空间的 cycle 指针链
//! - 核心命题"改进的迁移性" ↔ ConsciousnessTree 自进化闭环是否跨 cycle 稳定增长

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 一次 RSI 改进的可见/隐藏评测集合标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct _DatasetSplit {
    pub name: String,
}

/// 可运行工件: 智能体继承的起点 (代码 / 配置 / 技能权重等)。
#[derive(Debug, Clone, Default)]
pub(crate) struct _InheritedArtifact {
    pub id: String,
    pub kind: String,
    pub payload_ref: String,
}

/// 单次改进 rollout: 在继承工件上做实验, 保留/丢弃改动, 记录评分。
#[derive(Debug, Clone, Default)]
pub struct Rollout {
    pub id: String,
    pub parent_artifact_id: String,
    pub changes: Vec<String>,
    pub visible_score: f64,
    pub hidden_score: Option<f64>,
    pub kept: bool,
}

/// 版本历史: 跨 rollout 的工件演进链 (对应 KB cycle 指针链)。
#[derive(Debug, Clone, Default)]
pub struct VersionHistory {
    pub root_artifact_id: String,
    pub rollouts: Vec<Rollout>,
}

impl VersionHistory {
    /// 构造空版本历史, 以给定根工件为起点。
    pub fn new(root: _InheritedArtifact) -> Self {
        Self {
            root_artifact_id: root.id,
            rollouts: Vec::new(),
        }
    }

    /// 追加一次 rollout, 验证其父工件须为当前根或已有 rollout 产出。
    pub(crate) fn _append_rollout(&mut self, r: Rollout) -> Result<(), String> {
        if r.parent_artifact_id != self.root_artifact_id
            && !self.rollouts.iter().any(|x| x.id == r.parent_artifact_id)
        {
            return Err(format!(
                "rollout {} parent {} not in version history",
                r.id, r.parent_artifact_id
            ));
        }
        self.rollouts.push(r);
        Ok(())
    }

    /// 迁移性指标: 隐藏集相对可见集的评分保留比 (kept rollouts 平均)。
    /// 空时返回 0.0 (无迁移证据)。
    pub(crate) fn _transfer_ratio(&self) -> f64 {
        let kept: Vec<&Rollout> = self
            .rollouts
            .iter()
            .filter(|r| r.kept && r.hidden_score.is_some())
            .collect();
        if kept.is_empty() {
            return 0.0;
        }
        let num: f64 = kept.iter().map(|r| r.hidden_score.unwrap()).sum();
        let den: f64 = kept.iter().map(|r| r.visible_score).sum();
        if den == 0.0 {
            0.0
        } else {
            num / den
        }
    }
}

/// RSI-Exam 评测结构: 聚合可见/隐藏集与版本历史。
#[derive(Debug, Clone, Default)]
pub(crate) struct _RsiExam {
    pub visible_set: _DatasetSplit,
    pub hidden_set: _DatasetSplit,
    pub history: VersionHistory,
    /// 外部源标注 (rsi-exam.ai), 用于追溯吸收来源。
    pub source: String,
    /// 与 NeoTrix SEAL pipeline 自进化成熟度的映射注释。
    pub seal_mapping: HashMap<String, String>,
}

impl _RsiExam {
    pub fn new(root: _InheritedArtifact) -> Self {
        let mut seal_mapping = HashMap::new();
        seal_mapping.insert(
            "inherited_artifact".into(),
            "SEAL pipeline 上轮蒸馏/吸收产出的可运行工件".into(),
        );
        seal_mapping.insert(
            "visible_set/hidden_set".into(),
            "ConsciousnessTree 可见训练态 / 隐藏验证态".into(),
        );
        seal_mapping.insert(
            "rollout".into(),
            "SEAL 迭代 cycle (explore→distill→self_test→absorb)".into(),
        );
        seal_mapping.insert(
            "version_history".into(),
            "KB `experience` namespace 的 cycle 指针链".into(),
        );
        seal_mapping.insert(
            "_transfer_ratio".into(),
            "改进可迁移性 ↔ ConsciousnessTree 自进化闭环跨 cycle 稳定增长".into(),
        );
        Self {
            visible_set: _DatasetSplit {
                name: "visible".into(),
            },
            hidden_set: _DatasetSplit {
                name: "hidden".into(),
            },
            history: VersionHistory::new(root),
            source: "rsi-exam.ai".into(),
            seal_mapping,
        }
    }
}

impl SelfTest for _RsiExam {
    fn name(&self) -> &'static str {
        "_RsiExam"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        if self.source != "rsi-exam.ai" {
            errs.push(format!("unexpected source: {}", self.source));
        }
        if self.seal_mapping.is_empty() {
            errs.push("seal_mapping must be populated".into());
        }
        if self.history.root_artifact_id.is_empty() {
            errs.push("root_artifact_id must not be empty".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_exam_construct() {
        let root = _InheritedArtifact {
            id: "a0".into(),
            kind: "skill".into(),
            payload_ref: "kb://experience/cycle-001".into(),
        };
        let exam = _RsiExam::new(root);
        assert_eq!(exam.source, "rsi-exam.ai");
        assert_eq!(exam.visible_set.name, "visible");
        assert_eq!(exam.hidden_set.name, "hidden");
        assert!(!exam.seal_mapping.is_empty());
        assert!(exam.self_test().is_ok());
    }

    #[test]
    fn test_rollout_append_and_transfer() {
        let root = _InheritedArtifact {
            id: "a0".into(),
            kind: "skill".into(),
            payload_ref: "kb://experience/cycle-001".into(),
        };
        let mut exam = _RsiExam::new(root);
        exam.history
            ._append_rollout(Rollout {
                id: "r1".into(),
                parent_artifact_id: "a0".into(),
                changes: vec!["tune temperature".into()],
                visible_score: 0.8,
                hidden_score: Some(0.72),
                kept: true,
            })
            .unwrap();
        // 非法父工件应被拒绝。
        assert!(exam
            .history
            ._append_rollout(Rollout {
                id: "r2".into(),
                parent_artifact_id: "ghost".into(),
                changes: vec![],
                visible_score: 0.0,
                hidden_score: None,
                kept: false,
            })
            .is_err());
        assert!(
            (exam.history._transfer_ratio() - 0.9).abs() < 1e-9,
            "_transfer_ratio 应≈0.9 (float), got {}",
            exam.history._transfer_ratio()
        );
    }

    #[test]
    fn test_transfer_ratio_empty() {
        let root = _InheritedArtifact::default();
        let exam = _RsiExam::new(root);
        assert_eq!(exam.history._transfer_ratio(), 0.0);
    }
}
