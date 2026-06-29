use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// ERLHeuristicPool — Experiential Reflective Learning 启发式缓存
///
/// ERL (arXiv 2603.24639) 启发的结构化启发式记忆:
/// 从经验轨迹中提取可复用的启发式规则 (trigger_condition → recommended_action)。
/// 每个启发式包含触发条件、推荐动作、成功/失败计数、置信度。
/// 新任务到来时检索最相关的启发式注入上下文。

#[derive(Debug, Clone)]
pub struct Heuristic {
    pub id: u64,
    pub trigger_condition: String,
    pub recommended_action: String,
    pub rationale: String,
    pub source_domain: String,
    pub success_count: u32,
    pub fail_count: u32,
    pub confidence: f64,
    pub last_used: u64,
    pub timestamp: u64,
}

impl Heuristic {
    pub fn total_attempts(&self) -> u32 { self.success_count + self.fail_count }
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts() == 0 { return 0.5; }
        self.success_count as f64 / self.total_attempts() as f64
    }
    pub fn relevance_score(&self, task_description: &str, domain: &str) -> f64 {
        let mut score = self.confidence * 0.4 + self.success_rate() * 0.3;
        if self.source_domain == domain { score += 0.2; }
        let desc_lower = task_description.to_lowercase();
        let cond_lower = self.trigger_condition.to_lowercase();
        if desc_lower.contains(&cond_lower) || cond_lower.contains(&desc_lower) { score += 0.3; }
        score.min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct ERLHeuristicPoolConfig {
    pub max_heuristics: usize,
    pub min_evidence_for_active: u32,
    pub default_confidence: f64,
}

impl Default for ERLHeuristicPoolConfig {
    fn default() -> Self {
        Self { max_heuristics: 200, min_evidence_for_active: 2, default_confidence: 0.5 }
    }
}

#[derive(Debug, Clone)]
pub struct ERLHeuristicPool {
    config: ERLHeuristicPoolConfig,
    heuristics: HashMap<u64, Heuristic>,
    domain_index: HashMap<String, Vec<u64>>,
    next_id: u64,
}

impl ERLHeuristicPool {
    pub fn new(config: ERLHeuristicPoolConfig) -> Self {
        Self { config, heuristics: HashMap::new(), domain_index: HashMap::new(), next_id: 1 }
    }

    pub fn extract_heuristic(
        &mut self,
        trigger_condition: String,
        recommended_action: String,
        rationale: String,
        source_domain: String,
        success: bool,
    ) -> u64 {
        for id in self.heuristics.clone().keys() {
            if self.heuristics[id].trigger_condition == trigger_condition {
                let h = self.heuristics.get_mut(id).unwrap();
                if success { h.success_count += 1; } else { h.fail_count += 1; }
                h.confidence = (h.confidence + if success { 0.1 } else { -0.1 }).clamp(0.0, 1.0);
                h.last_used = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                return *id;
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let heuristic = Heuristic {
            id, trigger_condition: trigger_condition.clone(), recommended_action, rationale,
            source_domain: source_domain.clone(),
            success_count: if success { 1 } else { 0 },
            fail_count: if success { 0 } else { 1 },
            confidence: self.config.default_confidence, last_used: now, timestamp: now,
        };
        if self.heuristics.len() >= self.config.max_heuristics {
            if let Some(&mid) = self.heuristics.iter()
                .min_by(|a, b| a.1.last_used.cmp(&b.1.last_used)).map(|(id, _)| id)
            {
                self.heuristics.remove(&mid);
                for ids in self.domain_index.values_mut() { ids.retain(|i| *i != mid); }
            }
        }
        self.heuristics.insert(id, heuristic);
        self.domain_index.entry(source_domain).or_default().push(id);
        id
    }

    pub fn retrieve_relevant(&self, task_description: &str, domain: &str, top_k: usize) -> Vec<&Heuristic> {
        let mut scored: Vec<_> = self.heuristics.values()
            .map(|h| (h, h.relevance_score(task_description, domain)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter()
            .take(top_k)
            .filter(|(h, s)| *s > 0.3 && h.total_attempts() >= self.config.min_evidence_for_active)
            .map(|(h, _)| h)
            .collect()
    }

    pub fn record_outcome(&mut self, heuristic_id: u64, success: bool) {
        if let Some(h) = self.heuristics.get_mut(&heuristic_id) {
            if success { h.success_count += 1; } else { h.fail_count += 1; }
            h.confidence = (h.confidence + if success { 0.05 } else { -0.1 }).clamp(0.0, 1.0);
            h.last_used = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        }
    }

    pub fn heuristic_count(&self) -> usize { self.heuristics.len() }
    pub fn active_count(&self) -> usize {
        self.heuristics.values().filter(|h| h.total_attempts() >= self.config.min_evidence_for_active).count()
    }
    pub fn domain_count(&self) -> usize { self.domain_index.len() }

    pub fn summary(&self) -> String {
        format!("heuristics={} ({} active) domains={}", self.heuristic_count(), self.active_count(), self.domain_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pool() { let p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default()); assert_eq!(p.heuristic_count(), 0); }

    #[test]
    fn test_extract_basic() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        let id = p.extract_heuristic("ece>0.15".into(), "recalibrate".into(), "ECE drift".into(), "calibration".into(), true);
        assert_eq!(id, 1); assert_eq!(p.heuristic_count(), 1);
    }

    #[test]
    fn test_duplicate_merged() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        let id1 = p.extract_heuristic("ece>0.15".into(), "recalibrate".into(), "drift".into(), "calibration".into(), true);
        let id2 = p.extract_heuristic("ece>0.15".into(), "recalibrate2".into(), "drift2".into(), "calibration".into(), false);
        assert_eq!(id1, id2);
        assert_eq!(p.heuristics[&id1].success_count, 1);
        assert_eq!(p.heuristics[&id1].fail_count, 1);
    }

    #[test]
    fn test_retrieve_by_domain() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        p.extract_heuristic("a".into(), "a1".into(), "r".into(), "calibration".into(), true);
        p.extract_heuristic("b".into(), "b1".into(), "r".into(), "memory".into(), true);
        let r = p.retrieve_relevant("test", "calibration", 5);
        assert!(r.iter().any(|h| h.source_domain == "calibration"));
    }

    #[test]
    fn test_top_k() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        for i in 0..10 {
            let id = p.extract_heuristic(format!("c{}", i), "action".into(), "r".into(), "test".into(), true);
            for _ in 0..3 { p.record_outcome(id, true); }
        }
        assert!(p.retrieve_relevant("test", "test", 3).len() <= 3);
    }

    #[test]
    fn test_record_outcome() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        let id = p.extract_heuristic("t".into(), "x".into(), "r".into(), "d".into(), true);
        let before = p.heuristics[&id].confidence;
        p.record_outcome(id, false);
        assert!(p.heuristics[&id].confidence < before);
    }

    #[test]
    fn test_active_count() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig { min_evidence_for_active: 2, ..Default::default() });
        let id = p.extract_heuristic("t".into(), "x".into(), "r".into(), "d".into(), true);
        assert_eq!(p.active_count(), 0);
        p.record_outcome(id, true);
        assert_eq!(p.active_count(), 1);
    }

    #[test]
    fn test_eviction() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig { max_heuristics: 3, ..Default::default() });
        for i in 0..5 { p.extract_heuristic(format!("c{}", i), "x".into(), "r".into(), "d".into(), true); }
        assert_eq!(p.heuristic_count(), 3);
    }

    #[test]
    fn test_summary() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        p.extract_heuristic("t".into(), "x".into(), "r".into(), "d".into(), true);
        assert!(p.summary().contains("heuristics"));
    }

    #[test]
    fn test_retrieve_empty() {
        let p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        assert!(p.retrieve_relevant("none", "unknown", 5).is_empty());
    }

    #[test]
    fn test_record_nonexistent_noop() {
        let mut p = ERLHeuristicPool::new(ERLHeuristicPoolConfig::default());
        p.record_outcome(999, true);
        assert_eq!(p.heuristic_count(), 0);
    }
}
