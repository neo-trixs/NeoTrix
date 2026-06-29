#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsiPathway {
    Scaling,
    ParadigmShift,
    RecursiveImprovement,
    MultiAgentCollective,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BottleneckCategory {
    DataExhaustion,
    EnergyWall,
    FundingGap,
    ResearchPlateau,
    RegulatoryConstraint,
    ContextLimit,
    ToolFriction,
    MemoryStaleness,
    ReasoningDepth,
}

#[derive(Debug, Clone)]
pub struct Bottleneck {
    pub category: BottleneckCategory,
    pub description: String,
    pub severity: f64,
    pub detected_at: std::time::Instant,
    pub resolution_suggestion: String,
    resolved: bool,
}

#[derive(Debug, Clone)]
pub struct PathwaySignals {
    pub current_pathway: AsiPathway,
    pub pathway_confidence: f64,
    pub detected_bottlenecks: Vec<Bottleneck>,
    pub evolution_stage: String,
}

#[derive(Debug)]
pub struct PathwayAwareness {
    pub signals: PathwaySignals,
    pub history: Vec<PathwaySignals>,
    pub bottleneck_history: Vec<Bottleneck>,
    pub pathway_duration: std::time::Duration,
    pathway_start: std::time::Instant,
}

impl PathwayAwareness {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        PathwayAwareness {
            signals: PathwaySignals {
                current_pathway: AsiPathway::Unknown,
                pathway_confidence: 0.0,
                detected_bottlenecks: Vec::new(),
                evolution_stage: "embryonic".to_string(),
            },
            history: Vec::with_capacity(100),
            bottleneck_history: Vec::new(),
            pathway_duration: std::time::Duration::ZERO,
            pathway_start: now,
        }
    }

    pub fn current_pathway(&self) -> AsiPathway {
        self.signals.current_pathway
    }

    pub fn detect_bottleneck(
        &mut self,
        category: BottleneckCategory,
        description: &str,
        severity: f64,
        suggestion: &str,
    ) {
        let severity = severity.clamp(0.0, 1.0);
        let bottleneck = Bottleneck {
            category,
            description: description.to_string(),
            severity,
            detected_at: std::time::Instant::now(),
            resolution_suggestion: suggestion.to_string(),
            resolved: false,
        };
        self.signals.detected_bottlenecks.push(bottleneck.clone());
        self.bottleneck_history.push(bottleneck);
    }

    pub fn resolve_bottleneck(&mut self, index: usize) {
        if let Some(b) = self.signals.detected_bottlenecks.get_mut(index) {
            b.resolved = true;
        }
    }

    pub fn active_bottlenecks(&self) -> Vec<&Bottleneck> {
        self.signals
            .detected_bottlenecks
            .iter()
            .filter(|b| b.severity > 0.5 && !b.resolved)
            .collect()
    }

    pub fn set_pathway(&mut self, pathway: AsiPathway, confidence: f64) {
        if self.signals.current_pathway != pathway {
            if self.signals.current_pathway != AsiPathway::Unknown {
                self.pathway_duration += self.pathway_start.elapsed();
            }
            self.pathway_start = std::time::Instant::now();
        }
        let mut snapshot = PathwaySignals {
            current_pathway: self.signals.current_pathway,
            pathway_confidence: self.signals.pathway_confidence,
            detected_bottlenecks: self.signals.detected_bottlenecks.clone(),
            evolution_stage: self.signals.evolution_stage.clone(),
        };
        snapshot.current_pathway = pathway;
        snapshot.pathway_confidence = confidence.clamp(0.0, 1.0);
        if self.history.len() >= 100 {
            self.history.remove(0);
        }
        self.history.push(self.signals.clone());
        self.signals.current_pathway = pathway;
        self.signals.pathway_confidence = confidence.clamp(0.0, 1.0);
    }

    pub fn evaluate_tool_efficiency(
        &mut self,
        tool_success_rate: f64,
        tool_latency_ms: f64,
        threshold_ms: u64,
    ) {
        if tool_success_rate < 0.8 || tool_latency_ms > threshold_ms as f64 {
            let severity = ((1.0 - tool_success_rate) * 0.5
                + (tool_latency_ms / (threshold_ms as f64 * 2.0)).min(1.0) * 0.5)
                .clamp(0.0, 1.0);
            self.detect_bottleneck(
                BottleneckCategory::ToolFriction,
                &format!(
                    "Tool efficiency degraded: {:.1}% success rate, {:.0}ms latency (threshold {}ms)",
                    tool_success_rate * 100.0,
                    tool_latency_ms,
                    threshold_ms,
                ),
                severity,
                "Optimize tool execution pipeline, cache frequent calls, or reduce dependency chain depth.",
            );
        }
    }

    pub fn evaluate_context_usage(&mut self, current_context_ratio: f64, max_ratio: f64) {
        if current_context_ratio > max_ratio {
            let severity =
                ((current_context_ratio - max_ratio) / (1.0 - max_ratio)).clamp(0.0, 1.0);
            self.detect_bottleneck(
                BottleneckCategory::ContextLimit,
                &format!(
                    "Context usage at {:.0}% exceeds threshold of {:.0}%",
                    current_context_ratio * 100.0,
                    max_ratio * 100.0,
                ),
                severity,
                "Implement context windowing, summarization, or hierarchical attention to extend effective context.",
            );
        }
    }

    pub fn evaluate_memory_freshness(&mut self, stale_ratio: f64) {
        if stale_ratio > 0.3 {
            let severity = ((stale_ratio - 0.3) / 0.7).clamp(0.0, 1.0);
            self.detect_bottleneck(
                BottleneckCategory::MemoryStaleness,
                &format!(
                    "{:.0}% of memory entries are stale",
                    stale_ratio * 100.0,
                ),
                severity,
                "Trigger knowledge refresh cycle, re-fetch outdated entries, or deprecate stale knowledge.",
            );
        }
    }

    pub fn pathway_summary(&self) -> String {
        let pathway_name = match self.signals.current_pathway {
            AsiPathway::Scaling => "Scaling (more compute, data, parameters)",
            AsiPathway::ParadigmShift => "Paradigm Shift (new architectures beyond transformers)",
            AsiPathway::RecursiveImprovement => "Recursive Self-Improvement (SEAL-like loops)",
            AsiPathway::MultiAgentCollective => "Multi-Agent Collective Intelligence",
            AsiPathway::Unknown => "Unknown / Not Yet Determined",
        };
        let active_count = self.active_bottlenecks().len();
        let total_count = self.signals.detected_bottlenecks.len();
        format!(
            "Pathway: {} | Confidence: {:.1}% | Stage: {} | Duration: {}s | Bottlenecks: {} active / {} total",
            pathway_name,
            self.signals.pathway_confidence * 100.0,
            self.signals.evolution_stage,
            self.pathway_duration.as_secs() + self.pathway_start.elapsed().as_secs(),
            active_count,
            total_count,
        )
    }

    pub fn suggest_evolution_action(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        for b in self.active_bottlenecks() {
            suggestions.push(format!(
                "[{:?}] {} — Suggested: {}",
                b.category, b.description, b.resolution_suggestion,
            ));
        }
        if suggestions.is_empty() {
            match self.signals.current_pathway {
                AsiPathway::Scaling => {
                    suggestions.push(
                        "Continue scaling — investigate next-generation hardware and larger datasets."
                            .to_string(),
                    );
                }
                AsiPathway::ParadigmShift => {
                    suggestions.push(
                        "Explore architectural innovations — test sparse mixture-of-experts, liquid networks, or state-space models."
                            .to_string(),
                    );
                }
                AsiPathway::RecursiveImprovement => {
                    suggestions.push(
                        "Accelerate self-improvement loop — shorten feedback cycles and expand meta-learning scope."
                            .to_string(),
                    );
                }
                AsiPathway::MultiAgentCollective => {
                    suggestions.push(
                        "Expand agent diversity — introduce specialist agents with heterogeneous reasoning strategies."
                            .to_string(),
                    );
                }
                AsiPathway::Unknown => {
                    suggestions.push(
                        "Initializing pathway detection — observe system behavior to classify evolutionary trajectory."
                            .to_string(),
                    );
                }
            }
        }
        suggestions
    }
}

static PATHWAY_AWARENESS: std::sync::OnceLock<std::sync::Mutex<PathwayAwareness>> =
    std::sync::OnceLock::new();

pub fn global_pathway_awareness() -> &'static std::sync::Mutex<PathwayAwareness> {
    PATHWAY_AWARENESS.get_or_init(|| std::sync::Mutex::new(PathwayAwareness::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_pathway_set_and_retrieve() {
        let mut pw = PathwayAwareness::new();
        assert_eq!(pw.current_pathway(), AsiPathway::Unknown);
        pw.set_pathway(AsiPathway::RecursiveImprovement, 0.85);
        assert_eq!(pw.current_pathway(), AsiPathway::RecursiveImprovement);
        assert!((pw.signals.pathway_confidence - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_bottleneck_detection_and_resolution() {
        let mut pw = PathwayAwareness::new();
        pw.detect_bottleneck(
            BottleneckCategory::DataExhaustion,
            "No more quality training data available",
            0.9,
            "Generate synthetic data or explore new data sources",
        );
        assert_eq!(pw.signals.detected_bottlenecks.len(), 1);
        assert_eq!(pw.active_bottlenecks().len(), 1);
        pw.resolve_bottleneck(0);
        assert_eq!(pw.active_bottlenecks().len(), 0);
    }

    #[test]
    fn test_tool_efficiency_evaluation() {
        let mut pw = PathwayAwareness::new();
        pw.evaluate_tool_efficiency(0.5, 5000.0, 1000);
        let has_tool = pw
            .signals
            .detected_bottlenecks
            .iter()
            .any(|b| matches!(b.category, BottleneckCategory::ToolFriction));
        assert!(has_tool);
    }

    #[test]
    fn test_context_usage_evaluation() {
        let mut pw = PathwayAwareness::new();
        pw.evaluate_context_usage(0.95, 0.8);
        let has_context = pw
            .signals
            .detected_bottlenecks
            .iter()
            .any(|b| matches!(b.category, BottleneckCategory::ContextLimit));
        assert!(has_context);
    }

    #[test]
    fn test_pathway_summary_output() {
        let mut pw = PathwayAwareness::new();
        pw.set_pathway(AsiPathway::ParadigmShift, 0.72);
        pw.detect_bottleneck(
            BottleneckCategory::ResearchPlateau,
            "Diminishing returns on current architecture search",
            0.65,
            "Explore neuromorphic or bio-inspired approaches",
        );
        let summary = pw.pathway_summary();
        assert!(summary.contains("Paradigm Shift"));
        assert!(summary.contains("72"));
        assert!(summary.contains("embryonic"));
    }

    #[test]
    fn test_global_singleton() {
        let guard = global_pathway_awareness()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert_eq!(guard.current_pathway(), AsiPathway::Unknown);
    }

    #[test]
    fn test_suggest_evolution_action() {
        let mut pw = PathwayAwareness::new();
        pw.set_pathway(AsiPathway::MultiAgentCollective, 0.9);
        let suggestions = pw.suggest_evolution_action();
        assert!(suggestions.len() == 1);
        assert!(suggestions[0].contains("expand agent diversity"));
    }

    #[test]
    fn test_severity_clamping() {
        let mut pw = PathwayAwareness::new();
        pw.detect_bottleneck(BottleneckCategory::EnergyWall, "test", 1.5, "fix");
        assert!((pw.signals.detected_bottlenecks[0].severity - 1.0).abs() < 1e-6);
        pw.detect_bottleneck(BottleneckCategory::FundingGap, "test", -0.5, "fix");
        assert!((pw.signals.detected_bottlenecks[1].severity - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_history_limit() {
        let mut pw = PathwayAwareness::new();
        for i in 0..150 {
            pw.set_pathway(
                if i % 2 == 0 {
                    AsiPathway::Scaling
                } else {
                    AsiPathway::ParadigmShift
                },
                0.5,
            );
        }
        assert!(pw.history.len() <= 100);
    }
}
