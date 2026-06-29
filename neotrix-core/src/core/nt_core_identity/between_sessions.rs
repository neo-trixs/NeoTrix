#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReflectionLayer {
    Event,
    Meaning,
    Identity,
}

#[derive(Debug, Clone)]
pub struct ReflectionEntry {
    pub layer: ReflectionLayer,
    pub content: String,
    pub timestamp: u64,
    pub salience: f64,
}

#[derive(Debug, Clone)]
pub struct SessionMemory {
    pub entries: Vec<ReflectionEntry>,
    pub session_id: u64,
    pub started_at: u64,
}

impl SessionMemory {
    pub fn new(session_id: u64) -> Self {
        Self {
            entries: vec![],
            session_id,
            started_at: 0,
        }
    }
    pub fn record(&mut self, layer: ReflectionLayer, content: &str, salience: f64) {
        self.entries.push(ReflectionEntry {
            layer,
            content: content.into(),
            timestamp: 0,
            salience: salience.clamp(0.0, 1.0),
        });
    }
    pub fn distill_principles(&self) -> Vec<String> {
        let mut p: Vec<String> = self
            .entries
            .iter()
            .filter(|e| e.layer == ReflectionLayer::Identity && e.salience > 0.7)
            .map(|e| e.content.clone())
            .collect();
        p.sort();
        p.dedup();
        p
    }
}

#[derive(Debug, Clone)]
pub struct BetweenSessionsReflector {
    pub session_memories: Vec<SessionMemory>,
    pub derived_principles: Vec<String>,
    pub max_principles: usize,
}

impl BetweenSessionsReflector {
    pub fn new() -> Self {
        Self {
            session_memories: vec![],
            derived_principles: vec![],
            max_principles: 20,
        }
    }
    pub fn end_session(&mut self, memory: SessionMemory) {
        for p in memory.distill_principles() {
            if !self.derived_principles.contains(&p)
                && self.derived_principles.len() < self.max_principles
            {
                self.derived_principles.push(p);
            }
        }
        self.session_memories.push(memory);
    }
    pub fn idle_reflect(&mut self) {
        let mut p: Vec<String> = self
            .session_memories
            .iter()
            .flat_map(|m| m.distill_principles())
            .collect();
        p.sort();
        p.dedup();
        p.truncate(self.max_principles);
        self.derived_principles = p;
    }
    pub fn principles(&self) -> &[String] {
        &self.derived_principles
    }
}

impl Default for BetweenSessionsReflector {
    fn default() -> Self {
        Self::new()
    }
}
