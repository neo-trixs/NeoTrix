use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum EvolutionEventType {
    ProposalCreated,
    GuardEvaluated,
    CodeModification,
    GateSwap,
    SkillUpdate,
    TestPassed,
    TestFailed,
    BenchmarkImproved,
    BenchmarkRegressed,
    Rollback,
}

#[derive(Debug, Clone)]
pub struct EvolutionEvent {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: EvolutionEventType,
    pub description: String,
    pub module_affected: String,
    pub outcome_score: f64,
    pub context_tags: Vec<String>,
    pub causal_parent_ids: Vec<u64>,
}

pub struct EvolutionTrace {
    events: Vec<EvolutionEvent>,
    next_id: u64,
    max_events: usize,
}

impl EvolutionTrace {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_events.min(128)),
            next_id: 1,
            max_events,
        }
    }

    pub fn record_event(
        &mut self,
        event_type: EvolutionEventType,
        description: &str,
        module: &str,
        score: f64,
        tags: Vec<String>,
        parent_ids: Vec<u64>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let event = EvolutionEvent {
            id,
            timestamp: Self::now(),
            event_type,
            description: description.to_string(),
            module_affected: module.to_string(),
            outcome_score: score,
            context_tags: tags,
            causal_parent_ids: parent_ids,
        };
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }
        self.events.push(event);
        id
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn get_recent(&self, n: usize) -> Vec<&EvolutionEvent> {
        self.events.iter().rev().take(n).collect()
    }

    pub fn get_by_module(&self, module: &str) -> Vec<&EvolutionEvent> {
        self.events
            .iter()
            .filter(|e| e.module_affected == module)
            .collect()
    }

    pub fn get_by_type(&self, event_type: EvolutionEventType) -> Vec<&EvolutionEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    pub fn chains(&self) -> Vec<Vec<&EvolutionEvent>> {
        let event_refs: Vec<&EvolutionEvent> = self.events.iter().collect();
        let mut roots: Vec<&EvolutionEvent> = Vec::new();
        for event in &event_refs {
            if event.causal_parent_ids.is_empty() {
                roots.push(event);
            }
        }
        let mut result = Vec::new();
        for root in roots {
            let mut chain = vec![root];
            self.extend_chain(root.id, &event_refs, &mut chain, &mut result);
        }
        result
    }

    fn extend_chain<'a>(
        &self,
        current_id: u64,
        event_refs: &[&'a EvolutionEvent],
        chain: &mut Vec<&'a EvolutionEvent>,
        result: &mut Vec<Vec<&'a EvolutionEvent>>,
    ) {
        let children: Vec<&'a EvolutionEvent> = event_refs
            .iter()
            .filter(|e| e.causal_parent_ids.contains(&current_id))
            .copied()
            .collect();
        if children.is_empty() {
            result.push(chain.clone());
            return;
        }
        for child in children {
            chain.push(child);
            self.extend_chain(child.id, event_refs, chain, result);
            chain.pop();
        }
    }

    pub fn events(&self) -> &[EvolutionEvent] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

pub struct EvolutionCausalGraph {
    adjacency: HashMap<u64, Vec<u64>>,
    event_outcomes: HashMap<u64, f64>,
}

impl EvolutionCausalGraph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            event_outcomes: HashMap::new(),
        }
    }

    pub fn build(trace: &EvolutionTrace) -> Self {
        let mut graph = Self::new();
        let events: Vec<_> = trace.events().to_vec();

        for event in &events {
            let id = event.id;
            if !graph.event_outcomes.contains_key(&id) {
                graph.event_outcomes.insert(id, event.outcome_score);
            }
            if !graph.adjacency.contains_key(&id) {
                graph.adjacency.insert(id, Vec::new());
            }
        }

        for event in &events {
            for &pid in &event.causal_parent_ids {
                graph.adjacency.entry(pid).or_default().push(event.id);
            }
        }

        let mut by_module: HashMap<String, Vec<&EvolutionEvent>> = HashMap::new();
        for event in &events {
            by_module
                .entry(event.module_affected.clone())
                .or_default()
                .push(event);
        }
        for (_module, module_events) in &by_module {
            for i in 0..module_events.len().saturating_sub(1) {
                let a = module_events[i];
                let b = module_events[i + 1];
                if !graph
                    .adjacency
                    .get(&a.id)
                    .map_or(false, |v| v.contains(&b.id))
                {
                    graph.adjacency.entry(a.id).or_default().push(b.id);
                }
            }
        }

        graph
    }

    fn dfs_paths(&self, node: u64, visited: &mut Vec<u64>, all_paths: &mut Vec<Vec<u64>>) {
        visited.push(node);
        let children = self.adjacency.get(&node);
        let has_children = children.map_or(false, |v| {
            let unvisited: Vec<_> = v.iter().filter(|c| !visited.contains(c)).collect();
            !unvisited.is_empty()
        });
        if !has_children {
            all_paths.push(visited.clone());
        } else if let Some(children) = children {
            for &child in children {
                if !visited.contains(&child) {
                    self.dfs_paths(child, visited, all_paths);
                }
            }
        }
        visited.pop();
    }

    pub fn success_paths(&self) -> Vec<Vec<u64>> {
        let roots: Vec<u64> = self
            .adjacency
            .keys()
            .filter(|&id| {
                self.event_outcomes.get(id).copied().unwrap_or(0.0) >= 0.0
                    && !self.adjacency.values().any(|v| v.contains(id))
            })
            .copied()
            .collect();

        let mut all_paths = Vec::new();
        for &root in &roots {
            let mut visited = Vec::new();
            self.dfs_paths(root, &mut visited, &mut all_paths);
        }

        all_paths
            .into_iter()
            .filter(|path| {
                path.iter()
                    .all(|id| self.event_outcomes.get(id).copied().unwrap_or(0.0) >= 0.0)
            })
            .collect()
    }

    pub fn failure_patterns(&self) -> Vec<Vec<u64>> {
        let mut all_paths = Vec::new();
        let all_nodes: Vec<u64> = self.adjacency.keys().copied().collect();
        for &node in &all_nodes {
            let mut visited = Vec::new();
            self.dfs_paths(node, &mut visited, &mut all_paths);
        }

        all_paths
            .into_iter()
            .filter(|path| {
                let last_outcome = path
                    .last()
                    .and_then(|id| self.event_outcomes.get(id))
                    .copied()
                    .unwrap_or(0.0);
                last_outcome < 0.0
            })
            .collect()
    }

    pub fn most_influential_module(&self, trace: &EvolutionTrace) -> Option<String> {
        let mut module_impact: HashMap<String, f64> = HashMap::new();
        for event in trace.events() {
            *module_impact
                .entry(event.module_affected.clone())
                .or_insert(0.0) += event.outcome_score.abs();
        }
        module_impact
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(m, _)| m)
    }

    pub fn predict_success(&self, description: &str, _module: &str, _tags: &[String]) -> f64 {
        let _query_words: Vec<&str> = description.split_whitespace().collect();

        let mut candidates: Vec<f64> = Vec::new();
        let mut weights: Vec<f64> = Vec::new();

        for (&id, &score) in &self.event_outcomes {
            if !self.adjacency.contains_key(&id) {
                continue;
            }
            candidates.push(score);
            weights.push(1.0);
        }

        let by_module: Vec<f64> = self.event_outcomes.iter().map(|(_, &s)| s).collect();

        if by_module.is_empty() {
            return 0.0;
        }

        let sum: f64 = candidates
            .iter()
            .zip(weights.iter())
            .map(|(s, w)| s * w)
            .sum();
        let w_sum: f64 = weights.iter().sum();
        if w_sum == 0.0 {
            let avg: f64 = by_module.iter().sum::<f64>() / by_module.len() as f64;
            avg
        } else {
            sum / w_sum
        }
    }
}

impl Default for EvolutionCausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EvolutionPredictor {
    trace: EvolutionTrace,
    graph: EvolutionCausalGraph,
}

impl EvolutionPredictor {
    pub fn new(max_events: usize) -> Self {
        Self {
            trace: EvolutionTrace::new(max_events),
            graph: EvolutionCausalGraph::new(),
        }
    }

    pub fn record_and_analyze(
        &mut self,
        event_type: EvolutionEventType,
        description: &str,
        module: &str,
        score: f64,
        tags: Vec<String>,
        parent_ids: Vec<u64>,
    ) -> u64 {
        let id = self
            .trace
            .record_event(event_type, description, module, score, tags, parent_ids);
        self.graph = EvolutionCausalGraph::build(&self.trace);
        id
    }

    pub fn predict_outcome(&self, description: &str, module: &str, tags: &[String]) -> (f64, f64) {
        let module_events: Vec<&EvolutionEvent> = self.trace.get_by_module(module);
        let total_events = self.trace.len();

        if total_events == 0 {
            return (0.0, 0.0);
        }

        let predicted = self.graph.predict_success(description, module, tags);

        let module_familiarity = if total_events > 0 {
            module_events.len() as f64 / total_events as f64
        } else {
            0.0
        };

        let confidence = (module_familiarity * 0.6 + 0.2).clamp(0.0, 0.95);

        (predicted.clamp(-1.0, 1.0), confidence)
    }

    pub fn suggest_parents(&self, description: &str, module: &str) -> Vec<u64> {
        let recent = self.trace.get_recent(10);
        let mut candidates: Vec<(u64, f64)> = Vec::new();

        for event in recent {
            let mut score = 0.0;
            if event.module_affected == module {
                score += 2.0;
            }
            let desc_words: Vec<&str> = description.split_whitespace().collect();
            let event_words: Vec<&str> = event.description.split_whitespace().collect();
            let overlap = desc_words
                .iter()
                .filter(|w| event_words.contains(w))
                .count();
            score += overlap as f64 * 0.5;
            if score > 0.0 {
                candidates.push((event.id, score));
            }
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.into_iter().map(|(id, _)| id).take(3).collect()
    }

    pub fn summary_report(&self) -> String {
        let total = self.trace.len();
        let positive = self
            .trace
            .events()
            .iter()
            .filter(|e| e.outcome_score > 0.0)
            .count();
        let negative = self
            .trace
            .events()
            .iter()
            .filter(|e| e.outcome_score < 0.0)
            .count();
        let modules: std::collections::HashSet<String> = self
            .trace
            .events()
            .iter()
            .map(|e| e.module_affected.clone())
            .collect();
        let success_paths = self.graph.success_paths().len();
        let failure_paths = self.graph.failure_patterns().len();

        let mut report = String::new();
        report.push_str(&format!("=== Evolution Health Report ===\n"));
        report.push_str(&format!("Total events tracked: {}\n", total));
        report.push_str(&format!("Positive outcomes: {}\n", positive));
        report.push_str(&format!("Negative outcomes: {}\n", negative));
        report.push_str(&format!("Modules affected: {}\n", modules.len()));
        report.push_str(&format!("Success chains: {}\n", success_paths));
        report.push_str(&format!("Failure chains: {}\n", failure_paths));
        if !modules.is_empty() {
            let top = self.graph.most_influential_module(&self.trace);
            if let Some(m) = top {
                report.push_str(&format!("Most influential module: {}\n", m));
            }
        }
        report
    }

    pub fn trace(&self) -> &EvolutionTrace {
        &self.trace
    }

    pub fn graph(&self) -> &EvolutionCausalGraph {
        &self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_single_event() {
        let mut trace = EvolutionTrace::new(100);
        let id = trace.record_event(
            EvolutionEventType::ProposalCreated,
            "add sparse attention",
            "attention_head",
            0.8,
            vec!["attention".to_string()],
            vec![],
        );
        assert_eq!(trace.len(), 1);
        let event = &trace.events()[0];
        assert_eq!(event.id, id);
        assert_eq!(event.event_type, EvolutionEventType::ProposalCreated);
        assert_eq!(event.module_affected, "attention_head");
        assert!((event.outcome_score - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_record_causal_chain() {
        let mut trace = EvolutionTrace::new(100);
        let p1 = trace.record_event(
            EvolutionEventType::ProposalCreated,
            "refactor gate swap",
            "core",
            0.5,
            vec![],
            vec![],
        );
        let p2 = trace.record_event(
            EvolutionEventType::CodeModification,
            "apply gate swap",
            "core",
            0.3,
            vec![],
            vec![p1],
        );
        let p3 = trace.record_event(
            EvolutionEventType::TestPassed,
            "all tests green",
            "core",
            0.6,
            vec![],
            vec![p2],
        );
        let _p4 = trace.record_event(
            EvolutionEventType::BenchmarkImproved,
            "latency -12%",
            "core",
            0.9,
            vec![],
            vec![p3],
        );

        assert_eq!(trace.len(), 4);
        assert_eq!(trace.events()[1].causal_parent_ids, vec![p1]);
        assert_eq!(trace.events()[2].causal_parent_ids, vec![p2]);
        assert_eq!(trace.events()[3].causal_parent_ids, vec![p3]);
    }

    #[test]
    fn test_extract_causal_chains_linear() {
        let mut trace = EvolutionTrace::new(100);
        let a = trace.record_event(
            EvolutionEventType::ProposalCreated,
            "a",
            "m",
            0.0,
            vec![],
            vec![],
        );
        let b = trace.record_event(
            EvolutionEventType::CodeModification,
            "b",
            "m",
            0.0,
            vec![],
            vec![a],
        );
        let c = trace.record_event(
            EvolutionEventType::TestPassed,
            "c",
            "m",
            0.0,
            vec![],
            vec![b],
        );
        let chains = trace.chains();
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].len(), 3);
        assert_eq!(chains[0][0].id, a);
        assert_eq!(chains[0][1].id, b);
        assert_eq!(chains[0][2].id, c);
    }

    #[test]
    fn test_build_causal_graph() {
        let mut trace = EvolutionTrace::new(100);
        let a = trace.record_event(
            EvolutionEventType::ProposalCreated,
            "add ctx",
            "context",
            0.7,
            vec![],
            vec![],
        );
        let b = trace.record_event(
            EvolutionEventType::CodeModification,
            "impl ctx",
            "context",
            0.5,
            vec![],
            vec![a],
        );
        let c = trace.record_event(
            EvolutionEventType::BenchmarkImproved,
            "ctx +10%",
            "context",
            0.9,
            vec![],
            vec![b],
        );

        let graph = EvolutionCausalGraph::build(&trace);
        assert!(graph.success_paths().len() > 0);
        assert!(graph.event_outcomes.contains_key(&a));
        assert!(graph.event_outcomes.contains_key(&b));
        assert!(graph.event_outcomes.contains_key(&c));
    }

    #[test]
    fn test_success_paths() {
        let mut trace = EvolutionTrace::new(100);
        let a = trace.record_event(
            EvolutionEventType::ProposalCreated,
            "good",
            "m",
            0.5,
            vec![],
            vec![],
        );
        let _b = trace.record_event(
            EvolutionEventType::TestPassed,
            "pass",
            "m",
            0.3,
            vec![],
            vec![a],
        );
        let graph = EvolutionCausalGraph::build(&trace);
        let paths = graph.success_paths();
        assert!(paths.iter().any(|p| p.len() >= 2));
    }

    #[test]
    fn test_failure_patterns() {
        let mut trace = EvolutionTrace::new(100);
        let a = trace.record_event(
            EvolutionEventType::ProposalCreated,
            "bad idea",
            "m",
            -0.5,
            vec![],
            vec![],
        );
        let b = trace.record_event(
            EvolutionEventType::TestFailed,
            "fail",
            "m",
            -0.8,
            vec![],
            vec![a],
        );
        let graph = EvolutionCausalGraph::build(&trace);
        let failures = graph.failure_patterns();
        assert!(failures.iter().any(|p| p.last().copied() == Some(b)));
    }

    #[test]
    fn test_predictor_trains_and_predicts() {
        let mut predictor = EvolutionPredictor::new(100);
        predictor.record_and_analyze(
            EvolutionEventType::CodeModification,
            "add cache",
            "core",
            0.7,
            vec!["perf".to_string()],
            vec![],
        );
        predictor.record_and_analyze(
            EvolutionEventType::CodeModification,
            "refactor loop",
            "core",
            0.5,
            vec!["perf".to_string()],
            vec![],
        );
        predictor.record_and_analyze(
            EvolutionEventType::CodeModification,
            "optimize query",
            "core",
            0.8,
            vec!["perf".to_string()],
            vec![],
        );
        predictor.record_and_analyze(
            EvolutionEventType::CodeModification,
            "add index",
            "core",
            0.6,
            vec!["perf".to_string()],
            vec![],
        );
        predictor.record_and_analyze(
            EvolutionEventType::CodeModification,
            "reduce alloc",
            "core",
            0.9,
            vec!["perf".to_string()],
            vec![],
        );

        let (predicted, confidence) =
            predictor.predict_outcome("improve throughput", "core", &["perf".to_string()]);
        assert!(predicted >= 0.0);
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_predictor_confidence_drops_for_novel_module() {
        let mut predictor = EvolutionPredictor::new(100);
        for i in 0..5 {
            predictor.record_and_analyze(
                EvolutionEventType::CodeModification,
                &format!("known change {}", i),
                "familiar_mod",
                0.5,
                vec![],
                vec![],
            );
        }

        let (_, familiar_conf) = predictor.predict_outcome("another change", "familiar_mod", &[]);
        let (_, novel_conf) = predictor.predict_outcome("new thing", "novel_mod", &[]);

        assert!(novel_conf < familiar_conf);
    }

    #[test]
    fn test_summary_report() {
        let mut predictor = EvolutionPredictor::new(100);
        predictor.record_and_analyze(
            EvolutionEventType::ProposalCreated,
            "plan A",
            "core",
            0.5,
            vec![],
            vec![],
        );
        predictor.record_and_analyze(
            EvolutionEventType::BenchmarkImproved,
            "win",
            "core",
            0.8,
            vec![],
            vec![],
        );
        let report = predictor.summary_report();
        assert!(report.contains("Total events tracked: 2"));
        assert!(report.contains("Positive outcomes: 2"));
    }

    #[test]
    fn test_suggest_parents() {
        let mut predictor = EvolutionPredictor::new(100);
        let a = predictor.record_and_analyze(
            EvolutionEventType::CodeModification,
            "fix memory leak",
            "core",
            0.7,
            vec![],
            vec![],
        );
        predictor.record_and_analyze(
            EvolutionEventType::CodeModification,
            "add feature flag",
            "ui",
            0.3,
            vec![],
            vec![],
        );
        predictor.record_and_analyze(
            EvolutionEventType::BenchmarkImproved,
            "memory -20%",
            "core",
            0.9,
            vec![],
            vec![a],
        );

        let suggestions = predictor.suggest_parents("fix another memory issue", "core");
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&a));
    }

    #[test]
    fn test_most_influential_module() {
        let mut trace = EvolutionTrace::new(100);
        trace.record_event(
            EvolutionEventType::CodeModification,
            "change a",
            "alpha",
            0.9,
            vec![],
            vec![],
        );
        trace.record_event(
            EvolutionEventType::CodeModification,
            "change b",
            "alpha",
            0.8,
            vec![],
            vec![],
        );
        trace.record_event(
            EvolutionEventType::CodeModification,
            "change c",
            "beta",
            0.1,
            vec![],
            vec![],
        );

        let graph = EvolutionCausalGraph::build(&trace);
        let top = graph.most_influential_module(&trace);
        assert_eq!(top, Some("alpha".to_string()));
    }

    #[test]
    fn test_filter_by_type() {
        let mut trace = EvolutionTrace::new(100);
        trace.record_event(
            EvolutionEventType::ProposalCreated,
            "p1",
            "m",
            0.0,
            vec![],
            vec![],
        );
        trace.record_event(
            EvolutionEventType::CodeModification,
            "c1",
            "m",
            0.0,
            vec![],
            vec![],
        );
        trace.record_event(
            EvolutionEventType::ProposalCreated,
            "p2",
            "m",
            0.0,
            vec![],
            vec![],
        );

        let proposals = trace.get_by_type(EvolutionEventType::ProposalCreated);
        assert_eq!(proposals.len(), 2);
    }

    #[test]
    fn test_max_events_eviction() {
        let mut trace = EvolutionTrace::new(3);
        trace.record_event(
            EvolutionEventType::ProposalCreated,
            "a",
            "m",
            0.0,
            vec![],
            vec![],
        );
        trace.record_event(
            EvolutionEventType::ProposalCreated,
            "b",
            "m",
            0.0,
            vec![],
            vec![],
        );
        trace.record_event(
            EvolutionEventType::ProposalCreated,
            "c",
            "m",
            0.0,
            vec![],
            vec![],
        );
        trace.record_event(
            EvolutionEventType::ProposalCreated,
            "d",
            "m",
            0.0,
            vec![],
            vec![],
        );
        assert_eq!(trace.len(), 3);
    }

    #[test]
    fn test_empty_predictor_returns_zero() {
        let predictor = EvolutionPredictor::new(100);
        let (predicted, confidence) = predictor.predict_outcome("anything", "unknown", &[]);
        assert!((predicted - 0.0).abs() < 1e-6);
        assert!((confidence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_success_paths() {
        let graph = EvolutionCausalGraph::new();
        let paths = graph.success_paths();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_empty_failure_patterns() {
        let graph = EvolutionCausalGraph::new();
        let failures = graph.failure_patterns();
        assert!(failures.is_empty());
    }
}
