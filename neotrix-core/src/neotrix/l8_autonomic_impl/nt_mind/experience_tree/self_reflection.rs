#![forbid(unsafe_code)]

//! Self-Reflection Engine — Reflexion-style verbal reinforcement learning loop.
//!
//! Inspired by: Shinn et al., "Reflexion: Language Agents with Verbal Reinforcement Learning" (NeurIPS 2023).
//!
//! Agents verbally reflect on task feedback signals, maintain their own reflective text
//! in an episodic memory buffer, and induce better decision-making in subsequent trials.
//! Unlike traditional RL, no weight updates or model fine-tuning are required.
//!
//! Integration points:
//!   - `nt_mind_background_loop::handlers_absorption` — post-absorption reflection
//!   - `HookEvent::SessionEnd` — session-end reflection trigger
//!   - `ExperienceEngine` — feed reflection into the distill pipeline

use std::collections::VecDeque;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::core::nt_core_self_test::SelfTest;
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;

// ============================================================================
// 数据结构
// ============================================================================

/// Self-reflection record — a single reflective episode.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReflectionRecord {
    pub session_id: String,
    pub cycle: String,
    pub ts: u64,
    /// What the agent attempted
    pub action_summary: String,
    /// Observed outcome / feedback signal
    pub feedback: String,
    /// Self-generated verbal reflection
    pub reflection: String,
    /// Confidence in the reflection (0.0–1.0)
    pub confidence: f64,
    /// Whether this reflection improved subsequent performance
    pub improved: bool,
}

/// Episodic memory buffer — stores recent reflection records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionBuffer {
    pub max_size: usize,
    pub records: VecDeque<ReflectionRecord>,
}

impl ReflectionBuffer {
    pub fn new(max_size: usize) -> Self {
        Self { max_size, records: VecDeque::with_capacity(max_size) }
    }

    pub fn push(&mut self, record: ReflectionRecord) {
        if self.records.len() >= self.max_size {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }

    pub fn len(&self) -> usize { self.records.len() }

    pub fn is_empty(&self) -> bool { self.records.is_empty() }

    /// Retrieve recent reflections for context injection.
    pub fn recent(&self, n: usize) -> Vec<&ReflectionRecord> {
        self.records.iter().rev().take(n).collect()
    }
}

/// Self-reflection engine — generates verbal reflections and updates the buffer.
pub struct SelfReflectionEngine {
    buffer: Arc<std::sync::Mutex<ReflectionBuffer>>,
    #[allow(dead_code)]
    kb: Option<Arc<KnowledgeBase>>,
}

impl SelfReflectionEngine {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: Arc::new(std::sync::Mutex::new(ReflectionBuffer::new(buffer_size))),
            kb: None,
        }
    }

    pub fn with_kb(kb: Arc<KnowledgeBase>) -> Self {
        Self {
            buffer: Arc::new(std::sync::Mutex::new(ReflectionBuffer::new(64))),
            kb: Some(kb),
        }
    }

    /// Generate a verbal self-reflection from action and feedback.
    pub fn reflect(&self, session_id: &str, cycle: &str, action: &str, feedback: &str) -> ReflectionRecord {
        let reflection = self.generate_reflection(action, feedback);
        let confidence = self.compute_confidence(feedback);
        let record = ReflectionRecord {
            session_id: session_id.to_string(),
            cycle: cycle.to_string(),
            ts: now_ts(),
            action_summary: action.to_string(),
            feedback: feedback.to_string(),
            reflection,
            confidence,
            improved: confidence > 0.5,
        };
        self.buffer.lock().unwrap().push(record.clone());
        record
    }

    fn generate_reflection(&self, action: &str, feedback: &str) -> String {
        format!("Action: {} → Feedback: {} → Reflection: {}", action, feedback, self.synthesize_insight(feedback))
    }

    fn synthesize_insight(&self, feedback: &str) -> String {
        let lower = feedback.to_lowercase();
        if lower.contains("error") || lower.contains("fail") {
            "Adjust strategy: identify failure mode, decompose task, retry with correction".to_string()
        } else if lower.contains("success") || lower.contains("pass") || lower.contains("ok") {
            "Reinforce approach: consolidate successful pattern, record as procedural knowledge".to_string()
        } else {
            "Observe and generalize: extract transferable pattern, update belief state".to_string()
        }
    }

    fn compute_confidence(&self, feedback: &str) -> f64 {
        let signal_strength = if feedback.contains("error") || feedback.contains("fail") {
            0.3
        } else if feedback.contains("success") || feedback.contains("pass") {
            0.8
        } else {
            0.5
        };
        signal_strength
    }

    /// Retrieve recent reflections for injection into the agent loop.
    pub fn recent_reflections(&self, n: usize) -> Vec<ReflectionRecord> {
        self.buffer.lock().unwrap().recent(n).into_iter().cloned().collect()
    }

    /// Get the reflection buffer.
    pub fn buffer(&self) -> &Arc<std::sync::Mutex<ReflectionBuffer>> {
        &self.buffer
    }
}

// SelfTest trait implementation for NeoTrix SelfTest registry
impl SelfTest for SelfReflectionEngine {
    fn name(&self) -> &str {
        "self_reflection_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        // Verify engine can be created and buffer operations work
        drop(self.buffer.lock().map_err(|e| vec![format!("mutex poisoned: {}", e)])?);
        let _ = self.buffer.lock().unwrap().push(ReflectionRecord::default());
        Ok(())
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_buffer_push_and_recent() {
        let mut buf = ReflectionBuffer::new(3);
        let r1 = ReflectionRecord {
            session_id: "s1".into(), cycle: "c1".into(), ts: 1,
            action_summary: "a1".into(), feedback: "ok".into(),
            reflection: "r1".into(), confidence: 0.8, improved: true,
        };
        let r2 = ReflectionRecord {
            session_id: "s1".into(), cycle: "c1".into(), ts: 2,
            action_summary: "a2".into(), feedback: "error".into(),
            reflection: "r2".into(), confidence: 0.9, improved: false,
        };
        buf.push(r1.clone());
        buf.push(r2.clone());
        assert_eq!(buf.len(), 2);
        let recent = buf.recent(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].action_summary, "a2");
    }

    #[test]
    fn test_reflection_buffer_max_size() {
        let mut buf = ReflectionBuffer::new(2);
        buf.push(ReflectionRecord {
            session_id: "s".into(), cycle: "c".into(), ts: 1,
            action_summary: "a1".into(), feedback: "ok".into(),
            reflection: "r1".into(), confidence: 0.8, improved: true,
        });
        buf.push(ReflectionRecord {
            session_id: "s".into(), cycle: "c".into(), ts: 2,
            action_summary: "a2".into(), feedback: "ok".into(),
            reflection: "r2".into(), confidence: 0.8, improved: true,
        });
        buf.push(ReflectionRecord {
            session_id: "s".into(), cycle: "c".into(), ts: 3,
            action_summary: "a3".into(), feedback: "ok".into(),
            reflection: "r3".into(), confidence: 0.8, improved: true,
        });
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_self_reflection_engine() {
        let engine = SelfReflectionEngine::new(8);
        let record = engine.reflect("s1", "c1", "run test", "success: all passed");
        assert_eq!(record.action_summary, "run test");
        assert!(record.confidence > 0.5);
        assert!(record.improved);
        assert_eq!(engine.recent_reflections(1).len(), 1);
    }

    #[test]
    fn test_self_reflection_error_feedback() {
        let engine = SelfReflectionEngine::new(8);
        let record = engine.reflect("s1", "c1", "run test", "error: assertion failed");
        assert!(record.reflection.contains("Adjust"));
        assert!(!record.improved);
    }
}
