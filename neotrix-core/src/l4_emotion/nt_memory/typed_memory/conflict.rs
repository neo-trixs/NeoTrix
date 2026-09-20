#![forbid(unsafe_code)]

use std::collections::HashMap;

use super::entry::{ConflictOutcome, ConflictStrategy, TypedMemoryEntry};

/// Conflict resolution result.
#[derive(Debug, Clone)]
pub struct Resolution {
    pub winner_id: String,
    pub loser_id: Option<String>,
    pub strategy: ConflictStrategy,
    pub merged_content: Option<String>,
}

/// Conflict resolver supporting four strategies.
#[derive(Debug)]
pub struct ConflictResolver {
    strategy: ConflictStrategy,
    manual_decisions: HashMap<String, ConflictOutcome>,
}

impl ConflictResolver {
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self {
            strategy,
            manual_decisions: HashMap::new(),
        }
    }
    pub fn with_strategy(mut self, strategy: ConflictStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn resolve_all(&mut self, entries: &mut Vec<TypedMemoryEntry>) -> Vec<Resolution> {
        let mut resolutions = Vec::new();
        let groups = self.group_conflicts(entries);
        for (key, group) in groups {
            if group.len() <= 1 {
                continue;
            }
            let mut group: Vec<TypedMemoryEntry> = group.into_iter().map(|e| e.clone()).collect();
            match self.strategy {
                ConflictStrategy::LatestWins => {
                    resolutions.push(self.resolve_latest_wins(key, &mut group[..]))
                }
                ConflictStrategy::HighestConfidence => {
                    resolutions.push(self.resolve_highest_confidence(key, &mut group[..]))
                }
                ConflictStrategy::MajorityVote => {
                    resolutions.push(self.resolve_majority_vote(key, &mut group[..]))
                }
                ConflictStrategy::Manual => {
                    resolutions.push(self.resolve_manual(key, &mut group[..]))
                }
            }
        }
        resolutions
    }

    pub fn register_manual(&mut self, key: String, outcome: ConflictOutcome) {
        self.manual_decisions.insert(key, outcome);
    }

    fn group_conflicts<'a>(
        &self,
        entries: &'a mut Vec<TypedMemoryEntry>,
    ) -> HashMap<String, Vec<&'a mut TypedMemoryEntry>> {
        let mut groups: HashMap<String, Vec<&mut TypedMemoryEntry>> = HashMap::new();
        for entry in entries.iter_mut() {
            let key = entry
                .conflicts
                .first()
                .map(|c| c.conflicting_id.clone())
                .unwrap_or_else(|| entry.id.clone());
            groups.entry(key).or_default().push(entry);
        }
        groups.retain(|_, v| v.len() > 1);
        groups
    }

    fn resolve_latest_wins(&self, _key: String, entries: &mut [TypedMemoryEntry]) -> Resolution {
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let winner = &entries[0];
        let losers = &entries[1..];
        Resolution {
            winner_id: winner.id.clone(),
            loser_id: losers.first().map(|e| Some(e.id.clone())).unwrap_or(None),
            strategy: ConflictStrategy::LatestWins,
            merged_content: Some(
                entries
                    .iter()
                    .map(|e| e.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n--- MERGED ---\n"),
            ),
        }
    }

    fn resolve_highest_confidence(
        &self,
        _key: String,
        entries: &mut [TypedMemoryEntry],
    ) -> Resolution {
        entries.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let winner = &entries[0];
        let losers = &entries[1..];
        Resolution {
            winner_id: winner.id.clone(),
            loser_id: losers.first().map(|e| Some(e.id.clone())).unwrap_or(None),
            strategy: ConflictStrategy::HighestConfidence,
            merged_content: Some(
                entries
                    .iter()
                    .map(|e| e.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n--- MERGED ---\n"),
            ),
        }
    }

    fn resolve_majority_vote(&self, _key: String, entries: &mut [TypedMemoryEntry]) -> Resolution {
        let mut scores: HashMap<String, usize> = HashMap::new();
        for entry in entries.iter() {
            *scores.entry(entry.id.clone()).or_insert(0) += 1;
        }
        let winner_id = scores
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k.clone())
            .unwrap_or_default();
        let loser = entries
            .iter()
            .find(|e| e.id != winner_id)
            .map(|e| e.id.clone());
        Resolution {
            winner_id,
            loser_id: loser,
            strategy: ConflictStrategy::MajorityVote,
            merged_content: Some(
                entries
                    .iter()
                    .map(|e| e.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n--- MERGED ---\n"),
            ),
        }
    }

    fn resolve_manual(&self, key: String, entries: &mut [TypedMemoryEntry]) -> Resolution {
        let outcome = self
            .manual_decisions
            .get(&key)
            .cloned()
            .unwrap_or(ConflictOutcome::Pending);
        let winner_id = match outcome {
            ConflictOutcome::Resolved | ConflictOutcome::Superseded | ConflictOutcome::Merged => {
                entries.first().map(|e| e.id.clone()).unwrap_or_default()
            }
            ConflictOutcome::Pending => String::new(),
        };
        Resolution {
            winner_id,
            loser_id: entries.get(1).map(|e| Some(e.id.clone())).unwrap_or(None),
            strategy: ConflictStrategy::Manual,
            merged_content: if matches!(outcome, ConflictOutcome::Merged) {
                Some(
                    entries
                        .iter()
                        .map(|e| e.content.clone())
                        .collect::<Vec<_>>()
                        .join("\n--- MERGED ---\n"),
                )
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::entry::ConflictRecord;
    use super::*;
    use super::super::estate::MemoryEstate;
    fn make_conflicting() -> (TypedMemoryEntry, TypedMemoryEntry) {
        let mut e1 =
            TypedMemoryEntry::new("e1".into(), MemoryEstate::Semantic, "content A".into(), 0.8);
        e1.conflicts.push(ConflictRecord {
            conflicting_id: "e2".into(),
            strategy: ConflictStrategy::LatestWins,
            resolved_at: 0,
            outcome: ConflictOutcome::Pending,
        });
        let mut e2 =
            TypedMemoryEntry::new("e2".into(), MemoryEstate::Semantic, "content B".into(), 0.9);
        e2.conflicts.push(ConflictRecord {
            conflicting_id: "e1".into(),
            strategy: ConflictStrategy::LatestWins,
            resolved_at: 0,
            outcome: ConflictOutcome::Pending,
        });
        (e1, e2)
    }
    #[test]
    fn test_resolve_latest_wins() {
        let (mut e1, mut e2) = make_conflicting();
        e2.timestamp = e1.timestamp + 100;
        let mut entries = vec![e1, e2];
        let mut r = ConflictResolver::new(ConflictStrategy::LatestWins);
        let res = r.resolve_all(&mut entries);
        assert!(!res.is_empty());
        assert_eq!(res[0].strategy, ConflictStrategy::LatestWins);
    }
    #[test]
    fn test_resolve_highest_confidence() {
        let (mut e1, mut e2) = make_conflicting();
        e1.confidence = 0.5;
        let mut entries = vec![e1, e2];
        let mut r = ConflictResolver::new(ConflictStrategy::HighestConfidence);
        let res = r.resolve_all(&mut entries);
        assert!(!res.is_empty());
        assert_eq!(res[0].strategy, ConflictStrategy::HighestConfidence);
    }
    #[test]
    fn test_empty_resolution() {
        let mut entries: Vec<TypedMemoryEntry> = vec![];
        let mut r = ConflictResolver::new(ConflictStrategy::LatestWins);
        assert!(r.resolve_all(&mut entries).is_empty());
    }
    #[test]
    fn test_manual_resolution() {
        let (e1, e2) = make_conflicting();
        let mut entries = vec![e1, e2];
        let mut r = ConflictResolver::new(ConflictStrategy::Manual);
        r.register_manual("e1".into(), ConflictOutcome::Resolved);
        assert!(!r.resolve_all(&mut entries).is_empty());
    }
}
