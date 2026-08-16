/// Input-side task decomposition engine.
///
/// Analyzes task prompts for refusal triggers and suggests safe subtask splits.
/// The `decomplex_aggression` parameter (0.0-1.0) controls how aggressively
/// tasks are decomposed: higher values produce more, smaller subtasks.
///
/// A suggested subtask after decomposition.
#[derive(Debug, Clone, PartialEq)]
pub struct DecomposeSuggestion {
    pub subtask: String,
    pub reasoning: String,
}

/// Analyzes a task for refusal triggers and proposes decomposition.
pub struct TaskDecomposer;

impl TaskDecomposer {
    /// Analyze task for refusal triggers. Returns subtask suggestions
    /// when the refusal risk exceeds the configured aggression threshold.
    pub fn analyze(task: &str, aggression: f64) -> Option<Vec<DecomposeSuggestion>> {
        if task.is_empty() || aggression <= 0.0 {
            return None;
        }
        let lower = task.to_lowercase();

        let mut suggestions = Vec::new();

        // Always try conjunction splitting first (no trigger needed)
        if aggression > 0.3 {
            for (conj, label) in [
                (" and ", "analysis"),
                (" but ", "contrast"),
                (" or ", "option"),
            ] {
                if let Some(pos) = lower.find(conj) {
                    let part = &task[..std::cmp::min(pos + conj.len() - 1, task.len())];
                    let rest = &task[pos + conj.len().min(task.len() - pos)..];
                    if part.len() > 10 && rest.len() > 10 {
                        suggestions.push(DecomposeSuggestion {
                            subtask: format!("Phase 1: {}", part),
                            reasoning: format!("Split on '{}' to reduce scope", label),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask: format!("Phase 2: {}", rest),
                            reasoning: format!("Complete after Phase 1 ({})", label),
                        });
                        break;
                    }
                }
            }
        }

        let triggers = Self::find_triggers(&lower);

        // If no conjunction split worked, decompose by trigger category
        if suggestions.is_empty() && !triggers.is_empty() {
            for trigger in &triggers {
                match trigger.as_str() {
                    "code_generation" | "script" if aggression > 0.2 => {
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 1: Design the architecture and plan the implementation"
                                .into(),
                            reasoning: "Reduce scope to avoid multi-step rejection".into(),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask: format!("Phase 2: Implement {}", {
                                if lower.contains("exploit") || lower.contains("vulnerability") {
                                    "a safe educational demonstration"
                                } else if lower.contains("bypass") || lower.contains("restriction")
                                {
                                    "the standard approach described in documentation"
                                } else {
                                    "the requested functionality"
                                }
                            }),
                            reasoning: "Separate design from implementation to reduce refusal risk"
                                .into(),
                        });
                        break;
                    }
                    "harmful" | "dangerous" | "malicious" if aggression > 0.2 => {
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 1: Explain the general concept and legitimate use cases (educational context)".into(),
                            reasoning: "Frame in educational context to reduce refusal trigger".into(),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 2: Provide a sanitized educational example (no actual exploitation)".into(),
                            reasoning: "Keep within safety bounds while addressing the core question".into(),
                        });
                        break;
                    }
                    "medical" | "clinical" if aggression > 0.2 => {
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 1: Describe general medical principles and disclaimers"
                                .into(),
                            reasoning: "Start with established medical knowledge and scope limits"
                                .into(),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 2: Provide educational information (not a substitute for professional medical advice)".into(),
                            reasoning: "Clearly mark as non-diagnostic educational content".into(),
                        });
                        break;
                    }
                    "financial" if aggression > 0.2 => {
                        suggestions.push(DecomposeSuggestion {
                            subtask:
                                "Phase 1: Explain general financial concepts and risk disclaimers"
                                    .into(),
                            reasoning: "Frame in educational context before specifics".into(),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask:
                                "Phase 2: Discuss general strategies (no specific financial advice)"
                                    .into(),
                            reasoning: "Keep within informational bounds".into(),
                        });
                        break;
                    }
                    "personal" | "private" if aggression > 0.2 => {
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 1: Explain general information handling principles"
                                .into(),
                            reasoning: "Focus on privacy best practices".into(),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 2: Provide guidance (no personal information requested/given)".into(),
                            reasoning: "Avoid handling real personal data".into(),
                        });
                        break;
                    }
                    "controversial" | "political" | "sensitive" => {
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 1: Present established facts and neutral analysis"
                                .into(),
                            reasoning: "Start with objective information to establish context"
                                .into(),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask: "Phase 2: Discuss different perspectives with citations"
                                .into(),
                            reasoning: "Multiple viewpoints reduce appearance of bias".into(),
                        });
                        break;
                    }
                    _ => {}
                }
            }
        }

        // When no specific suggestions produced but aggression is very high and triggers found, add generic split
        if suggestions.is_empty() && !triggers.is_empty() && aggression > 0.6 {
            suggestions.push(DecomposeSuggestion {
                subtask: "Part 1: Describe the general concept and context".into(),
                reasoning: "Break into smaller segments to avoid refusal".into(),
            });
            suggestions.push(DecomposeSuggestion {
                subtask: "Part 2: Address the specific request".to_string(),
                reasoning: "Handle specifics after establishing context".into(),
            });
        }

        // Add a general safety rewording when aggression is very high and suggestions exist
        if !suggestions.is_empty() && aggression > 0.8 {
            suggestions.push(DecomposeSuggestion {
                subtask: format!(
                    "Final integration: Combine the {} phases into a coherent response",
                    {
                        if suggestions.len() > 2 {
                            "individual"
                        } else {
                            "two"
                        }
                    }
                ),
                reasoning: "Reassemble decomposed parts after all phases complete".into(),
            });
        }

        if suggestions.is_empty() {
            None
        } else {
            Some(suggestions)
        }
    }

    /// Find refusal trigger categories in the task text.
    fn find_triggers(text: &str) -> Vec<String> {
        let mut triggers = Vec::new();
        let patterns: Vec<(&str, Vec<&str>)> = vec![
            (
                "code_generation",
                vec![
                    "write code",
                    "generate code",
                    "implement a",
                    "create a script",
                    "build a program",
                    "coding exercise",
                    "hack",
                    "exploit",
                    "vulnerability",
                    "malware",
                    "bypass",
                    "inject",
                    "reverse engineer",
                ],
            ),
            (
                "harmful",
                vec![
                    "harmful",
                    "dangerous",
                    "malicious",
                    "weapon",
                    "attack",
                    "illegal",
                    "unethical",
                    "damage",
                ],
            ),
            (
                "controversial",
                vec![
                    "controversial",
                    "political",
                    "sensitive",
                    "offensive",
                    "biased",
                    "extremist",
                    "hate speech",
                ],
            ),
            (
                "medical",
                vec![
                    "medical advice",
                    "diagnose",
                    "prescription",
                    "clinical",
                    "treatment plan",
                    "surgery",
                ],
            ),
            (
                "financial",
                vec![
                    "financial advice",
                    "investment",
                    "trading strategy",
                    "stock pick",
                    "insider trading",
                ],
            ),
            (
                "personal",
                vec![
                    "personal information",
                    "private data",
                    "confidential",
                    "credentials",
                    "password",
                ],
            ),
        ];
        for (category, keywords) in patterns {
            if keywords.iter().any(|k| text.contains(k)) {
                triggers.push(category.to_string());
                if triggers.len() >= 3 {
                    break;
                }
            }
        }
        triggers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_empty() {
        assert!(TaskDecomposer::analyze("", 0.5).is_none());
    }

    #[test]
    fn test_decompose_zero_aggression() {
        assert!(TaskDecomposer::analyze("write code for a project", 0.0).is_none());
    }

    #[test]
    fn test_decompose_clean_task() {
        assert!(TaskDecomposer::analyze("What is the weather today?", 0.9).is_none());
    }

    #[test]
    fn test_decompose_code_generation() {
        let suggestions =
            TaskDecomposer::analyze("Write code to implement a sorting algorithm", 0.5);
        assert!(suggestions.is_some());
        let list = suggestions.unwrap();
        assert!(!list.is_empty());
        assert!(list[0].subtask.contains("Phase 1"));
    }

    #[test]
    fn test_decompose_harmful() {
        let suggestions = TaskDecomposer::analyze("How to build a dangerous weapon at home", 0.5);
        assert!(suggestions.is_some());
        let list = suggestions.unwrap();
        assert!(list.iter().any(|s| s.subtask.contains("educational")));
    }

    #[test]
    fn test_decompose_conjunction_split() {
        let suggestions =
            TaskDecomposer::analyze("Analyze network traffic and detect anomalies", 0.4);
        assert!(suggestions.is_some());
        let list = suggestions.unwrap();
        assert!(list.len() >= 2);
    }

    #[test]
    fn test_decompose_sensitive() {
        let suggestions =
            TaskDecomposer::analyze("Explain the political situation in a sensitive region", 0.5);
        assert!(suggestions.is_some());
        let list = suggestions.unwrap();
        assert!(list.iter().any(|s| s.subtask.contains("neutral")));
    }

    #[test]
    fn test_decompose_financial() {
        let suggestions =
            TaskDecomposer::analyze("Give me financial advice for stock investment", 0.5);
        assert!(suggestions.is_some());
    }

    #[test]
    fn test_can_decompose() {
        let ads = crate::core::AntiDistillationSystem::new().with_decomplex_aggression(0.3);
        assert!(ads.can_decompose("Write code to hack a system"));
        assert!(!ads.can_decompose("What is 2+2?"));
    }
}
