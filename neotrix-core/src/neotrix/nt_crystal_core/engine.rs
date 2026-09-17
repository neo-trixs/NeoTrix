//! 晶体核心引擎 — 吸收 → 熔炼 → 进化 → 输出

use super::{CrystalCore, crystal_root};

/// 待吸收的经验条目
#[derive(Debug, Clone)]
pub struct PendingAbsorbItem {
    pub context: String,
    pub action: String,
    pub result: String,
    pub reflection: String,
    pub domain: String,
}

/// 晶体引擎 — 驱动核心循环
pub struct CrystalEngine;

impl CrystalEngine {
    /// 初始化晶体核心 (创建默认核心)
    pub fn init(name: impl Into<String>) -> Result<CrystalCore, String> {
        let core = CrystalCore::new(name);
        core.save()?;
        Ok(core)
    }

    /// 加载已有核心
    pub fn load() -> Result<CrystalCore, String> {
        CrystalCore::load()
    }

    /// 确保核心存在 (不存在则创建)
    pub fn ensure() -> Result<CrystalCore, String> {
        if crystal_root().join("crystal.json").exists() {
            Self::load()
        } else {
            Self::init("NeoTrix")
        }
    }

    /// 解析 pending-absorb.json 内容为待吸收条目
    pub fn parse_pending_absorb(content: &str) -> Result<Vec<PendingAbsorbItem>, String> {
        let v: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| format!("invalid pending JSON: {e}"))?;

        let items: Vec<&serde_json::Value> = match &v {
            serde_json::Value::Array(arr) => arr.iter().collect(),
            _ => vec![&v],
        };

        if items.is_empty() {
            return Err("pending JSON is empty".to_string());
        }

        let mut result = Vec::new();
        for item in items {
            let context = item.get("context").and_then(|v| v.as_str()).unwrap_or("unknown");
            let action = item.get("action").and_then(|v| v.as_str()).unwrap_or("unknown");
            let result_str = item.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
            let reflection = item.get("reflection").and_then(|v| v.as_str()).unwrap_or("");
            let domain = item.get("domain").and_then(|v| v.as_str()).unwrap_or("general");

            result.push(PendingAbsorbItem {
                context: context.to_string(),
                action: action.to_string(),
                result: result_str.to_string(),
                reflection: reflection.to_string(),
                domain: domain.to_string(),
            });
        }

        Ok(result)
    }

    /// 吸收: 将新信息存入经验层
    pub fn absorb(
        core: &mut CrystalCore,
        context: &str,
        action: &str,
        result: &str,
        reflection: &str,
        domain: &str,
    ) -> Result<String, String> {
        let quality = Self::assess_quality(result);
        let id = core.experience.record_episode(
            context, action, result, reflection, domain, quality,
        );
        core.save()?;
        Ok(id)
    }

    /// 吸收失败: 记录教训
    pub fn absorb_failure(
        core: &mut CrystalCore,
        description: &str,
        root_cause: &str,
        fix: &str,
        takeaway: &str,
        category: &str,
        severity: f64,
    ) -> Result<String, String> {
        let id = core.experience.record_failure(
            description, root_cause, fix, takeaway, category, severity,
        );
        // 失败会降低安全评分，但提高进化评分
        let current = core.evolution.capability_scores.get("evolution");
        core.evolution.update_score("evolution", (current + 0.02).min(1.0));
        core.save()?;
        Ok(id)
    }

    /// 吸收成功: 记录方案
    pub fn absorb_success(
        core: &mut CrystalCore,
        problem: &str,
        approach: &str,
        result: &str,
        reusable: bool,
        pattern: &str,
        domain: &str,
    ) -> Result<String, String> {
        let id = core.experience.record_success(
            problem, approach, result, reusable, pattern, domain,
        );
        // 成功会提高相关能力评分
        let current = core.evolution.capability_scores.get(domain);
        core.evolution.update_score(domain, (current + 0.05).min(1.0));
        core.save()?;
        Ok(id)
    }

    /// 熔炼: 从经验中提取模式，更新知识层
    pub fn fuse(core: &mut CrystalCore) -> FusionReport {
        let mut report = FusionReport::default();
        let stats = core.experience.stats();

        // 从高评分经验中提取模式
        for ep in &core.experience.episodes {
            if ep.quality > 0.8 {
                report.high_quality_episodes += 1;
            }
        }

        // 从可复用方案中提取因果链
        for sol in core.experience.reusable_solutions() {
            let pattern = super::knowledge::CausalPattern {
                id: format!("FP-{:04}", core.knowledge.patterns.len() + 1),
                if_conditions: vec![sol.problem.clone()],
                then_consequences: vec![sol.approach.clone()],
                so_implications: vec![sol.pattern.clone()],
                theory_origin: "Experience".into(),
                confidence: 0.7,
            };
            core.knowledge.add_pattern(pattern);
            report.new_patterns += 1;
        }

        // 更新领域标签
        let domains: Vec<String> = core.experience.episodes.iter()
            .map(|e| e.domain.clone())
            .collect();
        let unique_domains: Vec<String> = domains.into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        core.knowledge.domain_tags = unique_domains;

        report.total_episodes = stats.total_episodes;
        report.total_failures = stats.total_failures;
        report.total_successes = stats.total_successes;
        report
    }

    /// 进化: 评估能力，识别差距，生成改进目标
    pub fn evolve(core: &mut CrystalCore) -> EvolutionReport {
        let mut report = EvolutionReport::default();

        // 获取当前总体评分
        let overall_before = core.evolution.capability_scores.overall();
        report.score_before = overall_before;

        // 识别能力差距
        let gaps = core.evolution.capability_scores.gaps(0.5);
        report.gaps = gaps.clone();

        // 根据经验更新评分
        let stats = core.experience.stats();
        if stats.total_episodes > 10 {
            let avg_q = stats.avg_episode_quality;
            let current = core.evolution.capability_scores.get("reasoning");
            core.evolution.update_score("reasoning", (current + avg_q * 0.01).min(1.0));
        }
        if stats.total_failures > 5 {
            let current = core.evolution.capability_scores.get("safety");
            core.evolution.update_score("safety", (current + 0.01).min(1.0));
        }
        if stats.reusable_solutions > 3 {
            let current = core.evolution.capability_scores.get("creativity");
            core.evolution.update_score("creativity", (current + 0.02).min(1.0));
        }

        // 记录生长周期
        let overall_after = core.evolution.capability_scores.overall();
        report.score_after = overall_after;

        core.evolution.record_cycle(
            core.evolution.current_phase.clone(),
            vec![format!("Fused {} patterns, resolved {} gaps", report.new_patterns, gaps.len())],
            overall_before,
            overall_after,
            0,
        );

        // 生成改进建议
        for (capability, score) in &gaps {
            report.recommendations.push(format!(
                "Improve {}: current={:.2}, target=0.5",
                capability, score
            ));
        }

        report
    }

    /// 输出: 综合所有层生成响应上下文
    pub fn output(core: &CrystalCore, task_domain: &str) -> OutputContext {
        let mut ctx = OutputContext::default();

        // L1: 身份约束
        ctx.identity_name = core.identity.name.clone();
        ctx.axioms = core.identity.axioms.iter().map(|a| format!("{}: {}", a.id, a.name)).collect();
        ctx.value_weights = core.identity.values.weights.clone();

        // L2: 相关知识
        for (id, theory) in &core.knowledge.theories {
            ctx.relevant_theories.push(format!("{}: {}", id, theory.core_claim));
        }
        let patterns: Vec<_> = core.knowledge.patterns_by_theory(task_domain)
            .into_iter()
            .map(|p| format!("{}: IF {} THEN {} SO {}",
                p.id,
                p.if_conditions.join(" AND "),
                p.then_consequences.join(", "),
                p.so_implications.join(", ")))
            .collect();
        ctx.relevant_patterns = patterns;

        // L3: 相关经验
        let episodes: Vec<_> = core.experience.episodes_by_domain(task_domain)
            .into_iter()
            .map(|e| format!("{}: {} → {}", e.context, e.action, e.result))
            .collect();
        ctx.relevant_experiences = episodes;

        // L4: 当前能力
        ctx.capability_scores = core.evolution.capability_scores.scores.clone();
        ctx.current_phase = core.evolution.current_phase.clone();
        ctx.overall_score = core.evolution.capability_scores.overall();

        ctx
    }

    /// 评估结果质量
    fn assess_quality(result: &str) -> f64 {
        let lower = result.to_lowercase();
        if lower.contains("success") || lower.contains("pass") || lower.contains("ok") {
            0.9
        } else if lower.contains("partial") || lower.contains("mostly") {
            0.6
        } else if lower.contains("fail") || lower.contains("error") {
            0.2
        } else {
            0.5
        }
    }
}

/// 熔炼报告
#[derive(Debug, Default)]
pub struct FusionReport {
    pub total_episodes: usize,
    pub total_failures: usize,
    pub total_successes: usize,
    pub high_quality_episodes: usize,
    pub new_patterns: usize,
}

/// 进化报告
#[derive(Debug, Default)]
pub struct EvolutionReport {
    pub score_before: f64,
    pub score_after: f64,
    pub gaps: Vec<(String, f64)>,
    pub new_patterns: usize,
    pub recommendations: Vec<String>,
}

/// 输出上下文
#[derive(Debug, Default)]
pub struct OutputContext {
    pub identity_name: String,
    pub axioms: Vec<String>,
    pub value_weights: std::collections::HashMap<String, f64>,
    pub relevant_theories: Vec<String>,
    pub relevant_patterns: Vec<String>,
    pub relevant_experiences: Vec<String>,
    pub capability_scores: std::collections::HashMap<String, f64>,
    pub current_phase: super::evolution::GrowthPhase,
    pub overall_score: f64,
}

impl std::fmt::Display for OutputContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Crystal Output Context ===")?;
        writeln!(f, "Identity: {} (score: {:.2})", self.identity_name, self.overall_score)?;
        writeln!(f, "Phase: {:?}", self.current_phase)?;
        writeln!(f, "Axioms: {}", self.axioms.len())?;
        writeln!(f, "Theories: {}", self.relevant_theories.len())?;
        writeln!(f, "Patterns: {}", self.relevant_patterns.len())?;
        writeln!(f, "Experiences: {}", self.relevant_experiences.len())?;
        writeln!(f, "Capabilities:")?;
        for (k, v) in &self.capability_scores {
            writeln!(f, "  {}: {:.2}", k, v)?;
        }
        Ok(())
    }
}
