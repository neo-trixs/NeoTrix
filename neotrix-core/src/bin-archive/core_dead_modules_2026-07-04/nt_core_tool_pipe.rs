//! B127 — Tool training data pipeline.
//!
//! Records tool call traces and builds training examples for future
//! DPO-style tool-use training (ToolLLM style).
//!
//! Architecture:
//!   ToolCallTrace: single tool call record
//!   ToolEpisode: a sequence of tool calls for one task
//!   ToolPipe: persistent store with dedup + sampling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Single tool call record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallTrace {
    pub tool_name: String,
    pub arguments: String,
    pub result: String,
    pub success: bool,
    pub latency_ms: u64,
    pub token_cost: u32,
    pub timestamp: u64,
}

/// A complete tool-use episode for one task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEpisode {
    pub task: String,
    pub task_type: String,
    pub calls: Vec<ToolCallTrace>,
    pub task_success: bool,
    pub total_cost: u32,
    pub total_latency_ms: u64,
    pub timestamp: u64,
}

/// Training example extracted from an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTrainingExample {
    pub task: String,
    pub task_type: String,
    pub chosen_tool: String,
    pub chosen_args: String,
    pub rejected_tool: String,
    pub rejected_args: String,
    pub preference_score: f64,
}

/// Tool training data pipeline.
#[derive(Debug, Clone)]
pub struct ToolPipe {
    /// All recorded episodes
    episodes: Vec<ToolEpisode>,
    /// Extracted training examples
    examples: Vec<ToolTrainingExample>,
    /// Max episodes to retain
    max_episodes: usize,
    /// Dedup set: (task, tool_name) → count
    dedup: HashMap<(String, String), u32>,
}

impl Default for ToolPipe {
    fn default() -> Self {
        Self::new(500)
    }
}

impl ToolPipe {
    pub fn new(max_episodes: usize) -> Self {
        Self {
            episodes: Vec::with_capacity(max_episodes),
            examples: Vec::with_capacity(max_episodes * 2),
            max_episodes,
            dedup: HashMap::new(),
        }
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }

    /// Record a single tool call trace.
    pub fn record_call(&self, tool_name: &str, arguments: &str, result: &str, success: bool, latency_ms: u64, token_cost: u32) -> ToolCallTrace {
        ToolCallTrace {
            tool_name: tool_name.to_string(),
            arguments: arguments.to_string(),
            result: result.to_string(),
            success,
            latency_ms,
            token_cost,
            timestamp: Self::now(),
        }
    }

    /// Finish and store an episode from collected traces.
    pub fn finish_episode(&mut self, task: &str, task_type: &str, calls: Vec<ToolCallTrace>, task_success: bool) {
        if calls.is_empty() {
            return;
        }

        let total_cost: u32 = calls.iter().map(|c| c.token_cost).sum();
        let total_latency: u64 = calls.iter().map(|c| c.latency_ms).sum();

        let episode = ToolEpisode {
            task: task.to_string(),
            task_type: task_type.to_string(),
            calls,
            task_success,
            total_cost,
            total_latency_ms: total_latency,
            timestamp: Self::now(),
        };

        // Dedup: skip if same (task, tool_name) seen >3 times
        let dedup_key = (task.to_string(), task_type.to_string());
        let count = self.dedup.entry(dedup_key).or_insert(0);
        if *count > 3 {
            return;
        }
        *count += 1;

        if self.episodes.len() >= self.max_episodes {
            self.episodes.remove(0);
        }
        self.episodes.push(episode);

        // Auto-extract training examples
        self.extract_examples();
    }

    /// Extract DPO-style training examples from episodes.
    /// For each episode with mixed success/failure calls, create a preference pair.
    fn extract_examples(&mut self) {
        for episode in &self.episodes {
            let successes: Vec<&ToolCallTrace> = episode.calls.iter().filter(|c| c.success).collect();
            let failures: Vec<&ToolCallTrace> = episode.calls.iter().filter(|c| !c.success).collect();

            if successes.is_empty() || failures.is_empty() {
                continue;
            }

            for good in &successes {
                for bad in &failures {
                    // Only pair if same tool (preference is about arguments/context)
                    if good.tool_name == bad.tool_name {
                        continue;
                    }
                    let example = ToolTrainingExample {
                        task: episode.task.clone(),
                        task_type: episode.task_type.clone(),
                        chosen_tool: good.tool_name.clone(),
                        chosen_args: good.arguments.clone(),
                        rejected_tool: bad.tool_name.clone(),
                        rejected_args: bad.arguments.clone(),
                        preference_score: if episode.task_success { 1.0 } else { -1.0 },
                    };
                    // Avoid duplicates
                    let is_dup = self.examples.iter().any(|e|
                        e.task == example.task && e.chosen_tool == example.chosen_tool
                    );
                    if !is_dup {
                        self.examples.push(example);
                    }
                }
            }
        }
    }

    /// Sample training examples for DPO training.
    pub fn sample_examples(&self, n: usize) -> Vec<&ToolTrainingExample> {
        if self.examples.is_empty() {
            return vec![];
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n = n.min(self.examples.len());
        let mut indices: Vec<usize> = (0..self.examples.len()).collect();
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let idx = rng.gen_range(i..self.examples.len());
            indices.swap(i, idx);
            result.push(&self.examples[indices[i]]);
        }
        result
    }

    /// Get stats summary.
    pub fn stats(&self) -> ToolPipeStats {
        let total_tokens: u32 = self.episodes.iter().map(|e| e.total_cost).sum();
        let total_latency: u64 = self.episodes.iter().map(|e| e.total_latency_ms).sum();
        let success_rate = if self.episodes.is_empty() {
            0.0
        } else {
            self.episodes.iter().filter(|e| e.task_success).count() as f64 / self.episodes.len() as f64
        };
        ToolPipeStats {
            episodes: self.episodes.len(),
            examples: self.examples.len(),
            total_tokens,
            total_latency_ms: total_latency,
            success_rate,
        }
    }

    pub fn episode_count(&self) -> usize { self.episodes.len() }
    pub fn example_count(&self) -> usize { self.examples.len() }
}

#[derive(Debug, Clone)]
pub struct ToolPipeStats {
    pub episodes: usize,
    pub examples: usize,
    pub total_tokens: u32,
    pub total_latency_ms: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_call(tool: &str, success: bool) -> ToolCallTrace {
        ToolCallTrace {
            tool_name: tool.to_string(),
            arguments: "{}".into(),
            result: if success { "ok".into() } else { "error".into() },
            success,
            latency_ms: 100,
            token_cost: 50,
            timestamp: 0,
        }
    }

    #[test]
    fn test_finish_episode_stores_episode() {
        let mut pipe = ToolPipe::new(100);
        let calls = vec![make_call("search", true), make_call("compute", true)];
        pipe.finish_episode("math task", "math", calls, true);
        assert_eq!(pipe.episode_count(), 1);
    }

    #[test]
    fn test_empty_episode_ignored() {
        let mut pipe = ToolPipe::new(100);
        pipe.finish_episode("test", "general", vec![], true);
        assert_eq!(pipe.episode_count(), 0);
    }

    #[test]
    fn test_extract_examples_creates_preference() {
        let mut pipe = ToolPipe::new(100);
        let calls = vec![
            make_call("search", true),
            make_call("compute", false),
        ];
        pipe.finish_episode("task", "general", calls, true);
        assert!(pipe.example_count() >= 1, "should extract preference from mixed episode");
    }

    #[test]
    fn test_sample_examples() {
        let mut pipe = ToolPipe::new(100);
        for i in 0..5 {
            let calls = vec![
                make_call(&format!("tool_a_{}", i), true),
                make_call(&format!("tool_b_{}", i), false),
            ];
            pipe.finish_episode(&format!("task_{}", i), "general", calls, true);
        }
        let sampled = pipe.sample_examples(3);
        assert!(!sampled.is_empty());
        assert!(sampled.len() <= 3);
    }

    #[test]
    fn test_stats() {
        let mut pipe = ToolPipe::new(100);
        let calls = vec![make_call("search", true)];
        pipe.finish_episode("task", "general", calls, true);
        let stats = pipe.stats();
        assert_eq!(stats.episodes, 1);
        assert!(stats.success_rate > 0.0);
    }

    #[test]
    fn test_dedup_limits_repeats() {
        let mut pipe = ToolPipe::new(100);
        for _ in 0..10 {
            let calls = vec![make_call("search", true)];
            pipe.finish_episode("same_task", "general", calls, true);
        }
        assert!(pipe.episode_count() < 10, "dedup should limit repeats, got {}", pipe.episode_count());
    }
}
