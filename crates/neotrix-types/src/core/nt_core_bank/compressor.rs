use crate::core::layered_memory::{LayeredMemory, MemoryLayer, MemoryEntry};

/// Compression strategy for different context types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompactStrategy {
    /// Truncate tool descriptions to N chars
    TruncateToolDescs(usize),
    /// Remove entries below importance threshold
    DropLowImportance(f64),
    /// Keep only top N entries by importance
    KeepTop(usize),
    /// No compression
    None,
}

/// Compresses context before LLM calls to stay within token budgets.
pub struct ContextCompressor {
    pub tool_desc_max_chars: usize,
    pub memory_importance_threshold: f64,
    pub max_history_entries: usize,
}

impl Default for ContextCompressor {
    fn default() -> Self {
        Self {
            tool_desc_max_chars: 50,
            memory_importance_threshold: 0.2,
            max_history_entries: 20,
        }
    }
}

impl ContextCompressor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compress tool descriptions — truncate each to max chars.
    pub fn compress_tool_descs(&self, descriptions: &mut Vec<String>) -> usize {
        let mut compressed = 0;
        for desc in descriptions.iter_mut() {
            if desc.len() > self.tool_desc_max_chars {
                desc.truncate(self.tool_desc_max_chars);
                desc.push_str("...");
                compressed += 1;
            }
        }
        compressed
    }

    /// Compact memory layers — remove entries below importance threshold.
    pub fn compress_memory(&self, memory: &mut LayeredMemory) -> usize {
        let mut removed = 0;
        for layer in &[MemoryLayer::L2Session, MemoryLayer::L3Knowledge, MemoryLayer::L4Archive] {
            let entries: Vec<MemoryEntry> = memory.query_layer(layer).into_iter().cloned().collect();
            let keep: Vec<MemoryEntry> = entries.into_iter()
                .filter(|e| e.importance >= self.memory_importance_threshold)
                .collect();
            removed += memory.clear_layer(layer) - keep.len();
            for entry in keep {
                memory.store(entry);
            }
        }
        removed
    }

    /// Truncate message history to keep only the most recent entries.
    pub fn truncate_history<T>(&self, history: &mut Vec<T>) -> usize
    where
        T: Clone,
    {
        if history.len() > self.max_history_entries {
            let excess = history.len() - self.max_history_entries;
            *history = history.split_off(history.len() - self.max_history_entries);
            excess
        } else {
            0
        }
    }

    /// Run all compression strategies.
    pub fn compress_all(
        &self,
        tool_descs: &mut Vec<String>,
        memory: &mut LayeredMemory,
        history: &mut Vec<String>,
    ) -> CompressionReport {
        let tool_compressed = self.compress_tool_descs(tool_descs);
        let memory_removed = self.compress_memory(memory);
        let history_truncated = self.truncate_history(history);
        CompressionReport {
            tool_descs_compressed: tool_compressed,
            memory_entries_removed: memory_removed,
            history_entries_truncated: history_truncated,
        }
    }
}

/// Report of what was compressed.
#[derive(Debug, Clone)]
pub struct CompressionReport {
    pub tool_descs_compressed: usize,
    pub memory_entries_removed: usize,
    pub history_entries_truncated: usize,
}

impl CompressionReport {
    pub fn total_savings(&self) -> usize {
        self.tool_descs_compressed + self.memory_entries_removed + self.history_entries_truncated
    }

    pub fn is_empty(&self) -> bool {
        self.total_savings() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::layered_memory::*;

    #[test]
    fn test_compress_tool_descs() {
        let compressor = ContextCompressor::new();
        let mut descs = vec![
            "A very long tool description that exceeds the maximum character limit".to_string(),
            "short".to_string(),
        ];
        let count = compressor.compress_tool_descs(&mut descs);
        assert_eq!(count, 1);
        assert!(descs[0].ends_with("..."));
        assert_eq!(descs[0].len(), 53);
        assert_eq!(descs[1], "short");
    }

    #[test]
    fn test_truncate_history() {
        let compressor = ContextCompressor::new();
        let mut history: Vec<String> = (0..100).map(|i| format!("entry {}", i)).collect();
        let removed = compressor.truncate_history(&mut history);
        assert_eq!(removed, 80);
        assert_eq!(history.len(), compressor.max_history_entries);
        assert_eq!(history[0], "entry 80");
    }

    #[test]
    fn test_compress_memory() {
        let compressor = ContextCompressor::new();
        let mut memory = LayeredMemory::new();

        memory.store(MemoryEntry::new("Important L2", MemoryLayer::L2Session, 0.9));
        memory.store(MemoryEntry::new("Trivial L2", MemoryLayer::L2Session, 0.05));
        memory.store(MemoryEntry::new("Important L3", MemoryLayer::L3Knowledge, 0.8));
        memory.store(MemoryEntry::new("Trivial L3", MemoryLayer::L3Knowledge, 0.1));

        let removed = compressor.compress_memory(&mut memory);
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_compress_all() {
        let compressor = ContextCompressor::new();
        let mut memory = LayeredMemory::new();
        memory.store(MemoryEntry::new("test", MemoryLayer::L2Session, 0.1));

        let mut descs = vec!["long description here that should be truncated".to_string()];
        let mut history: Vec<String> = (0..50).map(|i| format!("msg {}", i)).collect();

        let report = compressor.compress_all(&mut descs, &mut memory, &mut history);
        assert!(report.total_savings() > 0);
        assert!(!report.is_empty());
    }

    #[test]
    fn test_empty_report() {
        let compressor = ContextCompressor::new();
        let mut memory = LayeredMemory::new();
        let mut descs = vec!["short".to_string()];
        let mut history = vec!["only".to_string()];

        let report = compressor.compress_all(&mut descs, &mut memory, &mut history);
        assert!(report.is_empty());
    }

    #[test]
    fn test_compressor_defaults() {
        let compressor = ContextCompressor::default();
        assert_eq!(compressor.tool_desc_max_chars, 50);
        assert_eq!(compressor.memory_importance_threshold, 0.2);
        assert_eq!(compressor.max_history_entries, 20);
    }

    #[test]
    fn test_custom_strategy() {
        let compressor = ContextCompressor {
            tool_desc_max_chars: 10,
            memory_importance_threshold: 0.5,
            max_history_entries: 5,
        };
        let mut descs = vec!["1234567890extra".to_string()];
        compressor.compress_tool_descs(&mut descs);
        assert_eq!(descs[0].len(), 13);
    }
}
