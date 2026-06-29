use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::nt_core_hcube::hippocampal_trace::HippocampalMemory;
use crate::core::nt_core_hcube::sm2_scheduler::SM2Scheduler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceSegment {
    pub id: String,
    pub state: String,
    pub action: String,
    pub observation: String,
    pub reward: f64,
    pub task_type: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceTrace {
    pub id: String,
    pub segments: Vec<ExperienceSegment>,
    pub total_reward: f64,
    pub task_id: String,
    pub source: String,
}

pub struct ExperiencePool {
    traces: HashMap<String, ExperienceTrace>,
    max_traces: usize,
    task_index: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperiencePoolStats {
    pub total_traces: usize,
    pub total_segments: usize,
    pub avg_reward: f64,
    pub max_reward: f64,
    pub unique_task_types: usize,
}

impl ExperiencePool {
    pub fn new(max_traces: usize) -> Self {
        ExperiencePool {
            traces: HashMap::new(),
            max_traces,
            task_index: HashMap::new(),
        }
    }

    pub fn store(&mut self, trace: ExperienceTrace) {
        let id = trace.id.clone();
        for seg in &trace.segments {
            self.task_index
                .entry(seg.task_type.clone())
                .or_default()
                .push(id.clone());
        }
        self.traces.insert(id.clone(), trace);
        self.enforce_max();
    }

    pub fn retrieve(&self, task_type: &str, k: usize) -> Vec<&ExperienceTrace> {
        self.task_index
            .get(task_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.traces.get(id))
                    .take(k)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn retrieve_high_value(&self, min_reward: f64, k: usize) -> Vec<&ExperienceTrace> {
        let mut high: Vec<&ExperienceTrace> = self
            .traces
            .values()
            .filter(|t| t.total_reward > min_reward)
            .collect();
        high.sort_by(|a, b| {
            b.total_reward
                .partial_cmp(&a.total_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        high.truncate(k);
        high
    }

    pub fn strip_actions(&self, trace_id: &str) -> Vec<String> {
        self.traces
            .get(trace_id)
            .map(|trace| trace.segments.iter().map(|s| s.action.clone()).collect())
            .unwrap_or_default()
    }

    pub fn trace_similarity(a: &ExperienceTrace, b: &ExperienceTrace) -> f64 {
        let actions_a: Vec<&str> = a.segments.iter().map(|s| s.action.as_str()).collect();
        let actions_b: Vec<&str> = b.segments.iter().map(|s| s.action.as_str()).collect();

        let set_a: std::collections::HashSet<&str> = actions_a.iter().cloned().collect();
        let set_b: std::collections::HashSet<&str> = actions_b.iter().cloned().collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }

    pub fn len(&self) -> usize {
        self.traces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.traces.is_empty()
    }

    pub fn prune(&mut self, min_reward: f64) -> usize {
        let before = self.traces.len();
        self.traces.retain(|_, t| t.total_reward >= min_reward);
        self.rebuild_index();
        before - self.traces.len()
    }

    pub fn consolidate_to_hippocampus(&self, memory: &mut HippocampalMemory) {
        if self.traces.is_empty() {
            return;
        }
        for trace in self.traces.values() {
            for seg in &trace.segments {
                let seg_bytes: Vec<u8> =
                    format!("{}|{}|{}", seg.state, seg.action, seg.observation)
                        .bytes()
                        .collect();
                let key_bytes: Vec<u8> = seg.task_type.bytes().collect();
                let strength = (seg.reward.abs() / 10.0).clamp(0.1, 1.0);
                let tags = vec![
                    trace.source.clone(),
                    seg.task_type.clone(),
                    format!("reward_{:.1}", seg.reward),
                ];
                memory.store(seg_bytes, key_bytes, strength, tags);
            }
        }
    }

    pub fn schedule_sm2_reviews(&self, scheduler: &mut SM2Scheduler) {
        if self.traces.is_empty() {
            return;
        }
        for trace in self.traces.values() {
            let memory_id = format!("exp_{}", trace.id);
            let already = scheduler
                .all_items()
                .iter()
                .any(|it| it.memory_id == memory_id);
            if !already {
                let mut sig = Vec::new();
                for seg in &trace.segments {
                    let s = format!("{}|{}|{}", seg.state, seg.action, seg.observation);
                    sig.extend_from_slice(s.as_bytes());
                }
                scheduler.add_item(&memory_id, sig);
            }
        }
    }

    pub fn stats(&self) -> ExperiencePoolStats {
        let total_traces = self.traces.len();
        let total_segments: usize = self.traces.values().map(|t| t.segments.len()).sum();
        let rewards: Vec<f64> = self.traces.values().map(|t| t.total_reward).collect();
        let avg_reward = if rewards.is_empty() {
            0.0
        } else {
            rewards.iter().sum::<f64>() / rewards.len() as f64
        };
        let max_reward = rewards.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let unique_task_types = self.task_index.len();

        ExperiencePoolStats {
            total_traces,
            total_segments,
            avg_reward,
            max_reward,
            unique_task_types,
        }
    }

    fn enforce_max(&mut self) {
        while self.traces.len() > self.max_traces {
            let oldest_id = self
                .traces
                .iter()
                .min_by_key(|(_, t)| t.segments.first().map(|s| s.timestamp).unwrap_or(i64::MAX))
                .map(|(id, _)| id.clone());

            if let Some(id) = oldest_id {
                self.traces.remove(&id);
            } else {
                break;
            }
        }
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        self.task_index.clear();
        for trace in self.traces.values() {
            for seg in &trace.segments {
                self.task_index
                    .entry(seg.task_type.clone())
                    .or_default()
                    .push(trace.id.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segment(
        id: &str,
        state: &str,
        action: &str,
        observation: &str,
        reward: f64,
        task_type: &str,
        timestamp: i64,
    ) -> ExperienceSegment {
        ExperienceSegment {
            id: id.to_string(),
            state: state.to_string(),
            action: action.to_string(),
            observation: observation.to_string(),
            reward,
            task_type: task_type.to_string(),
            timestamp,
        }
    }

    fn make_trace(
        id: &str,
        task_id: &str,
        source: &str,
        total_reward: f64,
        segments: Vec<ExperienceSegment>,
    ) -> ExperienceTrace {
        ExperienceTrace {
            id: id.to_string(),
            segments,
            total_reward,
            task_id: task_id.to_string(),
            source: source.to_string(),
        }
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut pool = ExperiencePool::new(100);
        let seg = make_segment(
            "s1",
            "initial",
            "move_left",
            "moved",
            0.5,
            "navigation",
            1000,
        );
        let trace = make_trace("t1", "task1", "exploration", 2.0, vec![seg]);
        pool.store(trace);

        let results = pool.retrieve("navigation", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "t1");
    }

    #[test]
    fn test_retrieve_high_value() {
        let mut pool = ExperiencePool::new(100);
        let seg_low = make_segment("s1", "a", "idle", "nothing", 0.1, "mining", 1000);
        let seg_high = make_segment("s2", "b", "dig", "found_ore", 1.0, "mining", 1001);
        let trace_low = make_trace("t1", "task1", "exploration", 0.5, vec![seg_low]);
        let trace_high = make_trace("t2", "task2", "exploration", 5.0, vec![seg_high]);
        pool.store(trace_low);
        pool.store(trace_high);

        let results = pool.retrieve_high_value(1.0, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "t2");
    }

    #[test]
    fn test_strip_actions() {
        let mut pool = ExperiencePool::new(100);
        let segs = vec![
            make_segment("s1", "start", "move_up", "up", 0.5, "explore", 1000),
            make_segment("s2", "mid", "move_right", "right", 0.5, "explore", 1001),
            make_segment("s3", "end", "grab", "got_item", 1.0, "explore", 1002),
        ];
        let trace = make_trace("t1", "task1", "exploration", 2.0, segs);
        pool.store(trace);

        let actions = pool.strip_actions("t1");
        assert_eq!(actions, vec!["move_up", "move_right", "grab"]);
    }

    #[test]
    fn test_trace_similarity_identical() {
        let seg = make_segment("s1", "a", "jump", "flew", 0.5, "physics", 1000);
        let a = make_trace("t1", "task1", "exploration", 1.0, vec![seg.clone()]);
        let b = make_trace("t2", "task2", "exploration", 1.0, vec![seg]);

        let sim = ExperiencePool::trace_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_prune_low_value() {
        let mut pool = ExperiencePool::new(100);
        let seg_low = make_segment("s1", "a", "wait", "nothing", 0.1, "idle", 1000);
        let seg_high = make_segment("s2", "b", "process", "done", 0.8, "work", 1001);
        let trace_low = make_trace("t1", "task1", "exploration", 0.3, vec![seg_low]);
        let trace_high = make_trace("t2", "task2", "execution", 3.0, vec![seg_high]);
        pool.store(trace_low);
        pool.store(trace_high);

        let pruned = pool.prune(1.0);
        assert_eq!(pruned, 1);
        assert_eq!(pool.len(), 1);
        assert!(pool.traces.contains_key("t2"));
    }

    #[test]
    fn test_stats() {
        let mut pool = ExperiencePool::new(100);
        let seg_a = make_segment("s1", "x", "scan", "found", 0.6, "survey", 1000);
        let seg_b = make_segment("s2", "y", "drill", "hole", 0.9, "survey", 1001);
        let trace_a = make_trace("t1", "task1", "exploration", 4.0, vec![seg_a]);
        let trace_b = make_trace("t2", "task2", "exploration", 6.0, vec![seg_b]);
        pool.store(trace_a);
        pool.store(trace_b);

        let stats = pool.stats();
        assert_eq!(stats.total_traces, 2);
        assert_eq!(stats.total_segments, 2);
        assert!((stats.avg_reward - 5.0).abs() < 1e-6);
        assert!((stats.max_reward - 6.0).abs() < 1e-6);
        assert_eq!(stats.unique_task_types, 1);
    }

    #[test]
    fn test_max_traces_enforced() {
        let mut pool = ExperiencePool::new(3);
        for i in 0..5 {
            let seg = make_segment(
                &format!("s{}", i),
                "st",
                "act",
                "obs",
                0.5,
                "test",
                1000 + i,
            );
            let trace = make_trace(&format!("t{}", i), "task", "exploration", 1.0, vec![seg]);
            pool.store(trace);
        }
        assert!(pool.len() <= 3);
    }
}
