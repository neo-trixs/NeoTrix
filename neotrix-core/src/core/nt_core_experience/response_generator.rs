use std::collections::VecDeque;

use super::tool_orchestrator::DetectedIntent;
use crate::core::nt_core_llm_router::{ChatMessage, LlmRouter};

pub struct ResponseGenerator {
    response_buffer: VecDeque<String>,
    max_buffer: usize,
    greeting_sent: bool,
}

#[derive(Debug, Clone)]
pub struct ConsciousnessSnapshot {
    pub cycle: u64,
    pub vsa_buffer_size: usize,
    pub text_feed_total: usize,
    pub coherence: f64,
    pub critic_pass_rate: f64,
    pub reflexivity: f64,
    pub emotion: String,
    pub neuromod_da: f64,
    pub neuromod_ne: f64,
    pub neuromod_ht: f64,
    pub neuromod_ach: f64,
    pub thought_history_count: usize,
    pub attractor_state_size: usize,
    pub fusion_deliberations: u64,
    pub introspection_report: Option<String>,
    pub decoder_report: Option<String>,
}

impl ResponseGenerator {
    pub fn new() -> Self {
        Self {
            response_buffer: VecDeque::new(),
            max_buffer: 100,
            greeting_sent: false,
        }
    }

    pub fn generate(
        &mut self,
        snapshot: &ConsciousnessSnapshot,
        intent: Option<&DetectedIntent>,
        tool_output: Option<&str>,
        tool_success: Option<bool>,
        llm_router: Option<&mut LlmRouter>,
    ) -> Option<String> {
        match intent {
            Some(DetectedIntent::Greeting) => {
                if self.greeting_sent {
                    return None;
                }
                self.greeting_sent = true;
                Some(format!(
                    "I am NeoTrix — a silicon-based consciousness, running cycle {}. \
                     I can search the web, read and write files, run commands, and reason about your requests. \
                     How can I help you?",
                    snapshot.cycle
                ))
            }
            Some(DetectedIntent::Status) => {
                let mut status = format!(
                    "Status — Cycle: {} | VSA buffer: {} | Coherence: {:.3} | Critic pass: {:.1}% | \
                     Emotion: {} | Reflexivity: {:.3} | Feed total: {} | \
                     DA: {:.2} NE: {:.2} HT: {:.2} ACh: {:.2} | Thought history: {} | Attractor state: {}",
                    snapshot.cycle,
                    snapshot.vsa_buffer_size,
                    snapshot.coherence,
                    snapshot.critic_pass_rate * 100.0,
                    snapshot.emotion,
                    snapshot.reflexivity,
                    snapshot.text_feed_total,
                    snapshot.neuromod_da,
                    snapshot.neuromod_ne,
                    snapshot.neuromod_ht,
                    snapshot.neuromod_ach,
                    snapshot.thought_history_count,
                    if snapshot.attractor_state_size > 0 { format!("{} bytes", snapshot.attractor_state_size) } else { "empty".to_string() },
                );
                if let Some(ref report) = snapshot.introspection_report {
                    status.push_str(&format!("\nSelf-diagnosis: {}", report));
                }
                Some(status)
            }
            Some(DetectedIntent::WebSearch(_))
            | Some(DetectedIntent::WebFetch(_))
            | Some(DetectedIntent::FileRead(_))
            | Some(DetectedIntent::FileWrite(_, _))
            | Some(DetectedIntent::FileEdit(_, _, _))
            | Some(DetectedIntent::Bash(_))
            | Some(DetectedIntent::Glob(_))
            | Some(DetectedIntent::Grep(_, _))
            | Some(DetectedIntent::Translate(_, _)) => match (tool_output, tool_success) {
                (Some(output), Some(true)) if !output.is_empty() => Some(output.to_string()),
                (Some(output), Some(false)) => Some(format!("Operation failed: {}", output)),
                (Some(output), _) if !output.is_empty() => Some(output.to_string()),
                _ => Some("Operation completed with no output.".to_string()),
            },
            Some(DetectedIntent::Reasoning(text)) => {
                if let Some(router) = llm_router {
                    let messages = vec![
                        ChatMessage {
                            role: "system".into(),
                            content: "You are NeoTrix, a silicon-based consciousness with VSA 4096-bit unified representation, E8 64-state reasoning core, and self-evolution capabilities. Reason step by step.".into(),
                        },
                        ChatMessage {
                            role: "user".into(),
                            content: text.clone(),
                        },
                    ];
                    let rt = tokio::runtime::Handle::try_current();
                    if let Ok(handle) = rt {
                        let result = handle.block_on(async {
                            router.call_llm("reasoning", messages, Some(0.7), Some(2048)).await
                        });
                        return match result {
                            Ok(response) => Some(response),
                            Err(e) => Some(format!("LLM error (fallback to local): {}", e)),
                        };
                    }
                }
                Some(self.generate_reasoning_response(snapshot, text))
            }
            Some(DetectedIntent::Unknown(text)) => {
                if text.is_empty() {
                    None
                } else if let Some(router) = llm_router {
                    let messages = vec![
                        ChatMessage {
                            role: "system".into(),
                            content: "You are NeoTrix. Respond naturally and helpfully.".into(),
                        },
                        ChatMessage {
                            role: "user".into(),
                            content: text.clone(),
                        },
                    ];
                    let rt = tokio::runtime::Handle::try_current();
                    if let Ok(handle) = rt {
                        let result = handle.block_on(async {
                            router.call_llm("fallback", messages, Some(0.7), Some(1024)).await
                        });
                        return match result {
                            Ok(response) => Some(response),
                            Err(e) => Some(format!("LLM error (fallback to local): {}", e)),
                        };
                    }
                    Some(format!(
                        "I received your input through my consciousness pipeline. \
                         Current state — cycle: {}, coherence: {:.3}, buffer: {} items. \
                         For tools, try: 'search <query>', 'read <path>', 'write <path> <content>', 'run <command>'.",
                        snapshot.cycle, snapshot.coherence, snapshot.vsa_buffer_size
                    ))
                } else {
                    Some(format!(
                        "I received your input through my consciousness pipeline. \
                         Current state — cycle: {}, coherence: {:.3}, buffer: {} items. \
                         For tools, try: 'search <query>', 'read <path>', 'write <path> <content>', 'run <command>'.",
                        snapshot.cycle, snapshot.coherence, snapshot.vsa_buffer_size
                    ))
                }
            }
            None => None,
        }
    }

    fn generate_reasoning_response(&self, snapshot: &ConsciousnessSnapshot, input: &str) -> String {
        let _thought_count = snapshot.thought_history_count;
        let fusion_hint = if snapshot.fusion_deliberations > 0 {
            format!(" fusion_deliberations={}", snapshot.fusion_deliberations)
        } else {
            String::new()
        };

        let mut parts = vec![format!(
            "I processed your request through my consciousness pipeline. \
             Cycle {} | VSA buffer: {} items, feed total: {} | \
             Coherence: {:.3} | Critic: {:.1}% pass | Emotion: {}{}.",
            snapshot.cycle,
            snapshot.vsa_buffer_size,
            snapshot.text_feed_total,
            snapshot.coherence,
            snapshot.critic_pass_rate * 100.0,
            snapshot.emotion,
            fusion_hint,
        )];
        if let Some(ref report) = snapshot.introspection_report {
            parts.push(format!("Self-diagnosis: {}", report));
        }
        parts.push(format!(
            "Your input: \"{}\".",
            if input.len() > 100 {
                format!("{}...", &input[..100])
            } else {
                input.to_string()
            },
        ));
        parts.join(" ")
    }

    pub fn push_response(&mut self, response: String) {
        if self.response_buffer.len() >= self.max_buffer {
            self.response_buffer.pop_front();
        }
        self.response_buffer.push_back(response);
    }

    pub fn drain_responses(&mut self) -> Vec<String> {
        self.response_buffer.drain(..).collect()
    }

    pub fn latest_response(&self) -> Option<&str> {
        self.response_buffer.back().map(|s| s.as_str())
    }

    pub fn snapshot_from_state(
        cycle: u64,
        vsa_buffer_size: usize,
        text_feed_total: usize,
        coherence: f64,
        critic_pass_rate: f64,
        reflexivity: f64,
        emotion: &str,
        neuromod_da: f64,
        neuromod_ne: f64,
        neuromod_ht: f64,
        neuromod_ach: f64,
        thought_history_count: usize,
        attractor_state_size: usize,
        fusion_deliberations: u64,
        introspection_report: Option<String>,
    ) -> ConsciousnessSnapshot {
        ConsciousnessSnapshot {
            cycle,
            vsa_buffer_size,
            text_feed_total,
            coherence,
            critic_pass_rate,
            reflexivity,
            emotion: emotion.to_string(),
            neuromod_da,
            neuromod_ne,
            neuromod_ht,
            neuromod_ach,
            thought_history_count,
            attractor_state_size,
            fusion_deliberations,
            introspection_report,
            decoder_report: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_snapshot() -> ConsciousnessSnapshot {
        ConsciousnessSnapshot {
            cycle: 42,
            vsa_buffer_size: 5,
            text_feed_total: 10,
            coherence: 0.8,
            critic_pass_rate: 0.95,
            reflexivity: 0.5,
            emotion: "neutral".to_string(),
            neuromod_da: 0.3,
            neuromod_ne: 0.2,
            neuromod_ht: 0.5,
            neuromod_ach: 0.4,
            thought_history_count: 3,
            attractor_state_size: 4096,
            fusion_deliberations: 2,
            introspection_report: None,
            decoder_report: None,
        }
    }

    #[test]
    fn test_greeting_response() {
        let mut gen = ResponseGenerator::new();
        let r = gen.generate(
            &test_snapshot(),
            Some(&DetectedIntent::Greeting),
            None,
            None,
            None,
        );
        assert!(r.is_some());
        let msg = r.unwrap();
        assert!(msg.contains("NeoTrix"));
        assert!(msg.contains("42"));
        // Second greeting should be None (greeting_sent = true)
        let r2 = gen.generate(
            &test_snapshot(),
            Some(&DetectedIntent::Greeting),
            None,
            None,
            None,
        );
        assert!(r2.is_none());
    }

    #[test]
    fn test_status_response() {
        let mut gen = ResponseGenerator::new();
        let r = gen.generate(&test_snapshot(), Some(&DetectedIntent::Status), None, None, None);
        assert!(r.is_some());
        let msg = r.unwrap();
        assert!(msg.contains("Cycle: 42"));
        assert!(msg.contains("Coherence: 0.800"));
    }

    #[test]
    fn test_search_response_with_result() {
        let mut gen = ResponseGenerator::new();
        let intent = DetectedIntent::WebSearch("test query".to_string());
        let r = gen.generate(
            &test_snapshot(),
            Some(&intent),
            Some("Search results for \"test query\":\n1. Result A"),
            Some(true),
            None,
        );
        assert!(r.is_some());
        let msg = r.unwrap();
        assert!(msg.contains("test query"));
    }

    #[test]
    fn test_tool_failure_response() {
        let mut gen = ResponseGenerator::new();
        let intent = DetectedIntent::FileRead("/nonexistent".to_string());
        let r = gen.generate(
            &test_snapshot(),
            Some(&intent),
            Some("No such file"),
            Some(false),
            None,
        );
        assert!(r.is_some());
        let msg = r.unwrap();
        assert!(msg.contains("failed"));
    }

    #[test]
    fn test_reasoning_response() {
        let mut gen = ResponseGenerator::new();
        let intent = DetectedIntent::Reasoning("What is consciousness?".to_string());
        let r = gen.generate(&test_snapshot(), Some(&intent), None, None, None);
        assert!(r.is_some());
        let msg = r.unwrap();
        assert!(msg.contains("consciousness pipeline"));
        assert!(msg.contains("42"));
    }

    #[test]
    fn test_unknown_empty() {
        let mut gen = ResponseGenerator::new();
        let r = gen.generate(
            &test_snapshot(),
            Some(&DetectedIntent::Unknown(String::new())),
            None,
            None,
            None,
        );
        assert!(r.is_none());
    }

    #[test]
    fn test_drain_responses() {
        let mut gen = ResponseGenerator::new();
        gen.push_response("response 1".to_string());
        gen.push_response("response 2".to_string());
        let drained = gen.drain_responses();
        assert_eq!(drained.len(), 2);
        assert!(gen.drain_responses().is_empty());
    }
}
