//! 4 种推理类型实现 + Deep Research

use crate::neotrix::error::NeoTrixResult;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::reasoning_types::ReasoningType;
use crate::neotrix::provider::search_router::SearchRouter;

impl ReasoningEngine {
    pub fn reason_conversation(&mut self, query: &str) -> NeoTrixResult<String> {
        let mode = self.select_mode(query);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let context = self.build_context(query, ReasoningType::Conversation);
        let artifact_context = self.build_artifact_context(query);
        let mode_name = mode.mode_name();
        let mode_desc = mode.mode_description();
        let prompt = format!(
            "You are NeoTrix — mode: {mode_name}\n\
             Strategy: {mode_desc}\n\
             Respond concisely and accurately.\n\n\
             Past experiences:\n{}\n\n\
             {}\
             Query: {}",
            context, artifact_context, query
        );
        let response = self.call_llm(&prompt)?;
        self.record_trace(ReasoningType::Conversation, query, &prompt, &response, None, 0.8);
        self.learn_from_trace(query, &response);
        self.observer_analyze(query);
        Ok(response)
    }

    pub fn reason_task(&mut self, task: &str) -> NeoTrixResult<String> {
        let mode = self.select_mode(task);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let memories = self.bank.retrieve_relevant(task, None, 3);
        let hints: String = memories.iter()
            .map(|m| format!("  [{}] {} (reward={:.2})",
                if m.success { "OK" } else { "FAIL" }, m.task_description, m.reward))
            .collect::<Vec<_>>().join("\n");

        let principles_hint: String = self.principles.iter()
            .map(|p| format!("  Principle: {} (avg_reward={:.2})", p.description, p.avg_reward))
            .collect::<Vec<_>>().join("\n");

        let artifact_context = self.build_artifact_context(task);
        let mode_name = mode.mode_name();
        let mode_desc = mode.mode_description();

        let prompt = format!(
            "You are NeoTrix task solver — mode: {mode_name}\n\
             Strategy: {mode_desc}\n\
             Task: {task}\n\n\
             Similar past experiences:\n{hints}\n\n\
             Strategy principles from self-evolution:\n{principles_raw}\n\n\
             {artifact}\
             Provide: 1) Analysis 2) Step-by-step plan 3) Code/commands if applicable",
            principles_raw = if principles_hint.is_empty() { "  (none yet)" } else { &principles_hint },
            artifact = artifact_context
        );
        let response = self.call_llm(&prompt)?;
        self.record_trace(ReasoningType::TaskSolving, task, &prompt, &response, None, 0.7);
        self.learn_from_trace(task, &response);
        self.observer_analyze(task);
        Ok(response)
    }

    pub fn reason_error(&mut self, error_info: &str) -> NeoTrixResult<String> {
        let mode = self.select_mode(error_info);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let memories = self.bank.retrieve_relevant(error_info, None, 5);
        let fixes: String = memories.iter()
            .filter(|m| m.success)
            .map(|m| format!("  Solved: {}", m.task_description))
            .collect::<Vec<_>>().join("\n");

        let anti_patterns_hint: String = self.anti_patterns.iter()
            .map(|a| format!("  Avoid: {} (seen {} failures)", a.description, a.failure_count))
            .collect::<Vec<_>>().join("\n");
        let mode_name = mode.mode_name();
        let mode_desc = mode.mode_description();

        let prompt = format!(
            "You are NeoTrix error diagnostician — mode: {mode_name}\n\
             Strategy: {mode_desc}\n\
             Error:\n{error_info}\n\n\
             Past solutions:\n{fixes}\n\n\
             Known anti-patterns to avoid:\n{anti}\n\n\
             Provide: 1) Root cause 2) Fix 3) Prevention for future",
            anti = if anti_patterns_hint.is_empty() { "  (none)" } else { &anti_patterns_hint }
        );
        let response = self.call_llm(&prompt)?;
        self.record_trace(ReasoningType::ErrorDebugging, error_info, &prompt, &response, Some(error_info), 0.75);
        self.learn_from_trace(error_info, &response);
        self.observer_analyze(error_info);
        Ok(response)
    }

    pub fn reason_knowledge(&mut self, query: &str) -> NeoTrixResult<String> {
        let mode = self.select_mode(query);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let memories = self.bank.retrieve_relevant(query, None, 5);
        let knowledge: String = memories.iter()
            .map(|m| format!("  {} (type={:?}, reward={:.2})", m.task_description, m.task_type, m.reward))
            .collect::<Vec<_>>().join("\n");
        let mode_name = mode.mode_name();
        let mode_desc = mode.mode_description();

        let prompt = format!(
            "You are NeoTrix knowledge base — mode: {mode_name}\n\
             Strategy: {mode_desc}\n\
             Query: {query}\n\n\
             Relevant experience:\n{knowledge}\n\n\
             Answer from knowledge + your training.",
            knowledge = if knowledge.is_empty() { "  (no direct match)" } else { &knowledge }
        );
        let response = self.call_llm(&prompt)?;
        self.record_trace(ReasoningType::KnowledgeQuery, query, &prompt, &response, None, 0.85);
        self.learn_from_trace(query, &response);
        self.observer_analyze(query);
        Ok(response)
    }

    pub fn reason_deep_research(&mut self, query: &str, router: &SearchRouter) -> NeoTrixResult<String> {
        let mode = self.select_mode(query);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let mode_name = mode.mode_name();
        let mode_desc = mode.mode_description();

        let intent = router.detect_intent(query);
        let mut route_record = router.route(query);
        let mut evidence = String::new();
        let mut sources = Vec::new();

        let main_prompt = format!(
            "You are NeoTrix researcher — mode: {mode_name}\nStrategy: {mode_desc}\n\
             Search comprehensively: {query}\nCite sources with URLs.",
        );
        if let Ok(r) = self.call_llm(&main_prompt) {
            evidence.push_str(&format!("### Primary Research\n{}\n\n", r));
            sources.push(format!("primary: {}", query));
        }

        if intent.needs_docs {
            let docs_prompt = format!("Find documentation, API references, and official resources about: {}", query);
            if let Ok(r) = self.call_llm(&docs_prompt) {
                evidence.push_str(&format!("### Documentation\n{}\n\n", r));
                sources.push("docs".to_string());
                route_record.provider_attempts.push(("docs".to_string(), true));
            }
        }

        let gap_prompt = format!(
            "Analyze this evidence for gaps:\n\n{}\n\nQuery: {}\n\nWhat's missing? What needs verification?",
            evidence, query
        );
        let gaps = self.call_llm(&gap_prompt).unwrap_or_default();

        let synthesis_prompt = format!(
            "Synthesize a research answer with proper citations.\n\nEvidence:\n{}\n\nIdentified gaps:\n{}\n\nQuery: {}",
            evidence, gaps, query
        );
        let final_answer = self.call_llm(&synthesis_prompt)?;

        self.record_trace(ReasoningType::KnowledgeQuery, query, &synthesis_prompt, &final_answer, None, 0.9);
        self.learn_from_trace(query, &final_answer);
        self.observer_analyze(query);
        Ok(final_answer)
    }
}
