use crate::neotrix::error::NeoTrixResult;
use crate::neotrix::nt_mind::ReasoningType;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;

pub struct PrdInput {
    pub product_context: String,
    pub user_stories: Vec<String>,
    pub competitive_context: Vec<String>,
    pub design_notes: Option<String>,
}

impl PrdInput {
    pub fn new(context: &str) -> Self {
        Self {
            product_context: context.to_string(),
            user_stories: Vec::new(),
            competitive_context: Vec::new(),
            design_notes: None,
        }
    }
}

impl ReasoningEngine {
    pub fn reason_prd(&mut self, input: &PrdInput) -> NeoTrixResult<String> {
        let mode = self.select_mode("prd_generation");
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let prompt = build_prd_prompt(input);
        let response = self.call_llm(&prompt)?;
        let score = 0.7;

        self.record_trace(
            ReasoningType::PrdGeneration,
            &input.product_context,
            &prompt,
            &response,
            None,
            score,
        );

        self.learn_from_trace(&input.product_context, &response);
        self.observer_analyze(&input.product_context);

        Ok(response)
    }

    pub fn refine_prd(&mut self, input: &PrdInput, feedback: &str) -> NeoTrixResult<String> {
        let initial = self.reason_prd(input)?;
        let refinement_prompt = format!(
            "Improve the following PRD based on this feedback:\n\nFeedback: {}\n\nCurrent PRD:\n{}",
            feedback, initial
        );
        let mode = self.select_mode("prd_refinement");
        self.current_state = self.current_state.transition_to(mode);
        let response = self.call_llm(&refinement_prompt)?;
        Ok(response)
    }
}

fn build_prd_prompt(input: &PrdInput) -> String {
    let mut prompt = String::from(
        "You are a Senior Product Manager. Write a comprehensive Product Requirements Document (PRD) in Markdown.\n\n"
    );

    prompt.push_str("## Product Context\n");
    prompt.push_str(&input.product_context);
    prompt.push('\n');

    if !input.user_stories.is_empty() {
        prompt.push_str("\n## User Stories\n");
        for (i, story) in input.user_stories.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", i + 1, story));
        }
    }

    if !input.competitive_context.is_empty() {
        prompt.push_str("\n## Competitive Context\n");
        for ctx in &input.competitive_context {
            prompt.push_str(&format!("- {}\n", ctx));
        }
    }

    if let Some(notes) = &input.design_notes {
        prompt.push_str(&format!("\n## Design Notes\n{}\n", notes));
    }

    prompt.push_str("\n## Required Sections\n");
    prompt.push_str("1. **Overview** – one-paragraph summary\n");
    prompt.push_str("2. **Problem Statement** – what problem are we solving\n");
    prompt.push_str("3. **Target Users** – who this is for\n");
    prompt.push_str("4. **Features** – prioritized feature list with effort estimates\n");
    prompt.push_str("5. **Edge Cases** – what could go wrong\n");
    prompt.push_str("6. **Acceptance Criteria** – how we know it's done\n");
    prompt.push_str("7. **Success Metrics** – how we measure impact\n");
    prompt.push_str("8. **Risks & Mitigations** – what might block us\n");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prd_input_creation() {
        let input = PrdInput::new("A mobile app for task management");
        assert_eq!(input.product_context, "A mobile app for task management");
        assert!(input.user_stories.is_empty());
        assert!(input.competitive_context.is_empty());
        assert!(input.design_notes.is_none());
    }

    #[test]
    fn test_prd_input_with_stories() {
        let mut input = PrdInput::new("A todo app");
        input.user_stories.push("As a user, I can create tasks".to_string());
        input.user_stories.push("As a user, I can set due dates".to_string());
        assert_eq!(input.user_stories.len(), 2);
    }

    #[test]
    fn test_prd_prompt_contains_sections() {
        let input = PrdInput::new("Test product");
        let prompt = build_prd_prompt(&input);
        assert!(prompt.contains("Overview"));
        assert!(prompt.contains("Problem Statement"));
        assert!(prompt.contains("Target Users"));
        assert!(prompt.contains("Acceptance Criteria"));
        assert!(prompt.contains("Success Metrics"));
        assert!(prompt.contains("Risks & Mitigations"));
    }

    #[test]
    fn test_prd_type_variant() {
        let t = ReasoningType::PrdGeneration;
        assert_ne!(t, ReasoningType::General);
    }

    #[test]
    fn test_prd_prompt_contains_context() {
        let input = PrdInput::new("A social media analytics dashboard");
        let prompt = build_prd_prompt(&input);
        assert!(prompt.contains("social media analytics dashboard"));
        assert!(prompt.contains("Product Context"));
    }

    #[test]
    fn test_prd_prompt_with_competitive_context() {
        let mut input = PrdInput::new("A note-taking app");
        input.competitive_context.push("Notion has a rich editor".to_string());
        input.competitive_context.push("Obsidian has local-first sync".to_string());
        let prompt = build_prd_prompt(&input);
        assert!(prompt.contains("Notion has a rich editor"));
        assert!(prompt.contains("Obsidian has local-first sync"));
    }

    #[test]
    fn test_prd_prompt_with_design_notes() {
        let mut input = PrdInput::new("A fitness app");
        input.design_notes = Some("Use Apple Health integration".to_string());
        let prompt = build_prd_prompt(&input);
        assert!(prompt.contains("Apple Health integration"));
    }
}
