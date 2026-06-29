use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A synthesized tool — a reusable capability with signature and heuristic implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedTool {
    pub name: String,
    pub domain: String,
    pub version: u32,
    pub parameters: Vec<String>,
    pub returns: String,
    /// A symbolic heuristic for the tool's behavior
    pub heuristic: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Usage count
    pub usage_count: u64,
    /// Average success rate
    pub success_rate: f64,
}

/// ToolSynthesizer — synthesizes executable tools from CapabilitySynthesizer discoveries.
#[derive(Debug, Clone)]
pub struct ToolSynthesizer {
    /// Synthesized tools, indexed by name
    pub tools: HashMap<String, SynthesizedTool>,
    /// Maximum number of tools
    pub max_tools: usize,
    /// Minimum confidence for auto-activation
    pub activation_threshold: f64,
    /// Total synthesis cycles
    pub synthesis_count: u64,
}

impl ToolSynthesizer {
    pub fn new(max_tools: usize, activation_threshold: f64) -> Self {
        Self {
            tools: HashMap::new(),
            max_tools,
            activation_threshold,
            synthesis_count: 0,
        }
    }

    /// Synthesize a tool from a capability description.
    /// Parses the capability string to extract:
    /// - name: first word before ':'
    /// - domain: context from description
    /// - parameters: extracted from parentheses in description, e.g., "(a, b, c)"
    /// - heuristic: an NE-strategy heuristic derived from the description
    /// - confidence: current CapabilitySynthesizer confidence
    pub fn synthesize_from_capability(
        &mut self,
        domain: &str,
        capability_name: &str,
        description: &str,
        confidence: f64,
    ) -> SynthesizedTool {
        // Extract name: first token before ':' or first whitespace
        let name = if let Some(idx) = capability_name.find(':') {
            capability_name[..idx].trim().to_string()
        } else {
            capability_name
                .split_whitespace()
                .next()
                .unwrap_or(capability_name)
                .to_string()
        };
        if name.is_empty() {
            let fallback = format!("tool_{}", self.synthesis_count + 1);
            return self.synthesize_from_capability(domain, &fallback, description, confidence);
        }

        // Extract parameters: find parenthesized groups in description
        let mut parameters: Vec<String> = Vec::new();
        let mut depth = 0usize;
        let mut start = 0usize;
        for (i, ch) in description.char_indices() {
            match ch {
                '(' if depth == 0 => {
                    depth = 1;
                    start = i + 1;
                }
                '(' => depth += 1,
                ')' if depth == 1 => {
                    let content = &description[start..i];
                    for param in content.split(',') {
                        let p = param.trim().to_string();
                        if !p.is_empty() && !parameters.contains(&p) {
                            parameters.push(p);
                        }
                    }
                    depth = 0;
                }
                ')' => depth -= 1,
                _ => {}
            }
        }

        // Extract return type hint
        let returns = if description.contains("->") {
            description
                .split("->")
                .nth(1)
                .unwrap_or("string")
                .trim()
                .to_string()
        } else if description.to_lowercase().contains("return") {
            "string".to_string()
        } else {
            "string".to_string()
        };

        // Generate heuristic: "TuneParam:{extracted_param}:0.1" format
        let heuristic = if let Some(first_param) = parameters.first() {
            format!("TuneParam:{}:0.1", first_param)
        } else if let Some(first_word) = description.split_whitespace().next() {
            format!("TuneParam:{}:0.1", first_word.trim_matches(|c: char| !c.is_alphanumeric()))
        } else {
            format!("TuneParam:{}:0.1", name)
        };

        // Prevent duplicate names by versioning
        let version = if let Some(existing) = self.tools.get(&name) {
            existing.version + 1
        } else {
            1
        };

        let tool = SynthesizedTool {
            name: name.clone(),
            domain: domain.to_string(),
            version,
            parameters,
            returns,
            heuristic,
            confidence,
            usage_count: 0,
            success_rate: 0.5,
        };

        // Enforce max_tools limit — evict lowest confidence
        if self.tools.len() >= self.max_tools && !self.tools.contains_key(&name) {
            if let Some(worst) = self
                .tools
                .iter()
                .min_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, _)| k.clone())
            {
                self.tools.remove(&worst);
            }
        }

        self.synthesis_count += 1;
        self.tools.insert(name.clone(), tool.clone());
        tool
    }

    /// Activate a tool — returns its heuristic string for execution
    pub fn activate(&mut self, name: &str) -> Option<&str> {
        let tool = self.tools.get_mut(name)?;
        if tool.confidence >= self.activation_threshold {
            tool.usage_count += 1;
            Some(tool.heuristic.as_str())
        } else {
            None
        }
    }

    /// Record tool outcome
    pub fn record_outcome(&mut self, name: &str, success: bool) {
        if let Some(tool) = self.tools.get_mut(name) {
            tool.usage_count += 1;
            let n = tool.usage_count as f64;
            tool.success_rate = ((n - 1.0) * tool.success_rate + if success { 1.0 } else { 0.0 }) / n;
            if !success {
                tool.confidence *= 0.9;
            }
        }
    }

    /// Get currently active (high-confidence) tools
    pub fn active_tools(&self) -> Vec<&SynthesizedTool> {
        self.tools
            .values()
            .filter(|t| t.confidence >= self.activation_threshold && t.success_rate >= 0.5)
            .collect()
    }

    /// Suggest tools for a given domain
    pub fn suggest_for_domain(&self, domain: &str) -> Vec<&SynthesizedTool> {
        let domain_lower = domain.to_lowercase();
        let mut matches: Vec<&SynthesizedTool> = self
            .tools
            .values()
            .filter(|t| t.domain.to_lowercase() == domain_lower)
            .collect();
        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Prune low-confidence, rarely-used tools
    pub fn prune(&mut self, min_confidence: f64, min_usage: u64) -> usize {
        let before = self.tools.len();
        self.tools.retain(|_, t| t.confidence >= min_confidence && t.usage_count >= min_usage);
        before - self.tools.len()
    }

    /// Metrics for dashboard
    pub fn metrics(&self) -> serde_json::Value {
        let total = self.tools.len();
        let active = self.active_tools().len();
        let avg_confidence: f64 = if total > 0 {
            self.tools.values().map(|t| t.confidence).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let avg_success: f64 = if total > 0 {
            self.tools.values().map(|t| t.success_rate).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let total_usage: u64 = self.tools.values().map(|t| t.usage_count).sum();

        serde_json::json!({
            "total_tools": total,
            "active_tools": active,
            "synthesis_count": self.synthesis_count,
            "avg_confidence": format!("{:.3}", avg_confidence),
            "avg_success_rate": format!("{:.3}", avg_success),
            "total_usage": total_usage,
            "activation_threshold": self.activation_threshold,
            "max_tools": self.max_tools,
        })
    }
}

impl Default for ToolSynthesizer {
    fn default() -> Self {
        Self::new(100, 0.6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ts = ToolSynthesizer::new(50, 0.7);
        assert_eq!(ts.tools.len(), 0);
        assert_eq!(ts.max_tools, 50);
        assert_eq!(ts.activation_threshold, 0.7);
        assert_eq!(ts.synthesis_count, 0);
    }

    #[test]
    fn test_default() {
        let ts = ToolSynthesizer::default();
        assert_eq!(ts.max_tools, 100);
        assert_eq!(ts.activation_threshold, 0.6);
    }

    #[test]
    fn test_synthesize_simple() {
        let mut ts = ToolSynthesizer::default();
        let tool = ts.synthesize_from_capability("search", "web_search", "search the web (query, limit)", 0.85);
        assert_eq!(tool.name, "web_search");
        assert_eq!(tool.domain, "search");
        assert_eq!(tool.version, 1);
        assert_eq!(tool.parameters, vec!["query", "limit"]);
        assert_eq!(tool.heuristic, "TuneParam:query:0.1");
        assert_eq!(tool.confidence, 0.85);
        assert_eq!(ts.synthesis_count, 1);
    }

    #[test]
    fn test_synthesize_with_colon_name() {
        let mut ts = ToolSynthesizer::default();
        let tool = ts.synthesize_from_capability("code", "file_read: read a file (path)", "read files from disk", 0.9);
        assert_eq!(tool.name, "file_read");
        assert_eq!(tool.parameters, vec!["path"]);
    }

    #[test]
    fn test_synthesize_no_params() {
        let mut ts = ToolSynthesizer::default();
        let tool = ts.synthesize_from_capability("system", "ping", "check if system is alive", 0.5);
        assert_eq!(tool.name, "ping");
        assert!(tool.parameters.is_empty());
        assert!(tool.heuristic.contains("TuneParam:"));
    }

    #[test]
    fn test_versioning_on_duplicate_name() {
        let mut ts = ToolSynthesizer::default();
        let t1 = ts.synthesize_from_capability("code", "compile", "compile source (src)", 0.8);
        let t2 = ts.synthesize_from_capability("code", "compile", "compile with options (src, target)", 0.9);
        assert_eq!(t1.version, 1);
        assert_eq!(t2.version, 2);
        assert_eq!(ts.tools.len(), 1, "duplicate name overwrites");
        assert_eq!(ts.tools.get("compile").unwrap().version, 2);
    }

    #[test]
    fn test_activate_above_threshold() {
        let mut ts = ToolSynthesizer::default();
        ts.synthesize_from_capability("search", "web_search", "search the web (query)", 0.85);
        let heuristic = ts.activate("web_search");
        assert!(heuristic.is_some());
        assert_eq!(heuristic.unwrap(), "TuneParam:query:0.1");
        assert_eq!(ts.tools.get("web_search").unwrap().usage_count, 1);
    }

    #[test]
    fn test_activate_below_threshold() {
        let mut ts = ToolSynthesizer::default();
        ts.activation_threshold = 0.8;
        ts.synthesize_from_capability("search", "web_search", "search the web (query)", 0.5);
        assert!(ts.activate("web_search").is_none());
    }

    #[test]
    fn test_activate_unknown() {
        let mut ts = ToolSynthesizer::default();
        assert!(ts.activate("nonexistent").is_none());
    }

    #[test]
    fn test_record_outcome_success() {
        let mut ts = ToolSynthesizer::default();
        ts.synthesize_from_capability("code", "compile", "compile source (src)", 0.8);
        ts.record_outcome("compile", true);
        let tool = ts.tools.get("compile").unwrap();
        assert_eq!(tool.usage_count, 1);
        assert!((tool.success_rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_record_outcome_failure_reduces_confidence() {
        let mut ts = ToolSynthesizer::default();
        ts.synthesize_from_capability("code", "compile", "compile source (src)", 0.8);
        ts.record_outcome("compile", false);
        let tool = ts.tools.get("compile").unwrap();
        assert!((tool.confidence - 0.8 * 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_active_tools_filters_low_confidence() {
        let mut ts = ToolSynthesizer::default();
        ts.synthesize_from_capability("search", "good_tool", "search the web (query)", 0.9);
        ts.synthesize_from_capability("search", "bad_tool", "bad search (query)", 0.3);
        let active = ts.active_tools();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "good_tool");
    }

    #[test]
    fn test_suggest_for_domain() {
        let mut ts = ToolSynthesizer::default();
        ts.synthesize_from_capability("search", "web_search", "search the web (query)", 0.7);
        ts.synthesize_from_capability("code", "compile", "compile source (src)", 0.9);
        ts.synthesize_from_capability("search", "image_search", "search images (query)", 0.8);
        let suggestions = ts.suggest_for_domain("search");
        assert_eq!(suggestions.len(), 2);
        // Sorted by confidence descending
        assert_eq!(suggestions[0].name, "image_search");
        assert_eq!(suggestions[1].name, "web_search");
    }

    #[test]
    fn test_suggest_for_domain_empty() {
        let ts = ToolSynthesizer::default();
        let suggestions = ts.suggest_for_domain("unknown");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_prune() {
        let mut ts = ToolSynthesizer::new(10, 0.0);
        ts.synthesize_from_capability("a", "good", "high confidence tool (x)", 0.9);
        ts.record_outcome("good", true);
        ts.record_outcome("good", true);
        ts.synthesize_from_capability("a", "bad", "low confidence tool (x)", 0.1);
        let pruned = ts.prune(0.5, 1);
        assert_eq!(pruned, 1);
        assert!(ts.tools.contains_key("good"));
        assert!(!ts.tools.contains_key("bad"));
    }

    #[test]
    fn test_max_tools_eviction() {
        let mut ts = ToolSynthesizer::new(3, 0.0);
        ts.synthesize_from_capability("a", "tool_a", "tool a", 0.9);
        ts.synthesize_from_capability("b", "tool_b", "tool b", 0.8);
        ts.synthesize_from_capability("c", "tool_c", "tool c", 0.7);
        // This should evict tool_c (lowest confidence)
        ts.synthesize_from_capability("d", "tool_d", "tool d", 0.85);
        assert_eq!(ts.tools.len(), 3);
        assert!(ts.tools.contains_key("tool_a"));
        assert!(ts.tools.contains_key("tool_b"));
        assert!(ts.tools.contains_key("tool_d"));
        assert!(!ts.tools.contains_key("tool_c"));
    }

    #[test]
    fn test_metrics() {
        let mut ts = ToolSynthesizer::default();
        ts.synthesize_from_capability("search", "web_search", "search the web (query)", 0.8);
        ts.record_outcome("web_search", true);
        let m = ts.metrics();
        assert_eq!(m["total_tools"], 1);
        assert_eq!(m["active_tools"], 1);
        assert_eq!(m["synthesis_count"], 1);
        assert_eq!(m["total_usage"], 1);
    }

    #[test]
    fn test_synthesize_empty_name_fallback() {
        let mut ts = ToolSynthesizer::default();
        let tool = ts.synthesize_from_capability("test", ": leading colon", "some description", 0.5);
        assert_eq!(tool.name, "tool_1");
        assert_eq!(ts.tools.len(), 1);
    }

    #[test]
    fn test_synthesize_parenthesized_return_type() {
        let mut ts = ToolSynthesizer::default();
        let tool = ts.synthesize_from_capability("code", "parse", "parse input -> Result<String>", 0.7);
        assert_eq!(tool.returns, "Result<String>");
    }
}
