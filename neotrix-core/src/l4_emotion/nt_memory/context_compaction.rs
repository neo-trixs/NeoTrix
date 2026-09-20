#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCompactor { max_tokens: u32, threshold: f64 }

impl ContextCompactor {
    pub fn new(max: u32) -> Self { Self { max_tokens: max, threshold: 0.8 } }
    pub fn needs_compaction(&self, tokens: u32) -> bool { tokens as f64 > self.max_tokens as f64 * self.threshold }
    pub fn compact(&self, msgs: Vec<String>) -> (Vec<String>, usize) {
        if !self.needs_compaction(msgs.len() as u32 * 100) { return (msgs, 0); }
        let keep = msgs.len() / 2;
        (msgs.into_iter().skip(msgs.len() - keep).collect(), msgs.len() - keep)
    }
}
