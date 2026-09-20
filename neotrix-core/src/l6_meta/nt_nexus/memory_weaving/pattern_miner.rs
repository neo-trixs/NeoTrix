use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::config::WeaveConfig;
use super::session_bridge::{SessionBridge, SessionSummary};

/// A recurring theme or pattern detected across multiple sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Unique identifier for this pattern.
    pub id: String,
    /// Human-readable label derived from keyword clustering.
    pub label: String,
    /// Keywords that define this pattern (R-MEM12 episodic → semantic).
    pub keywords: Vec<String>,
    /// Session IDs where this pattern was observed.
    pub sessions: Vec<String>,
    /// Recurrence count across sessions.
    pub frequency: usize,
    /// Timestamp of first detection.
    pub first_seen: u64,
    /// Timestamp of most recent observation.
    pub last_seen: u64,
    /// Domain this pattern belongs to (e.g. "architecture", "debugging").
    pub domain: String,
}

impl Pattern {
    /// Jaccard similarity between two patterns' keyword sets.
    pub fn keyword_similarity(&self, other: &Pattern) -> f64 {
        let a: HashSet<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
        let b: HashSet<&str> = other.keywords.iter().map(|s| s.as_str()).collect();
        let intersection = a.intersection(&b).count();
        let union = a.union(&b).count();
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// A session's keyword fingerprint used for pattern mining.
#[derive(Debug, Clone)]
pub struct SessionFingerprint {
    pub session_id: String,
    pub keywords: Vec<String>,
    pub domain: String,
    pub timestamp: u64,
}

/// Mines recurring patterns across sessions (R-MEM12 distillation).
///
/// `PatternMiner` operates on session fingerprints and produces
/// candidate `Pattern` instances that exceed the configured frequency
/// threshold. It does not persist — the caller is responsible for
/// feeding results into `WeaveGraph` and `ConsolidationEngine`.
pub struct PatternMiner {
    config: WeaveConfig,
    /// Accumulated fingerprints from ingested sessions.
    fingerprints: Vec<SessionFingerprint>,
}

impl PatternMiner {
    pub fn new(config: WeaveConfig) -> Self {
        Self {
            config,
            fingerprints: Vec::new(),
        }
    }

    /// Ingest a session fingerprint for future mining.
    pub fn ingest(&mut self, fp: SessionFingerprint) {
        self.fingerprints.push(fp);
    }

    /// Ingest multiple fingerprints at once.
    pub fn ingest_batch(&mut self, fps: Vec<SessionFingerprint>) {
        self.fingerprints.extend(fps);
    }

    /// Mine patterns from all ingested fingerprints.
    ///
    /// Returns patterns whose frequency >= `config.min_pattern_frequency`.
    pub fn mine(&self) -> Vec<Pattern> {
        if self.fingerprints.is_empty() {
            return Vec::new();
        }

        // Group fingerprints by domain for intra-domain pattern detection
        let mut by_domain: HashMap<&str, Vec<&SessionFingerprint>> = HashMap::new();
        for fp in &self.fingerprints {
            by_domain
                .entry(fp.domain.as_str())
                .or_default()
                .push(fp);
        }

        let mut patterns = Vec::new();
        let mut pattern_counter: HashMap<String, usize> = HashMap::new();

        for (domain, fps) in &by_domain {
            // Build keyword → session set mapping
            let mut keyword_sessions: HashMap<&str, HashSet<&str>> = HashMap::new();
            for fp in fps {
                for kw in &fp.keywords {
                    keyword_sessions
                        .entry(kw.as_str())
                        .or_default()
                        .insert(&fp.session_id);
                }
            }

            // Cluster co-occurring keywords into pattern candidates
            let keywords: Vec<&str> = keyword_sessions.keys().copied().collect();
            for i in 0..keywords.len() {
                for j in (i + 1)..keywords.len() {
                    let kw_a = keywords[i];
                    let kw_b = keywords[j];
                    let sessions_a = &keyword_sessions[kw_a];
                    let sessions_b = &keyword_sessions[kw_b];

                    let intersection: HashSet<&str> =
                        sessions_a.intersection(sessions_b).copied().collect();

                    if intersection.len() < self.config.min_pattern_frequency {
                        continue;
                    }

                    // Generate a stable pattern ID from the keyword pair
                    let mut pair = vec![kw_a, kw_b];
                    pair.sort();
                    let pattern_key = format!("{}:{}", domain, pair.join("+"));

                    let counter = pattern_counter.entry(pattern_key.clone()).or_insert(0);
                    *counter += 1;

                    let label = format!("{}_{}+", domain, pair.join("_"));
                    let sessions: Vec<String> =
                        intersection.into_iter().map(|s| s.to_string()).collect();
                    let timestamps: Vec<u64> = fps
                        .iter()
                        .filter(|fp| sessions.contains(&fp.session_id))
                        .map(|fp| fp.timestamp)
                        .collect();

                    patterns.push(Pattern {
                        id: pattern_key,
                        label,
                        keywords: vec![kw_a.to_string(), kw_b.to_string()],
                        sessions: sessions.clone(),
                        frequency: sessions.len(),
                        first_seen: timestamps.iter().copied().min().unwrap_or(0),
                        last_seen: timestamps.iter().copied().max().unwrap_or(0),
                        domain: domain.to_string(),
                    });
                }
            }
        }

        // Deduplicate by pattern ID, keeping highest frequency
        let mut seen: HashMap<String, Pattern> = HashMap::new();
        for p in patterns {
            match seen.get_mut(&p.id) {
                Some(existing) => {
                    if p.frequency > existing.frequency {
                        *existing = p;
                    }
                }
                None => {
                    seen.insert(p.id.clone(), p);
                }
            }
        }

        let mut result: Vec<Pattern> = seen.into_values().collect();
        result.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        result
    }

    /// Number of ingested fingerprints.
    pub fn fingerprint_count(&self) -> usize {
        self.fingerprints.len()
    }

    /// Clear all ingested fingerprints.
    pub fn clear(&mut self) {
        self.fingerprints.clear();
    }

    /// Detect bridges between sessions based on shared patterns.
    /// Returns SessionBridge candidates with similarity scores.
    pub fn detect_bridges(&self) -> Vec<SessionBridge> {
        let patterns = self.mine();
        let mut session_pairs: HashMap<(String, String), Vec<&Pattern>> = HashMap::new();

        for pattern in &patterns {
            let sessions = &pattern.sessions;
            for i in 0..sessions.len() {
                for j in (i + 1)..sessions.len() {
                    let mut pair = [sessions[i].clone(), sessions[j].clone()];
                    pair.sort();
                    let key = (pair[0].clone(), pair[1].clone());
                    session_pairs.entry(key).or_default().push(pattern);
                }
            }
        }

        session_pairs
            .into_iter()
            .map(|((a, b), shared)| {
                let similarity = shared.len() as f64
                    / (self.fingerprint_count().max(1) as f64);
                SessionBridge::new(
                    &a,
                    &b,
                    shared.iter().map(|p| p.id.clone()).collect(),
                    similarity,
                )
            })
            .collect()
    }
}

/// Lightweight topic-level pattern for cross-session analysis.
///
/// Distinct from the keyword-clustering `Pattern` above; used by the
/// free functions `mine_patterns` / `promote_frequent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPattern {
    pub topic: String,
    pub frequency: u32,
    pub sessions: Vec<String>,
    pub avg_confidence: f64,
}

/// Mine recurring topic patterns across sessions.
///
/// For each unique topic appearing in `key_topics` across multiple sessions,
/// produces a `TopicPattern` with frequency = number of sessions containing it.
pub fn mine_patterns(sessions: &[SessionSummary]) -> Vec<TopicPattern> {
    use std::collections::HashMap;

    let mut topic_sessions: HashMap<&str, Vec<&str>> = HashMap::new();

    for session in sessions {
        for topic in &session.key_topics {
            topic_sessions
                .entry(topic.as_str())
                .or_default()
                .push(&session.session_id);
        }
    }

    topic_sessions
        .into_iter()
        .filter(|(_, s)| s.len() >= 2)
        .map(|(topic, session_ids)| TopicPattern {
            topic: topic.to_string(),
            frequency: session_ids.len() as u32,
            sessions: session_ids.into_iter().map(String::from).collect(),
            avg_confidence: 1.0,
        })
        .collect()
}

/// Keep only patterns whose frequency >= threshold.
pub fn promote_frequent(patterns: &[TopicPattern], threshold: u32) -> Vec<TopicPattern> {
    patterns
        .iter()
        .filter(|p| p.frequency >= threshold)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(session: &str, kws: Vec<&str>, domain: &str) -> SessionFingerprint {
        SessionFingerprint {
            session_id: session.to_string(),
            keywords: kws.into_iter().map(String::from).collect(),
            domain: domain.to_string(),
            timestamp: 1000,
        }
    }

    #[test]
    fn mine_no_fingerprints() {
        let miner = PatternMiner::new(WeaveConfig::default());
        assert!(miner.mine().is_empty());
    }

    #[test]
    fn mine_single_session_no_pattern() {
        let mut miner = PatternMiner::new(WeaveConfig::default());
        miner.ingest(fp("s1", vec!["a", "b"], "arch"));
        // Need at least min_pattern_frequency (2) sessions sharing keywords
        assert!(miner.mine().is_empty());
    }

    #[test]
    fn mine_shared_keywords() {
        let cfg = WeaveConfig {
            min_pattern_frequency: 2,
            ..Default::default()
        };
        let mut miner = PatternMiner::new(cfg);
        miner.ingest(fp("s1", vec!["rust", "trait", "error"], "arch"));
        miner.ingest(fp("s2", vec!["rust", "trait", "async"], "arch"));
        miner.ingest(fp("s3", vec!["python", "class"], "ml"));

        let patterns = miner.mine();
        // "rust"+"trait" should appear in s1 and s2
        assert!(patterns.iter().any(|p| p.frequency >= 2));
    }

    #[test]
    fn pattern_keyword_similarity() {
        let a = Pattern {
            id: "a".into(),
            label: "a".into(),
            keywords: vec!["x".into(), "y".into(), "z".into()],
            sessions: vec![],
            frequency: 3,
            first_seen: 0,
            last_seen: 0,
            domain: "d".into(),
        };
        let b = Pattern {
            id: "b".into(),
            label: "b".into(),
            keywords: vec!["y".into(), "z".into(), "w".into()],
            sessions: vec![],
            frequency: 3,
            first_seen: 0,
            last_seen: 0,
            domain: "d".into(),
        };
        // intersection={y,z}=2, union={x,y,z,w}=4 => 0.5
        assert!((a.keyword_similarity(&b) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn detect_bridges() {
        let cfg = WeaveConfig {
            min_pattern_frequency: 2,
            ..Default::default()
        };
        let mut miner = PatternMiner::new(cfg);
        miner.ingest(fp("s1", vec!["rust", "trait"], "arch"));
        miner.ingest(fp("s2", vec!["rust", "trait"], "arch"));
        miner.ingest(fp("s3", vec!["python"], "ml"));

        let bridges = miner.detect_bridges();
        // s1-s2 share "rust"+"trait"
        assert!(!bridges.is_empty());
        assert!(bridges.iter().any(|b| {
            (b.session_a == "s1" && b.session_b == "s2")
                || (b.session_a == "s2" && b.session_b == "s1")
        }));
    }

    #[test]
    fn clear_empties_fingerprints() {
        let mut miner = PatternMiner::new(WeaveConfig::default());
        miner.ingest(fp("s1", vec!["a"], "d"));
        assert_eq!(miner.fingerprint_count(), 1);
        miner.clear();
        assert_eq!(miner.fingerprint_count(), 0);
    }
}
