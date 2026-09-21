//! # NtSmartContext — 智能上下文加载
//!
//! 人类对话式上下文管理：不再每轮加载全部规则文档，而是
//!
//! 1. **按需加载**：每个规则/能力包注册 `triggers`（触发词），`route(query)`
//!    只把本轮命中的包载入上下文，token 上限封顶（最优解状态）。
//! 2. **主记忆分支**：`remember/recall` 维护跨轮常驻的关键事实与决策，
//!    以 `CompactionPriority::Critical` 的 `CompactionIntent` 接入
//!    `ContextBudget::assemble_with_intent`，压缩时永不丢失。
//! 3. **剪枝遗忘**：`advance_turn` 对非置顶包做 salience 衰减，`prune`
//!    移除长期不用 + 低显著性的包（Pinned 永不删除）。
//!
//! 与现有能力的接线 (R-P79)：
//! - 预算与压缩 → `super::context_budget::{ContextBudget, CompactionIntent}`
//! - 触发词思想 → `crate::skill_loader::{SkillEntry, SkillFilter}`
//!  （本模块是其轻量内存版：免文件 IO，纯函数可测）
//!
//! # Safety
//! - 纯内存结构，无 IO、无锁、无 unsafe (R-P1)。
//! - 生产代码无 `unwrap/expect/panic`，错误与越界全部饱和/钳制处理。

use super::context_budget::{
    AssembledContext, CompactionIntent, CompactionPriority, ContextBudget, SourceType,
};
use std::collections::HashMap;

/// Token 估算：与 `ContextBudget::assemble` 一致的 `len / 4` 启发式。
fn est_tokens(content: &str) -> usize {
    content.len() / 4
}

/// 规则包驻留种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtPackKind {
    /// 主记忆分支镜像：每轮必载，剪枝永不删除。
    Pinned,
    /// 触发词命中才加载。
    OnDemand,
    /// 用后即焚：失活即剪。
    Ephemeral,
}

/// 可加载的规则/能力包。
#[derive(Debug, Clone)]
pub struct NtRulePack {
    pub name: String,
    pub triggers: Vec<String>,
    pub kind: NtPackKind,
    /// 显著性 0.0..=1.0（注册与使用时钳制）。
    pub salience: f64,
    pub source: SourceType,
    pub content: String,
    pub last_used_turn: u64,
    pub use_count: u64,
}

impl NtRulePack {
    pub fn new(
        name: impl Into<String>,
        kind: NtPackKind,
        source: SourceType,
        content: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            triggers: Vec::new(),
            kind,
            salience: 0.5,
            source,
            content: content.into(),
            last_used_turn: 0,
            use_count: 0,
        }
    }

    pub fn with_triggers(mut self, triggers: &[&str]) -> Self {
        self.triggers = triggers.iter().map(|t| t.to_string()).collect();
        self
    }

    pub fn with_salience(mut self, salience: f64) -> Self {
        self.salience = salience.clamp(0.0, 1.0);
        self
    }
}

/// 主记忆分支节点：跨轮常驻的关键事实/决策。
#[derive(Debug, Clone)]
pub struct NtMemoryNode {
    pub key: String,
    pub content: String,
    pub salience: f64,
}

/// 剪枝报告。
#[derive(Debug, Clone)]
pub struct NtPruneReport {
    pub pruned: Vec<String>,
    pub freed_tokens: usize,
    pub remaining_packs: usize,
}

/// 本轮路由命中的包。
#[derive(Debug, Clone)]
pub struct NtRoutedPack {
    pub name: String,
    pub score: f64,
    pub tokens: usize,
}

/// 智能上下文：注册表 + 主记忆分支 + 轮次时钟。
pub struct NtSmartContext {
    packs: HashMap<String, NtRulePack>,
    main_branch: Vec<NtMemoryNode>,
    turn: u64,
    session_id: String,
    token_cap: usize,
    decay: f64,
    prune_threshold: f64,
    stale_turns: u64,
}

impl NtSmartContext {
    pub fn new(session_id: impl Into<String>, token_cap: usize) -> Self {
        Self {
            packs: HashMap::new(),
            main_branch: Vec::new(),
            turn: 0,
            session_id: session_id.into(),
            token_cap,
            decay: 0.9,
            prune_threshold: 0.2,
            stale_turns: 5,
        }
    }

    pub fn with_decay(mut self, decay: f64) -> Self {
        self.decay = decay.clamp(0.0, 1.0);
        self
    }

    pub fn with_prune_threshold(mut self, threshold: f64) -> Self {
        self.prune_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn with_stale_turns(mut self, turns: u64) -> Self {
        self.stale_turns = turns;
        self
    }

    pub fn turn(&self) -> u64 {
        self.turn
    }

    pub fn pack_count(&self) -> usize {
        self.packs.len()
    }

    pub fn branch_len(&self) -> usize {
        self.main_branch.len()
    }

    /// 注册规则包（同名覆盖）。注册轮即视为刚使用，防止新生包被误剪。
    pub fn register_pack(&mut self, mut pack: NtRulePack) {
        pack.salience = pack.salience.clamp(0.0, 1.0);
        pack.last_used_turn = self.turn;
        self.packs.insert(pack.name.clone(), pack);
    }

    /// 写入主记忆分支：同 key 合并（内容覆盖、显著性取高）。
    pub fn remember(&mut self, key: impl Into<String>, content: impl Into<String>, salience: f64) {
        let key = key.into();
        let salience = salience.clamp(0.0, 1.0);
        if let Some(node) = self.main_branch.iter_mut().find(|n| n.key == key) {
            node.content = content.into();
            if salience > node.salience {
                node.salience = salience;
            }
        } else {
            self.main_branch.push(NtMemoryNode {
                key,
                content: content.into(),
                salience,
            });
        }
    }

    /// 读出主记忆分支全文（压缩时的 Critical 注入源）。
    pub fn recall(&self) -> String {
        self.main_branch
            .iter()
            .map(|n| n.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 本轮路由：Pinned 必载 + 触发命中的包按分排序、token 封顶。
    /// 命中包自动记 `use_count/last_used_turn`（即“用到哪个加载哪个”的记账）。
    pub fn route(&mut self, query: &str) -> Vec<NtRoutedPack> {
        let q = query.to_lowercase();
        let mut scored: Vec<(String, f64)> = Vec::new();

        for pack in self.packs.values() {
            if pack.kind == NtPackKind::Pinned {
                scored.push((pack.name.clone(), f64::MAX));
                continue;
            }
            let hits = pack
                .triggers
                .iter()
                .filter(|t| !t.is_empty() && q.contains(&t.to_lowercase()))
                .count();
            if hits > 0 {
                scored.push((pack.name.clone(), hits as f64 * 2.0 + pack.salience));
            }
        }

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        let mut routed = Vec::new();
        let mut used = 0usize;
        for (name, score) in scored {
            let tokens = self.packs.get(&name).map_or(0, |p| est_tokens(&p.content));
            if used.saturating_add(tokens) > self.token_cap {
                continue;
            }
            used = used.saturating_add(tokens);
            if let Some(pack) = self.packs.get_mut(&name) {
                pack.use_count = pack.use_count.saturating_add(1);
                pack.last_used_turn = self.turn;
            }
            routed.push(NtRoutedPack {
                name,
                score,
                tokens,
            });
        }
        routed
    }

    /// 推进一轮：时钟 + 非置顶包显著性衰减。
    pub fn advance_turn(&mut self) {
        self.turn = self.turn.saturating_add(1);
        for pack in self.packs.values_mut() {
            if pack.kind != NtPackKind::Pinned {
                pack.salience = (pack.salience * self.decay).clamp(0.0, 1.0);
            }
        }
    }

    /// 剪枝：删长期不用 + 低显著性的包，Pinned 永生。
    /// Ephemeral 只要低于阈值就删；OnDemand 需同时失活超 `stale_turns`。
    pub fn prune(&mut self) -> NtPruneReport {
        let mut pruned = Vec::new();
        let mut freed = 0usize;
        let mut drop_names = Vec::new();

        for pack in self.packs.values() {
            if pack.kind == NtPackKind::Pinned {
                continue;
            }
            let stale = self.turn.saturating_sub(pack.last_used_turn) > self.stale_turns;
            let weak = pack.salience < self.prune_threshold;
            let drop = match pack.kind {
                NtPackKind::Ephemeral => weak,
                NtPackKind::OnDemand => weak && stale,
                NtPackKind::Pinned => false,
            };
            if drop {
                drop_names.push(pack.name.clone());
            }
        }

        for name in drop_names {
            if let Some(pack) = self.packs.remove(&name) {
                freed = freed.saturating_add(est_tokens(&pack.content));
                pruned.push(name);
            }
        }
        pruned.sort();
        NtPruneReport {
            pruned,
            freed_tokens: freed,
            remaining_packs: self.packs.len(),
        }
    }

    /// 主记忆分支 → Critical 意图（压缩时保留分支）。
    /// 空分支返回 `None`，调用方可直接跳过注入。
    pub fn main_branch_intent(&self) -> Option<CompactionIntent> {
        if self.main_branch.is_empty() {
            return None;
        }
        let content = self.recall();
        let reserve = est_tokens(&content);
        let peak = self
            .main_branch
            .iter()
            .map(|n| n.salience)
            .fold(0.0f64, f64::max);
        Some(CompactionIntent {
            session_id: self.session_id.clone(),
            previous_salience: peak.clamp(0.0, 1.0),
            target_cursor: self.turn.min(usize::MAX as u64) as usize,
            summary_blocks: vec![content],
            current_source: SourceType::KnowledgeBase,
            reserve_tokens: reserve,
            priority: CompactionPriority::Critical,
        })
    }

    /// 一轮组装：路由命中包 + 额外源 → 预算组装，主记忆分支走 Critical 注入。
    /// 这是能力机制的统一入口：调用方每轮只调这一个函数。
    pub fn assemble_turn(
        &mut self,
        budget: &ContextBudget,
        query: &str,
        extra: &[(SourceType, String)],
    ) -> AssembledContext {
        let routed = self.route(query);
        let mut sources: Vec<(SourceType, String)> = Vec::new();
        for r in &routed {
            if let Some(pack) = self.packs.get(&r.name) {
                sources.push((pack.source.clone(), pack.content.clone()));
            }
        }
        for (source, content) in extra {
            sources.push((source.clone(), content.clone()));
        }
        match self.main_branch_intent() {
            Some(intent) => budget.assemble_with_intent(&sources, &[intent]),
            None => budget.assemble(&sources),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(name: &str, kind: NtPackKind, triggers: &[&str], content: &str) -> NtRulePack {
        NtRulePack::new(name, kind, SourceType::System, content).with_triggers(triggers)
    }

    fn ctx() -> NtSmartContext {
        NtSmartContext::new("sess-test", 2000)
    }

    #[test]
    fn test_pinned_always_routed() {
        let mut c = ctx();
        c.register_pack(pack("p-rules", NtPackKind::Pinned, &[], "pinned rules"));
        let routed = c.route("完全无关的查询");
        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].name, "p-rules");
    }

    #[test]
    fn test_ondemand_only_on_trigger() {
        let mut c = ctx();
        c.register_pack(pack("pay", NtPackKind::OnDemand, &["支付", "pay"], "pay rules"));
        assert!(c.route("今天天气如何").is_empty());
        let routed = c.route("如何接入支付？");
        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].name, "pay");
    }

    #[test]
    fn test_route_respects_token_cap() {
        let mut c = NtSmartContext::new("s", 10);
        c.register_pack(pack("a", NtPackKind::Pinned, &[], &"x".repeat(20)));
        c.register_pack(pack("b", NtPackKind::Pinned, &[], &"y".repeat(20)));
        let routed = c.route("q");
        let total: usize = routed.iter().map(|r| r.tokens).sum();
        assert!(total <= 10, "routed {} tokens over cap 10", total);
    }

    #[test]
    fn test_route_marks_usage() {
        let mut c = ctx();
        c.register_pack(pack("pay", NtPackKind::OnDemand, &["pay"], "pay rules"));
        c.route("pay please");
        let p = c.packs.get("pay").unwrap();
        assert_eq!(p.use_count, 1);
        assert_eq!(p.last_used_turn, 0);
    }

    #[test]
    fn test_remember_upsert_and_recall() {
        let mut c = ctx();
        c.remember("goal", "上线支付", 0.9);
        c.remember("goal", "上线支付v2", 0.5);
        assert_eq!(c.branch_len(), 1);
        assert!(c.recall().contains("v2"));
        let node = &c.main_branch[0];
        assert!((node.salience - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_decay_only_non_pinned() {
        let mut c = ctx();
        c.register_pack(
            pack("p", NtPackKind::Pinned, &[], "p").with_salience(0.8),
        );
        c.register_pack(pack("o", NtPackKind::OnDemand, &["o"], "o").with_salience(0.8));
        c.advance_turn();
        assert!((c.packs.get("p").unwrap().salience - 0.8).abs() < 1e-9);
        assert!(c.packs.get("o").unwrap().salience < 0.8);
    }

    #[test]
    fn test_prune_drops_stale_ephemeral_keeps_pinned() {
        let mut c = NtSmartContext::new("s", 2000)
            .with_prune_threshold(0.5)
            .with_stale_turns(1);
        c.register_pack(pack("keep", NtPackKind::Pinned, &[], "k").with_salience(0.1));
        c.register_pack(pack("tmp", NtPackKind::Ephemeral, &["t"], "t").with_salience(0.1));
        for _ in 0..3 {
            c.advance_turn();
        }
        let report = c.prune();
        assert_eq!(report.pruned, vec!["tmp".to_string()]);
        assert!(c.packs.contains_key("keep"));
        assert_eq!(report.remaining_packs, 1);
    }

    #[test]
    fn test_prune_spares_fresh_registered() {
        let mut c = NtSmartContext::new("s", 2000).with_prune_threshold(0.9);
        c.register_pack(pack("fresh", NtPackKind::OnDemand, &["f"], "f").with_salience(0.1));
        let report = c.prune();
        assert!(report.pruned.is_empty());
    }

    #[test]
    fn test_main_branch_intent_none_when_empty() {
        let c = ctx();
        assert!(c.main_branch_intent().is_none());
    }

    #[test]
    fn test_assemble_turn_injects_branch_critical() {
        let mut c = ctx();
        c.register_pack(pack("pay", NtPackKind::OnDemand, &["pay"], "pay rules"));
        c.remember("decision", "选方案A", 0.95);
        let budget = ContextBudget::new(500);
        let assembled =
            c.assemble_turn(&budget, "pay 接入", &[(SourceType::Prompt, "hi".to_string())]);
        let kb: Vec<_> = assembled
            .slices
            .iter()
            .filter(|s| matches!(s.source, SourceType::KnowledgeBase))
            .collect();
        assert_eq!(kb.len(), 1);
        assert!(kb[0].content.contains("选方案A"));
        assert!(assembled.total_used <= assembled.budget);
    }
}
