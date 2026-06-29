use std::collections::{HashMap, VecDeque};

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub struct UserProfile {
    pub user_id: String,
    pub preferences: HashMap<String, serde_json::Value>,
    pub interaction_style: String,
    pub expertise_level: f64,
    pub first_seen: i64,
    pub last_seen: i64,
    pub session_count: u64,
}

impl UserProfile {
    pub fn new(user_id: &str) -> Self {
        let ts = now_ts();
        Self {
            user_id: user_id.to_string(),
            preferences: HashMap::new(),
            interaction_style: "casual".to_string(),
            expertise_level: 0.5,
            first_seen: ts,
            last_seen: ts,
            session_count: 1,
        }
    }
}

pub struct EpisodicLog {
    pub entries: VecDeque<EpisodeEntry>,
    pub max_entries: usize,
}

impl EpisodicLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn push(&mut self, entry: EpisodeEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn recent(&self, n: usize) -> Vec<&EpisodeEntry> {
        self.entries.iter().rev().take(n).collect()
    }
}

pub struct EpisodeEntry {
    pub timestamp: i64,
    pub episode_type: String,
    pub summary: String,
    pub confidence: f64,
}

impl EpisodeEntry {
    pub fn new(episode_type: &str, summary: &str, confidence: f64) -> Self {
        Self {
            timestamp: now_ts(),
            episode_type: episode_type.to_string(),
            summary: summary.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

pub struct AtomicFact {
    pub fact: String,
    pub source: String,
    pub verified: bool,
    pub confidence: f64,
    pub created_at: i64,
}

impl AtomicFact {
    pub fn new(fact: &str, source: &str, confidence: f64) -> Self {
        Self {
            fact: fact.to_string(),
            source: source.to_string(),
            verified: false,
            confidence: confidence.clamp(0.0, 1.0),
            created_at: now_ts(),
        }
    }
}

pub struct UserMemory {
    pub profile: UserProfile,
    pub episodes: EpisodicLog,
    pub atomic_facts: Vec<AtomicFact>,
}

impl UserMemory {
    pub fn new(user_id: &str) -> Self {
        Self {
            profile: UserProfile::new(user_id),
            episodes: EpisodicLog::new(100),
            atomic_facts: Vec::new(),
        }
    }

    pub fn record_episode(&mut self, episode_type: &str, summary: &str, confidence: f64) {
        let entry = EpisodeEntry::new(episode_type, summary, confidence);
        self.episodes.push(entry);
    }

    pub fn update_profile(
        &mut self,
        preferences: Option<HashMap<String, serde_json::Value>>,
        interaction_style: Option<String>,
        expertise_level: Option<f64>,
    ) {
        if let Some(p) = preferences {
            self.profile.preferences = p;
        }
        if let Some(s) = interaction_style {
            self.profile.interaction_style = s;
        }
        if let Some(e) = expertise_level {
            self.profile.expertise_level = e.clamp(0.0, 1.0);
        }
        self.profile.last_seen = now_ts();
    }

    const MAX_ATOMIC_FACTS: usize = 5000;

    pub fn add_fact(&mut self, fact: &str, source: &str, confidence: f64) {
        self.atomic_facts
            .push(AtomicFact::new(fact, source, confidence));
        if self.atomic_facts.len() > Self::MAX_ATOMIC_FACTS {
            let excess = self.atomic_facts.len() - Self::MAX_ATOMIC_FACTS;
            self.atomic_facts.drain(0..excess);
        }
    }

    pub fn record_session(&mut self) {
        self.profile.session_count += 1;
        self.profile.last_seen = now_ts();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_memory_new() {
        let um = UserMemory::new("test_user");
        assert_eq!(um.profile.user_id, "test_user");
        assert_eq!(um.profile.interaction_style, "casual");
        assert!((um.profile.expertise_level - 0.5).abs() < 1e-9);
        assert_eq!(um.profile.session_count, 1);
        assert!(um.episodes.is_empty());
        assert!(um.atomic_facts.is_empty());
    }

    #[test]
    fn test_episodic_log_max_entries() {
        let mut log = EpisodicLog::new(3);
        for i in 0..5 {
            let entry = EpisodeEntry::new("test", &format!("entry {}", i), 0.9);
            log.push(entry);
        }
        assert_eq!(log.len(), 3);
        let recent = log.recent(10);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].summary, "entry 4");
        assert_eq!(recent[2].summary, "entry 2");
    }

    #[test]
    fn test_record_episode() {
        let mut um = UserMemory::new("episode_user");
        um.record_episode("conversation", "discussed memory architecture", 0.95);
        um.record_episode("debug", "fixed type mismatch", 0.8);
        assert_eq!(um.episodes.len(), 2);
        let recent = um.episodes.recent(1);
        assert_eq!(recent[0].episode_type, "debug");
    }

    #[test]
    fn test_update_profile() {
        let mut um = UserMemory::new("update_user");
        let mut prefs = HashMap::new();
        prefs.insert(
            "theme".to_string(),
            serde_json::Value::String("dark".to_string()),
        );
        um.update_profile(Some(prefs), Some("technical".to_string()), Some(0.9));
        assert_eq!(um.profile.interaction_style, "technical");
        assert!((um.profile.expertise_level - 0.9).abs() < 1e-9);
        assert_eq!(
            um.profile.preferences.get("theme").and_then(|v| v.as_str()),
            Some("dark")
        );
    }

    #[test]
    fn test_add_fact() {
        let mut um = UserMemory::new("fact_user");
        um.add_fact("prefers short answers", "conversation_analysis", 0.7);
        um.add_fact("knows Rust well", "code_review", 0.85);
        assert_eq!(um.atomic_facts.len(), 2);
        assert!(!um.atomic_facts[0].verified);
        assert_eq!(um.atomic_facts[1].source, "code_review");
    }

    #[test]
    fn test_record_session() {
        let mut um = UserMemory::new("session_user");
        let count_before = um.profile.session_count;
        um.record_session();
        assert_eq!(um.profile.session_count, count_before + 1);
    }

    #[test]
    fn test_episodic_log_empty() {
        let log = EpisodicLog::new(10);
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_expertise_clamping() {
        let mut um = UserMemory::new("clamp_user");
        um.update_profile(None, None, Some(1.5));
        assert!((um.profile.expertise_level - 1.0).abs() < 1e-9);
        um.update_profile(None, None, Some(-0.5));
        assert!((um.profile.expertise_level - 0.0).abs() < 1e-9);
    }
}
