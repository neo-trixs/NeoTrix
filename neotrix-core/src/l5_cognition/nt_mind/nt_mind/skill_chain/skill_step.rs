//! 技能链步骤定义 — SkillStep 枚举
//!
//! 每个步骤描述一次原子技能调用，包含输入/输出类型元数据。
//! R-P123: 按认知域拆分步骤类型

use serde::{Deserialize, Serialize};

/// 技能步骤类型枚举
///
/// 四大认知域步骤: 头脑风暴 → 规划 → TDD 实现 → 审查
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillStep {
    /// 头脑风暴: 探索需求、生成方案
    Brainstorm {
        /// 主题描述
        topic: String,
    },
    /// 规划: 将目标分解为可执行计划
    Plan {
        /// 目标描述
        goal: String,
    },
    /// TDD 实现: 测试驱动开发
    Tdd {
        /// 功能描述
        feature: String,
    },
    /// 代码审查: 质量检查与反馈
    Review {
        /// 待审查代码内容或路径
        code: String,
    },
}

/// 步骤输入类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepInputType {
    /// 原始文本/主题
    Text,
    /// 结构化 JSON
    Json,
    /// 代码片段
    Code,
}

/// 步骤输出类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepOutputType {
    /// 方案列表
    Proposals,
    /// 执行计划
    Plan,
    /// 测试+实现结果
    Implementation,
    /// 审查报告
    ReviewReport,
}

impl SkillStep {
    /// 步骤人类可读名称
    pub fn name(&self) -> &'static str {
        match self {
            SkillStep::Brainstorm { .. } => "Brainstorm",
            SkillStep::Plan { .. } => "Plan",
            SkillStep::Tdd { .. } => "TDD",
            SkillStep::Review { .. } => "Review",
        }
    }

    /// 步骤输入类型
    pub fn input_type(&self) -> StepInputType {
        match self {
            SkillStep::Brainstorm { .. } => StepInputType::Text,
            SkillStep::Plan { .. } => StepInputType::Json,
            SkillStep::Tdd { .. } => StepInputType::Json,
            SkillStep::Review { .. } => StepInputType::Code,
        }
    }

    /// 步骤输出类型
    pub fn output_type(&self) -> StepOutputType {
        match self {
            SkillStep::Brainstorm { .. } => StepOutputType::Proposals,
            SkillStep::Plan { .. } => StepOutputType::Plan,
            SkillStep::Tdd { .. } => StepOutputType::Implementation,
            SkillStep::Review { .. } => StepOutputType::ReviewReport,
        }
    }

    /// 提取步骤内载荷为字符串
    pub fn payload(&self) -> &str {
        match self {
            SkillStep::Brainstorm { topic } => topic,
            SkillStep::Plan { goal } => goal,
            SkillStep::Tdd { feature } => feature,
            SkillStep::Review { code } => code,
        }
    }
}

impl std::fmt::Display for SkillStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillStep::Brainstorm { topic } => write!(f, "Brainstorm({})", topic),
            SkillStep::Plan { goal } => write!(f, "Plan({})", goal),
            SkillStep::Tdd { feature } => write!(f, "Tdd({})", feature),
            SkillStep::Review { code } => write!(f, "Review({})", code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_name() {
        assert_eq!(
            SkillStep::Brainstorm { topic: "x".into() }.name(),
            "Brainstorm"
        );
        assert_eq!(
            SkillStep::Plan { goal: "y".into() }.name(),
            "Plan"
        );
        assert_eq!(
            SkillStep::Tdd { feature: "z".into() }.name(),
            "TDD"
        );
        assert_eq!(
            SkillStep::Review { code: "fn main(){}".into() }.name(),
            "Review"
        );
    }

    #[test]
    fn test_input_output_types() {
        let bs = SkillStep::Brainstorm { topic: "t".into() };
        assert_eq!(bs.input_type(), StepInputType::Text);
        assert_eq!(bs.output_type(), StepOutputType::Proposals);

        let plan = SkillStep::Plan { goal: "g".into() };
        assert_eq!(plan.input_type(), StepInputType::Json);
        assert_eq!(plan.output_type(), StepOutputType::Plan);

        let tdd = SkillStep::Tdd { feature: "f".into() };
        assert_eq!(tdd.input_type(), StepInputType::Json);
        assert_eq!(tdd.output_type(), StepOutputType::Implementation);

        let rev = SkillStep::Review { code: "c".into() };
        assert_eq!(rev.input_type(), StepInputType::Code);
        assert_eq!(rev.output_type(), StepOutputType::ReviewReport);
    }

    #[test]
    fn test_payload() {
        assert_eq!(
            SkillStep::Brainstorm { topic: "hello".into() }.payload(),
            "hello"
        );
        assert_eq!(
            SkillStep::Review { code: "fn main(){}".into() }.payload(),
            "fn main(){}"
        );
    }

    #[test]
    fn test_display() {
        let step = SkillStep::Tdd { feature: "auth".into() };
        assert_eq!(format!("{}", step), "Tdd(auth)");
    }

    #[test]
    fn test_serde_roundtrip() {
        let step = SkillStep::Brainstorm { topic: "test".into() };
        let json = serde_json::to_string(&step).unwrap();
        let back: SkillStep = serde_json::from_str(&json).unwrap();
        assert_eq!(step, back);
    }
}
