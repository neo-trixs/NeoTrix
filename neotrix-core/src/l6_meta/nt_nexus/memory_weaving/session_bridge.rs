use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Compact representation of a session's salient features for cross-session analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub key_topics: Vec<String>,
    pub patterns: Vec<String>,
    pub timestamp: chrono::NaiveDateTime,
}

/// A directed connection between two sessions sharing overlapping patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    pub from_session: String,
    pub to_session: String,
    pub shared_patterns: Vec<String>,
    pub strength: f64,
}

/// Find bridges between sessions that share at least `min_overlap` patterns.
///
/// Strength is computed as `shared_count / total_unique_patterns` (Jaccard-like).
pub fn find_bridges(sessions: &[SessionSummary], min_overlap: usize) -> Vec<Bridge> {
    let mut bridges = Vec::new();

    for i in 0..sessions.len() {
        for j in (i + 1)..sessions.len() {
            let a = &sessions[i];
            let b = &sessions[j];

            let set_a: HashSet<&str> = a.patterns.iter().map(|s| s.as_str()).collect();
            let set_b: HashSet<&str> = b.patterns.iter().map(|s| s.as_str()).collect();

            let shared: Vec<String> = set_a
                .intersection(&set_b)
                .map(|s| s.to_string())
                .collect();

            if shared.len() < min_overlap {
                continue;
            }

            let union: HashSet<&str> = set_a.union(&set_b).copied().collect();
            let strength = if union.is_empty() {
                0.0
            } else {
                shared.len() as f64 / union.len() as f64
            };

            bridges.push(Bridge {
                from_session: a.session_id.clone(),
                to_session: b.session_id.clone(),
                shared_patterns: shared,
                strength,
            });
        }
    }

    bridges.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
    bridges
}

/// Internal bridge type used by `PatternMiner::detect_bridges`.
#[derive(Debug, Clone)]
pub struct SessionBridge {
    pub session_a: String,
    pub session_b: String,
    pub shared_pattern_ids: Vec<String>,
    pub similarity: f64,
}

impl SessionBridge {
    pub fn new(session_a: &str, session_b: &str, shared_pattern_ids: Vec<String>, similarity: f64) -> Self {
        Self {
            session_a: session_a.to_string(),
            session_b: session_b.to_string(),
            shared_pattern_ids,
            similarity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(id: &str, patterns: Vec<&str>) -> SessionSummary {
        SessionSummary {
            session_id: id.to_string(),
            key_topics: vec![],
            patterns: patterns.into_iter().map(String::from).collect(),
            timestamp: chrono::DateTime::from_timestamp(1_000_000, 0).unwrap().naive_utc(),
        }
    }

    #[test]
    fn no_overlap_no_bridges() {
        let sessions = vec![summary("s1", vec!["a"]), summary("s2", vec!["b"])];
        let bridges = find_bridges(&sessions, 1);
        assert!(bridges.is_empty());
    }

    #[test]
    fn sufficient_overlap() {
        let sessions = vec![
            summary("s1", vec!["a", "b", "c"]),
            summary("s2", vec!["b", "c", "d"]),
        ];
        let bridges = find_bridges(&sessions, 2);
        assert_eq!(bridges.len(), 1);
        assert_eq!(bridges[0].shared_patterns.len(), 2);
        assert!(bridges[0].strength > 0.0);
    }

    #[test]
    fn below_threshold_filtered() {
        let sessions = vec![
            summary("s1", vec!["a"]),
            summary("s2", vec!["a", "b", "c"]),
        ];
        let bridges = find_bridges(&sessions, 2);
        assert!(bridges.is_empty());
    }

    #[test]
    fn strength_is_jaccard() {
        let sessions = vec![
            summary("s1", vec!["x", "y"]),
            summary("s2", vec!["y", "z"]),
        ];
        let bridges = find_bridges(&sessions, 1);
        assert_eq!(bridges.len(), 1);
        // shared={y}=1, union={x,y,z}=3 => 1/3
        assert!((bridges[0].strength - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn three_sessions_pairwise() {
        let sessions = vec![
            summary("s1", vec!["a", "b"]),
            summary("s2", vec!["b", "c"]),
            summary("s3", vec!["a", "b", "c"]),
        ];
        let bridges = find_bridges(&sessions, 1);
        // s1-s2: shared={b}, s1-s3: shared={a,b}, s2-s3: shared={b,c}
        assert_eq!(bridges.len(), 3);
    }

    #[test]
    fn bridges_sorted_by_strength() {
        let sessions = vec![
            summary("s1", vec!["a", "b", "c", "d"]),
            summary("s2", vec!["c", "d"]),
            summary("s3", vec!["a", "b", "c", "d"]),
        ];
        let bridges = find_bridges(&sessions, 1);
        assert!(bridges.len() >= 2);
        // s1-s3 should have higher strength than s1-s2
        assert!(bridges[0].strength >= bridges[1].strength);
    }

    #[test]
    fn session_bridge_new() {
        let sb = SessionBridge::new("a", "b", vec!["p1".into()], 0.5);
        assert_eq!(sb.session_a, "a");
        assert_eq!(sb.session_b, "b");
        assert_eq!(sb.similarity, 0.5);
    }
}
