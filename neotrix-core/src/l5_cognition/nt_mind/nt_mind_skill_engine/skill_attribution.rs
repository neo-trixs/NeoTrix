//! skill_attribution — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 差分归因记录 (arxiv 2608.11888 SkillTriage 吸收): 单个技能的激活统计与
/// procedure-heavy 风险标记。
pub struct SkillAttribution {
    pub name: String,
    pub category: String,
    pub activations: u32,
    pub over_validation_score: u32,
    pub procedure_heavy: bool,
    pub flagged: bool,
    /// self-benchmark 钩子 (Phase 4): 真实成功/失败调用计数, 驱动 promote/demote 校准。
    pub success_count: u32,
    pub last_used_at: i64,
}

impl SkillAttribution {
    /// 记录一次真实调用结果 (成功/失败), 供 `rebalance` 做 promote/demote 校准。
    pub fn record_outcome(&mut self, success: bool) {
        self.last_used_at = chrono::Utc::now().timestamp();
        if success {
            self.success_count += 1;
        }
    }
}

/// 技能树层级统计 (G6, AgentSkillOS 吸收): 巡检报告的数据载体。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct _SkillTreeStats {
    pub total_skills: usize,
    pub categories: HashMap<String, usize>,
    pub roots: usize,
    pub orphans: usize,
    pub max_depth: usize,
}

pub(crate) fn parse_array_field(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner.split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        trimmed.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
