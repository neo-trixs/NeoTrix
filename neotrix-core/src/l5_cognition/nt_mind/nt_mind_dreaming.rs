use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DreamDreamMemoryEntry {
    pub id: String,
    pub content: Vec<u8>,
    pub importance: f64,
    pub access_count: u64,
    pub last_accessed: Option<std::time::Instant>,
}

#[derive(Debug, Clone)]
pub struct DreamConfig {
    pub importance_threshold: f64,
    pub forget_threshold: f64,
    pub consolidation_cooldown: Duration,
    pub max_memories: usize,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            importance_threshold: 0.7,
            forget_threshold: 0.3,
            consolidation_cooldown: Duration::from_secs(3600),
            max_memories: 10000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DreamEntry {
    pub promoted_count: usize,
    pub forgotten_count: usize,
    pub insights: Vec<String>,
}

pub struct DreamDiary {
    entries: Vec<DreamEntry>,
}

impl DreamDiary {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, entry: DreamEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[DreamEntry] {
        &self.entries
    }

    pub fn total_promoted(&self) -> usize {
        self.entries.iter().map(|e| e.promoted_count).sum()
    }

    pub fn total_forgotten(&self) -> usize {
        self.entries.iter().map(|e| e.forgotten_count).sum()
    }
}

impl Default for DreamDiary {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DreamReport {
    pub promoted_count: usize,
    pub forgotten_count: usize,
    pub remaining_count: usize,
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub trigger: String,
    pub action: String,
    pub outcome: String,
    pub confidence: f64,
}

pub struct DreamConsolidator {
    config: DreamConfig,
    diary: DreamDiary,
}

impl DreamConsolidator {
    pub fn new(config: DreamConfig) -> Self {
        Self {
            config,
            diary: DreamDiary::new(),
        }
    }

    pub fn consolidate(&mut self, memories: &mut Vec<DreamMemoryEntry>) -> DreamReport {
        let mut promoted = Vec::new();
        let mut forgotten = Vec::new();

        memories.retain(|memory| {
            if memory.importance > self.config.importance_threshold {
                promoted.push(memory.clone());
                true
            } else if memory.importance < self.config.forget_threshold {
                forgotten.push(memory.clone());
                false
            } else {
                true
            }
        });

        if memories.len() > self.config.max_memories {
            memories.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            memories.truncate(self.config.max_memories);
        }

        let remaining = memories.len();
        let insights = self.extract_insights(memories);

        self.diary.record(DreamEntry {
            promoted_count: promoted.len(),
            forgotten_count: forgotten.len(),
            insights,
        });

        DreamReport {
            promoted_count: promoted.len(),
            forgotten_count: forgotten.len(),
            remaining_count: remaining,
        }
    }

    pub fn promote(&self, memory: &mut DreamMemoryEntry) {
        memory.importance = (memory.importance + 0.1).min(1.0);
    }

    pub fn discard(&self, memory: &mut DreamMemoryEntry) {
        memory.importance = 0.0;
    }

    pub fn cool_down(&self, memory: &mut DreamMemoryEntry) {
        memory.importance = (memory.importance - 0.05).max(0.0);
    }

    pub fn distill_lessons(&self, _memories: &[DreamMemoryEntry]) -> Vec<Lesson> {
        Vec::new()
    }

    pub fn diary(&self) -> &DreamDiary {
        &self.diary
    }

    pub fn config(&self) -> &DreamConfig {
        &self.config
    }

    fn extract_insights(&self, _memories: &[DreamMemoryEntry]) -> Vec<String> {
        Vec::new()
    }
}

impl Default for DreamConsolidator {
    fn default() -> Self {
        Self::new(DreamConfig::default())
    }
}
