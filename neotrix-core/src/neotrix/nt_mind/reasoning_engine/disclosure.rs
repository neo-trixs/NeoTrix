use serde::{Deserialize, Serialize};

use crate::neotrix::nt_mind::reasoning_types::ReasoningTrace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureLevel {
    pub level: u8,
    pub label: String,
}

impl DisclosureLevel {
    pub fn all() -> Vec<Self> {
        vec![
            Self { level: 0, label: "analogy".into() },
            Self { level: 1, label: "details".into() },
            Self { level: 2, label: "code".into() },
            Self { level: 3, label: "full".into() },
        ]
    }
}

pub fn render_progressive(trace: &ReasoningTrace, level: u8) -> String {
    match level {
        0 => render_analogy(trace),
        1 => render_details(trace),
        2 => render_code(trace),
        _ => render_full(trace),
    }
}

fn render_analogy(trace: &ReasoningTrace) -> String {
//    let mut output = format!("## {} — Analogy\n", trace.action);
    for step in &trace.steps {
        if !step.why.is_empty() && step.why != "TBD" {
            output += &format!("- {}\n", step.why);
        }
    }
    if !trace.decision.is_empty() && trace.decision != "TBD" {
        output += &format!("\n**Decision**: {}\n", trace.decision);
    }
    output
}

fn render_details(trace: &ReasoningTrace) -> String {
//    let mut output = format!("## {} — Details\n", trace.action);
    for step in &trace.steps {
        if !step.what.is_empty() && step.what != "TBD" {
            output += &format!("### {}\n", step.what);
        }
        if !step.framework.as_deref().unwrap_or("").is_empty() {
            output += &format!("_Framework_: {}\n", step.framework.as_deref().unwrap_or(""));
        }
        if !step.alternatives.is_empty() {
            output += "**Alternatives considered**:\n";
            for alt in &step.alternatives {
                output += &format!("- {}\n", alt);
            }
        }
    }
    output
}

fn render_code(trace: &ReasoningTrace) -> String {
//    let mut output = format!("## {} — Implementation\n", trace.action);
    for step in &trace.steps {
        output += &format!("```\n// Step: {}\n{}\n```\n", step.action, step.output);
    }
    output
}

fn render_full(trace: &ReasoningTrace) -> String {
//    let mut output = format!("## {} — Full Trace\n", trace.action);
    for (i, step) in trace.steps.iter().enumerate() {
        output += &format!("### Step {}: {}\n", i + 1, step.action);
        output += &format!("- **WHAT**: {}\n", step.what);
        output += &format!("- **WHY**: {}\n", step.why);
        if let Some(fw) = &step.framework {
            output += &format!("- **Framework**: {}\n", fw);
        }
        if !step.alternatives.is_empty() {
            output += "- **Alternatives**:\n";
            for alt in &step.alternatives {
                output += &format!("  - {}\n", alt);
            }
        }
        output += &format!("- **Decision**: {}\n", step.decision);
        output += &format!("- **Input**: `{}`\n", step.input);
        output += &format!("- **Output**: `{}`\n", step.output);
    }
    output
}

pub fn render_trace(trace: &ReasoningTrace, level: u8) -> String {
    render_progressive(trace, level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::reasoning_types::TraceStep;

    #[test]
    fn test_analogy_level_renders_why_only() {
        let mut trace = ReasoningTrace::new("test_action");
        trace.add_step(TraceStep {
            action: "query".into(),
            what: "Query LLM for capability update".into(),
            why: "LLM output is unconstrained; normalize() prevents dimension pollution".into(),
            framework: Some("variance argument: 1/√d_k scaling".into()),
            alternatives: vec!["Skip normalization".into(), "Clamp without normalize".into()],
            decision: "normalize() preserves relative ranking while bounding magnitude".into(),
            input: "...".into(),
            output: "...".into(),
            timestamp: chrono::Utc::now().timestamp(),
        });
        trace.decision = "Use normalize()".into();

        let result = render_progressive(&trace, 0);
        assert!(result.contains("LLM output is unconstrained"));
        assert!(result.contains("Use normalize()"));
        assert!(!result.contains("Skip normalization"));
    }

    #[test]
    fn test_full_level_renders_all() {
        let mut trace = ReasoningTrace::new("test_action");
        trace.add_step(TraceStep {
            action: "query".into(),
            what: "Query LLM".into(),
            why: "Normalize output".into(),
            framework: None,
            alternatives: vec![],
            decision: "Use normalize".into(),
            input: "raw_data".into(),
            output: "normalized".into(),
            timestamp: chrono::Utc::now().timestamp(),
        });

        let result = render_progressive(&trace, 3);
        assert!(result.contains("WHAT"));
        assert!(result.contains("WHY"));
        assert!(result.contains("Input"));
        assert!(result.contains("Output"));
    }

    #[test]
    fn test_details_level_renders_what_and_framework() {
        let mut trace = ReasoningTrace::new("test_action");
        trace.add_step(TraceStep {
            action: "query".into(),
            what: "Query LLM".into(),
            why: "Normalize output".into(),
            framework: Some("1/√d_k scaling".into()),
            alternatives: vec!["Skip".into()],
            decision: "Use normalize".into(),
            input: "raw".into(),
            output: "norm".into(),
            timestamp: chrono::Utc::now().timestamp(),
        });

        let result = render_progressive(&trace, 1);
        assert!(result.contains("Query LLM"));
        assert!(result.contains("1/√d_k scaling"));
        assert!(result.contains("Alternatives considered"));
    }

    #[test]
    fn test_code_level_renders_output() {
        let mut trace = ReasoningTrace::new("test_action");
        trace.add_step(TraceStep {
            action: "embed".into(),
            what: "Embed".into(),
            why: "Need vectors".into(),
            framework: None,
            alternatives: vec![],
            decision: "Use BERT".into(),
            input: "text".into(),
            output: "vec![0.1, 0.2, 0.3]".into(),
            timestamp: chrono::Utc::now().timestamp(),
        });

        let result = render_progressive(&trace, 2);
        assert!(result.contains("Step: embed"));
        assert!(result.contains("vec![0.1, 0.2, 0.3]"));
    }

    #[test]
    fn test_empty_trace() {
        let trace = ReasoningTrace::new("empty");
        let r0 = render_progressive(&trace, 0);
        let r3 = render_progressive(&trace, 3);
        assert!(r0.contains("empty"));
        assert!(r3.contains("empty"));
        assert!(!r0.contains("TBD"));
    }

    #[test]
    fn test_disclosure_level_all_returns_four() {
        let levels = DisclosureLevel::all();
        assert_eq!(levels.len(), 4);
        assert_eq!(levels[0].label, "analogy");
        assert_eq!(levels[3].label, "full");
    }

    #[test]
    fn test_render_trace_wrapper() {
        let trace = ReasoningTrace::new("wrapper");
        let result = render_trace(&trace, 3);
        assert!(result.contains("wrapper"));
    }
}
