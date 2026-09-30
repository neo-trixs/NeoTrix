//! NT-ORCHESTRATOR — 进化闭环调度器（Phase 1 缺陷修复：#1 无调度器、#2 无回灌）
//!
//! 把孤岛模块串成 tick：
//! ```text
//! 外部观测 → predict.observe（FEP 探索）→ awaken.cycle（自醒）
//!          → 定时 evolve（进化）→ 定时 save（落盘）
//! 模型输出 → remember_inference（回灌，来源标注）→ 同上循环
//! ```
//! reflect 由 cycle 内部按 reflect_every 触发（Generative Agents 映射）。
//! 纯调度逻辑；重 IO（save/cocoon）由调用方配频。无 unwrap / expect / panic；无 `[]` 索引。

use super::consciousness::{CrystalConsciousness, MemoryType};
use super::engine::CrystalEngine;
use super::{CocoonStore, CrystalCore, NtAwakenLoop, NtPredictLoop};

/// 调度配置
#[derive(Debug, Clone)]
pub struct NtOrchestratorConfig {
    /// 每 tick 自醒轮数
    pub awaken_rounds: usize,
    /// 验证奖励门
    pub reward_floor: f64,
    /// 每 N 个 cycle 触发 reflect（透传给 awaken）
    pub reflect_every: u64,
    /// 注意力捕获阈值（透传给 predict）
    pub capture_threshold: f64,
    /// 每 N tick 做一次 evolve（对手过程 + 熔炼 + 评分）
    pub evolve_every: u64,
    /// evolve tick 是否落盘 crystal.json
    pub save_on_evolve: bool,
    /// evolve tick 是否同步 cocoons（477K 全量遍历，贵，默认关，按需开）
    pub sync_cocoons_on_evolve: bool,
    /// predict EMA 系数
    pub predict_alpha: f64,
}

impl Default for NtOrchestratorConfig {
    fn default() -> Self {
        Self {
            awaken_rounds: 4,
            reward_floor: 0.5,
            reflect_every: 5,
            capture_threshold: 0.6,
            evolve_every: 3,
            save_on_evolve: true,
            sync_cocoons_on_evolve: false,
            predict_alpha: 0.3,
        }
    }
}

/// 单 tick 报告
#[derive(Debug, Default)]
pub struct OrchestratorReport {
    pub tick: u64,
    pub observed: usize,
    pub avg_surprise: f64,
    pub proposed: usize,
    pub chosen: usize,
    pub rejected: usize,
    pub avg_reward: f64,
    pub reflected: bool,
    pub evolved: bool,
    pub saved: bool,
    pub cocoons_synced: bool,
    pub error: Option<String>,
}

/// 回灌来源（标注进 content 前缀，可扫描过滤）
#[derive(Debug, Clone)]
pub enum InferenceSource {
    /// LLM 生成：模型名
    Llm(String),
    /// AgentJev 决策：问题 id
    AgentJev(String),
    /// 人类回灌
    Human,
}

impl InferenceSource {
    fn tag(&self) -> String {
        match self {
            Self::Llm(m) => format!("llm/{m}"),
            Self::AgentJev(q) => format!("agentjev/{q}"),
            Self::Human => "human".to_string(),
        }
    }
}

/// 进化闭环调度器（持有子循环状态：课程访问计数 + 域惊异 EMA）
pub struct NtOrchestrator {
    awaken: NtAwakenLoop,
    predict: NtPredictLoop,
    config: NtOrchestratorConfig,
    ticks: u64,
}

impl NtOrchestrator {
    pub fn new(config: NtOrchestratorConfig) -> Self {
        let alpha = config.predict_alpha;
        Self {
            awaken: NtAwakenLoop::new(),
            predict: NtPredictLoop::new(alpha),
            config,
            ticks: 0,
        }
    }

    /// 单 tick：观测 → 自醒 →（定时）进化 →（定时）落盘
    ///
    /// observations: (内容, 域, 置信度) 外部观测；空则只做自醒。
    pub fn tick(
        &mut self,
        consciousness: &mut CrystalConsciousness,
        core: &mut CrystalCore,
        observations: &[(String, String, f64)],
    ) -> OrchestratorReport {
        self.ticks += 1;
        let mut rep = OrchestratorReport {
            tick: self.ticks,
            ..Default::default()
        };

        // 1. 探索：外部观测先过 FEP（surprise + EMA + 注意力捕获），再入记忆
        let mut surprise_sum = 0.0;
        for (content, domain, confidence) in observations {
            let (_, s) = self.predict.observe(
                consciousness,
                content,
                domain,
                *confidence,
                self.config.capture_threshold,
            );
            surprise_sum += s;
            rep.observed += 1;
        }
        if rep.observed > 0 {
            rep.avg_surprise = surprise_sum / rep.observed as f64;
        }

        // 2. 自醒：选题 → 推理 → 验证 → 奖惩（reflect 由内部 reflect_every 触发）
        let cycle = self.awaken.cycle(
            consciousness,
            core,
            self.config.awaken_rounds,
            self.config.reward_floor,
            self.config.reflect_every,
        );
        rep.proposed = cycle.proposed;
        rep.chosen = cycle.chosen;
        rep.rejected = cycle.rejected;
        rep.avg_reward = cycle.avg_reward;
        rep.reflected = cycle.reflected;

        // 3. 定时进化 + 落盘
        if self.ticks % self.config.evolve_every.max(1) == 0 {
            let _ = CrystalEngine::evolve(core);
            rep.evolved = true;
            if self.config.save_on_evolve {
                match core.save() {
                    Ok(()) => rep.saved = true,
                    Err(e) => rep.error = Some(e),
                }
            }
            if self.config.sync_cocoons_on_evolve && rep.error.is_none() {
                let mut store = CocoonStore::load();
                store.sync_from_consciousness(consciousness);
                match store.save() {
                    Ok(()) => rep.cocoons_synced = true,
                    Err(e) => rep.error = Some(e),
                }
            }
        }

        rep
    }

    /// 模型输出回灌（缺陷 #2 修复）：LLM 生成 / AgentJev 决策 / 人类 → 记忆。
    ///
    /// 来源标注进 content 前缀 `[src:{tag}]`，域保持语义域（可按域召回），
    /// 类型记 Fact（观测性事实）。空内容或与既有 Pattern 重复（缺陷 #4）
    /// 则拒绝并返回 None，防垃圾进记忆。
    pub fn remember_inference(
        consciousness: &mut CrystalConsciousness,
        content: &str,
        source: &InferenceSource,
        domain: &str,
        confidence: f64,
    ) -> Option<String> {
        let body = content.trim();
        if body.is_empty() {
            return None;
        }
        let tagged = format!("[src:{}] {body}", source.tag());
        let id = consciousness.remember(
            tagged,
            MemoryType::Fact,
            domain,
            confidence.clamp(0.0, 1.0),
        );
        if consciousness.is_duplicate_pattern(&id) {
            consciousness.memories.remove(&id);
            return None;
        }
        Some(id)
    }

    /// 按来源前缀扫描回灌记忆（与 remember_inference 配对）
    pub fn recall_inference(
        consciousness: &CrystalConsciousness,
        source_tag: &str,
        limit: usize,
    ) -> Vec<String> {
        let prefix = format!("[src:{source_tag}]");
        consciousness
            .memories
            .values()
            .filter(|m| m.content.starts_with(prefix.as_str()))
            .take(limit.max(1))
            .map(|m| m.id.clone())
            .collect()
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// 当前自醒轮数（自迭代循环爬坡时读取）
    pub fn awaken_rounds(&self) -> usize {
        self.config.awaken_rounds
    }

    /// 设置自醒轮数（自迭代循环 ±1 爬坡；下限 1 防停转）
    pub fn set_awaken_rounds(&mut self, n: usize) {
        self.config.awaken_rounds = n.max(1);
    }

    /// 爬取优先级透出（predict EMA → 调用方决定何时 push 到 crawl_queue）
    pub fn crawl_priorities(&self) -> Vec<(String, f64)> {
        self.predict.crawl_priorities()
    }
}

impl Default for NtOrchestrator {
    fn default() -> Self {
        Self::new(NtOrchestratorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::super::consciousness::MemoryType;
    use super::*;

    fn seeded() -> (CrystalCore, CrystalConsciousness) {
        let core = CrystalCore::new("t");
        let mut c = CrystalConsciousness::new("t");
        for i in 0..4 {
            c.remember(format!("事实{i}"), MemoryType::Fact, "d", 0.8);
        }
        (core, c)
    }

    #[test]
    fn test_tick_runs_observe_and_awaken() {
        let (mut core, mut c) = seeded();
        let mut orch = NtOrchestrator::default();
        let obs = vec![("新现象 需要 解释".to_string(), "d".to_string(), 0.7)];
        let rep = orch.tick(&mut c, &mut core, &obs);
        assert_eq!(rep.tick, 1);
        assert_eq!(rep.observed, 1);
        assert!(rep.avg_surprise >= 0.0);
        assert!(rep.proposed > 0, "awaken must propose");
        assert_eq!(rep.chosen + rep.rejected, rep.proposed);
    }

    #[test]
    fn test_tick_empty_observations_only_awakens() {
        let (mut core, mut c) = seeded();
        let mut orch = NtOrchestrator::default();
        let rep = orch.tick(&mut c, &mut core, &[]);
        assert_eq!(rep.observed, 0);
        assert!(rep.proposed > 0);
    }

    #[test]
    fn test_evolve_fires_on_schedule() {
        let (mut core, mut c) = seeded();
        let mut orch = NtOrchestrator::new(NtOrchestratorConfig {
            evolve_every: 2,
            save_on_evolve: false,
            ..Default::default()
        });
        let r1 = orch.tick(&mut c, &mut core, &[]);
        assert!(!r1.evolved);
        let r2 = orch.tick(&mut c, &mut core, &[]);
        assert!(r2.evolved);
        assert!(!r2.saved, "save disabled must not save");
    }

    #[test]
    fn test_remember_inference_tags_and_recalls() {
        let (_, mut c) = seeded();
        let id = NtOrchestrator::remember_inference(
            &mut c,
            "淬火 提升 韧性",
            &InferenceSource::Llm("minimind-3".to_string()),
            "physics",
            0.75,
        )
        .unwrap();
        let m = c.memories.get(&id).unwrap();
        assert!(m.content.starts_with("[src:llm/minimind-3]"));
        assert_eq!(m.domain, "physics");
        let found = NtOrchestrator::recall_inference(&c, "llm/minimind-3", 10);
        assert!(found.contains(&id));
        // 空内容拒绝
        assert!(NtOrchestrator::remember_inference(
            &mut c,
            "   ",
            &InferenceSource::Human,
            "d",
            0.5
        )
        .is_none());
    }

    #[test]
    fn test_agentjev_source_tag() {
        let (_, mut c) = seeded();
        let id = NtOrchestrator::remember_inference(
            &mut c,
            "测试通过",
            &InferenceSource::AgentJev("done".to_string()),
            "ci",
            0.92,
        )
        .unwrap();
        assert!(c.memories.get(&id).unwrap().content.starts_with("[src:agentjev/done]"));
    }

    #[test]
    fn test_duplicate_pattern_rejected_on_ingest() {
        use super::super::consciousness::MemoryType as MT;
        let (_, mut c) = seeded();
        // 既有 Pattern
        c.remember("火焰 燃烧 释放 大量 热量", MT::Pattern, "physics", 0.9);
        // 近重复回灌 → 拒绝（5/6=0.83 ≥ 0.8；前缀占 1 个 token）
        let dup = NtOrchestrator::remember_inference(
            &mut c,
            "火焰 燃烧 释放 大量 热量",
            &InferenceSource::Llm("m".to_string()),
            "physics",
            0.8,
        );
        assert!(dup.is_none(), "near-duplicate of Pattern must be rejected");
        // 无关内容 → 通过
        let ok = NtOrchestrator::remember_inference(
            &mut c,
            "量子 纠缠 贝尔 不等式 验证",
            &InferenceSource::Llm("m".to_string()),
            "physics",
            0.8,
        );
        assert!(ok.is_some());
    }

    #[test]
    fn test_is_duplicate_pattern_direct() {
        use super::super::consciousness::MemoryType as MT;
        let (_, mut c) = seeded();
        let p = c.remember("光合作用 需要 叶绿素 阳光", MT::Pattern, "bio", 0.9);
        assert!(!c.is_duplicate_pattern(&p), "self must not match");
        let f1 = c.remember("光合作用 需要 叶绿素 阳光", MT::Fact, "bio", 0.8);
        assert!(c.is_duplicate_pattern(&f1), "identical content must dup");
        let f2 = c.remember("暗物质 只 参与 引力 作用", MT::Fact, "astro", 0.8);
        assert!(!c.is_duplicate_pattern(&f2), "unrelated must not dup");
        assert!(!c.is_duplicate_pattern("M-nonexistent"), "missing id safe");
    }

    #[test]
    fn test_crawl_priorities_passthrough() {
        let (mut core, mut c) = seeded();
        let mut orch = NtOrchestrator::default();
        let obs = vec![("暗物质 只 参与 引力".to_string(), "astro".to_string(), 0.8)];
        orch.tick(&mut c, &mut core, &obs);
        let ranks = orch.crawl_priorities();
        assert!(!ranks.is_empty());
        assert_eq!(ranks.first().map(|(d, _)| d.as_str()), Some("astro"));
    }
}
