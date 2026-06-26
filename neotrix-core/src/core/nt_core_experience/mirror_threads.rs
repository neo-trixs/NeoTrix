// REVIVED Evo 4
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ThreadType {
    Goals,
    Reasoning,
    Memory,
}

impl ThreadType {
    pub fn label(&self) -> &'static str {
        match self {
            ThreadType::Goals => "Goals",
            ThreadType::Reasoning => "Reasoning",
            ThreadType::Memory => "Memory",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CognitiveThread {
    pub id: u64,
    pub thread_type: ThreadType,
    pub content: String,
    pub confidence: f64,
    pub created_at: u64,
    pub last_activated: u64,
    pub activation_count: u64,
}

#[derive(Debug, Clone)]
pub struct ThreadManagerStats {
    pub total_threads: usize,
    pub goals_count: usize,
    pub reasoning_count: usize,
    pub memory_count: usize,
    pub avg_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ThreadManager {
    threads: Vec<CognitiveThread>,
    max_threads: usize,
    next_id: u64,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            threads: Vec::new(),
            max_threads: 6,
            next_id: 0,
        }
    }

    pub fn with_max_threads(mut self, max: usize) -> Self {
        self.max_threads = max;
        self
    }

    pub fn spawn_thread(&mut self, thread_type: ThreadType, content: &str, cycle: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        if self.threads.len() >= self.max_threads {
            let oldest_idx = self
                .threads
                .iter()
                .enumerate()
                .min_by_key(|(_, t)| t.last_activated)
                .map(|(i, _)| i);
            if let Some(idx) = oldest_idx {
                self.threads.remove(idx);
            }
        }

        self.threads.push(CognitiveThread {
            id,
            thread_type,
            content: content.to_string(),
            confidence: 0.5,
            created_at: cycle,
            last_activated: cycle,
            activation_count: 1,
        });
        id
    }

    pub fn update_thread(&mut self, id: u64, content: &str, confidence: f64, cycle: u64) -> bool {
        if let Some(thread) = self.threads.iter_mut().find(|t| t.id == id) {
            thread.content = content.to_string();
            thread.confidence = confidence.clamp(0.0, 1.0);
            thread.last_activated = cycle;
            thread.activation_count += 1;
            true
        } else {
            false
        }
    }

    pub fn synthesize_narrative(&self, cycle: u64) -> String {
        if self.threads.is_empty() {
            return format!("[cycle {}] no active threads", cycle);
        }
        let mut parts: Vec<String> = self
            .threads
            .iter()
            .map(|t| format!("[{}] {}", t.thread_type.label(), t.content))
            .collect();
        parts.insert(0, format!("[cycle {}] parallel synthesis:", cycle));
        parts.join("\n")
    }

    pub fn thread_count(&self, thread_type: ThreadType) -> usize {
        self.threads
            .iter()
            .filter(|t| t.thread_type == thread_type)
            .count()
    }

    pub fn stats(&self) -> ThreadManagerStats {
        let total = self.threads.len();
        let goals = self.thread_count(ThreadType::Goals);
        let reasoning = self.thread_count(ThreadType::Reasoning);
        let memory = self.thread_count(ThreadType::Memory);
        let avg_conf = if total == 0 {
            0.0
        } else {
            self.threads.iter().map(|t| t.confidence).sum::<f64>() / total as f64
        };
        ThreadManagerStats {
            total_threads: total,
            goals_count: goals,
            reasoning_count: reasoning,
            memory_count: memory,
            avg_confidence: avg_conf,
        }
    }

    pub fn prune_stale(&mut self, max_age: u64, current_cycle: u64) {
        self.threads
            .retain(|t| current_cycle.saturating_sub(t.last_activated) <= max_age);
    }
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_manager_new() {
        let tm = ThreadManager::new();
        assert_eq!(tm.threads.len(), 0);
        assert_eq!(tm.max_threads, 6);
    }

    #[test]
    fn test_spawn_thread() {
        let mut tm = ThreadManager::new();
        let id1 = tm.spawn_thread(ThreadType::Goals, "find resources", 1);
        let id2 = tm.spawn_thread(ThreadType::Reasoning, "analyze patterns", 1);
        let id3 = tm.spawn_thread(ThreadType::Memory, "recall prior session", 1);
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(tm.threads.len(), 3);
        assert!(tm
            .threads
            .iter()
            .any(|t| t.thread_type == ThreadType::Memory));
    }

    #[test]
    fn test_synthesize_narrative() {
        let mut tm = ThreadManager::new();
        tm.spawn_thread(ThreadType::Goals, "explore", 1);
        tm.spawn_thread(ThreadType::Reasoning, "deduce", 1);
        let narrative = tm.synthesize_narrative(1);
        assert!(narrative.contains("Goals"));
        assert!(narrative.contains("Reasoning"));
        assert!(narrative.contains("explore"));
        assert!(narrative.contains("deduce"));
    }

    #[test]
    fn test_update_thread() {
        let mut tm = ThreadManager::new();
        let id = tm.spawn_thread(ThreadType::Goals, "initial", 1);
        assert!(tm.update_thread(id, "updated", 0.9, 2));
        let t = tm.threads.iter().find(|t| t.id == id).unwrap();
        assert_eq!(t.content, "updated");
        assert!((t.confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_update_nonexistent() {
        let mut tm = ThreadManager::new();
        assert!(!tm.update_thread(999, "nope", 0.5, 1));
    }

    #[test]
    fn test_prune_stale() {
        let mut tm = ThreadManager::new();
        tm.spawn_thread(ThreadType::Goals, "old", 1);
        tm.spawn_thread(ThreadType::Goals, "new", 10);
        tm.prune_stale(5, 10);
        assert_eq!(tm.threads.len(), 1);
        assert_eq!(tm.threads[0].content, "new");
    }

    #[test]
    fn test_stats() {
        let mut tm = ThreadManager::new();
        tm.spawn_thread(ThreadType::Goals, "g", 1);
        tm.spawn_thread(ThreadType::Reasoning, "r", 1);
        tm.spawn_thread(ThreadType::Memory, "m", 1);
        let s = tm.stats();
        assert_eq!(s.total_threads, 3);
        assert_eq!(s.goals_count, 1);
        assert_eq!(s.reasoning_count, 1);
        assert_eq!(s.memory_count, 1);
    }

    #[test]
    fn test_max_threads_eviction() {
        let mut tm = ThreadManager::with_max_threads(ThreadManager::new(), 2);
        tm.spawn_thread(ThreadType::Goals, "a", 1);
        tm.spawn_thread(ThreadType::Goals, "b", 2);
        tm.spawn_thread(ThreadType::Goals, "c", 3);
        assert_eq!(tm.threads.len(), 2);
    }

    #[test]
    fn test_synthesize_narrative_empty() {
        let tm = ThreadManager::new();
        let n = tm.synthesize_narrative(5);
        assert!(n.contains("no active threads"));
    }

    #[test]
    fn test_confidence_clamp() {
        let mut tm = ThreadManager::new();
        let id = tm.spawn_thread(ThreadType::Goals, "test", 1);
        tm.update_thread(id, "test", 1.5, 2);
        assert!((tm.threads[0].confidence - 1.0).abs() < 0.01);
        tm.update_thread(id, "test", -0.5, 3);
        assert!((tm.threads[0].confidence - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_default() {
        let tm: ThreadManager = Default::default();
        assert_eq!(tm.threads.len(), 0);
    }
}
