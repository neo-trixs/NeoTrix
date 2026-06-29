use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::QuantizedVSA;

const MAX_NARRATIVE_EVENTS: usize = 500;
const MAX_SESSION_IDS: usize = 100;
const IDENTITY_VSA_DIM: usize = 64;
const NARRATIVE_PATH: &str = ".neotrix/narrative.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEvent {
    pub session_id: String,
    pub timestamp: u64,
    pub summary: String,
    pub reward: f64,
    pub duration_ms: u64,
    pub key_insights: Vec<String>,
    pub vsa_fingerprint: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentity {
    pub identity_vsa: Vec<u8>,
    pub session_ids: Vec<String>,
    pub first_seen: u64,
    pub last_seen: u64,
    pub coherence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NarrativeMergeStrategy {
    Union,
    Intersect,
    ConfidenceWeighted,
    LatestWins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeConsolidationReport {
    pub recurring_themes: Vec<String>,
    pub identity_drift: f64,
    pub coherence_trend: Vec<f64>,
    pub total_sessions: u64,
    pub total_events: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedEpisode {
    pub cycle_range: (usize, usize),
    pub summary_text: String,
    pub key_events: Vec<String>,
    pub emotional_valence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMemory {
    pub topic: String,
    pub key_facts: Vec<String>,
    pub access_count: u32,
    pub last_accessed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalStatus {
    Active,
    Paused,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: f64,
    pub created_at: u64,
    pub status: GoalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub error_desc: String,
    pub correction: String,
    pub outcome: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    pub formality: f64,
    pub verbosity: f64,
    pub technical_depth: f64,
    pub domain_focus: Vec<String>,
}

impl Default for StyleProfile {
    fn default() -> Self {
        Self {
            formality: 0.5,
            verbosity: 0.5,
            technical_depth: 0.5,
            domain_focus: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub consistency: f64,
    pub session_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSelf {
    pub current_goals: Vec<String>,
    pub active_context: String,
    pub emotional_state: f64,
}

impl Default for WorkingSelf {
    fn default() -> Self {
        Self {
            current_goals: Vec::new(),
            active_context: String::new(),
            emotional_state: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NarrativeSelf {
    pub session_id: String,
    pub session_start: u64,
    pub session_end: Option<u64>,
    pub session_count: u64,
    #[serde(default)]
    pub session_ids: Arc<Vec<String>>,
    pub personal_narrative: Arc<VecDeque<NarrativeEvent>>,
    pub current_session_summary: String,
    pub current_session_reward: f64,
    pub current_session_start: u64,
    pub current_session_insights: Arc<Vec<String>>,
    pub total_reward: f64,
    pub situated_memory: VecDeque<StoredMemory>,
    pub active_goals: Vec<Goal>,
    pub correction_log: VecDeque<Correction>,
    pub style_profile: StyleProfile,
    pub role_history: VecDeque<Role>,
    pub working_self: WorkingSelf,
    pub consolidated_episodes: Vec<ConsolidatedEpisode>,
}

impl Clone for NarrativeSelf {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            session_start: self.session_start,
            session_end: self.session_end,
            session_count: self.session_count,
            session_ids: Arc::clone(&self.session_ids),
            personal_narrative: Arc::clone(&self.personal_narrative),
            current_session_summary: self.current_session_summary.clone(),
            current_session_reward: self.current_session_reward,
            current_session_start: self.current_session_start,
            current_session_insights: Arc::clone(&self.current_session_insights),
            total_reward: self.total_reward,
            situated_memory: self.situated_memory.clone(),
            active_goals: self.active_goals.clone(),
            correction_log: self.correction_log.clone(),
            style_profile: self.style_profile.clone(),
            role_history: self.role_history.clone(),
            working_self: self.working_self.clone(),
            consolidated_episodes: self.consolidated_episodes.clone(),
        }
    }
}

impl Default for NarrativeSelf {
    fn default() -> Self {
        Self::new()
    }
}

impl NarrativeSelf {
    pub fn new() -> Self {
        let session_id = format!("ses_{:x}", rand::random::<u64>());
        Self {
            session_id,
            session_start: now_secs(),
            session_end: None,
            session_count: 0,
            session_ids: Arc::new(Vec::with_capacity(MAX_SESSION_IDS)),
            personal_narrative: Arc::new(VecDeque::with_capacity(MAX_NARRATIVE_EVENTS)),
            current_session_summary: String::new(),
            current_session_reward: 0.0,
            current_session_start: now_secs(),
            current_session_insights: Arc::new(Vec::with_capacity(16)),
            total_reward: 0.0,
            situated_memory: VecDeque::new(),
            active_goals: Vec::new(),
            correction_log: VecDeque::new(),
            style_profile: StyleProfile::default(),
            role_history: VecDeque::new(),
            working_self: WorkingSelf::default(),
            consolidated_episodes: Vec::new(),
        }
    }

    pub fn record_iteration(&mut self, summary: &str, reward: f64, insight: Option<String>) {
        self.current_session_summary = summary.to_string();
        self.current_session_reward += reward;
        self.total_reward += reward;
        if let Some(i) = insight {
            if !self.current_session_insights.contains(&i) {
                Arc::make_mut(&mut self.current_session_insights).push(i);
            }
        }
    }

    pub fn end_session(&mut self) -> NarrativeEvent {
        let elapsed_secs = now_secs().saturating_sub(self.current_session_start);
        let summary = self.current_session_summary.clone();
        let vsa_bytes = QuantizedVSA::seeded_random(
            summary
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64)),
            64,
        );
        let event = NarrativeEvent {
            session_id: self.session_id.clone(),
            timestamp: now_secs(),
            summary,
            reward: self.current_session_reward,
            duration_ms: elapsed_secs * 1000,
            key_insights: (*self.current_session_insights).clone(),
            vsa_fingerprint: Some(vsa_bytes),
        };
        Arc::make_mut(&mut self.personal_narrative).push_back(event.clone());

        let ids = Arc::make_mut(&mut self.session_ids);
        if !ids.contains(&self.session_id) {
            ids.push(self.session_id.clone());
        }
        if ids.len() > MAX_SESSION_IDS {
            ids.remove(0);
        }

        if self.personal_narrative.len() > MAX_NARRATIVE_EVENTS {
            Arc::make_mut(&mut self.personal_narrative).pop_front();
        }
        self.session_end = Some(now_secs());
        self.session_count += 1;
        event
    }

    pub fn start_new_session(&mut self) {
        let _ = self.end_session();
        self.session_id = format!("ses_{:x}", rand::random::<u64>());
        self.session_start = now_secs();
        self.session_end = None;
        self.current_session_summary = String::new();
        self.current_session_reward = 0.0;
        self.current_session_start = now_secs();
        Arc::make_mut(&mut self.current_session_insights).clear();
    }

    pub fn narrative_summary(&self, max_events: usize) -> String {
        let count = self.personal_narrative.len().min(max_events);
        let recent: Vec<&NarrativeEvent> =
            self.personal_narrative.iter().rev().take(count).collect();
        let mut out = format!(
            "Sessions: {} | Total reward: {:.3}\n",
            self.session_count, self.total_reward
        );
        for (i, e) in recent.iter().enumerate().rev() {
            out.push_str(&format!(
                "  [{}. {}] reward={:.3} duration={}s insights={}\n",
                i + 1,
                e.summary.chars().take(60).collect::<String>(),
                e.reward,
                e.duration_ms / 1000,
                e.key_insights.len(),
            ));
        }
        out
    }

    pub fn merge(&mut self, other: NarrativeSelf, strategy: NarrativeMergeStrategy) {
        let other_events: Vec<NarrativeEvent> = other.personal_narrative.iter().cloned().collect();
        let self_events: Vec<NarrativeEvent> = self.personal_narrative.iter().cloned().collect();

        let merged = match strategy {
            NarrativeMergeStrategy::Union => {
                let mut all = self_events;
                for e in other_events {
                    if !all
                        .iter()
                        .any(|x| x.timestamp == e.timestamp && x.summary == e.summary)
                    {
                        all.push(e);
                    }
                }
                all
            }
            NarrativeMergeStrategy::Intersect => {
                let other_summaries: HashSet<&str> =
                    other_events.iter().map(|e| e.summary.as_str()).collect();
                self_events
                    .into_iter()
                    .filter(|e| other_summaries.contains(e.summary.as_str()))
                    .collect()
            }
            NarrativeMergeStrategy::ConfidenceWeighted => {
                let mut seen: HashMap<&str, &NarrativeEvent> = HashMap::new();
                for e in self_events.iter().chain(other_events.iter()) {
                    let key = e.summary.as_str();
                    if let Some(existing) = seen.get(key) {
                        if e.reward > existing.reward {
                            seen.insert(key, e);
                        }
                    } else {
                        seen.insert(key, e);
                    }
                }
                seen.into_values().cloned().collect()
            }
            NarrativeMergeStrategy::LatestWins => {
                let mut seen: HashMap<&str, &NarrativeEvent> = HashMap::new();
                for e in self_events.iter().chain(other_events.iter()) {
                    let key = e.summary.as_str();
                    if let Some(existing) = seen.get(key) {
                        if e.timestamp > existing.timestamp {
                            seen.insert(key, e);
                        }
                    } else {
                        seen.insert(key, e);
                    }
                }
                seen.into_values().cloned().collect()
            }
        };

        let mut bounded: VecDeque<NarrativeEvent> = merged.into_iter().collect();
        while bounded.len() > MAX_NARRATIVE_EVENTS {
            bounded.pop_front();
        }
        self.personal_narrative = Arc::new(bounded);

        let other_ids: Vec<&str> = other.session_ids.iter().map(|s| s.as_str()).collect();
        let self_ids = Arc::make_mut(&mut self.session_ids);
        for sid in other_ids {
            if !self_ids.iter().any(|x| x == sid) {
                if self_ids.len() >= MAX_SESSION_IDS {
                    self_ids.remove(0);
                }
                self_ids.push(sid.to_string());
            }
        }

        self.total_reward = self.total_reward.max(other.total_reward);
        self.session_count = self.session_count.max(other.session_count);
    }

    pub fn consolidate(&self) -> NarrativeConsolidationReport {
        let events: Vec<&NarrativeEvent> = self.personal_narrative.iter().collect();

        let total_sessions = self.session_count;
        let total_events = events.len() as u64;

        let recurring_themes = {
            let mut theme_sessions: HashMap<String, HashSet<String>> = HashMap::new();
            for e in &events {
                for token in e.summary.split_whitespace() {
                    let t = token
                        .trim_matches(|c: char| c.is_ascii_punctuation())
                        .to_lowercase();
                    if t.len() < 3 {
                        continue;
                    }
                    theme_sessions
                        .entry(t)
                        .or_default()
                        .insert(e.session_id.clone());
                }
            }
            let mut themes: Vec<(String, usize)> = theme_sessions
                .into_iter()
                .filter(|(_, sessions)| sessions.len() >= 3)
                .map(|(theme, sessions)| (theme, sessions.len()))
                .collect();
            themes.sort_by(|a, b| b.1.cmp(&a.1));
            themes.into_iter().take(10).map(|(t, _)| t).collect()
        };

        let identity_drift = {
            let first = events.first().and_then(|e| e.vsa_fingerprint.as_ref());
            let last = events.last().and_then(|e| e.vsa_fingerprint.as_ref());
            match (first, last) {
                (Some(a), Some(b)) => {
                    let a_vals: Vec<f64> = a.iter().map(|&x| x as f64 / 255.0).collect();
                    let b_vals: Vec<f64> = b.iter().map(|&x| x as f64 / 255.0).collect();
                    let dot: f64 = a_vals.iter().zip(&b_vals).map(|(x, y)| x * y).sum();
                    let norm_a: f64 = a_vals.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
                    let norm_b: f64 = b_vals.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
                    1.0 - (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
                }
                _ => 0.0,
            }
        };

        let coherence_trend = {
            let mut per_session: Vec<(String, Vec<f64>)> = Vec::new();
            for e in &events {
                let sid = &e.session_id;
                if let Some(entry) = per_session.iter_mut().find(|(id, _)| id == sid) {
                    entry.1.push(e.reward);
                } else {
                    per_session.push((sid.clone(), vec![e.reward]));
                }
            }
            per_session
                .iter()
                .map(|(_, rewards)| {
                    if rewards.len() < 2 {
                        return 1.0;
                    }
                    let mean = rewards.iter().sum::<f64>() / rewards.len() as f64;
                    let variance = rewards.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                        / rewards.len() as f64;
                    if variance.is_nan() || mean.abs() < 1e-10 {
                        1.0
                    } else {
                        let cv = variance.sqrt() / mean.abs();
                        (1.0 - cv.min(1.0)).max(0.0)
                    }
                })
                .collect()
        };

        NarrativeConsolidationReport {
            recurring_themes,
            identity_drift,
            coherence_trend,
            total_sessions,
            total_events,
        }
    }

    pub fn build_identity(&self) -> NarrativeIdentity {
        let report = self.consolidate();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for theme in &report.recurring_themes {
            theme.hash(&mut hasher);
        }
        let avg_reward = if self.session_count > 0 {
            self.total_reward / self.session_count as f64
        } else {
            0.0
        };
        avg_reward.to_bits().hash(&mut hasher);
        self.session_count.hash(&mut hasher);
        let seed = hasher.finish();

        let identity_vsa = QuantizedVSA::seeded_random(seed, IDENTITY_VSA_DIM);

        let events: Vec<&NarrativeEvent> = self.personal_narrative.iter().collect();
        let first_seen = events.first().map(|e| e.timestamp).unwrap_or(now_secs());
        let last_seen = events.last().map(|e| e.timestamp).unwrap_or(now_secs());

        let session_ids: Vec<String> = self.session_ids.to_vec();

        NarrativeIdentity {
            identity_vsa,
            session_ids,
            first_seen,
            last_seen,
            coherence_score: report.coherence_trend.iter().copied().fold(0.0, f64::max),
        }
    }

    /// Consolidate episodes from a range of consciousness cycles.
    /// Reads RECORD step outputs (narrative events) in the given cycle range,
    /// extracts key events, emotional markers, and decision points,
    /// and stores a ConsolidatedEpisode.
    pub fn consolidate_episode(
        &mut self,
        from_cycle: usize,
        to_cycle: usize,
    ) -> Option<&ConsolidatedEpisode> {
        let events: Vec<&NarrativeEvent> = self.personal_narrative.iter().collect();
        if events.is_empty() {
            return None;
        }
        let mut key_events: Vec<String> = Vec::new();
        let mut valence_sum = 0.0;
        let mut valence_count = 0usize;
        for event in &events {
            if event.duration_ms >= 100 {
                key_events.push(format!(
                    "{} reward={:.3} dur={}ms",
                    event.summary.chars().take(80).collect::<String>(),
                    event.reward,
                    event.duration_ms,
                ));
            }
            valence_sum += event.reward;
            valence_count += 1;
            for insight in &event.key_insights {
                if !key_events.iter().any(|k| k.contains(insight)) {
                    key_events.push(insight.clone());
                }
            }
        }
        let emotional_valence = if valence_count > 0 {
            (valence_sum / valence_count as f64).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let summary_text = if key_events.is_empty() {
            "no notable events".to_string()
        } else if key_events.len() == 1 {
            key_events[0].clone()
        } else {
            format!(
                "{} events from cycles {}-{} (valence={:.3})",
                key_events.len(),
                from_cycle,
                to_cycle,
                emotional_valence,
            )
        };
        let episode = ConsolidatedEpisode {
            cycle_range: (from_cycle, to_cycle),
            summary_text,
            key_events,
            emotional_valence,
        };
        self.consolidated_episodes.push(episode);
        self.consolidated_episodes.last()
    }

    pub fn save(&self) {
        let path = crate::core::nt_core_util::home_dir().join(NARRATIVE_PATH);
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::warn!("failed to create dir: {}", e);
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("tmp");
            if let Err(e) = std::fs::write(&tmp, json) {
                log::warn!("failed to write: {}", e);
            }
            if tmp.exists() {
                if let Err(e) = std::fs::rename(&tmp, &path) {
                    log::warn!("failed to rename: {}", e);
                }
            }
        }
    }

    pub fn load() -> Option<Self> {
        let path = crate::core::nt_core_util::home_dir().join(NARRATIVE_PATH);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<NarrativeSelf>(&s).ok())
    }

    pub fn record_situated_memory(&mut self, topic: &str, fact: &str) {
        let now = now_secs();
        if let Some(existing) = self.situated_memory.iter_mut().find(|m| m.topic == topic) {
            if !existing.key_facts.contains(&fact.to_string()) {
                existing.key_facts.push(fact.to_string());
            }
            existing.access_count = existing.access_count.saturating_add(1);
            existing.last_accessed = now;
        } else {
            self.situated_memory.push_back(StoredMemory {
                topic: topic.to_string(),
                key_facts: vec![fact.to_string()],
                access_count: 1,
                last_accessed: now,
            });
        }
    }

    pub fn recall_situated(&self, topic: &str) -> Vec<&str> {
        self.situated_memory
            .iter()
            .filter(|m| m.topic == topic)
            .flat_map(|m| m.key_facts.iter().map(|f| f.as_str()))
            .collect()
    }

    pub fn set_goal(&mut self, desc: &str, priority: f64) {
        let id = format!("goal_{:x}", rand::random::<u64>());
        self.active_goals.push(Goal {
            id,
            description: desc.to_string(),
            priority,
            created_at: now_secs(),
            status: GoalStatus::Active,
        });
    }

    pub fn update_goal_status(&mut self, id: &str, status: GoalStatus) {
        if let Some(goal) = self.active_goals.iter_mut().find(|g| g.id == id) {
            goal.status = status;
        }
    }

    pub fn log_correction(&mut self, error: &str, correction: &str, outcome: f64) {
        self.correction_log.push_back(Correction {
            error_desc: error.to_string(),
            correction: correction.to_string(),
            outcome,
            timestamp: now_secs(),
        });
    }

    pub fn update_style(&mut self, formality: f64, verbosity: f64, depth: f64) {
        self.style_profile.formality = formality.clamp(0.0, 1.0);
        self.style_profile.verbosity = verbosity.clamp(0.0, 1.0);
        self.style_profile.technical_depth = depth.clamp(0.0, 1.0);
    }

    pub fn adopt_role(&mut self, name: &str) {
        if let Some(existing) = self
            .role_history
            .iter_mut()
            .find(|r: &&mut Role| r.name == name)
        {
            existing.session_count = existing.session_count.saturating_add(1);
        } else {
            self.role_history.push_back(Role {
                name: name.to_string(),
                consistency: 1.0,
                session_count: 1,
            });
        }
    }

    pub fn current_role_consistency(&self) -> f64 {
        self.role_history
            .back()
            .map(|r| r.consistency)
            .unwrap_or(0.0)
    }

    pub fn set_working_context(&mut self, context: &str, emotional_valence: f64) {
        self.working_self.active_context = context.to_string();
        self.working_self.emotional_state = emotional_valence.clamp(-1.0, 1.0);
        for goal in &self.active_goals {
            if goal.status == GoalStatus::Active {
                let desc_lower = goal.description.to_lowercase();
                let context_lower = context.to_lowercase();
                if context_lower.contains(&desc_lower) || desc_lower.contains(&context_lower) {
                    if !self.working_self.current_goals.contains(&goal.id) {
                        self.working_self.current_goals.push(goal.id.clone());
                    }
                }
            }
        }
    }

    pub fn gate_retrieval(&self, memory: &NarrativeEvent) -> f64 {
        if self.working_self.current_goals.is_empty() {
            return 0.5;
        }
        let goal_descriptions: Vec<String> = self
            .working_self
            .current_goals
            .iter()
            .filter_map(|gid| {
                self.active_goals
                    .iter()
                    .find(|g| &g.id == gid)
                    .map(|g| g.description.clone())
            })
            .collect();
        if goal_descriptions.is_empty() {
            return 0.5;
        }
        let goal_words: Vec<String> = goal_descriptions
            .iter()
            .flat_map(|d| {
                d.split_whitespace()
                    .map(|w| {
                        w.trim_matches(|c: char| c.is_ascii_punctuation())
                            .to_lowercase()
                    })
                    .filter(|w| w.len() > 2)
            })
            .collect();
        if goal_words.is_empty() {
            return 0.5;
        }
        let memory_text: String =
            format!("{} {}", memory.summary, memory.key_insights.join(" ")).to_lowercase();
        let overlap: usize = goal_words
            .iter()
            .filter(|w| memory_text.contains(w.as_str()))
            .count();
        (overlap as f64 / goal_words.len() as f64).clamp(0.0, 1.0)
    }

    pub fn narrative_continuity_score(&self) -> f64 {
        let memory_score = if self.situated_memory.is_empty() {
            0.0
        } else {
            let accessed: f64 = self
                .situated_memory
                .iter()
                .filter(|m| m.access_count > 0)
                .count() as f64;
            accessed / self.situated_memory.len() as f64
        };
        let goal_score = if self.active_goals.is_empty() {
            0.0
        } else {
            let completed: f64 = self
                .active_goals
                .iter()
                .filter(|g| g.status == GoalStatus::Completed)
                .count() as f64;
            completed / self.active_goals.len() as f64
        };
        let correction_score = if self.correction_log.is_empty() {
            0.0
        } else {
            let avg_outcome: f64 = self.correction_log.iter().map(|c| c.outcome).sum::<f64>()
                / self.correction_log.len() as f64;
            avg_outcome.clamp(0.0, 1.0)
        };
        let style_score = {
            let f = self.style_profile.formality;
            let v = self.style_profile.verbosity;
            let d = self.style_profile.technical_depth;
            let mean = (f + v + d) / 3.0;
            let variance = ((f - mean).powi(2) + (v - mean).powi(2) + (d - mean).powi(2)) / 3.0;
            1.0 - (variance.sqrt()).min(1.0)
        };
        let role_score = self.current_role_consistency();
        (memory_score + goal_score + correction_score + style_score + role_score) / 5.0
    }
}

fn now_secs() -> u64 {
    crate::core::nt_core_time::unix_now_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_narrative_new_session() {
        let n = NarrativeSelf::new();
        assert!(n.session_id.starts_with("ses_"));
        assert!(n.session_end.is_none());
    }

    #[test]
    fn test_record_iteration_accumulates() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("task A", 0.5, Some("insight 1".into()));
        n.record_iteration("task B", 0.3, Some("insight 2".into()));
        assert!((n.current_session_reward - 0.8).abs() < 0.001);
        assert!((n.total_reward - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_end_session_creates_event() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("test task", 0.7, None);
        let event = n.end_session();
        assert_eq!(event.session_id, n.session_id);
    }

    #[test]
    fn test_start_new_session_rotates() {
        let mut n = NarrativeSelf::new();
        let first_id = n.session_id.clone();
        n.record_iteration("session A", 1.0, None);
        n.start_new_session();
        assert_ne!(n.session_id, first_id);
        assert_eq!(n.session_count, 1);
        assert_eq!(n.personal_narrative.len(), 1);
    }

    #[test]
    fn test_narrative_summary_format() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("important discovery", 0.9, Some("found pattern".into()));
        n.end_session();
        let summary = n.narrative_summary(10);
        assert!(summary.contains("Sessions:"));
        assert!(summary.contains("reward="));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("save test", 0.5, None);
        n.end_session();

        n.save();
        let loaded = NarrativeSelf::load();
        assert!(loaded.is_some());
        if let Some(l) = loaded {
            assert_eq!(l.personal_narrative.len(), n.personal_narrative.len());
            assert_eq!(l.session_count, n.session_count);
        }
    }

    #[test]
    fn test_multiple_sessions_tracked() {
        let mut n = NarrativeSelf::new();
        for i in 0..3 {
            n.record_iteration(&format!("session {}", i), i as f64 * 0.5, None);
            n.start_new_session();
        }
        assert_eq!(n.session_count, 3);
        assert_eq!(n.personal_narrative.len(), 3);
    }

    #[test]
    fn test_max_narrative_events() {
        let mut n = NarrativeSelf::new();
        for i in 0..MAX_NARRATIVE_EVENTS + 10 {
            n.record_iteration(&format!("event {}", i), 0.1, None);
            n.start_new_session();
        }
        assert!(n.personal_narrative.len() <= MAX_NARRATIVE_EVENTS);
    }

    #[test]
    fn test_merge_combines_events() {
        let mut a = NarrativeSelf::new();
        let mut b = NarrativeSelf::new();
        a.record_iteration("alpha", 0.5, None);
        a.end_session();
        b.record_iteration("beta", 0.3, None);
        b.end_session();
        let count_before = a.personal_narrative.len();
        a.merge(b, NarrativeMergeStrategy::Union);
        assert!(a.personal_narrative.len() >= count_before + 1);
    }

    #[test]
    fn test_merge_respects_max_events() {
        let mut a = NarrativeSelf::new();
        for i in 0..MAX_NARRATIVE_EVENTS {
            a.record_iteration(&format!("a-event {}", i), 0.1, None);
            a.end_session();
            a.start_new_session();
        }
        let mut b = NarrativeSelf::new();
        for i in 0..100 {
            b.record_iteration(&format!("b-event {}", i), 0.1, None);
            b.end_session();
            b.start_new_session();
        }
        a.merge(b, NarrativeMergeStrategy::Union);
        assert!(a.personal_narrative.len() <= MAX_NARRATIVE_EVENTS);
    }

    #[test]
    fn test_consolidation_recurring_themes() {
        let mut n = NarrativeSelf::new();
        for _ in 0..5 {
            n.record_iteration("deep learning exploration", 0.8, Some("new insight".into()));
            n.end_session();
            n.start_new_session();
        }
        n.record_iteration("unrelated walk", 0.1, None);
        n.end_session();
        let report = n.consolidate();
        assert!(!report.recurring_themes.is_empty());
        assert_eq!(report.total_sessions, 6);
    }

    #[test]
    fn test_build_identity_deterministic() {
        let mut a = NarrativeSelf::new();
        a.record_iteration("identical session", 0.7, Some("insight".into()));
        a.end_session();
        let id1 = a.build_identity();
        let id2 = a.build_identity();
        assert_eq!(id1.identity_vsa, id2.identity_vsa);
        assert_eq!(id1.session_ids, id2.session_ids);
        assert!((id1.coherence_score - id2.coherence_score).abs() < 1e-6);
    }

    #[test]
    fn test_identity_different_for_different_narratives() {
        let mut a = NarrativeSelf::new();
        let mut b = NarrativeSelf::new();
        a.record_iteration("exploring AI safety", 0.9, None);
        a.end_session();
        b.record_iteration("analyzing market trends", 0.3, None);
        b.end_session();
        let id_a = a.build_identity();
        let id_b = b.build_identity();
        assert_ne!(id_a.identity_vsa, id_b.identity_vsa);
    }

    #[test]
    fn test_record_situated_memory_adds_fact() {
        let mut n = NarrativeSelf::new();
        n.record_situated_memory("math", "2+2=4");
        assert_eq!(n.situated_memory.len(), 1);
        assert_eq!(n.situated_memory[0].key_facts.len(), 1);
    }

    #[test]
    fn test_record_situated_memory_merges_same_topic() {
        let mut n = NarrativeSelf::new();
        n.record_situated_memory("math", "2+2=4");
        n.record_situated_memory("math", "3+5=8");
        assert_eq!(n.situated_memory.len(), 1);
        assert_eq!(n.situated_memory[0].key_facts.len(), 2);
        assert_eq!(n.situated_memory[0].access_count, 2);
    }

    #[test]
    fn test_recall_situated_by_topic() {
        let mut n = NarrativeSelf::new();
        n.record_situated_memory("physics", "E=mc^2");
        n.record_situated_memory("math", "2+2=4");
        let facts = n.recall_situated("physics");
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0], "E=mc^2");
    }

    #[test]
    fn test_set_goal_creates_active_goal() {
        let mut n = NarrativeSelf::new();
        n.set_goal("find meaning", 0.9);
        assert_eq!(n.active_goals.len(), 1);
        assert_eq!(n.active_goals[0].status, GoalStatus::Active);
        assert!((n.active_goals[0].priority - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_update_goal_status() {
        let mut n = NarrativeSelf::new();
        n.set_goal("test", 0.5);
        let id = n.active_goals[0].id.clone();
        n.update_goal_status(&id, GoalStatus::Completed);
        assert_eq!(n.active_goals[0].status, GoalStatus::Completed);
    }

    #[test]
    fn test_log_correction_appends() {
        let mut n = NarrativeSelf::new();
        n.log_correction("wrong answer", "recompute", 0.8);
        assert_eq!(n.correction_log.len(), 1);
        assert_eq!(n.correction_log[0].error_desc, "wrong answer");
    }

    #[test]
    fn test_update_style_clamps() {
        let mut n = NarrativeSelf::new();
        n.update_style(1.5, -0.3, 0.7);
        assert!((n.style_profile.formality - 1.0).abs() < 0.001);
        assert!((n.style_profile.verbosity - 0.0).abs() < 0.001);
        assert!((n.style_profile.technical_depth - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_adopt_role_creates_new() {
        let mut n = NarrativeSelf::new();
        n.adopt_role("teacher");
        assert_eq!(n.role_history.len(), 1);
        assert_eq!(n.role_history[0].name, "teacher");
    }

    #[test]
    fn test_adopt_role_increments_existing() {
        let mut n = NarrativeSelf::new();
        n.adopt_role("mentor");
        n.adopt_role("mentor");
        assert_eq!(n.role_history.len(), 1);
        assert_eq!(n.role_history[0].session_count, 2);
    }

    #[test]
    fn test_current_role_consistency_default_zero() {
        let n = NarrativeSelf::new();
        assert!((n.current_role_consistency() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_set_working_context_activates_goals() {
        let mut n = NarrativeSelf::new();
        n.set_goal("explore quantum", 0.9);
        n.set_working_context("explore quantum computing", 0.5);
        assert!(!n.working_self.current_goals.is_empty());
    }

    #[test]
    fn test_gate_retrieval_returns_half_when_no_goals() {
        let n = NarrativeSelf::new();
        let mem = NarrativeEvent {
            session_id: "s1".into(),
            timestamp: 0,
            summary: "anything".into(),
            reward: 0.0,
            duration_ms: 0,
            key_insights: vec![],
            vsa_fingerprint: None,
        };
        assert!((n.gate_retrieval(&mem) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_narrative_continuity_score_range() {
        let n = NarrativeSelf::new();
        let score = n.narrative_continuity_score();
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_narrative_continuity_score_increases_with_data() {
        let mut n = NarrativeSelf::new();
        let base = n.narrative_continuity_score();
        n.record_situated_memory("topic", "fact");
        n.set_goal("complete", 1.0);
        n.log_correction("err", "fix", 1.0);
        n.update_style(0.8, 0.3, 0.7);
        n.adopt_role("expert");
        let improved = n.narrative_continuity_score();
        assert!(improved >= base);
    }

    #[test]
    fn test_gate_retrieval_keyword_overlap() {
        let mut n = NarrativeSelf::new();
        n.set_goal("explore quantum mechanics", 1.0);
        n.set_working_context("explore quantum mechanics now", 0.0);
        let mem = NarrativeEvent {
            session_id: "s1".into(),
            timestamp: 0,
            summary: "learning about quantum mechanics".into(),
            reward: 0.0,
            duration_ms: 0,
            key_insights: vec![],
            vsa_fingerprint: None,
        };
        let score = n.gate_retrieval(&mem);
        assert!(score > 0.0);
    }
}
