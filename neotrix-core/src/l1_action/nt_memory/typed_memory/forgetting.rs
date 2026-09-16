#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::estate::MemoryEstate;
use super::entry::TypedMemoryEntry;

/// Exponential decay forgetting policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayPolicy {
    pub decay_rate: f64,
    pub half_life_seconds: f64,
}

impl DecayPolicy {
    pub fn new(decay_rate: f64) -> Self { Self { decay_rate, half_life_seconds: if decay_rate > 0.0 { 0.6931471805599453 / decay_rate } else { f64::MAX } } }
    pub fn default() -> Self { Self::new(0.0001) }
    pub fn compute_decay(&self, entry: &TypedMemoryEntry) -> f64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let age = (now - entry.timestamp) as f64;
        if self.decay_rate <= 0.0 || age <= 0.0 { 1.0 } else { (-self.decay_rate * age).exp() }
    }
}

/// Retention policy based on access patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub min_access_threshold: u64,
    pub max_age_without_access_seconds: i64,
    pub access_weight: f64,
}

impl RetentionPolicy {
    pub fn new(min_access_threshold: u64, max_age_without_access_seconds: i64) -> Self { Self { min_access_threshold, max_age_without_access_seconds, access_weight: 0.5 } }
    pub fn default() -> Self { Self::new(5, 2_592_000) }
    pub fn should_forget(&self, entry: &TypedMemoryEntry) -> bool {
        if entry.access_count >= self.min_access_threshold { return false; }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        now - entry.timestamp > self.max_age_without_access_seconds
    }
    pub fn retention_score(&self, entry: &TypedMemoryEntry) -> f64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let age = (now - entry.timestamp) as f64;
        let access_factor = (entry.access_count as f64 * self.access_weight).min(1.0);
        let age_factor = (age / 2_592_000.0).min(1.0);
        access_factor * (1.0 - age_factor)
    }
}

/// Compression policy: summarize old memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionPolicy {
    pub compress_age_seconds: i64,
    pub target_compression_ratio: f64,
    pub summary_method: CompressionMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMethod { Extractive, Abstractive, KeywordOnly }

impl CompressionPolicy {
    pub fn new(compress_age_seconds: i64) -> Self { Self { compress_age_seconds, target_compression_ratio: 0.5, summary_method: CompressionMethod::Extractive } }
    pub fn default() -> Self { Self::new(2_592_000) }
    pub fn should_compress(&self, entry: &TypedMemoryEntry) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        (now - entry.timestamp) > self.compress_age_seconds
    }
    pub fn compress(&self, entry: &TypedMemoryEntry) -> CompressedMemory {
        let words: Vec<&str> = entry.content.split_whitespace().collect();
        let target_len = (words.len() as f64 * self.target_compression_ratio) as usize;
        let summary = words.into_iter().take(target_len.max(10)).collect::<Vec<_>>().join(" ");
        let ratio = if entry.content.len() > 0 { summary.len() as f64 / entry.content.len() as f64 } else { 0.0 };
        CompressedMemory { original_id: entry.id.clone(), summary, original_estate: entry.estate, compression_ratio: ratio, compressed_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedMemory { pub original_id: String, pub summary: String, pub original_estate: MemoryEstate, pub compression_ratio: f64, pub compressed_at: i64 }

/// Forget action types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForgetAction { Decayed, Forgiven, Compressed, Retained }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgetEvent { pub entry_id: String, pub action: ForgetAction, pub timestamp: i64, pub reason: String }

/// Policy-driven forgetting system.
#[derive(Debug)]
pub struct PolicyDrivenForgetting {
    decay_policy: DecayPolicy,
    retention_policy: RetentionPolicy,
    compression_policy: CompressionPolicy,
    forget_log: Vec<ForgetEvent>,
}

impl PolicyDrivenForgetting {
    pub fn new() -> Self { Self { decay_policy: DecayPolicy::default(), retention_policy: RetentionPolicy::default(), compression_policy: CompressionPolicy::default(), forget_log: Vec::new() } }
    pub fn with_decay(mut self, decay_rate: f64) -> Self { self.decay_policy = DecayPolicy::new(decay_rate); self }
    pub fn with_retention(mut self, min_access: u64, max_age: i64) -> Self { self.retention_policy = RetentionPolicy::new(min_access, max_age); self }
    pub fn with_compression(mut self, age_seconds: i64) -> Self { self.compression_policy = CompressionPolicy::new(age_seconds); self }

    pub fn apply_decay(&mut self, entry: &mut TypedMemoryEntry) -> ForgetAction {
        let factor = self.decay_policy.compute_decay(entry);
        let old = entry.confidence; entry.decay_confidence(factor);
        let action = if entry.confidence < 0.1 { ForgetAction::Decayed } else { ForgetAction::Retained };
        self.forget_log.push(ForgetEvent { entry_id: entry.id.clone(), action: action.clone(), timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64, reason: format!("Decayed from {:.3} to {:.3}", old, entry.confidence) });
        action
    }

    pub fn apply_retention(&mut self, entry: &TypedMemoryEntry) -> ForgetAction {
        if self.retention_policy.should_forget(entry) {
            self.forget_log.push(ForgetEvent { entry_id: entry.id.clone(), action: ForgetAction::Forgiven, timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64, reason: "Low access + old".to_string() });
            ForgetAction::Forgiven
        } else { ForgetAction::Retained }
    }

    pub fn apply_compression(&mut self, entry: &TypedMemoryEntry) -> ForgetAction {
        if self.compression_policy.should_compress(entry) {
            let compressed = self.compression_policy.compress(entry);
            self.forget_log.push(ForgetEvent { entry_id: compressed.original_id.clone(), action: ForgetAction::Compressed, timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64, reason: format!("Compressed ratio={:.3}", compressed.compression_ratio) });
            ForgetAction::Compressed
        } else { ForgetAction::Retained }
    }

    pub fn apply_all(&mut self, entry: &mut TypedMemoryEntry) -> Vec<ForgetAction> {
        let decay = self.apply_decay(entry);
        let retention = self.apply_retention(entry);
        let compression = self.apply_compression(entry);
        vec![decay, retention, compression]
    }
    pub fn forget_log(&self) -> &[ForgetEvent] { &self.forget_log }
}

impl Default for PolicyDrivenForgetting { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn make_entry(estate: MemoryEstate, age_seconds: i64) -> TypedMemoryEntry {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let mut e = TypedMemoryEntry::new(format!("test-{}", estate.as_str()), estate, format!("content for {}", estate.as_str()), 0.8);
        e.timestamp = now - age_seconds; e
    }
    #[test] fn test_decay_policy() { let p = DecayPolicy::new(0.1); let mut e = make_entry(MemoryEstate::Semantic, 1000); let f = p.compute_decay(&e); assert!(f < 1.0); e.decay_confidence(f); assert!(e.confidence < 0.8); }
    #[test] fn test_retention_should_forget() { let p = RetentionPolicy::new(5, 1000); let e = make_entry(MemoryEstate::Episodic, 2000); assert!(p.should_forget(&e)); }
    #[test] fn test_compression() { let p = CompressionPolicy::new(1000); let e = make_entry(MemoryEstate::Procedural, 2000); assert!(p.should_compress(&e)); let c = p.compress(&e); assert!(c.compression_ratio < 1.0); }
    #[test] fn test_forgotting_apply_all() { let mut f = PolicyDrivenForgetting::new(); let mut e = make_entry(MemoryEstate::Episodic, 5000000); let actions = f.apply_all(&mut e); assert_eq!(actions.len(), 3); assert!(!f.forget_log().is_empty()); }
}
