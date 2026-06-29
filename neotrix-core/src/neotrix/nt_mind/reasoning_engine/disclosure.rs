use crate::neotrix::nt_mind::reasoning_types::ReasoningTrace;

#[derive(Debug, Clone)]
pub struct DisclosureLevel {
    pub level: u8,
    pub label: String,
}

impl DisclosureLevel {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                level: 0,
                label: "summary".into(),
            },
            Self {
                level: 1,
                label: "details".into(),
            },
            Self {
                level: 2,
                label: "full".into(),
            },
            Self {
                level: 3,
                label: "complete".into(),
            },
        ]
    }
}

pub fn render_progressive(trace: &ReasoningTrace, level: u8) -> String {
    let base = format!(
        "## {} ({:?})\nConfidence: {:.2} | Success: {}",
        trace.task, trace.reasoning_type, trace.outcome_score, trace.success
    );
    match level {
        0 => base,
        1 => format!("{}\nPrompt: {}", base, trace.prompt),
        2 => format!("{}\nLLM Response: {}", base, trace.llm_response),
        _ => {
            let mut out = format!(
                "{}\nPrompt: {}\nResponse: {}",
                base, trace.prompt, trace.llm_response
            );
            if let Some(ref err) = trace.error_context {
                out += &format!("\nError: {}", err);
            }
            out
        }
    }
}

pub fn render_trace(trace: &ReasoningTrace, level: u8) -> String {
    render_progressive(trace, level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::reasoning_types::{
        PerspectiveLens, ReasoningMethod, ReasoningType,
    };

    #[test]
    fn test_summary_level() {
        let trace = ReasoningTrace {
            id: "test-1".into(),
            reasoning_type: ReasoningType::TaskSolving,
            reasoning_method: Some(ReasoningMethod::FirstPrinciples),
            perspective_lens: Some(PerspectiveLens::Builder),
            task: "optimize query".into(),
            prompt: "How to optimize?".into(),
            llm_response: "Use indexing".into(),
            error_context: None,
            outcome_score: 0.9,
            success: true,
            timestamp: 0,
            vsa_tag: None,
        };
        let result = render_progressive(&trace, 0);
        assert!(result.contains("optimize query"));
        assert!(!result.contains("Use indexing"));
    }

    #[test]
    fn test_full_level_renders_all() {
        let trace = ReasoningTrace {
            id: "test-2".into(),
            reasoning_type: ReasoningType::KnowledgeQuery,
            reasoning_method: None,
            perspective_lens: None,
            task: "search API".into(),
            prompt: "Find REST endpoints".into(),
            llm_response: "GET /api/v1".into(),
            error_context: Some("timeout".into()),
            outcome_score: 0.5,
            success: false,
            timestamp: 0,
            vsa_tag: None,
        };
        let result = render_progressive(&trace, 3);
        assert!(result.contains("search API"));
        assert!(result.contains("GET /api/v1"));
        assert!(result.contains("timeout"));
    }

    #[test]
    fn test_disclosure_level_all_returns_four() {
        let levels = DisclosureLevel::all();
        assert_eq!(levels.len(), 4);
        assert_eq!(levels[0].label, "summary");
        assert_eq!(levels[3].label, "complete");
    }

    #[test]
    fn test_render_trace_wrapper() {
        let trace = ReasoningTrace {
            id: "wrapper".into(),
            reasoning_type: ReasoningType::General,
            reasoning_method: None,
            perspective_lens: None,
            task: "wrapper test".into(),
            prompt: "".into(),
            llm_response: "".into(),
            error_context: None,
            outcome_score: 0.0,
            success: true,
            timestamp: 0,
            vsa_tag: None,
        };
        let result = render_trace(&trace, 3);
        assert!(result.contains("wrapper test"));
    }
}
