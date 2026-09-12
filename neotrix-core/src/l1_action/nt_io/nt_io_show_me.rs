//! Show-Me 交互式可视化辅助 (NT-IO)
//!
//! 吸收源: humanlayer/skills/tree/main/plugins/show-me
//! 成熟度: C1 (unit-tested stub, 无外部可视化运行时集成)
//!
//! 核心能力: 将结构化数据/步骤转换为交互式可视化辅助规格
//! (如分步高亮、节点面板), 本 stub 负责 aid 规格生成与校验。

use crate::core::nt_core_self_test::SelfTest;

/// 可视化辅助条目 (一个可高亮/可点击的步骤或实体)。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VisualStep {
    pub id: String,
    pub label: String,
}

/// 可视化辅助 trait — 把步骤序列映射为可交互 aid 规格。
pub(crate) trait VisualAid: Send + Sync {
    /// 生成 aid 规格: 返回步骤数, 或 None 当步骤为空/含重复 id。
    fn build_aid(&self, steps: &[VisualStep]) -> Option<usize>;
    /// 校验步骤 id 唯一且非空。
    fn is_valid(&self, steps: &[VisualStep]) -> bool;
}

/// 默认实现: 唯一 id 校验 + 计数式规格生成。
#[derive(Default)]
pub(crate) struct ShowMeAid;

impl VisualAid for ShowMeAid {
    fn build_aid(&self, steps: &[VisualStep]) -> Option<usize> {
        if self.is_valid(steps) {
            Some(steps.len())
        } else {
            None
        }
    }

    fn is_valid(&self, steps: &[VisualStep]) -> bool {
        if steps.is_empty() {
            return false;
        }
        let mut seen = std::collections::HashSet::new();
        steps.iter().all(|s| {
            !s.id.is_empty() && !s.label.is_empty() && seen.insert(s.id.as_str())
        })
    }
}

/// T1 SelfTest: 验证 aid 构建器存在且唯一性校验生效。
#[derive(Default)]
pub(crate) struct ShowMeSelfTest;

impl SelfTest for ShowMeSelfTest {
    fn name(&self) -> &str {
        "nt_io_show_me"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let a = ShowMeAid;
        let steps = vec![
            VisualStep { id: "1".into(), label: "load".into() },
            VisualStep { id: "2".into(), label: "render".into() },
        ];
        match a.build_aid(&steps) {
            Some(2) => Ok(()),
            _ => Err(vec!["nt_io_show_me: valid aid failed to build".into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_valid_aid() {
        let a = ShowMeAid;
        let steps = vec![
            VisualStep { id: "1".into(), label: "a".into() },
            VisualStep { id: "2".into(), label: "b".into() },
        ];
        assert_eq!(a.build_aid(&steps), Some(2));
    }

    #[test]
    fn test_rejects_duplicate_ids() {
        let a = ShowMeAid;
        let steps = vec![
            VisualStep { id: "1".into(), label: "a".into() },
            VisualStep { id: "1".into(), label: "b".into() },
        ];
        assert!(!a.is_valid(&steps));
        assert_eq!(a.build_aid(&steps), None);
    }

    #[test]
    fn test_rejects_empty() {
        let a = ShowMeAid;
        assert!(!a.is_valid(&[]));
        assert_eq!(a.build_aid(&[]), None);
    }
}
