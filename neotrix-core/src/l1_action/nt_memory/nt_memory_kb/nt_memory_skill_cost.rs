//! NT-MEMORY 技能成本感知 (D22) — 每次技能加载计入 token 成本, 渐进披露 (薄入口)。
//!
//! 参照: the-librarian ('every skill costs tokens') +
//!       awesome-codex-skills (SKILL.md 契约 + 渐进披露, 技能越薄越好)。
//! 机制: 每次技能加载记录 token 消耗与调用频率, 暴露高成本/低回报技能,
//! 推动渐进披露 (先薄入口再按需加载正文)。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单次技能加载成本记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLoad {
    pub skill: String,
    /// 本次消耗的 token 估计 (按字符数 / 4 粗估)。
    pub tokens: usize,
    /// 是否完整加载 (true=全量正文, false=薄入口)。
    pub full: bool,
}

/// 技能成本台账。
#[derive(Debug, Clone, Default)]
pub struct SkillCostLedger {
    /// skill → 累计 token
    pub totals: HashMap<String, usize>,
    /// skill → 调用次数
    pub calls: HashMap<String, usize>,
    /// 逐次记录 (用于审计)
    pub history: Vec<SkillLoad>,
    /// 渐进披露阈值: 超过后建议下次只加载薄入口
    pub disclosure_threshold: usize,
}

impl SkillCostLedger {
    pub fn new(disclosure_threshold: usize) -> Self {
        Self {
            totals: HashMap::new(),
            calls: HashMap::new(),
            history: Vec::new(),
            disclosure_threshold,
        }
    }

    /// 记录一次加载。content 是实际加载的正文, full 标记是否全量。
    pub fn record(&mut self, skill: &str, content: &str, full: bool) {
        let tokens = estimate_tokens(content);
        *self.totals.entry(skill.to_string()).or_insert(0) += tokens;
        *self.calls.entry(skill.to_string()).or_insert(0) += 1;
        self.history.push(SkillLoad {
            skill: skill.to_string(),
            tokens,
            full,
        });
    }

    /// 该技能累计 token。
    pub fn total(&self, skill: &str) -> usize {
        self.totals.get(skill).copied().unwrap_or(0)
    }

    /// 该技能调用次数。
    pub fn call_count(&self, skill: &str) -> usize {
        self.calls.get(skill).copied().unwrap_or(0)
    }

    /// 平均每次加载成本。
    pub fn avg_cost(&self, skill: &str) -> f64 {
        let calls = self.call_count(skill);
        if calls == 0 {
            0.0
        } else {
            self.total(skill) as f64 / calls as f64
        }
    }

    /// 渐进披露建议: 累计成本超阈值 → 建议下次只加载薄入口。
    pub fn should_use_thin_entry(&self, skill: &str) -> bool {
        self.total(skill) > self.disclosure_threshold
    }

    /// 高成本技能排行 (降序, 前 n)。
    pub fn top_cost(&self, n: usize) -> Vec<(String, usize, usize)> {
        let mut v: Vec<(String, usize, usize)> = self
            .totals
            .iter()
            .map(|(k, v)| (k.clone(), *v, self.calls.get(k).copied().unwrap_or(0)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.truncate(n);
        v
    }

    /// 薄入口摘要 (渐进披露): 只给出技能名 + 成本, 不含正文。
    pub fn thin_brief(&self) -> String {
        if self.history.is_empty() {
            return "No skills loaded yet.".to_string();
        }
        let mut out = String::from("技能成本 (渐进披露, 薄入口):\n");
        for (skill, tokens, calls) in self.top_cost(10) {
            out.push_str(&format!("- {}: {} tokens / {} calls (avg {:.0})\n", skill, tokens, calls, tokens as f64 / calls.max(1) as f64));
        }
        out
    }
}

/// token 粗估: 字符数 / 4 (英文), 中文按字计。
pub fn estimate_tokens(content: &str) -> usize {
    let cjk: usize = content.chars().filter(|c| (*c as u32) > 0x2E7F).count();
    let rest = content.chars().count().saturating_sub(cjk);
    cjk + rest / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_tracks_total_and_calls() {
        let mut ledger = SkillCostLedger::new(1000);
        ledger.record("rev-officer", "loads a moderately long body of text here", true);
        ledger.record("rev-officer", "second", true);
        assert_eq!(ledger.call_count("rev-officer"), 2);
        assert!(ledger.total("rev-officer") > 0);
        assert_eq!(ledger.history.len(), 2);
    }

    #[test]
    fn avg_and_top_cost() {
        let mut ledger = SkillCostLedger::new(1000);
        ledger.record("a", "short", true);
        ledger.record("b", &"long".repeat(40), true);
        let top = ledger.top_cost(1);
        assert_eq!(top[0].0, "b", "b costlier than a");
        assert!(ledger.avg_cost("b") > ledger.avg_cost("a"));
    }

    #[test]
    fn thin_entry_suggested_after_threshold() {
        let mut ledger = SkillCostLedger::new(10);
        ledger.record("heavy", &"x".repeat(100), true);
        assert!(ledger.should_use_thin_entry("heavy"), "exceeded threshold → thin entry");
        let brief = ledger.thin_brief();
        assert!(brief.contains("heavy"));
    }

    #[test]
    fn cjk_tokens_counted_per_char() {
        let en = estimate_tokens("abcd efgh");
        let cjk = estimate_tokens("中文测试四个字");
        assert!(cjk >= 4, "CJK counted per char, got {}", cjk);
        assert!(en < 10);
    }
}
