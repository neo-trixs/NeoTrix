use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Instant;

/// Global smart router singleton
use crate::core::nt_core_util;

pub static SMART_ROUTER: LazyLock<Mutex<SmartRouter>> = LazyLock::new(|| {
    Mutex::new(SmartRouter::new())
});

/// Task complexity levels for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TaskComplexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Critical,
}

impl TaskComplexity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Trivial => "trivial",
            Self::Simple => "simple",
            Self::Moderate => "moderate",
            Self::Complex => "complex",
            Self::Critical => "critical",
        }
    }

    pub fn weight(&self) -> u8 {
        match self {
            Self::Trivial => 1,
            Self::Simple => 2,
            Self::Moderate => 3,
            Self::Complex => 4,
            Self::Critical => 5,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Trivial, Self::Simple, Self::Moderate, Self::Complex, Self::Critical]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trivial" => Some(Self::Trivial),
            "simple" => Some(Self::Simple),
            "moderate" => Some(Self::Moderate),
            "complex" => Some(Self::Complex),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }

    /// Classify a prompt/task by analyzing its characteristics
    pub fn classify(prompt: &str, context: &TaskContext) -> Self {
        // Heuristic scoring
        let mut score: u8 = 0;

        // 1. Prompt length factor (>500 chars pushes up)
        if context.prompt_length > 2000 {
            score += 2;
        } else if context.prompt_length > 800 {
            score += 1;
        }

        // 2. File mentions
        if context.mentions_files {
            score += 1;
            if context.file_count > 3 {
                score += 1;
            }
        }

        // 3. Git context
        if context.has_git_context {
            score += 1;
        }

        // 4. Keyword analysis (cumulative — all matching keywords add)
        let lower = prompt.to_lowercase();
        let critical_keywords = ["security", "deploy", "production", "audit", "permission"];
        let complex_keywords = ["architect", "design", "refactor", "migrate", "scalab"];
        let moderate_keywords = ["debug", "fix", "implement", "feature", "add ", "integrat"];

        for kw in &critical_keywords {
            if lower.contains(kw) { score += 3; }
        }
        for kw in &complex_keywords {
            if lower.contains(kw) { score += 2; }
        }
        for kw in &moderate_keywords {
            if lower.contains(kw) { score += 1; }
        }

        // 5. Technical terms density
        let tech_terms = ["api", "database", "function", "class", "struct", "impl",
                           "trait", "async", "generic", "macro", "thread", "mutex",
                           "http", "json", "serde", "tokio", "wasm"];
        let term_count = tech_terms.iter().filter(|t| lower.contains(*t)).count();
        if term_count > 5 {
            score += 2;
        } else if term_count > 0 {
            score += 1;
        }

        match score {
            0 => Self::Trivial,
            1..=3 => Self::Simple,
            4..=6 => Self::Moderate,
            7..=9 => Self::Complex,
            _ => Self::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub prompt_length: usize,
    pub mentions_files: bool,
    pub file_count: usize,
    pub has_git_context: bool,
    pub keywords: Vec<String>,
}

impl TaskContext {
    pub fn new(prompt: &str) -> Self {
        let lower = prompt.to_lowercase();
        let common_keywords = ["api", "function", "class", "struct", "impl", "trait",
                                "debug", "fix", "refactor", "design", "architect",
                                "security", "deploy", "test", "feature", "migrate"];
        let keywords: Vec<String> = common_keywords.iter()
            .filter(|kw| lower.contains(*kw))
            .map(|s| s.to_string())
            .collect();

        let file_extensions = [".rs", ".py", ".js", ".ts", ".json", ".toml", ".md",
                               ".html", ".css", ".go", ".rb", ".java", ".cpp", ".h"];
        let mentions_files = file_extensions.iter().any(|ext| lower.contains(ext));
        let file_count = file_extensions.iter()
            .filter(|ext| lower.contains(*ext))
            .count();

        let has_git_context = lower.contains("git ") || lower.contains("commit") ||
                              lower.contains("branch") || lower.contains("diff ") ||
                              lower.contains("stash") || lower.contains("pr ") ||
                              lower.contains("merge");

        Self {
            prompt_length: prompt.len(),
            mentions_files,
            file_count,
            has_git_context,
            keywords,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub complexity: TaskComplexity,
    pub provider: String,
    pub model: String,
    pub cost_per_1k_in: f64,
    pub cost_per_1k_out: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    pub total_routes: u64,
    pub routes_by_complexity: HashMap<String, u64>,
    pub estimated_savings: f64,
    pub total_tokens_saved: u64,
    pub flagship_cost: f64,
    pub actual_cost: f64,
}

impl RouterStats {
    fn new() -> Self {
        let mut routes_by_complexity = HashMap::new();
        for c in TaskComplexity::all() {
            routes_by_complexity.insert(c.label().to_string(), 0);
        }
        Self {
            total_routes: 0,
            routes_by_complexity,
            estimated_savings: 0.0,
            total_tokens_saved: 0,
            flagship_cost: 0.0,
            actual_cost: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub complexity: TaskComplexity,
    pub provider: String,
    pub model: String,
    pub estimated_cost: f64,
    pub flagship_cost: f64,
    pub savings: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRouter {
    pub rules: HashMap<TaskComplexity, RoutingRule>,
    pub enabled: bool,
    pub stats: RouterStats,
    pub flagship_reference: RoutingRule,
}

impl SmartRouter {
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        rules.insert(TaskComplexity::Trivial, RoutingRule {
            complexity: TaskComplexity::Trivial,
            provider: "opencode".to_string(),
            model: "gpt-4o-mini".to_string(),
            cost_per_1k_in: 0.0015,
            cost_per_1k_out: 0.006,
        });
        rules.insert(TaskComplexity::Simple, RoutingRule {
            complexity: TaskComplexity::Simple,
            provider: "opencode".to_string(),
            model: "gpt-4o-mini".to_string(),
            cost_per_1k_in: 0.0015,
            cost_per_1k_out: 0.006,
        });
        rules.insert(TaskComplexity::Moderate, RoutingRule {
            complexity: TaskComplexity::Moderate,
            provider: "opencode".to_string(),
            model: "gpt-4o".to_string(),
            cost_per_1k_in: 0.01,
            cost_per_1k_out: 0.03,
        });
        let flagship = RoutingRule {
            complexity: TaskComplexity::Complex,
            provider: "opencode".to_string(),
            model: "flagship".to_string(),
            cost_per_1k_in: 0.01,
            cost_per_1k_out: 0.03,
        };
        rules.insert(TaskComplexity::Complex, flagship.clone());
        rules.insert(TaskComplexity::Critical, RoutingRule {
            complexity: TaskComplexity::Critical,
            provider: "opencode".to_string(),
            model: "flagship".to_string(),
            cost_per_1k_in: 0.01,
            cost_per_1k_out: 0.03,
        });

        Self {
            rules,
            enabled: true,
            stats: RouterStats::new(),
            flagship_reference: flagship,
        }
    }

    pub fn flagships(&self) -> f64 {
        self.flagship_reference.cost_per_1k_in + self.flagship_reference.cost_per_1k_out
    }

    /// Route a task to the appropriate provider/model
    pub fn route(&mut self, _prompt: &str, context: &TaskContext) -> RoutingDecision {
        let complexity = TaskComplexity::classify(_prompt, context);
        let rule = self.rules.get(&complexity)
            .cloned()
            .unwrap_or_else(|| self.flagship_reference.clone());

        let estimated_tokens = (context.prompt_length / 4).max(100) as f64;
        let estimated_cost = (estimated_tokens / 1000.0) * (rule.cost_per_1k_in + rule.cost_per_1k_out);
        let flagship_cost = (estimated_tokens / 1000.0) *
            (self.flagship_reference.cost_per_1k_in + self.flagship_reference.cost_per_1k_out);
        let savings = flagship_cost - estimated_cost;

        self.stats.total_routes += 1;
        *self.stats.routes_by_complexity.entry(complexity.label().to_string()).or_insert(0) += 1;
        self.stats.estimated_savings += savings.max(0.0);
        self.stats.total_tokens_saved += (savings.max(0.0) * 1000.0) as u64;
        self.stats.flagship_cost += flagship_cost;
        self.stats.actual_cost += estimated_cost;

        if !self.enabled {
            return RoutingDecision {
                complexity,
                provider: self.flagship_reference.provider.clone(),
                model: self.flagship_reference.model.clone(),
                estimated_cost: flagship_cost,
                flagship_cost,
                savings: 0.0,
            };
        }

        RoutingDecision {
            complexity,
            provider: rule.provider,
            model: rule.model,
            estimated_cost,
            flagship_cost,
            savings,
        }
    }

    pub fn get_provider_config(&self, complexity: TaskComplexity) -> Option<&RoutingRule> {
        self.rules.get(&complexity)
    }

    pub fn set_rule(&mut self, complexity: TaskComplexity, provider: &str, model: &str, cost_in: f64, cost_out: f64) {
        self.rules.insert(complexity, RoutingRule {
            complexity,
            provider: provider.to_string(),
            model: model.to_string(),
            cost_per_1k_in: cost_in,
            cost_per_1k_out: cost_out,
        });
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn savings_report(&self) -> String {
        let names: Vec<String> = TaskComplexity::all().iter().map(|c| {
            let count = self.stats.routes_by_complexity.get(c.label()).copied().unwrap_or(0);
            format!("  {}: {} routes", c.label(), count)
        }).collect();

        format!(
            "📊 Smart Router Report\n\
             Status: {}\n\
             Total routes: {}\n\
             \n\
             Routes by complexity:\n{}\n\
             \n\
             Actual cost:       ${:.6}\n\
             Flagship cost:     ${:.6}\n\
             Estimated savings: ${:.6}\n\
             Tokens saved:      {}",
            if self.enabled { "✅ enabled" } else { "⛔ disabled" },
            self.stats.total_routes,
            names.join("\n"),
            self.stats.actual_cost,
            self.stats.flagship_cost,
            self.stats.estimated_savings,
            self.stats.total_tokens_saved,
        )
    }

    pub fn reset_stats(&mut self) {
        self.stats = RouterStats::new();
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = nt_core_util::home_dir().join(".neotrix");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("smart_router.json");
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("Serialize: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| format!("Write: {}", e))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("Rename: {}", e))?;
        Ok(())
    }

    pub fn load() -> Self {
        let path = nt_core_util::home_dir().join(".neotrix").join("smart_router.json");
        match std::fs::read_to_string(&path) {
            Ok(json) => {
                match serde_json::from_str(&json) {
                    Ok(router) => router,
                    Err(_) => Self::new(),
                }
            }
            Err(_) => Self::new(),
        }
    }
}

impl Default for SmartRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Cognitive Router ────────────────────────────────────────────────────────

/// Cognitive system selection for task routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CognitiveSystem {
    System1, // Intuition — fast E8 pattern match, <100ms
    System2, // Logic — full symbolic reasoning, no latency cap
    System3, // Metacognition — self-reflection, verification
}

impl CognitiveSystem {
    pub fn label(&self) -> &'static str {
        match self {
            Self::System1 => "system1-intuition",
            Self::System2 => "system2-logic",
            Self::System3 => "system3-metacognition",
        }
    }
}

/// Cognitive routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveDecision {
    pub system: CognitiveSystem,
    pub complexity: TaskComplexity,
    pub confidence: f64,
    pub latency_budget_ms: u64,
    pub uncertainty: f64,
    pub requires_verification: bool,
    pub reasoning: String,
}

/// Cognitive router statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveRouterStats {
    pub total_routes: u64,
    pub routes_by_system: HashMap<String, u64>,
    pub system1_latency_avg_ms: f64,
    pub system2_latency_avg_ms: f64,
    pub system3_latency_avg_ms: f64,
}

impl CognitiveRouterStats {
    fn new() -> Self {
        let mut routes_by_system = HashMap::new();
        routes_by_system.insert("system1-intuition".to_string(), 0);
        routes_by_system.insert("system2-logic".to_string(), 0);
        routes_by_system.insert("system3-metacognition".to_string(), 0);
        Self {
            total_routes: 0,
            routes_by_system,
            system1_latency_avg_ms: 0.0,
            system2_latency_avg_ms: 0.0,
            system3_latency_avg_ms: 0.0,
        }
    }
}

/// CognitiveRouter — assigns tasks to cognitive systems (System1/2/3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveRouter {
    pub system1_threshold: f64,
    pub system2_threshold: f64,
    pub uncertainty_threshold: f64,
    pub enabled: bool,
    pub stats: CognitiveRouterStats,
}

impl CognitiveRouter {
    pub fn new() -> Self {
        Self {
            system1_threshold: 3.0, // Moderate → S1
            system2_threshold: 4.0, // Complex → S2
            uncertainty_threshold: 0.7,
            enabled: true,
            stats: CognitiveRouterStats::new(),
        }
    }

    /// Route a task to the appropriate cognitive system
    pub fn route(&mut self, prompt: &str, context: &TaskContext) -> CognitiveDecision {
        let start = Instant::now();
        let complexity = TaskComplexity::classify(prompt, context);
        let complexity_score = complexity.weight() as f64;
        let uncertainty = self.estimate_uncertainty(prompt);
        let requires_verification = self.needs_verification(prompt);

        let (system, latency_budget_ms, reasoning, confidence) = if !self.enabled {
            (CognitiveSystem::System2, 5000, "cognitive routing disabled, fallback S2".to_string(), 0.5)
        } else if complexity_score <= self.system1_threshold && uncertainty < self.uncertainty_threshold {
            (CognitiveSystem::System1, 100, "complexity low and uncertainty below threshold → fast S1 intuition".to_string(), 1.0 - uncertainty)
        } else if complexity_score <= self.system2_threshold {
            (CognitiveSystem::System2, 5000, "complexity within S2 range → full symbolic reasoning".to_string(), 0.8 - uncertainty * 0.3)
        } else {
            (CognitiveSystem::System3, u64::MAX, "high complexity or critical → S3 self-reflection".to_string(), 0.5 - uncertainty * 0.2)
        };

        let elapsed = start.elapsed().as_millis() as f64;
        self.stats.total_routes += 1;
        *self.stats.routes_by_system.entry(system.label().to_string()).or_insert(0) += 1;
        match system {
            CognitiveSystem::System1 => {
                self.stats.system1_latency_avg_ms = self.stats.system1_latency_avg_ms * 0.9 + elapsed * 0.1;
            }
            CognitiveSystem::System2 => {
                self.stats.system2_latency_avg_ms = self.stats.system2_latency_avg_ms * 0.9 + elapsed * 0.1;
            }
            CognitiveSystem::System3 => {
                self.stats.system3_latency_avg_ms = self.stats.system3_latency_avg_ms * 0.9 + elapsed * 0.1;
            }
        }

        CognitiveDecision {
            system,
            complexity,
            confidence,
            latency_budget_ms,
            uncertainty,
            requires_verification,
            reasoning,
        }
    }

    /// Estimate task uncertainty from prompt content
    fn estimate_uncertainty(&self, prompt: &str) -> f64 {
        let lower = prompt.to_lowercase();
        let mut score = 0.0;

        // Ambiguity indicators
        let ambiguity_words = [
            "maybe", "could", "perhaps", "possibly", "might", "not sure",
            "unsure", "maybe not", "i think", "probably", "what if",
            "not certain", "vague", "unclear", "ambiguous",
        ];
        for w in &ambiguity_words {
            if lower.contains(w) {
                score += 0.12;
            }
        }

        // Contradiction indicators
        let contradiction_pairs = [
            ("but also", 0.15),
            ("on the other hand", 0.12),
            ("however", 0.08),
            ("although", 0.08),
            ("alternatively", 0.10),
            ("instead", 0.06),
        ];
        for (phrase, val) in &contradiction_pairs {
            if lower.contains(phrase) {
                score += val;
            }
        }

        // Question mark increases uncertainty
        let q_count = prompt.matches('?').count();
        score += (q_count as f64) * 0.05;

        // Short vague prompts are more uncertain
        if prompt.len() < 20 {
            score += 0.15;
        }

        // Well-structured lists decrease uncertainty
        if lower.contains("- ") || lower.contains("1. ") || lower.contains("* ") {
            score = (score - 0.1).max(0.0);
        }

        score.min(1.0)
    }

    /// Check if a response requires post-hoc verification
    fn needs_verification(&self, prompt: &str) -> bool {
        let lower = prompt.to_lowercase();
        let verification_keywords = [
            "security", "deploy", "production", "audit", "permission",
            "password", "token", "secret", "key", "encrypt",
            "auth", "critical", "safety", "dangerous", "delete",
            "sudo", "root", "admin", "firewall", "certificate",
        ];
        verification_keywords.iter().any(|kw| lower.contains(kw))
    }

    /// Set routing thresholds
    pub fn set_thresholds(&mut self, s1: f64, s2: f64, uncertainty: f64) {
        self.system1_threshold = s1;
        self.system2_threshold = s2;
        self.uncertainty_threshold = uncertainty;
    }
}

impl Default for CognitiveRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration: SmartRouter can use CognitiveRouter as pre-routing step
impl SmartRouter {
    /// Combined cognitive + cost-aware routing
    pub fn route_cognitive(
        &mut self,
        prompt: &str,
        context: &TaskContext,
        cognitive: &mut CognitiveRouter,
    ) -> (CognitiveDecision, RoutingDecision) {
        let cognitive_decision = cognitive.route(prompt, context);
        let routing_decision = self.route(prompt, context);
        (cognitive_decision, routing_decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_complexity_labels() {
        assert_eq!(TaskComplexity::Trivial.label(), "trivial");
        assert_eq!(TaskComplexity::Critical.label(), "critical");
    }

    #[test]
    fn test_complexity_weights() {
        assert_eq!(TaskComplexity::Trivial.weight(), 1);
        assert_eq!(TaskComplexity::Critical.weight(), 5);
    }

    #[test]
    fn test_complexity_from_str() {
        assert_eq!(TaskComplexity::from_str("trivial"), Some(TaskComplexity::Trivial));
        assert_eq!(TaskComplexity::from_str("Complex"), Some(TaskComplexity::Complex));
        assert_eq!(TaskComplexity::from_str("unknown"), None);
    }

    #[test]
    fn test_classify_trivial_prompt() {
        let prompt = "hello world";
        let ctx = TaskContext::new(prompt);
        let result = TaskComplexity::classify(prompt, &ctx);
        assert_eq!(result, TaskComplexity::Trivial);
    }

    #[test]
    fn test_classify_simple_question() {
        let prompt = "what is an api and how does it work";
        let ctx = TaskContext::new(prompt);
        let result = TaskComplexity::classify(prompt, &ctx);
        assert_eq!(result, TaskComplexity::Simple, "what/how question with tech term 'api' should be Simple, got {:?}", result);
    }

    #[test]
    fn test_classify_complex_refactor() {
        let prompt = "refactor the database module to use async traits and implement connection pooling with tokio";
        let ctx = TaskContext::new(prompt);
        let result = TaskComplexity::classify(prompt, &ctx);
        assert!(result == TaskComplexity::Complex || result == TaskComplexity::Moderate,
            "refactor+async+tokio should be Moderate+, got {:?}", result);
    }

    #[test]
    fn test_classify_critical_security() {
        let prompt = "security audit of the authentication system before production deployment";
        let ctx = TaskContext::new(prompt);
        let result = TaskComplexity::classify(prompt, &ctx);
        assert_eq!(result, TaskComplexity::Critical,
            "security+production+deployment should be Critical, got {:?}", result);
    }

    #[test]
    fn test_new_router_defaults() {
        let router = SmartRouter::new();
        assert_eq!(router.rules.len(), 5);
        assert!(router.enabled);
        assert_eq!(router.stats.total_routes, 0);
    }

    #[test]
    fn test_route_trivial() {
        let mut router = SmartRouter::new();
        let ctx = TaskContext::new("hello");
        let decision = router.route("hello", &ctx);
        assert_eq!(decision.model, "gpt-4o-mini");
        assert!(decision.savings >= 0.0);
        assert_eq!(router.stats.total_routes, 1);
    }

    #[test]
    fn test_route_disabled() {
        let mut router = SmartRouter::new();
        router.set_enabled(false);
        let ctx = TaskContext::new("hello");
        let decision = router.route("hello", &ctx);
        assert_eq!(decision.model, "flagship");
    }

    #[test]
    fn test_set_rule() {
        let mut router = SmartRouter::new();
        router.set_rule(TaskComplexity::Simple, "local", "llama3", 0.0001, 0.0002);
        let rule = router.get_provider_config(TaskComplexity::Simple).unwrap();
        assert_eq!(rule.provider, "local");
        assert_eq!(rule.model, "llama3");
    }

    #[test]
    fn test_savings_report() {
        let mut router = SmartRouter::new();
        let ctx = TaskContext::new("hello");
        router.route("hello", &ctx);
        let report = router.savings_report();
        assert!(report.contains("Smart Router Report"));
        assert!(report.contains("1 route"));
    }

    #[test]
    fn test_reset_stats() {
        let mut router = SmartRouter::new();
        let ctx = TaskContext::new("hello");
        router.route("hello", &ctx);
        assert_eq!(router.stats.total_routes, 1);
        router.reset_stats();
        assert_eq!(router.stats.total_routes, 0);
    }

    #[test]
    fn test_task_context_detects_files() {
        let ctx = TaskContext::new("edit src/main.rs and src/lib.rs");
        assert!(ctx.mentions_files);
        assert!(ctx.file_count >= 1, "should detect .rs extension");
    }

    #[test]
    fn test_task_context_detects_git() {
        let ctx = TaskContext::new("git commit with message");
        assert!(ctx.has_git_context);
    }

    #[test]
    fn test_task_context_keywords() {
        let ctx = TaskContext::new("fix the function in the api module");
        assert!(ctx.keywords.contains(&"fix".to_string()));
        assert!(ctx.keywords.contains(&"function".to_string()));
        assert!(ctx.keywords.contains(&"api".to_string()));
    }

    #[test]
    fn test_persistence() {
        let mut router = SmartRouter::new();
        router.set_rule(TaskComplexity::Simple, "test-provider", "test-model", 0.01, 0.02);
        assert!(router.save().is_ok());
        let loaded = SmartRouter::load();
        let rule = loaded.get_provider_config(TaskComplexity::Simple).unwrap();
        assert_eq!(rule.provider, "test-provider");
    }

    #[test]
    fn test_routing_decision_fields() {
        let mut router = SmartRouter::new();
        let ctx = TaskContext::new("simple question");
        let d = router.route("simple question", &ctx);
        assert!(d.estimated_cost > 0.0, "estimated_cost should be > 0");
        assert!(d.flagship_cost > 0.0, "flagship_cost should be > 0");
        assert!(d.savings >= 0.0, "savings should be >= 0");
    }

    #[test]
    fn test_router_default_enabled() {
        let r = SmartRouter::new();
        assert!(r.enabled);
    }

    #[test]
    fn test_load_from_missing_file() {
        let router = SmartRouter::load();
        assert_eq!(router.rules.len(), 5);
    }

    #[test]
    fn test_all_complexities() {
        let all = TaskComplexity::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&TaskComplexity::Trivial));
        assert!(all.contains(&TaskComplexity::Critical));
    }

    // ─── Cognitive Router Tests ─────────────────────────────────────────────

    #[test]
    fn test_system1_simple_task() {
        let mut router = CognitiveRouter::new();
        let ctx = TaskContext::new("hello");
        let decision = router.route("hello", &ctx);
        assert_eq!(decision.system, CognitiveSystem::System1, "trivial task should route to S1");
        assert_eq!(decision.latency_budget_ms, 100, "S1 latency budget should be 100ms");
        assert!(decision.confidence > 0.5, "S1 should have high confidence");
    }

    #[test]
    fn test_system2_complex_task() {
        let mut router = CognitiveRouter::new();
        let prompt = "refactor the database module to use async traits and implement connection pooling with tokio";
        let ctx = TaskContext::new(prompt);
        let decision = router.route(prompt, &ctx);
        assert_eq!(decision.system, CognitiveSystem::System2, "complex refactor should route to S2");
        assert_eq!(decision.latency_budget_ms, 5000, "S2 latency budget should be 5000ms");
    }

    #[test]
    fn test_system3_uncertain_task() {
        let mut router = CognitiveRouter::new();
        let prompt = "maybe we could refactor this but also perhaps it's not clear what the alternative might be, possibly we need to reconsider";
        let ctx = TaskContext::new(prompt);
        let decision = router.route(prompt, &ctx);
        assert_eq!(decision.system, CognitiveSystem::System3, "highly uncertain task should route to S3");
        assert_eq!(decision.latency_budget_ms, u64::MAX, "S3 latency budget should be unlimited");
        assert!(decision.uncertainty > 0.5, "uncertainty should be high for ambiguous prompt");
    }

    #[test]
    fn test_uncertainty_estimation() {
        let router = CognitiveRouter::new();
        // Low uncertainty: clear, structured prompt
        let clear = router.estimate_uncertainty("implement login, add tests");
        // High uncertainty: ambiguous prompt
        let ambiguous = router.estimate_uncertainty("maybe we could possibly do something but also perhaps not sure");
        assert!(ambiguous > clear, "ambiguous prompt should have higher uncertainty than clear prompt");
        assert!(clear >= 0.0 && clear <= 1.0, "uncertainty should be 0-1");
        assert!(ambiguous >= 0.0 && ambiguous <= 1.0, "uncertainty should be 0-1");
    }

    #[test]
    fn test_requires_verification() {
        let mut router = CognitiveRouter::new();
        let ctx = TaskContext::new("security audit of production system");
        let decision = router.route("security audit of production system", &ctx);
        assert!(decision.requires_verification, "security prompts should require verification");
    }

    #[test]
    fn test_route_cognitive_integration() {
        let mut smart = SmartRouter::new();
        let mut cognitive = CognitiveRouter::new();
        let ctx = TaskContext::new("hello");
        let (cog_dec, route_dec) = smart.route_cognitive("hello", &ctx, &mut cognitive);
        assert_eq!(cog_dec.system, CognitiveSystem::System1);
        assert_eq!(route_dec.model, "gpt-4o-mini");
        assert_eq!(cognitive.stats.total_routes, 1);
    }
}
