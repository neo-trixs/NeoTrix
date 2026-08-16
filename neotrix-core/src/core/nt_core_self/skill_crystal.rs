use super::attention_head::AttentionDomain;
use super::reasoning_strategy::StrategyKind;
use super::thinking_trace::ThinkingTrace;
use std::path::Path;

/// 验证状态 — DeepZero assessment.j2 反幻觉协议吸收 (P0-1):
/// LLM/蒸馏产出的洞察必须带可复现验证字段，未验证 ≠ 有效。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VerificationStatus {
    /// 未提供验证契约（默认，反幻觉门标记）
    Unverified,
    /// 已提供验证契约但尚未执行验证
    Pending,
    /// 验证契约已执行且确认成立
    Confirmed,
    /// 验证契约已执行且被推翻
    Refuted,
}

/// 可复现验证契约 — 对应 DeepZero `confirms_if` 字段：
/// 给出"什么可观察结果能区分真实命中与假阳性"，让后续步骤能独立复现。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VerificationContract {
    /// 可观察结果描述（如 "输出缓冲区非零且匹配调用方从未提供的内存"）
    pub observable: String,
    /// 验证方法（如 "运行 PoC 检查返回值" / "重放 trace 检查行为改变"）
    pub method: String,
    /// 当前验证状态
    pub status: VerificationStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillCrystal {
    pub id: usize,
    pub name: String,
    pub pattern: String,
    pub effectiveness: f64,
    pub use_count: usize,
    pub source_trace_ids: Vec<usize>,
    pub tags: Vec<String>,
    pub strategy: StrategyKind,
    pub domain: AttentionDomain,
    pub created_at: usize,
    pub last_used: usize,
    /// 反幻觉验证契约（P0-1 吸收自 DeepZero assessment.j2）。
    /// `#[serde(default)]` 保证旧序列化数据反序列化时缺字段 → None，不破坏兼容。
    #[serde(default)]
    pub verification: Option<VerificationContract>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrystalRegistry {
    pub(crate) crystals: Vec<SkillCrystal>,
    pub max_crystals: usize,
    pub(crate) next_id: usize,
    pub auto_prune_enabled: bool,
}

impl CrystalRegistry {
    pub fn crystals(&self) -> &[SkillCrystal] {
        &self.crystals
    }
}

impl SkillCrystal {
    pub fn new(
        id: usize,
        name: &str,
        pattern: &str,
        strategy: StrategyKind,
        domain: AttentionDomain,
        iteration: usize,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            pattern: pattern.to_string(),
            effectiveness: 0.7,
            use_count: 0,
            source_trace_ids: Vec::new(),
            tags: Vec::new(),
            strategy,
            domain,
            created_at: iteration,
            last_used: iteration,
            verification: None,
        }
    }
}

impl Default for CrystalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CrystalRegistry {
    pub fn new() -> Self {
        Self {
            crystals: Vec::new(),
            max_crystals: 50,
            next_id: 0,
            auto_prune_enabled: true,
        }
    }

    pub fn extract_from_trace(&mut self, trace: &ThinkingTrace, iteration: usize) -> Option<usize> {
        if trace.grade.score() < 0.75 {
            return None;
        }

        let strategies = trace.strategies_used();
        if strategies.is_empty() {
            return None;
        }
        let dominant_strategy = strategies[0];

        let domains = trace.domains_used();
        let dominant_domain = if domains.is_empty() {
            AttentionDomain::PatternMatch
        } else {
            domains[0]
        };

        if let Some(existing) = self.find_similar(dominant_strategy, dominant_domain) {
            let existing_id = existing.id;
            let crystal = self.crystals.iter_mut().find(|c| c.id == existing_id)?;
            if !crystal.source_trace_ids.contains(&trace.id) {
                crystal.source_trace_ids.push(trace.id);
            }
            let old_eff = crystal.effectiveness;
            let new_eff = trace.grade.score() * 0.8 + 0.2;
            crystal.effectiveness = (old_eff + new_eff) / 2.0;
            crystal.last_used = iteration;
            crystal.use_count += 1;
            return Some(crystal.id);
        }

        let name = if trace.task.len() > 40 {
            let truncated: String = trace.task.chars().take(37).collect();
            format!("{}...", truncated)
        } else {
            trace.task.clone()
        };

        let pattern = {
            let mut descs: Vec<String> = strategies.iter().map(|s| s.label().to_string()).collect();
            descs.dedup();
            descs.join(", ")
        };

        let effectiveness = trace.grade.score() * 0.8 + 0.2;

        let tags: Vec<String> = {
            let mut t: Vec<String> = Vec::new();
            for step in &trace.steps {
                let label = step.domain.label().to_string();
                if !t.contains(&label) {
                    t.push(label);
                }
            }
            t
        };

        let crystal = SkillCrystal {
            id: self.next_id,
            name,
            pattern,
            effectiveness,
            use_count: 1,
            source_trace_ids: vec![trace.id],
            tags,
            strategy: dominant_strategy,
            domain: dominant_domain,
            created_at: iteration,
            last_used: iteration,
            verification: None,
        };

        let id = crystal.id;
        self.crystals.push(crystal);
        self.next_id += 1;

        if self.auto_prune_enabled && self.crystals.len() > self.max_crystals {
            self.auto_prune();
        }

        Some(id)
    }

    pub fn find_similar(
        &self,
        strategy: StrategyKind,
        domain: AttentionDomain,
    ) -> Option<&SkillCrystal> {
        self.crystals
            .iter()
            .find(|c| c.strategy == strategy && c.domain == domain)
    }

    pub fn merge_crystals(&mut self, id_a: usize, id_b: usize) -> Option<usize> {
        let idx_a = self.crystals.iter().position(|c| c.id == id_a)?;
        let idx_b = self.crystals.iter().position(|c| c.id == id_b)?;

        if idx_a == idx_b {
            return Some(id_a);
        }

        let b = self.crystals[idx_b].clone();
        let crystal = &mut self.crystals[idx_a];

        for tid in &b.source_trace_ids {
            if !crystal.source_trace_ids.contains(tid) {
                crystal.source_trace_ids.push(*tid);
            }
        }

        crystal.effectiveness = (crystal.effectiveness + b.effectiveness) / 2.0;
        crystal.use_count += b.use_count;
        if b.use_count > crystal.use_count - b.use_count {
            crystal.name = b.name;
        }
        crystal.last_used = crystal.last_used.max(b.last_used);
        if crystal.created_at > b.created_at {
            crystal.created_at = b.created_at;
        }

        for tag in &b.tags {
            if !crystal.tags.contains(tag) {
                crystal.tags.push(tag.clone());
            }
        }

        let merged_id = crystal.id;

        self.crystals.remove(idx_b);

        Some(merged_id)
    }

    pub fn prune_weak(&mut self, min_effectiveness: f64) -> usize {
        let before = self.crystals.len();
        self.crystals.retain(|c| {
            if c.effectiveness >= min_effectiveness {
                return true;
            }
            c.use_count > 3
        });
        before - self.crystals.len()
    }

    pub fn auto_prune(&mut self) -> usize {
        if self.crystals.len() <= self.max_crystals {
            return 0;
        }
        let to_remove = self.crystals.len() - self.max_crystals;

        let mut candidates: Vec<usize> = (0..self.crystals.len()).collect();
        candidates.sort_by(|&a, &b| {
            let ca = &self.crystals[a];
            let cb = &self.crystals[b];
            if ca.use_count == 0 && cb.use_count > 0 {
                return std::cmp::Ordering::Less;
            }
            if cb.use_count == 0 && ca.use_count > 0 {
                return std::cmp::Ordering::Greater;
            }
            ca.effectiveness
                .partial_cmp(&cb.effectiveness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let remove_indices: std::collections::HashSet<usize> =
            candidates.into_iter().take(to_remove).collect();

        let mut sorted: Vec<usize> = remove_indices.into_iter().collect();
        sorted.sort_unstable_by(|a, b| b.cmp(a));

        for idx in sorted {
            self.crystals.remove(idx);
        }

        to_remove
    }

    pub fn record_use(&mut self, id: usize, iteration: usize) {
        if let Some(crystal) = self.crystals.iter_mut().find(|c| c.id == id) {
            crystal.use_count += 1;
            crystal.last_used = iteration;
        }
    }

    pub fn summary(&self) -> String {
        if self.crystals.is_empty() {
            return "CrystalRegistry: 0 crystals".to_string();
        }
        let best_eff = self
            .crystals
            .iter()
            .map(|c| c.effectiveness)
            .fold(0.0_f64, f64::max);
        let total_use: usize = self.crystals.iter().map(|c| c.use_count).sum();
        let domain_counts: Vec<String> = {
            let mut map: std::collections::HashMap<AttentionDomain, usize> =
                std::collections::HashMap::new();
            for c in &self.crystals {
                *map.entry(c.domain).or_insert(0) += 1;
            }
            let mut pairs: Vec<(AttentionDomain, usize)> = map.into_iter().collect();
            pairs.sort_by_key(|(_, count)| *count);
            pairs.reverse();
            pairs
                .into_iter()
                .take(3)
                .map(|(d, count)| format!("{:?}:{}", d, count))
                .collect()
        };
        format!(
            "CrystalRegistry: {} crystals, best_eff={:.3}, total_use={}, top_domains=[{}]",
            self.crystals.len(),
            best_eff,
            total_use,
            domain_counts.join(", "),
        )
    }

    pub fn best_by_domain(&self, domain: AttentionDomain) -> Option<&SkillCrystal> {
        self.crystals
            .iter()
            .filter(|c| c.domain == domain)
            .max_by(|a, b| {
                a.effectiveness
                    .partial_cmp(&b.effectiveness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn best_by_strategy(&self, strategy: StrategyKind) -> Option<&SkillCrystal> {
        self.crystals
            .iter()
            .filter(|c| c.strategy == strategy)
            .max_by(|a, b| {
                a.effectiveness
                    .partial_cmp(&b.effectiveness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::thinking_trace::ReflectionGrade;
    use super::*;

    fn make_trace(
        id: usize,
        task: &str,
        grade: ReflectionGrade,
        strategy: StrategyKind,
        domain: AttentionDomain,
    ) -> ThinkingTrace {
        let mut trace = ThinkingTrace::new(id, task);
        trace.grade = grade;
        trace.steps.push(
            super::super::thinking_trace::ThinkingStep::new(1, "step", strategy)
                .with_domain(domain),
        );
        trace
    }

    #[test]
    fn test_registry_new() {
        let reg = CrystalRegistry::new();
        assert_eq!(reg.crystals.len(), 0);
        assert_eq!(reg.max_crystals, 50);
        assert_eq!(reg.next_id, 0);
    }

    #[test]
    fn test_extract_from_good_trace() {
        let mut reg = CrystalRegistry::new();
        let trace = make_trace(
            0,
            "fix rust borrow checker",
            ReflectionGrade::Good,
            StrategyKind::Reflection,
            AttentionDomain::Code,
        );
        let id = reg.extract_from_trace(&trace, 1);
        assert!(id.is_some());
        assert_eq!(reg.crystals.len(), 1);
        assert_eq!(reg.crystals[0].name, "fix rust borrow checker");
        assert_eq!(reg.crystals[0].strategy, StrategyKind::Reflection);
        assert_eq!(reg.crystals[0].domain, AttentionDomain::Code);
        assert!((reg.crystals[0].effectiveness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_extract_from_poor_trace() {
        let mut reg = CrystalRegistry::new();
        let trace = make_trace(
            0,
            "failed attempt",
            ReflectionGrade::Poor,
            StrategyKind::Direct,
            AttentionDomain::Code,
        );
        let id = reg.extract_from_trace(&trace, 1);
        assert!(id.is_none());
        assert_eq!(reg.crystals.len(), 0);
    }

    #[test]
    fn test_extract_long_cjk_task_no_panic() {
        // Regression: &trace.task[..37] sliced at byte 37 which lands
        // mid-CJK-char -> panic for any task > 40 bytes of multibyte text.
        // chars().take(37) truncates by character.
        let mut reg = CrystalRegistry::new();
        let long_task = "修复编译错误与类型系统缺陷".repeat(4);
        assert!(long_task.len() > 40);
        let trace = make_trace(
            0,
            &long_task,
            ReflectionGrade::Good,
            StrategyKind::Reflection,
            AttentionDomain::Code,
        );
        let id = reg.extract_from_trace(&trace, 1);
        assert!(id.is_some());
        let crystal = &reg.crystals[0];
        assert!(
            crystal.name.ends_with("..."),
            "long names must be ellipsized"
        );
        let name_chars = crystal.name.chars().count();
        assert!(
            name_chars <= 40,
            "truncated name must fit in 40 chars, got {}",
            name_chars
        );
        assert!(
            crystal.name.is_char_boundary(crystal.name.len() - 1) || crystal.name.ends_with('.')
        );
    }

    #[test]
    fn test_merge_crystals() {
        let mut reg = CrystalRegistry::new();
        reg.crystals.push(SkillCrystal::new(
            0,
            "debug CoT",
            "cot pattern",
            StrategyKind::ChainOfThought,
            AttentionDomain::Code,
            1,
        ));
        reg.crystals[0].source_trace_ids.push(0);
        reg.next_id = 1;
        reg.crystals.push(SkillCrystal::new(
            1,
            "debug direct",
            "direct pattern",
            StrategyKind::Direct,
            AttentionDomain::Code,
            2,
        ));
        reg.crystals[1].source_trace_ids.push(1);
        reg.next_id = 2;

        assert_eq!(reg.crystals.len(), 2);
        let merged = reg.merge_crystals(0, 1);
        assert!(merged.is_some());
        assert_eq!(reg.crystals.len(), 1);
        assert_eq!(reg.crystals[0].source_trace_ids.len(), 2);
    }

    #[test]
    fn test_prune_weak() {
        let mut reg = CrystalRegistry::new();
        reg.crystals.push(SkillCrystal::new(
            0,
            "weak",
            "pattern",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        ));
        reg.crystals[0].effectiveness = 0.3;
        reg.crystals.push(SkillCrystal::new(
            1,
            "strong",
            "pattern",
            StrategyKind::Reflection,
            AttentionDomain::Code,
            1,
        ));
        reg.crystals[1].effectiveness = 0.9;
        let removed = reg.prune_weak(0.5);
        assert_eq!(removed, 1);
        assert_eq!(reg.crystals.len(), 1);
        assert_eq!(reg.crystals[0].id, 1);
    }

    #[test]
    fn test_find_similar() {
        let mut reg = CrystalRegistry::new();
        let trace = make_trace(
            0,
            "refactor module",
            ReflectionGrade::Good,
            StrategyKind::Reflection,
            AttentionDomain::Code,
        );
        reg.extract_from_trace(&trace, 1);
        let found = reg.find_similar(StrategyKind::Reflection, AttentionDomain::Code);
        assert!(found.is_some());
        assert_eq!(found.expect("find_similar should return a crystal").id, 0);
        let not_found = reg.find_similar(StrategyKind::Direct, AttentionDomain::Code);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_auto_prune_over_max() {
        let mut reg = CrystalRegistry::new();
        reg.max_crystals = 3;
        for i in 0..5 {
            let domain = AttentionDomain::all()[i % AttentionDomain::all().len()];
            reg.crystals.push(SkillCrystal::new(
                i,
                &format!("crystal_{}", i),
                "test",
                StrategyKind::Direct,
                domain,
                1,
            ));
        }
        assert_eq!(reg.crystals.len(), 5);
        let removed = reg.auto_prune();
        assert_eq!(removed, 2);
        assert_eq!(reg.crystals.len(), 3);
    }

    #[test]
    fn test_best_by_domain_and_strategy() {
        let mut reg = CrystalRegistry::new();
        reg.crystals.push(SkillCrystal::new(
            0,
            "a",
            "p1",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        ));
        reg.crystals.push(SkillCrystal::new(
            1,
            "b",
            "p2",
            StrategyKind::Reflection,
            AttentionDomain::Code,
            1,
        ));
        reg.crystals.push(SkillCrystal::new(
            2,
            "c",
            "p3",
            StrategyKind::Direct,
            AttentionDomain::Planning,
            1,
        ));
        reg.crystals[0].effectiveness = 0.9;
        reg.crystals[1].effectiveness = 0.7;
        reg.crystals[2].effectiveness = 0.8;

        let by_domain = reg.best_by_domain(AttentionDomain::Code);
        assert!(by_domain.is_some());
        assert_eq!(
            by_domain
                .expect("best_by_domain should return a crystal")
                .id,
            0
        );

        let by_strategy = reg.best_by_strategy(StrategyKind::Direct);
        assert!(by_strategy.is_some());
        assert_eq!(
            by_strategy
                .expect("best_by_strategy should return a crystal")
                .id,
            0
        );
    }

    #[test]
    fn test_summary_format() {
        let mut reg = CrystalRegistry::new();
        let trace = make_trace(
            0,
            "test task",
            ReflectionGrade::Excellent,
            StrategyKind::Direct,
            AttentionDomain::Code,
        );
        reg.extract_from_trace(&trace, 1);
        let s = reg.summary();
        assert!(s.contains("CrystalRegistry: 1 crystals"));
    }

    #[test]
    fn test_merge_same_id_returns_unchanged() {
        let mut reg = CrystalRegistry::new();
        reg.crystals.push(SkillCrystal::new(
            0,
            "only",
            "pattern",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        ));
        let merged = reg.merge_crystals(0, 0);
        assert!(merged.is_some());
        assert_eq!(merged.expect("merge should return merged id"), 0);
        assert_eq!(reg.crystals.len(), 1);
    }

    #[test]
    fn test_record_use_updates_count_and_last_used() {
        let mut reg = CrystalRegistry::new();
        let trace = make_trace(
            0,
            "used skill",
            ReflectionGrade::Good,
            StrategyKind::Reflection,
            AttentionDomain::Code,
        );
        let id = reg
            .extract_from_trace(&trace, 1)
            .expect("extract_from_trace should return an id");
        assert_eq!(reg.crystals[0].use_count, 1);
        reg.record_use(id, 10);
        assert_eq!(reg.crystals[0].use_count, 2);
        assert_eq!(reg.crystals[0].last_used, 10);
    }
}
