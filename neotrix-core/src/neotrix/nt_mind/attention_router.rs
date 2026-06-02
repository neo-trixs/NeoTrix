use crate::core::nt_core_gwt::module_def::{SpecialistModule, SpecialistType};
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::cube::CubeEntry;
use crate::neotrix::nt_mind::self_iterating::harness_adapter::HarnessAdapter;

use super::hypercube_bridge::HyperCubeBridge;

use crate::neotrix::nt_world_crawl::config::{CrawlTopic, SeedEntry};
use crate::neotrix::nt_world_crawl::unified::UnifiedCrawler;

/// 路由结果 — GWT 竞争 + 知识检索的产出
pub struct RoutedContext {
    pub winning_topic: String,
    pub active_specialists: Vec<SpecialistType>,
    pub knowledge_lines: Vec<String>,
    pub salience_report: Vec<(SpecialistType, f64)>,
}

/// AttentionRouter — 知识驱动的推理意识内核
///
/// GWT 竞争 + KnowledgeHyperCube 检索 → 产生推理上下文
/// 三角闭环：注意力路由 → 知识检索 → 推理决策
pub struct AttentionRouter {
    pub workspace: GlobalWorkspace,
    pub bridge: HyperCubeBridge,
}

impl AttentionRouter {
    pub fn new() -> Self {
        let mut workspace = GlobalWorkspace::new(0.4);
        for st in &[
            SpecialistType::PatternMatcher,
            SpecialistType::AnomalyDetector,
            SpecialistType::KnowledgeIntegrator,
            SpecialistType::GoalPrioritizer,
            SpecialistType::RiskAssessor,
            SpecialistType::CreativityGenerator,
            SpecialistType::ReflectionEngine,
            SpecialistType::KnowledgeRetriever,
            SpecialistType::CodeAnalyzer,
            SpecialistType::Planner,
        ] {
            workspace.register(SpecialistModule::new(*st, format!("{:?}", st)));
        }
        Self {
            workspace,
            bridge: HyperCubeBridge::new(),
        }
    }

    pub fn wm(&mut self) -> &mut GlobalWorkspace {
        &mut self.workspace
    }

    /// 主入口：分析上下文 → GWT 竞争 → 知识检索 → RoutedContext
    pub fn route(&mut self, context: &str) -> RoutedContext {
        let lower = context.to_lowercase();
        let specialist_salience = self.compute_salience(&lower);

        // Activate specialists with their salience scores
        for (st, salience) in &specialist_salience {
            if let Some(s) = self.workspace.specialist_by_type_mut(st) {
                s.activate(*salience);
            }
        }

        // Pick winner: highest salience
        let winner = specialist_salience
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(st, _)| st.name().to_string())
            .unwrap_or_default();

        self.workspace.active_content = Some(winner.clone());
        self.workspace.broadcast_history.push(winner.clone());

        let active: Vec<SpecialistType> = self
            .workspace
            .active_specialists()
            .iter()
            .map(|m| m.module_type)
            .collect();

        let mut knowledge_lines: Vec<String> = Vec::new();
        for st in &active {
            let entries = self.retrieve_for_specialist(*st, context);
            for e in entries {
                knowledge_lines
                    .push(format!("[{}] {} ({})", st.short_name(), e.label, e.source));
            }
        }

        self.workspace.decay_all(0.3);

        RoutedContext {
            winning_topic: winner,
            active_specialists: active,
            knowledge_lines,
            salience_report: specialist_salience,
        }
    }

    /// 根据上下文关键词计算 7 个 Specialist 的 salience
    fn compute_salience(&self, lower: &str) -> Vec<(SpecialistType, f64)> {
        let terms = [
            (
                SpecialistType::PatternMatcher,
                0.7,
                &[
                    "pattern", "repeat", "template", "structure", "trend",
                    "similar", "common", "regular", "cycle", "algorithm",
                ][..],
            ),
            (
                SpecialistType::AnomalyDetector,
                0.7,
                &[
                    "error", "bug", "crash", "fail", "unusual", "exception",
                    "unexpected", "wrong", "broken", "issue", "problem",
                ][..],
            ),
            (
                SpecialistType::KnowledgeIntegrator,
                0.6,
                &[
                    "knowledge", "learn", "understand", "combine", "integrate",
                    "synthesize", "connect", "relate", "overview", "survey",
                ][..],
            ),
            (
                SpecialistType::GoalPrioritizer,
                0.7,
                &[
                    "goal", "plan", "priority", "objective", "milestone",
                    "strategy", "roadmap", "next", "schedule", "deadline",
                ][..],
            ),
            (
                SpecialistType::RiskAssessor,
                0.7,
                &[
                    "risk", "nt_shield", "danger", "warn", "vulnerability",
                    "threat", "safe", "protect", "audit", "breach",
                ][..],
            ),
            (
                SpecialistType::CreativityGenerator,
                0.6,
                &[
                    "creative", "novel", "innovate", "design", "imagine",
                    "invent", "explore", "possibility", "brainstorm", "idea",
                ][..],
            ),
            (
                SpecialistType::ReflectionEngine,
                0.6,
                &[
                    "reflect", "review", "improve", "optimize", "evolve",
                    "retrospect", "lesson", "growth", "iterate", "meta",
                ][..],
            ),
        ];

        let mut scores: Vec<(SpecialistType, f64)> = terms
            .iter()
            .map(|(st, base, keywords)| {
                let mut count = 0usize;
                for kw in *keywords {
                    if lower.contains(kw) {
                        count += 1;
                    }
                }
                let salience = (*base + count as f64 * 0.05).clamp(0.0, 1.0);
                (*st, salience)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    /// 为特定 Specialist 从超立方体检索相关知识
    fn retrieve_for_specialist(
        &self,
        st: SpecialistType,
        context: &str,
    ) -> Vec<CubeEntry> {
        let query = specialist_query_coord(st, context);
        self.bridge.query(&query, 4)
    }

    /// Analyze hypercube gap report and return sparse CrawlTopics that need more data
    pub fn sparse_topics(&self) -> Vec<CrawlTopic> {
        let gap_reports = self.bridge.analyze_gaps();
        let mut topics = Vec::new();
        // Map 8 dim indices to CrawlTopic (aligns with analyze_gaps dim 0..8)
        const DIM_TOPIC: [CrawlTopic; 8] = [
            CrawlTopic::LawAndGovernance,      // 0
            CrawlTopic::ScienceAndTechnology,  // 1
            CrawlTopic::PhilosophyAndEthics,   // 2
            CrawlTopic::HumanitiesAndCulture,  // 3
            CrawlTopic::HistoryAndArcheology,  // 4
            CrawlTopic::NewsAndMedia,          // 5
            CrawlTopic::PolicyAndRegulation,   // 6
            CrawlTopic::ArtsAndLiterature,     // 7
        ];
        for report in &gap_reports {
            if report.sparsity_score > 0.7 && report.dim_index < 8 {
                topics.push(DIM_TOPIC[report.dim_index]);
            }
        }
        topics
    }

    /// Seed the nt_world_crawl with URLs from sparse hypercube domains
    pub fn seed_nt_world_crawl_from_gaps(&self, nt_world_crawl: &mut UnifiedCrawler) {
        let sparse_topics = self.sparse_topics();
        for topic in sparse_topics {
            let url = format!("https://en.wikipedia.org/wiki/{}", topic.name());
            let seed = SeedEntry {
                url,
                topic,
                depth: 2,
                enabled: true,
            };
            nt_world_crawl.add_seeds(vec![seed]);
        }
    }

    /// 吸收推理结果到超立方体
    pub fn absorb_reasoning_result(
        &mut self,
        topic: &str,
        _result: &str,
        source: &str,
    ) {
        let coord = HyperCoord::with(DimensionAxis::Abstraction, 0.7);
        self.bridge.hypercube.insert(&coord, source, topic);
    }

    /// Set the environment context for Life-Harness adaptation.
    pub fn set_environment(&mut self, env: &str) {
        self.workspace.current_environment = Some(env.to_string());
    }

    /// Register a harness profile for the current environment.
    /// Returns true if the profile was registered and specialists got activation boosts.
    pub fn register_harness_profile(&mut self, env: &str, profile: &crate::neotrix::nt_mind::self_iterating::harness_adapter::HarnessProfile) -> bool {
        self.workspace.harness_adapter.register_profile(env, profile.clone());
        let activated = self.workspace.harness_adapter.activate(env).is_some();
        if activated {
            for (_, m) in self.workspace.specialists.iter_mut() {
                if let Some(adaptations) = profile.specialist_adaptations.get(&m.specialist_type) {
                    for a in adaptations {
                        m.record_harness_evidence(env, a);
                    }
                }
            }
        }
        activated
    }

    pub fn harness_adapter_mut(&mut self) -> &mut HarnessAdapter {
        &mut self.workspace.harness_adapter
    }

    /// 注入种子知识到超立方体（若为空）
    pub fn seed_knowledge(&mut self) {
        // All coords spaced by >= 0.1 per axis to avoid float-precision merge (< 0.05)
        let seeds: Vec<(&str, &str, HyperCoord)> = vec![
            ("deductive-reasoning",
             "Infer specific conclusions from general principles using syllogisms",
             HyperCoord::with(DimensionAxis::Abstraction, 0.95)),
            ("inductive-reasoning",
             "Generalize patterns from specific observations",
             HyperCoord::with(DimensionAxis::Abstraction, 0.75)),
            ("abductive-reasoning",
             "Infer best explanation from observed evidence",
             HyperCoord::with(DimensionAxis::Abstraction, 0.55)),
            ("analogical-reasoning",
             "Transfer knowledge from familiar domains via structural alignment",
             HyperCoord::with(DimensionAxis::Abstraction, 0.35)),
            ("causal-reasoning",
             "Identify cause-effect through counterfactual analysis",
             HyperCoord::with(DimensionAxis::Abstraction, 0.65)),
            ("system-1-intuition",
             "Fast automatic associative pattern matching",
             HyperCoord::with(DimensionAxis::Abstraction, 0.15)),
            ("system-2-analysis",
             "Slow deliberate analytical step-by-step verification",
             HyperCoord::with(DimensionAxis::Certainty, 0.85)),
            ("error-detection",
             "Identify discrepancies between expected and observed states",
             HyperCoord::with(DimensionAxis::Certainty, 0.95)),
            ("goal-decomposition",
             "Break high-level objectives into executable sub-tasks",
             HyperCoord::with(DimensionAxis::Agency, 0.85)),
            ("risk-assessment",
             "Evaluate probability and impact of adverse outcomes",
             HyperCoord::with(DimensionAxis::Agency, 0.65)),
        ];
        for (label, summary, coord) in seeds {
            self.bridge.hypercube.insert(&coord, label, summary);
        }
    }

    /// 构建可注入 ReasoningEngine prompt 的知识上下文
    pub fn build_knowledge_prompt_suffix(&self, context: &RoutedContext) -> String {
        if context.knowledge_lines.is_empty() {
            return String::new();
        }
        let header = format!(
            "\n[Knowledge from HyperCube — activated: {}]\n",
            context
                .active_specialists
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let body = context.knowledge_lines.join("\n");
        format!("{}{}\n", header, body)
    }
}

impl Default for AttentionRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Specialist → HyperCoord 映射 ====================

fn specialist_query_coord(st: SpecialistType, context: &str) -> HyperCoord {
    let lower = context.to_lowercase();
    let mut coord = HyperCoord::new();
    match st {
        SpecialistType::PatternMatcher => {
            coord.set(DimensionAxis::Abstraction, 0.6);
            coord.set(DimensionAxis::Certainty, 0.5);
        }
        SpecialistType::KnowledgeRetriever => {
            coord.set(DimensionAxis::Abstraction, 0.5);
            coord.set(DimensionAxis::Scale, 0.5);
        }
        SpecialistType::CodeAnalyzer => {
            coord.set(DimensionAxis::Abstraction, 0.3);
            coord.set(DimensionAxis::Certainty, 0.8);
        }
        SpecialistType::Planner => {
            coord.set(DimensionAxis::Abstraction, 0.7);
            coord.set(DimensionAxis::Agency, 0.6);
        }
        SpecialistType::AnomalyDetector => {
            coord.set(DimensionAxis::Certainty, 0.8);
            coord.set(DimensionAxis::Scale, 0.5);
            if lower.contains("nt_shield") || lower.contains("vulnerability") {
                coord.set(DimensionAxis::Domain, 0.7);
            }
        }
        SpecialistType::KnowledgeIntegrator => {
            coord.set(DimensionAxis::Abstraction, 0.8);
            coord.set(DimensionAxis::Agency, 0.7);
            if lower.contains("science") {
                coord.set(DimensionAxis::Domain, 0.6);
            }
        }
        SpecialistType::GoalPrioritizer => {
            coord.set(DimensionAxis::Agency, 0.85);
            coord.set(DimensionAxis::Time, 0.6);
        }
        SpecialistType::RiskAssessor => {
            coord.set(DimensionAxis::Agency, 0.7);
            coord.set(DimensionAxis::Domain, 0.6);
            coord.set(DimensionAxis::Certainty, 0.3);
        }
        SpecialistType::CreativityGenerator => {
            coord.set(DimensionAxis::Abstraction, 0.4);
            coord.set(DimensionAxis::Certainty, 0.2);
        }
        SpecialistType::ReflectionEngine => {
            coord.set(DimensionAxis::Abstraction, 0.9);
            coord.set(DimensionAxis::Agency, 0.3);
        }
        SpecialistType::MetaCognitionAnalyst => {
            coord.set(DimensionAxis::Abstraction, 0.85);
            coord.set(DimensionAxis::Scale, 0.7);
            coord.set(DimensionAxis::Time, 0.5);
        }
        SpecialistType::AISecurity => {
            coord.set(DimensionAxis::Domain, 0.8);
            coord.set(DimensionAxis::Certainty, 0.7);
            if lower.contains("prompt injection") || lower.contains("jailbreak") {
                coord.set(DimensionAxis::Abstraction, 0.6);
            }
        }
        SpecialistType::ImageGenerator => {
            coord.set(DimensionAxis::Abstraction, 0.5);
            coord.set(DimensionAxis::Domain, 0.6);
            coord.set(DimensionAxis::Agency, 0.5);
        }
    }
    coord
}

// ==================== SpecialistType 扩展方法 ====================

impl SpecialistType {
    pub fn name(&self) -> &'static str {
        match self {
            SpecialistType::PatternMatcher => "pattern-matcher",
            SpecialistType::AnomalyDetector => "anomaly-detector",
            SpecialistType::KnowledgeRetriever => "knowledge-retriever",
            SpecialistType::CodeAnalyzer => "code-analyzer",
            SpecialistType::Planner => "planner",
            SpecialistType::KnowledgeIntegrator => "knowledge-integrator",
            SpecialistType::GoalPrioritizer => "goal-prioritizer",
            SpecialistType::RiskAssessor => "risk-assessor",
            SpecialistType::CreativityGenerator => "creativity-generator",
            SpecialistType::ReflectionEngine => "reflection-engine",
            SpecialistType::MetaCognitionAnalyst => "meta-cognition-analyst",
            SpecialistType::AISecurity => "ai-nt_shield",
            SpecialistType::ImageGenerator => "image-generator",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            SpecialistType::PatternMatcher => "PM",
            SpecialistType::AnomalyDetector => "AD",
            SpecialistType::KnowledgeRetriever => "KR",
            SpecialistType::CodeAnalyzer => "CA",
            SpecialistType::Planner => "PL",
            SpecialistType::KnowledgeIntegrator => "KI",
            SpecialistType::GoalPrioritizer => "GP",
            SpecialistType::RiskAssessor => "RA",
            SpecialistType::CreativityGenerator => "CG",
            SpecialistType::ReflectionEngine => "RE",
            SpecialistType::MetaCognitionAnalyst => "MA",
            SpecialistType::AISecurity => "AS",
            SpecialistType::ImageGenerator => "IG",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_ten_specialists() {
        let router = AttentionRouter::new();
        assert_eq!(router.workspace.specialists.len(), 10);
    }

    #[test]
    fn test_new_creates_empty_hypercube() {
        let router = AttentionRouter::new();
        assert_eq!(router.bridge.hypercube.cell_count(), 0);
    }

    #[test]
    fn test_seed_knowledge_populates_hypercube() {
        let mut router = AttentionRouter::new();
        let before = router.bridge.hypercube.cell_count();
        router.seed_knowledge();
        let after = router.bridge.hypercube.cell_count();
        eprintln!("cell_count: before={} after={}", before, after);
        assert!(after >= 10, "expected >= 10, got {}", after);
    }

    #[test]
    fn test_route_returns_winning_topic() {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        let result = router.route("find patterns in the error logs and suggest fixes");
        assert!(!result.winning_topic.is_empty());
    }

    #[test]
    fn test_route_with_anomaly_context_activates_anomaly_detector() {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        let result = router.route("critical bug in production, nt_shield vulnerability detected");
        assert!(result.active_specialists.contains(&SpecialistType::AnomalyDetector));
    }

    #[test]
    fn test_route_with_goal_context_activates_goal_prioritizer() {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        let result = router.route("plan next sprint goals and set milestones");
        assert!(result.active_specialists.contains(&SpecialistType::GoalPrioritizer));
    }

    #[test]
    fn test_route_with_creative_context_activates_creativity() {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        let result = router.route("brainstorm novel design ideas for the new interface");
        assert!(result.active_specialists.contains(&SpecialistType::CreativityGenerator));
    }

    #[test]
    fn test_route_retrieves_knowledge_from_hypercube() {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        let result = router.route("analyze patterns in system behavior");
        assert!(!result.knowledge_lines.is_empty());
    }

    #[test]
    fn test_absorb_reasoning_result_adds_entry() {
        let mut router = AttentionRouter::new();
        let before = router.bridge.hypercube.cell_count();
        router.absorb_reasoning_result("test-topic", "test result", "test-source");
        assert_eq!(router.bridge.hypercube.cell_count(), before + 1);
    }

    #[test]
    fn test_build_knowledge_prompt_suffix_references_active_specialists() {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        let ctx = router.route("find patterns in error data");
        let suffix = router.build_knowledge_prompt_suffix(&ctx);
        assert!(!suffix.is_empty());
        assert!(suffix.contains("Knowledge from HyperCube"));
    }

    #[test]
    fn test_compute_salience_ranks_anomaly_highest_for_error_context() {
        let router = AttentionRouter::new();
        let scores = router.compute_salience("error crash bug exception");
        let anomaly = scores.iter().find(|(st, _)| *st == SpecialistType::AnomalyDetector);
        let pattern = scores.iter().find(|(st, _)| *st == SpecialistType::PatternMatcher);
        assert!(anomaly.is_some());
        assert!(pattern.is_some());
        assert!(anomaly.expect("anomaly should be ok in test").1 > pattern.expect("anomaly should be ok in test").1);
    }

    #[test]
    fn test_route_empty_context_returns_some_result() {
        let mut router = AttentionRouter::new();
        let result = router.route("");
        assert!(!result.active_specialists.is_empty());
    }

    #[test]
    fn test_decay_happens_after_route() {
        let mut router = AttentionRouter::new();
        let _before: Vec<f64> = router.workspace.specialists.values().map(|s| s.activation).collect();
        router.route("test");
        let after: Vec<f64> = router.workspace.specialists.values().map(|s| s.activation).collect();
        assert!(after.iter().all(|&a| a <= 0.8));
    }

    #[test]
    fn test_specialist_type_name_and_shortname() {
        assert_eq!(SpecialistType::PatternMatcher.name(), "pattern-matcher");
        assert_eq!(SpecialistType::AnomalyDetector.short_name(), "AD");
        assert_eq!(SpecialistType::KnowledgeIntegrator.short_name(), "KI");
        assert_eq!(SpecialistType::GoalPrioritizer.short_name(), "GP");
        assert_eq!(SpecialistType::RiskAssessor.short_name(), "RA");
        assert_eq!(SpecialistType::CreativityGenerator.short_name(), "CG");
        assert_eq!(SpecialistType::ReflectionEngine.short_name(), "RE");
    }

    #[test]
    fn test_salience_all_zero_for_irrelevant_context() {
        let router = AttentionRouter::new();
        let scores = router.compute_salience("xyzzzzz qwerty");
        assert!(scores.iter().all(|(_, s)| *s < 0.71));
    }

    #[test]
    fn test_query_coord_differs_by_specialist() {
        let pt_coord = specialist_query_coord(SpecialistType::PatternMatcher, "");
        let ki_coord = specialist_query_coord(SpecialistType::KnowledgeIntegrator, "");
        let pt_abstraction = pt_coord.get(&DimensionAxis::Abstraction);
        let ki_abstraction = ki_coord.get(&DimensionAxis::Abstraction);
        assert!((pt_abstraction - ki_abstraction).abs() > 0.01);
    }
}
