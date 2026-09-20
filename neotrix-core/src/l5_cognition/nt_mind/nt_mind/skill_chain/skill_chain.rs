//! 技能链 — SkillChain 结构体 + ChainStatus 枚举
//!
//! 管理有序步骤序列，支持推进/暂停/查询状态
//! R-P123: 链本身是认知域编排单元

use serde::{Deserialize, Serialize};

use super::chain_config::ChainConfig;
use super::skill_step::SkillStep;

/// 链执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainStatus {
    /// 正在执行
    Running,
    /// 已暂停，可恢复
    Paused,
    /// 全部步骤完成
    Complete,
    /// 执行失败
    Failed,
}

impl std::fmt::Display for ChainStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainStatus::Running => write!(f, "Running"),
            ChainStatus::Paused => write!(f, "Paused"),
            ChainStatus::Complete => write!(f, "Complete"),
            ChainStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// 技能链 — 有序步骤序列 + 执行索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChain {
    /// 链中所有步骤
    pub steps: Vec<SkillStep>,
    /// 当前执行索引 (指向下一步要执行的位置)
    pub current_idx: usize,
    /// 链配置
    pub config: ChainConfig,
    /// 链名称/标识
    pub name: String,
}

impl SkillChain {
    /// 从链类型名称和步骤列表创建新链
    pub fn new(chain_type: &str, steps: Vec<SkillStep>) -> Self {
        Self {
            steps,
            current_idx: 0,
            config: ChainConfig::default(),
            name: chain_type.to_string(),
        }
    }

    /// 带自定义配置创建
    pub fn with_config(chain_type: &str, steps: Vec<SkillStep>, config: ChainConfig) -> Self {
        Self {
            steps,
            current_idx: 0,
            config,
            name: chain_type.to_string(),
        }
    }

    /// 获取下一步 (不推进索引)
    pub fn next_step(&self) -> Option<&SkillStep> {
        self.steps.get(self.current_idx)
    }

    /// 推进一步，返回被跳过的步骤
    pub fn advance(&mut self) -> Option<SkillStep> {
        if self.current_idx < self.steps.len() {
            let step = self.steps[self.current_idx].clone();
            self.current_idx += 1;
            Some(step)
        } else {
            None
        }
    }

    /// 链是否已执行完毕
    pub fn is_complete(&self) -> bool {
        self.current_idx >= self.steps.len()
    }

    /// 当前执行状态
    pub fn current_status(&self) -> ChainStatus {
        if self.is_complete() {
            ChainStatus::Complete
        } else if self.current_idx > 0 {
            ChainStatus::Running
        } else {
            ChainStatus::Running
        }
    }

    /// 暂停链 (仅当正在运行时有效)
    pub fn pause(&mut self) -> bool {
        if self.current_status() == ChainStatus::Running && !self.is_complete() {
            // 暂停状态通过外部管理，此处标记当前索引即可
            true
        } else {
            false
        }
    }

    /// 已完成的步骤数
    pub fn completed_count(&self) -> usize {
        self.current_idx
    }

    /// 总步骤数
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// 进度百分比 (0.0 ~ 1.0)
    pub fn progress(&self) -> f64 {
        if self.steps.is_empty() {
            return 1.0;
        }
        self.current_idx as f64 / self.steps.len() as f64
    }
}

/// 预定义链类型构建器
impl SkillChain {
    /// 构建 Brainstorm → Plan → Tdd → Review 标准链
    pub fn standard_development(topic: &str) -> Self {
        let steps = vec![
            SkillStep::Brainstorm { topic: topic.to_string() },
            SkillStep::Plan { goal: format!("Plan for: {}", topic) },
            SkillStep::Tdd { feature: format!("Implement: {}", topic) },
            SkillStep::Review { code: String::new() },
        ];
        Self::new("standard_development", steps)
    }

    /// 构建 Brainstorm → Plan 快速规划链
    pub fn quick_plan(topic: &str) -> Self {
        let steps = vec![
            SkillStep::Brainstorm { topic: topic.to_string() },
            SkillStep::Plan { goal: format!("Plan for: {}", topic) },
        ];
        Self::new("quick_plan", steps)
    }

    /// 构建纯审查链
    pub fn review_only(code: &str) -> Self {
        let steps = vec![
            SkillStep::Review { code: code.to_string() },
        ];
        Self::new("review_only", steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_chain() -> SkillChain {
        SkillChain::new(
            "test",
            vec![
                SkillStep::Brainstorm { topic: "a".into() },
                SkillStep::Plan { goal: "b".into() },
                SkillStep::Tdd { feature: "c".into() },
                SkillStep::Review { code: "d".into() },
            ],
        )
    }

    #[test]
    fn test_new_chain() {
        let chain = make_test_chain();
        assert_eq!(chain.name, "test");
        assert_eq!(chain.total_steps(), 4);
        assert_eq!(chain.completed_count(), 0);
        assert!(!chain.is_complete());
    }

    #[test]
    fn test_next_step() {
        let chain = make_test_chain();
        let next = chain.next_step().unwrap();
        assert_eq!(next.name(), "Brainstorm");
    }

    #[test]
    fn test_advance() {
        let mut chain = make_test_chain();
        let step = chain.advance().unwrap();
        assert_eq!(step.name(), "Brainstorm");
        assert_eq!(chain.completed_count(), 1);
        assert_eq!(chain.next_step().unwrap().name(), "Plan");
    }

    #[test]
    fn test_advance_to_completion() {
        let mut chain = make_test_chain();
        for _ in 0..4 {
            assert!(chain.advance().is_some());
        }
        assert!(chain.is_complete());
        assert!(chain.advance().is_none());
    }

    #[test]
    fn test_status_running() {
        let chain = make_test_chain();
        assert_eq!(chain.current_status(), ChainStatus::Running);
    }

    #[test]
    fn test_status_complete() {
        let mut chain = make_test_chain();
        for _ in 0..4 { chain.advance(); }
        assert_eq!(chain.current_status(), ChainStatus::Complete);
    }

    #[test]
    fn test_progress() {
        let mut chain = make_test_chain();
        assert_eq!(chain.progress(), 0.0);
        chain.advance();
        assert_eq!(chain.progress(), 0.25);
        chain.advance();
        assert_eq!(chain.progress(), 0.5);
    }

    #[test]
    fn test_progress_empty_chain() {
        let chain = SkillChain::new("empty", vec![]);
        assert_eq!(chain.progress(), 1.0);
    }

    #[test]
    fn test_pause() {
        let mut chain = make_test_chain();
        assert!(chain.pause());
    }

    #[test]
    fn test_standard_development() {
        let chain = SkillChain::standard_development("auth module");
        assert_eq!(chain.total_steps(), 4);
        assert_eq!(chain.steps[0], SkillStep::Brainstorm { topic: "auth module".into() });
    }

    #[test]
    fn test_quick_plan() {
        let chain = SkillChain::quick_plan("deploy");
        assert_eq!(chain.total_steps(), 2);
    }

    #[test]
    fn test_review_only() {
        let chain = SkillChain::review_only("fn main(){}");
        assert_eq!(chain.total_steps(), 1);
    }

    #[test]
    fn test_serde_roundtrip() {
        let chain = make_test_chain();
        let json = serde_json::to_string(&chain).unwrap();
        let back: SkillChain = serde_json::from_str(&json).unwrap();
        assert_eq!(chain.name, back.name);
        assert_eq!(chain.steps.len(), back.steps.len());
        assert_eq!(chain.current_idx, back.current_idx);
    }

    #[test]
    fn test_with_config() {
        let cfg = ChainConfig::minimal();
        let chain = SkillChain::with_config(
            "custom",
            vec![SkillStep::Brainstorm { topic: "t".into() }],
            cfg.clone(),
        );
        assert_eq!(chain.name, "custom");
        assert_eq!(chain.config.max_retries_per_step, 1);
    }

    #[test]
    fn test_chain_status_display() {
        assert_eq!(format!("{}", ChainStatus::Running), "Running");
        assert_eq!(format!("{}", ChainStatus::Paused), "Paused");
        assert_eq!(format!("{}", ChainStatus::Complete), "Complete");
        assert_eq!(format!("{}", ChainStatus::Failed), "Failed");
    }

    #[test]
    fn test_advance_returns_step_and_advances() {
        let mut chain = make_test_chain();
        let step = chain.advance().unwrap();
        assert_eq!(step.name(), "Brainstorm");
        assert_eq!(chain.current_idx, 1);
    }

    #[test]
    fn test_advance_past_end() {
        let mut chain = make_test_chain();
        for _ in 0..4 {
            chain.advance();
        }
        assert!(chain.advance().is_none());
        assert!(chain.is_complete());
    }

    #[test]
    fn test_next_step_does_not_advance() {
        let chain = make_test_chain();
        let step1 = chain.next_step().unwrap();
        let step2 = chain.next_step().unwrap();
        assert_eq!(step1.name(), step2.name());
    }

    #[test]
    fn test_pause_on_running_chain() {
        let mut chain = make_test_chain();
        // Chain is running (even at index 0)
        assert!(chain.pause());
    }

    #[test]
    fn test_pause_on_complete_chain() {
        let mut chain = make_test_chain();
        for _ in 0..4 {
            chain.advance();
        }
        // Chain is complete — pause should fail
        assert!(!chain.pause());
    }

    #[test]
    fn test_progress_at_each_step() {
        let mut chain = make_test_chain();
        assert_eq!(chain.progress(), 0.0);
        chain.advance();
        assert_eq!(chain.progress(), 0.25);
        chain.advance();
        assert_eq!(chain.progress(), 0.5);
        chain.advance();
        assert_eq!(chain.progress(), 0.75);
        chain.advance();
        assert_eq!(chain.progress(), 1.0);
    }

    #[test]
    fn test_standard_development_steps_content() {
        let chain = SkillChain::standard_development("auth");
        assert_eq!(chain.steps[0], SkillStep::Brainstorm { topic: "auth".into() });
        assert_eq!(chain.steps[1], SkillStep::Plan { goal: "Plan for: auth".into() });
        assert_eq!(chain.steps[2], SkillStep::Tdd { feature: "Implement: auth".into() });
        assert!(matches!(chain.steps[3], SkillStep::Review { .. }));
    }

    #[test]
    fn test_chain_name() {
        let chain = SkillChain::new("my_chain", vec![]);
        assert_eq!(chain.name, "my_chain");
    }

    #[test]
    fn test_chain_config_preserved() {
        let mut cfg = ChainConfig::default();
        cfg.max_retries_per_step = 10;
        let chain = SkillChain::with_config("c", vec![], cfg);
        assert_eq!(chain.config.max_retries_per_step, 10);
    }
}
