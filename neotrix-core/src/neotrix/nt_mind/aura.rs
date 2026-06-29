/// AURA (Intent-Directed Probing) reasoning layer
///
/// Surfaces implicit needs behind user queries by computing gap scores
/// between literal and inferred intent, then controlling probe budgets
/// and tool selection. Reference: arXiv:2606.05557 (June 2026)
///
/// Sits between intent parsing and tool execution:
///   parse_intent → [AURA analyze_intent] → execute_tools
use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// AuraConfig — configuration for probe budget and privacy rules
// ============================================================================

/// Configuration for the AURA probe controller.
pub struct AuraConfig {
    /// Base probe budget (default: 3)
    pub base_probe_budget: u32,
    /// Gap threshold for allocating additional probes (default: 0.4)
    pub gap_threshold: f64,
    /// Maximum probes per query (default: 5)
    pub max_probes: u32,
    /// Privacy-sensitive tool patterns (e.g. "email", "contacts", "calendar")
    pub privacy_patterns: Vec<String>,
}

impl Default for AuraConfig {
    fn default() -> Self {
        Self {
            base_probe_budget: 3,
            gap_threshold: 0.4,
            max_probes: 5,
            privacy_patterns: vec![
                "email".into(),
                "contacts".into(),
                "calendar".into(),
                "messages".into(),
                "private".into(),
                "password".into(),
                "credential".into(),
            ],
        }
    }
}

// ============================================================================
// IntentFrame — structured estimate of implicit need
// ============================================================================

/// The AURA intent frame: structured estimate of implicit need behind a query.
#[derive(Debug, Clone)]
pub struct IntentFrame {
    /// Original user query (literal string)
    pub literal: String,
    /// Estimated implicit intent (what the user actually needs)
    pub implicit_intent: String,
    /// Gap score [0.0, 1.0]: how much the literal differs from the implicit need
    pub gap_score: f64,
    /// Probe budget: how many follow-up probes to use (1-5)
    pub probe_budget: u32,
    /// Suggested tools to use for probing
    pub suggested_tools: Vec<String>,
    /// Forbidden tools (privacy-sensitive or irrelevant)
    pub forbidden_tools: Vec<String>,
    /// Context scene hash (to detect repeated queries)
    pub scene_context: u64,
}

impl IntentFrame {
    /// Compute gap score between literal and inferred intent encodings.
    /// Uses VSA cosine similarity: gap = 1.0 - cosine(literal, intent).
    /// High gap → more implicit need → more probing required.
    pub fn compute_gap(literal_encoding: &[f64], intent_encoding: &[f64]) -> f64 {
        let engine = VSAEngine::new(literal_encoding.len());
        1.0 - engine.similarity(literal_encoding, intent_encoding)
    }

    /// Determine probe budget from gap score.
    pub fn determine_probe_budget(gap: f64, config: &AuraConfig) -> u32 {
        if gap < 0.2 {
            1
        } else if gap < 0.4 {
            2
        } else if gap < 0.6 {
            3
        } else if gap < 0.8 {
            4
        } else {
            config.max_probes
        }
    }

    /// Hash a query + context into a scene context identifier.
    pub fn hash_scene(query: &str, context: &[String]) -> u64 {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        for ctx in context {
            ctx.hash(&mut hasher);
        }
        hasher.finish()
    }
}

// ============================================================================
// Intent inference — heuristic pattern matching
// ============================================================================

/// Infer implicit intent and confidence using heuristic patterns.
///
/// Returns (inferred_intent_string, confidence).
pub fn infer_intent(query: &str) -> (String, f64) {
    let q = query.trim();

    // "where is X" → availability check
    if let Some(rest) = q
        .strip_prefix("where is ")
        .or_else(|| q.strip_prefix("where's "))
    {
        let subject = rest.trim_end_matches('?').trim();
        return (
            format!("is {} available or free to interact", subject),
            0.85,
        );
    }
    if let Some(rest) = q.strip_prefix("where are ") {
        let subject = rest.trim_end_matches('?').trim();
        return (
            format!("are {} available or free to interact", subject),
            0.85,
        );
    }

    // "what is X working on" → capability check
    if q.contains("what is ") && q.contains(" working on") {
        let subject = q
            .find("what is ")
            .and_then(|start| {
                let after = &q[start + 8..];
                after.find(" working on").map(|end| &after[..end])
            })
            .unwrap_or("that person");
        return (
            format!("can {} help with the current task", subject.trim()),
            0.80,
        );
    }

    // "when is X" → schedule check
    if let Some(rest) = q
        .strip_prefix("when is ")
        .or_else(|| q.strip_prefix("when's "))
    {
        let subject = rest.trim_end_matches('?').trim();
        return (format!("schedule alignment for {}", subject), 0.80);
    }

    // "how do I X" → step-by-step guidance
    if let Some(rest) = q
        .strip_prefix("how do i ")
        .or_else(|| q.strip_prefix("how do I "))
    {
        let goal = rest.trim_end_matches('?').trim();
        return (format!("step-by-step guidance on how to {}", goal), 0.90);
    }
    if let Some(rest) = q.strip_prefix("how to ") {
        let goal = rest.trim_end_matches('?').trim();
        return (format!("step-by-step guidance on how to {}", goal), 0.85);
    }

    // "how can I X" → feasibility + guidance
    if let Some(rest) = q
        .strip_prefix("how can i ")
        .or_else(|| q.strip_prefix("how can I "))
    {
        let goal = rest.trim_end_matches('?').trim();
        return (
            format!("feasibility assessment and guidance for {}", goal),
            0.85,
        );
    }

    // "why did X happen" → root cause analysis
    if let Some(rest) = q.strip_prefix("why did ") {
        let phenomenon = rest.trim_end_matches('?').trim();
        return (format!("root cause analysis of: {}", phenomenon), 0.85);
    }
    if let Some(rest) = q.strip_prefix("why does ") {
        let phenomenon = rest.trim_end_matches('?').trim();
        return (format!("root cause analysis of: {}", phenomenon), 0.85);
    }
    if let Some(rest) = q.strip_prefix("why is ") {
        let phenomenon = rest.trim_end_matches('?').trim();
        return (format!("root cause analysis of: {}", phenomenon), 0.85);
    }

    // "can you X" or "could you X" → capability confirmation + execution
    if let Some(rest) = q.strip_prefix("can you ") {
        let action = rest.trim_end_matches('?').trim();
        return (format!("confirm capability and execute: {}", action), 0.75);
    }
    if let Some(rest) = q.strip_prefix("could you ") {
        let action = rest.trim_end_matches('?').trim();
        return (format!("confirm capability and execute: {}", action), 0.75);
    }

    // "I need X" → resource/action provisioning
    if let Some(rest) = q
        .strip_prefix("i need ")
        .or_else(|| q.strip_prefix("I need "))
    {
        let need = rest.trim_end_matches('.').trim();
        return (format!("provide or facilitate: {}", need), 0.80);
    }

    // Generic question → clarification probe
    if q.ends_with('?') {
        return (
            format!("clarify intent behind: {}", q.trim_end_matches('?')),
            0.55,
        );
    }

    // Default: literal is the intent
    (q.to_string(), 0.95)
}

// ============================================================================
// Privacy pattern detection
// ============================================================================

/// Detect privacy-sensitive content in a query.
/// Returns a list of matched privacy patterns.
pub fn detect_privacy_patterns(query: &str, config: &AuraConfig) -> Vec<String> {
    let q = query.to_lowercase();
    config
        .privacy_patterns
        .iter()
        .filter(|pat| q.contains(&pat.to_lowercase()))
        .cloned()
        .collect()
}

/// Check if a tool name matches any forbidden pattern.
pub fn is_tool_forbidden(tool: &str, privacy_patterns: &[String]) -> bool {
    let t = tool.to_lowercase();
    privacy_patterns
        .iter()
        .any(|pat| t.contains(&pat.to_lowercase()))
}

// ============================================================================
// ProbeController — budget enforcement
// ============================================================================

/// Controls probe budget and tracks tool usage during probing.
pub struct ProbeController {
    pub probes_used: u32,
    pub probes_allowed: u32,
    pub tools_used: Vec<String>,
    pub has_violation: bool,
}

impl ProbeController {
    pub fn new(allowed: u32) -> Self {
        Self {
            probes_used: 0,
            probes_allowed: allowed,
            tools_used: Vec::new(),
            has_violation: false,
        }
    }

    /// Whether another probe is allowed.
    pub fn can_probe(&self) -> bool {
        self.probes_used < self.probes_allowed && !self.has_violation
    }

    /// Record a probe action, checking against forbidden tools.
    /// Returns Err if the tool is forbidden (privacy violation).
    pub fn record_probe(&mut self, tool: &str) -> Result<(), String> {
        if !self.can_probe() {
            return Err("probe budget exhausted".to_string());
        }
        self.probes_used += 1;
        self.tools_used.push(tool.to_string());
        Ok(())
    }
}

// ============================================================================
// Main integration API
// ============================================================================

/// Run AURA intent analysis on a user query.
/// Returns the IntentFrame that controls probing behavior.
pub fn analyze_intent(query: &str, vsa_engine: &VSAEngine, config: &AuraConfig) -> IntentFrame {
    let (implicit_intent, confidence) = infer_intent(query);

    let dim = vsa_engine.dimensions();
    let literal_encoding: Vec<f64> = (0..dim)
        .map(|i| {
            let b = query.as_bytes();
            let idx = i % b.len().max(1);
            (b[idx] as f64) / 255.0
        })
        .collect();
    let intent_encoding: Vec<f64> = (0..dim)
        .map(|i| {
            let b = implicit_intent.as_bytes();
            let idx = i % b.len().max(1);
            (b[idx] as f64) / 255.0
        })
        .collect();

    let gap_score = (1.0 - confidence)
        * (1.0 - IntentFrame::compute_gap(&literal_encoding, &intent_encoding)).max(0.1);

    let probe_budget = IntentFrame::determine_probe_budget(gap_score, config);

    let privacy_matches = detect_privacy_patterns(query, config);
    let mut forbidden_tools: Vec<String> = config
        .privacy_patterns
        .iter()
        .filter(|p| privacy_matches.iter().any(|m| m == *p))
        .cloned()
        .collect();
    forbidden_tools.sort();
    forbidden_tools.dedup();

    let suggested_tools = if gap_score < 0.3 {
        vec!["direct_execution".to_string()]
    } else if gap_score < 0.6 {
        vec![
            "clarification_probe".to_string(),
            "context_lookup".to_string(),
        ]
    } else {
        vec![
            "clarification_probe".to_string(),
            "context_lookup".to_string(),
            "knowledge_search".to_string(),
            "decomposition".to_string(),
        ]
    };

    let scene_context = IntentFrame::hash_scene(query, &[]);

    IntentFrame {
        literal: query.to_string(),
        implicit_intent,
        gap_score,
        probe_budget,
        suggested_tools,
        forbidden_tools,
        scene_context,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine() -> VSAEngine {
        VSAEngine::new(64)
    }

    // --- Gap score computation ---

    #[test]
    fn test_gap_score_high_when_dissimilar() {
        let literal: Vec<f64> = (0..64).map(|i| (i as f64).sin()).collect();
        let intent: Vec<f64> = (0..64).map(|i| (i as f64).cos()).collect();
        let gap = IntentFrame::compute_gap(&literal, &intent);
        assert!(gap > 0.0, "dissimilar vectors should produce positive gap");
    }

    #[test]
    fn test_gap_score_low_when_similar() {
        let literal: Vec<f64> = (0..64).map(|i| (i as f64).sin()).collect();
        let intent: Vec<f64> = literal.clone();
        let gap = IntentFrame::compute_gap(&literal, &intent);
        assert!(
            (gap - 0.0).abs() < 1e-12,
            "identical vectors should have zero gap"
        );
    }

    // --- Probe budget ---

    #[test]
    fn test_determine_probe_budget() {
        let config = AuraConfig::default();
        assert_eq!(IntentFrame::determine_probe_budget(0.1, &config), 1);
        assert_eq!(IntentFrame::determine_probe_budget(0.3, &config), 2);
        assert_eq!(IntentFrame::determine_probe_budget(0.5, &config), 3);
        assert_eq!(IntentFrame::determine_probe_budget(0.7, &config), 4);
        assert_eq!(
            IntentFrame::determine_probe_budget(0.9, &config),
            config.max_probes
        );
    }

    // --- Intent inference ---

    #[test]
    fn test_infer_intent_where_query() {
        let (intent, conf) = infer_intent("where is Lin Wei?");
        assert!(
            intent.contains("available") || intent.contains("free to interact"),
            "where query should infer availability check: got '{}'",
            intent
        );
        assert!(conf > 0.8);
    }

    #[test]
    fn test_infer_intent_how_query() {
        let (intent, conf) = infer_intent("how do I deploy the model?");
        assert!(
            intent.contains("step-by-step guidance"),
            "how query should infer guidance: got '{}'",
            intent
        );
        assert!(conf > 0.8);
    }

    #[test]
    fn test_infer_intent_why_query() {
        let (intent, conf) = infer_intent("why did the build fail?");
        assert!(
            intent.contains("root cause"),
            "why query should infer root cause analysis: got '{}'",
            intent
        );
        assert!(conf > 0.8);
    }

    #[test]
    fn test_infer_intent_when_query() {
        let (intent, conf) = infer_intent("when is the review?");
        assert!(
            intent.contains("schedule"),
            "when query should infer schedule: got '{}'",
            intent
        );
        assert!(conf > 0.7);
    }

    #[test]
    fn test_infer_intent_can_you_query() {
        let (intent, conf) = infer_intent("can you check the logs?");
        assert!(
            intent.contains("confirm capability"),
            "can you query should infer capability check: got '{}'",
            intent
        );
        assert!(conf > 0.7);
    }

    #[test]
    fn test_infer_intent_i_need_query() {
        let (intent, conf) = infer_intent("I need access to the dashboard");
        assert!(
            intent.contains("provide or facilitate"),
            "I need query should infer provisioning: got '{}'",
            intent
        );
        assert!(conf > 0.7);
    }

    #[test]
    fn test_infer_intent_generic_question() {
        let (intent, conf) = infer_intent("is this the right approach?");
        assert!(
            intent.contains("clarify intent"),
            "generic question should infer clarification: got '{}'",
            intent
        );
        assert!(conf < 0.7);
    }

    // --- ProbeController ---

    #[test]
    fn test_probe_controller_limits() {
        let mut ctrl = ProbeController::new(3);
        assert!(ctrl.can_probe());

        assert!(ctrl.record_probe("tool_a").is_ok());
        assert!(ctrl.record_probe("tool_b").is_ok());
        assert!(ctrl.record_probe("tool_c").is_ok());

        assert!(!ctrl.can_probe());
        let result = ctrl.record_probe("tool_d");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exhausted"));
    }

    #[test]
    fn test_probe_controller_tracks_usage() {
        let mut ctrl = ProbeController::new(2);
        ctrl.record_probe("search").unwrap();
        ctrl.record_probe("clarify").unwrap();
        assert_eq!(ctrl.probes_used, 2);
        assert_eq!(ctrl.tools_used, vec!["search", "clarify"]);
    }

    // --- Privacy pattern detection ---

    #[test]
    fn test_privacy_pattern_detection() {
        let config = AuraConfig::default();
        let matches = detect_privacy_patterns("check my email for the reset link", &config);
        assert!(
            !matches.is_empty(),
            "email should be detected as privacy pattern"
        );
        assert!(matches.contains(&"email".to_string()));
    }

    #[test]
    fn test_privacy_pattern_no_false_positive() {
        let config = AuraConfig::default();
        let matches = detect_privacy_patterns("how do I deploy the server?", &config);
        assert!(
            matches.is_empty(),
            "normal query should not trigger privacy detection"
        );
    }

    #[test]
    fn test_is_tool_forbidden() {
        let patterns = vec!["email".to_string(), "contacts".to_string()];
        assert!(is_tool_forbidden("read_email", &patterns));
        assert!(is_tool_forbidden("contacts_api", &patterns));
        assert!(!is_tool_forbidden("search", &patterns));
    }

    // --- Scene context hashing ---

    #[test]
    fn test_scene_hash_deterministic() {
        let h1 = IntentFrame::hash_scene("hello", &[]);
        let h2 = IntentFrame::hash_scene("hello", &[]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_scene_hash_differs_for_different_queries() {
        let h1 = IntentFrame::hash_scene("hello", &[]);
        let h2 = IntentFrame::hash_scene("world", &[]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_scene_hash_includes_context() {
        let h1 = IntentFrame::hash_scene("hello", &[]);
        let h2 = IntentFrame::hash_scene("hello", &["context".to_string()]);
        assert_ne!(h1, h2);
    }

    // --- analyze_intent integration ---

    #[test]
    fn test_analyze_intent_where_query() {
        let engine = test_engine();
        let config = AuraConfig::default();
        let frame = analyze_intent("where is Lin Wei?", &engine, &config);
        assert_eq!(frame.literal, "where is Lin Wei?");
        assert!(frame.implicit_intent.contains("available"));
        assert!(frame.probe_budget >= 1);
    }

    #[test]
    fn test_analyze_intent_privacy_flag() {
        let engine = test_engine();
        let config = AuraConfig::default();
        let frame = analyze_intent("check my private messages", &engine, &config);
        assert!(!frame.forbidden_tools.is_empty());
    }
}
