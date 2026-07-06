use std::collections::{HashMap, VecDeque};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::nt_memory_unify::{kv_get, kv_set};

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

    pub fn add_fact(&mut self, fact: &str, source: &str, confidence: f64) {
        self.atomic_facts.push(AtomicFact::new(fact, source, confidence));
    }

    pub fn record_session(&mut self) {
        self.profile.session_count += 1;
        self.profile.last_seen = now_ts();
    }

    pub fn save(&self, conn: &Connection) -> Result<(), String> {
        let ns = format!("user_memory.{}", self.profile.user_id);
        let profile_json = serde_json::to_string(&self.profile).map_err(|e| format!("serde: {}", e))?;
        kv_set(conn, &ns, "profile", &profile_json)?;

        let episodes_json = serde_json::to_string(
            &self.episodes.entries.iter().collect::<Vec<&EpisodeEntry>>()
        ).map_err(|e| format!("serde: {}", e))?;
        kv_set(conn, &ns, "episodes", &episodes_json)?;

        let facts_json = serde_json::to_string(&self.atomic_facts).map_err(|e| format!("serde: {}", e))?;
        kv_set(conn, &ns, "facts", &facts_json)?;

        Ok(())
    }

    pub fn load(conn: &Connection, user_id: &str) -> Option<Self> {
        let ns = format!("user_memory.{}", user_id);

        let profile_json = kv_get(conn, &ns, "profile").ok()??;
        let profile: UserProfile = serde_json::from_str(&profile_json).ok()?;

        let mut episodes = EpisodicLog::new(100);
        if let Some(ep_json) = kv_get(conn, &ns, "episodes").ok()? {
            if let Ok(entries) = serde_json::from_str::<Vec<EpisodeEntry>>(&ep_json) {
                for e in entries {
                    episodes.push(e);
                }
            }
        }

        let atomic_facts = if let Some(f_json) = kv_get(conn, &ns, "facts").ok()? {
            serde_json::from_str::<Vec<AtomicFact>>(&f_json).unwrap_or_default()
        } else {
            Vec::new()
        };

        Some(Self { profile, episodes, atomic_facts })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema;

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
        prefs.insert("theme".to_string(), serde_json::Value::String("dark".to_string()));
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

    #[test]
    fn test_save_load_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();

        let mut um = UserMemory::new("roundtrip_user");
        um.update_profile(None, Some("technical".into()), Some(0.85));
        um.record_episode("conversation", "discussed memory", 0.9);
        um.add_fact("prefers concise answers", "analysis", 0.75);
        um.record_session();
        assert!(um.save(&conn).is_ok());

        let loaded = UserMemory::load(&conn, "roundtrip_user").unwrap();
        assert_eq!(loaded.profile.user_id, "roundtrip_user");
        assert_eq!(loaded.profile.interaction_style, "technical");
        assert_eq!(loaded.episodes.len(), 1);
        assert_eq!(loaded.atomic_facts.len(), 1);
        assert_eq!(loaded.profile.session_count, 2);
    }

    #[test]
    fn test_load_nonexistent() {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        assert!(UserMemory::load(&conn, "no_such_user").is_none());
    }
}
