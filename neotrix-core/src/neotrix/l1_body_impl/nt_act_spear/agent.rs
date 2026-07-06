use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::prompt_registry::PromptRegistry;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Evaluating,
    Executing,
    Prompting,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub score: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonResult {
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    pub accepted: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishResult {
    pub accepted: bool,
    pub trajectory_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryEntry {
    pub tool: String,
    pub input: String,
    pub output: String,
    pub timestamp: i64,
    pub state: AgentState,
}

pub struct SpearAgent {
    pub state: AgentState,
    pub registry: Option<PromptRegistry>,
    pub trajectory: Vec<TrajectoryEntry>,
    pub iteration_count: u64,
}

const EVALUATION_KEYWORDS: &[&str] = &[
    "analysis", "reasoning", "evaluate", "optimize", "iterate",
    "code", "test", "verify", "design", "implement",
    "refactor", "debug", "benchmark", "metric", "score",
];

impl SpearAgent {
    pub fn new(registry: Option<PromptRegistry>) -> Self {
        Self {
            state: AgentState::Idle,
            registry,
            trajectory: Vec::new(),
            iteration_count: 0,
        }
    }

    pub fn transition(from: &AgentState, to: &AgentState) -> Result<(), String> {
        match (from, to) {
            (AgentState::Idle, AgentState::Evaluating)
            | (AgentState::Idle, AgentState::Finished)
            | (AgentState::Evaluating, AgentState::Executing)
            | (AgentState::Evaluating, AgentState::Prompting)
            | (AgentState::Evaluating, AgentState::Finished)
            | (AgentState::Executing, AgentState::Evaluating)
            | (AgentState::Executing, AgentState::Finished)
            | (AgentState::Prompting, AgentState::Evaluating)
            | (AgentState::Prompting, AgentState::Finished) => Ok(()),
            (AgentState::Finished, _) => {
                Err("Cannot transition from Finished: terminal state".to_string())
            }
            (f, t) => Err(format!("Invalid transition from {:?} to {:?}", f, t)),
        }
    }

    pub fn evaluate(&mut self, text: &str) -> EvaluationResult {
        let allowed = matches!(
            self.state,
            AgentState::Idle | AgentState::Executing | AgentState::Prompting
        );
        if !allowed {
            return EvaluationResult {
                score: 0.0,
                reasoning: format!("Cannot evaluate in state {:?}", self.state),
            };
        }
        if let Err(e) = Self::transition(&self.state, &AgentState::Evaluating) {
            return EvaluationResult {
                score: 0.0,
                reasoning: e,
            };
        }

        let lower = text.to_lowercase();
        let matches = EVALUATION_KEYWORDS
            .iter()
            .filter(|kw| lower.contains(*kw))
            .count();
        let score = (matches as f64 / EVALUATION_KEYWORDS.len() as f64 * 2.0).min(1.0);

        let reasoning = format!(
            "Scored {:.4}: found {}/{} key terms in text ({} chars)",
            score,
            matches,
            EVALUATION_KEYWORDS.len(),
            text.len(),
        );

        self.state = AgentState::Evaluating;
        self.iteration_count += 1;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.trajectory.push(TrajectoryEntry {
            tool: "evaluate".to_string(),
            input: text.to_string(),
            output: format!("score={:.4}, reasoning={}", score, reasoning),
            timestamp: now,
            state: AgentState::Evaluating,
        });

        EvaluationResult { score, reasoning }
    }

    pub fn execute_python(&mut self, code: &str) -> PythonResult {
        if self.state != AgentState::Evaluating {
            return PythonResult {
                output: String::new(),
                error: Some(format!(
                    "Cannot execute python in state {:?} (must be Evaluating)",
                    self.state
                )),
            };
        }

        let mut brace = 0i64;
        let mut paren = 0i64;
        let mut bracket = 0i64;
        for c in code.chars() {
            match c {
                '{' => brace += 1,
                '}' => brace -= 1,
                '(' => paren += 1,
                ')' => paren -= 1,
                '[' => bracket += 1,
                ']' => bracket -= 1,
                _ => {}
            }
        }

        if brace != 0 || paren != 0 || bracket != 0 {
            let err = format!(
                "Syntax error: unbalanced delimiters — braces:{}, parens:{}, brackets:{}",
                brace, paren, bracket
            );
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            self.trajectory.push(TrajectoryEntry {
                tool: "python".to_string(),
                input: code.to_string(),
                output: err.clone(),
                timestamp: now,
                state: AgentState::Executing,
            });
            return PythonResult {
                output: String::new(),
                error: Some(err),
            };
        }

        let line_count = code.lines().count();
        let output = format!(
            "[nt_act_spear-python: simulated]\nCode ({} chars, {} lines) syntax OK\n>>> Result: None",
            code.len(),
            line_count,
        );

        self.state = AgentState::Executing;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.trajectory.push(TrajectoryEntry {
            tool: "python".to_string(),
            input: code.to_string(),
            output: output.clone(),
            timestamp: now,
            state: AgentState::Executing,
        });

        PythonResult {
            output,
            error: None,
        }
    }

    pub fn set_prompt_instruction(&mut self, instruction: &str, priority: u8) -> PromptResult {
        if self.state != AgentState::Evaluating {
            return PromptResult {
                accepted: false,
                version: String::new(),
            };
        }

        let clamped = priority.min(10);
        let name = format!("nt_act_spear-p{}", clamped);

        let version = if let Some(ref mut reg) = self.registry {
            let entry = reg.register(&name, instruction, 0.0);
            format!("v{}", entry.version)
        } else {
            "v0-noregistry".to_string()
        };

        self.state = AgentState::Prompting;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.trajectory.push(TrajectoryEntry {
            tool: "set_prompt".to_string(),
            input: format!("[priority={}] {}", clamped, instruction),
            output: format!("accepted=true, version={}", version),
            timestamp: now,
            state: AgentState::Prompting,
        });

        PromptResult {
            accepted: true,
            version,
        }
    }

    pub fn finish(&mut self, score: f64, summary: &str) -> FinishResult {
        if self.state == AgentState::Finished {
            return FinishResult {
                accepted: false,
                trajectory_id: String::new(),
            };
        }

        let clamped = score.clamp(0.0, 1.0);
        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let trajectory_id = format!("nt_act_spear-{:020x}", ns);

        self.state = AgentState::Finished;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.trajectory.push(TrajectoryEntry {
            tool: "finish".to_string(),
            input: format!("score={:.4}, summary={}", clamped, summary),
            output: format!("trajectory_id={}", trajectory_id),
            timestamp: now,
            state: AgentState::Finished,
        });

        FinishResult {
            accepted: true,
            trajectory_id,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.state == AgentState::Finished
    }

    pub fn trajectory_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "SPEAR Agent | steps:{} state:{:?} iter:{}\n",
            self.trajectory.len(),
            self.state,
            self.iteration_count
        ));
        for (i, e) in self.trajectory.iter().enumerate() {
            out.push_str(&format!(
                "  [{:02}] {} -> {:?} | {} chars\n",
                i + 1,
                e.tool,
                e.state,
                e.input.len(),
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn now_ns() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    }

    #[test]
    fn test_idle_to_evaluating() {
        let mut agent = SpearAgent::new(None);
        assert_eq!(agent.state, AgentState::Idle);
        let result = agent.evaluate("test analysis code");
        assert!(result.score > 0.0);
        assert_eq!(agent.state, AgentState::Evaluating);
    }

    #[test]
    fn test_full_evaluate_python_set_prompt_cycle() {
        let reg = PromptRegistry::new(10);
        let mut agent = SpearAgent::new(Some(reg));

        assert_eq!(agent.state, AgentState::Idle);

        agent.evaluate("code analysis and design");
        assert_eq!(agent.state, AgentState::Evaluating);

        let pr = agent.execute_python("{x:1}");
        assert!(pr.error.is_none());
        assert_eq!(agent.state, AgentState::Executing);

        agent.evaluate("second evaluation");
        assert_eq!(agent.state, AgentState::Evaluating);

        let sr = agent.set_prompt_instruction("optimize for accuracy", 5);
        assert!(sr.accepted);
        assert!(sr.version.starts_with("v"));
        assert_eq!(agent.state, AgentState::Prompting);

        agent.evaluate("final check");
        assert_eq!(agent.state, AgentState::Evaluating);

        let fr = agent.finish(0.95, "all good");
        assert!(fr.accepted);
        assert!(fr.trajectory_id.starts_with("nt_act_spear-"));
        assert_eq!(agent.state, AgentState::Finished);
        assert!(agent.is_terminal());
    }

    #[test]
    fn test_invalid_transition_executing_to_prompting() {
        let mut agent = SpearAgent::new(None);
        assert_eq!(agent.state, AgentState::Idle);

        agent.evaluate("test");
        assert_eq!(agent.state, AgentState::Evaluating);

        agent.execute_python("{a:1}");
        assert_eq!(agent.state, AgentState::Executing);

        let sr = agent.set_prompt_instruction("try set prompt", 3);
        assert!(!sr.accepted);
        assert_eq!(agent.state, AgentState::Executing);
    }

    #[test]
    fn test_finish_is_terminal() {
        let mut agent = SpearAgent::new(None);
        agent.evaluate("analysis");
        let fr = agent.finish(0.8, "done");
        assert!(fr.accepted);
        assert!(agent.is_terminal());

        let fr2 = agent.finish(0.9, "again");
        assert!(!fr2.accepted);
        assert!(agent.is_terminal());

        let er = agent.evaluate("more");
        assert_eq!(er.score, 0.0);
        assert!(er.reasoning.contains("Finished"));
    }

    #[test]
    fn test_trajectory_summary() {
        let reg = PromptRegistry::new(10);
        let mut agent = SpearAgent::new(Some(reg));
        agent.evaluate("analysis code test design");
        agent.execute_python("x = 1\ny = 2");
        agent.evaluate("check results");
        agent.set_prompt_instruction("improve reasoning", 9);
        agent.finish(0.85, "completed");
        let summary = agent.trajectory_summary();
        assert!(summary.contains("steps:5"));
        assert!(summary.contains("evaluate"));
        assert!(summary.contains("python"));
        assert!(summary.contains("set_prompt"));
        assert!(summary.contains("finish"));
    }

    #[test]
    fn test_python_syntax_error() {
        let mut agent = SpearAgent::new(None);
        agent.evaluate("code test");
        let result = agent.execute_python("{x:1");
        assert!(result.error.is_some());
        assert!(result.error.expect("unexpected None/Err").contains("unbalanced"));
    }

    #[test]
    fn test_transition_table() {
        assert!(SpearAgent::transition(&AgentState::Idle, &AgentState::Evaluating).is_ok());
        assert!(SpearAgent::transition(&AgentState::Idle, &AgentState::Finished).is_ok());
        assert!(SpearAgent::transition(&AgentState::Evaluating, &AgentState::Executing).is_ok());
        assert!(SpearAgent::transition(&AgentState::Evaluating, &AgentState::Prompting).is_ok());
        assert!(SpearAgent::transition(&AgentState::Evaluating, &AgentState::Finished).is_ok());
        assert!(SpearAgent::transition(&AgentState::Executing, &AgentState::Evaluating).is_ok());
        assert!(SpearAgent::transition(&AgentState::Executing, &AgentState::Finished).is_ok());
        assert!(SpearAgent::transition(&AgentState::Prompting, &AgentState::Evaluating).is_ok());
        assert!(SpearAgent::transition(&AgentState::Prompting, &AgentState::Finished).is_ok());

        assert!(SpearAgent::transition(&AgentState::Finished, &AgentState::Idle).is_err());
        assert!(SpearAgent::transition(&AgentState::Finished, &AgentState::Evaluating).is_err());
        assert!(SpearAgent::transition(&AgentState::Idle, &AgentState::Executing).is_err());
        assert!(SpearAgent::transition(&AgentState::Idle, &AgentState::Prompting).is_err());
        assert!(SpearAgent::transition(&AgentState::Executing, &AgentState::Executing).is_err());
        assert!(SpearAgent::transition(&AgentState::Executing, &AgentState::Prompting).is_err());
        assert!(SpearAgent::transition(&AgentState::Prompting, &AgentState::Executing).is_err());
        assert!(SpearAgent::transition(&AgentState::Prompting, &AgentState::Prompting).is_err());
    }
}
