use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_self_test::SelfTest;

#[derive(Debug, Clone, PartialEq)]
pub enum CommitType {
    Feature,
    Fix,
    Refactor,
    Docs,
    Test,
}

impl CommitType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommitType::Feature => "feature",
            CommitType::Fix => "fix",
            CommitType::Refactor => "refactor",
            CommitType::Docs => "docs",
            CommitType::Test => "test",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitEvent {
    pub id: String,
    pub timestamp: u64,
    pub author: String,
    pub description: String,
    pub files_changed: Vec<String>,
    pub commit_type: CommitType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArcStatus {
    Active,
    Completed,
    Abandoned,
}

impl ArcStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArcStatus::Active => "active",
            ArcStatus::Completed => "completed",
            ArcStatus::Abandoned => "abandoned",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlotArc {
    pub name: String,
    pub description: String,
    pub commits: Vec<String>,
    pub status: ArcStatus,
}

#[derive(Debug, Clone)]
pub struct CharacterSheet {
    pub entity_name: String,
    pub first_seen: u64,
    pub last_modified: u64,
    pub commit_count: u64,
    pub tags: HashSet<String>,
    pub consistency_issues: Vec<String>,
}

const MAX_EVENTS: usize = 1000;

#[derive(Debug)]
pub struct NarrativeState {
    pub arcs: Vec<PlotArc>,
    pub characters: Vec<CharacterSheet>,
    pub events: VecDeque<CommitEvent>,
}

impl NarrativeState {
    pub fn new() -> Self {
        NarrativeState {
            arcs: Vec::new(),
            characters: Vec::new(),
            events: VecDeque::with_capacity(MAX_EVENTS),
        }
    }

    pub fn record_event(&mut self, event: CommitEvent) {
        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn arc_count(&self) -> usize {
        self.arcs.len()
    }

    pub fn character_count(&self) -> usize {
        self.characters.len()
    }
}

#[derive(Debug)]
pub struct NarrativeConsistencyChecker {
    state: NarrativeState,
    entity_index: HashMap<String, usize>,
    arc_index: HashMap<String, usize>,
}

impl NarrativeConsistencyChecker {
    pub fn new() -> Self {
        NarrativeConsistencyChecker {
            state: NarrativeState::new(),
            entity_index: HashMap::new(),
            arc_index: HashMap::new(),
        }
    }

    pub fn record_event(
        &mut self,
        id: String,
        author: String,
        description: String,
        files_changed: Vec<String>,
        commit_type: CommitType,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let event = CommitEvent {
            id: id.clone(),
            timestamp: now,
            author,
            description,
            files_changed: files_changed.clone(),
            commit_type,
        };

        self.state.record_event(event);

        for file in &files_changed {
            self.update_entity(file, &id, now);
        }
    }

    fn update_entity(&mut self, entity_name: &str, _commit_id: &str, now: u64) {
        if let Some(&idx) = self.entity_index.get(entity_name) {
            let sheet = &mut self.state.characters[idx];
            sheet.last_modified = now;
            sheet.commit_count += 1;
        } else {
            let mut tags = HashSet::new();
            let parts: Vec<&str> = entity_name.split('/').collect();
            if parts.len() > 1 {
                tags.insert(parts[0].to_string());
            }
            if entity_name.ends_with(".rs") {
                tags.insert("rust".to_string());
            } else if entity_name.ends_with(".py") {
                tags.insert("python".to_string());
            } else if entity_name.ends_with(".md") {
                tags.insert("documentation".to_string());
            }
            let sheet = CharacterSheet {
                entity_name: entity_name.to_string(),
                first_seen: now,
                last_modified: now,
                commit_count: 1,
                tags,
                consistency_issues: Vec::new(),
            };
            self.entity_index.insert(entity_name.to_string(), self.state.characters.len());
            self.state.characters.push(sheet);
        }
    }

    pub fn add_arc(&mut self, name: String, description: String, status: ArcStatus) {
        let arc = PlotArc {
            name: name.clone(),
            description,
            commits: Vec::new(),
            status,
        };
        self.arc_index.insert(name.clone(), self.state.arcs.len());
        self.state.arcs.push(arc);
    }

    pub fn link_commit_to_arc(&mut self, commit_id: &str, arc_name: &str) -> Result<(), String> {
        let idx = self.arc_index.get(arc_name).ok_or_else(|| format!("arc '{}' not found", arc_name))?;
        self.state.arcs[*idx].commits.push(commit_id.to_string());
        Ok(())
    }

    pub fn check_consistency(&mut self) -> Vec<String> {
        let mut issues = Vec::new();
        issues.extend(self.detect_arc_drift());
        issues.extend(self.detect_orphan_entities());
        issues.extend(self.detect_stale_arcs());
        issues
    }

    pub fn detect_arc_drift(&mut self) -> Vec<String> {
        let mut issues = Vec::new();
        for arc in &self.state.arcs {
            if arc.status != ArcStatus::Active {
                continue;
            }
            let recent_count = arc.commits.iter().filter(|_| true).count();
            if recent_count > 0 && !arc.description.is_empty() {
                let desc_words: HashSet<&str> = arc.description.split_whitespace().collect();
                for commit_id in &arc.commits {
                    if let Some(event) = self.state.events.iter().find(|e| &e.id == commit_id) {
                        let desc_lower = event.description.to_lowercase();
                        let desc_words_lower: HashSet<String> = desc_words.iter().map(|w| w.to_lowercase()).collect();
                        let has_overlap = desc_words_lower.iter().any(|w| desc_lower.contains(w.as_str()));
                        if !has_overlap {
                            issues.push(format!(
                                "arc '{}': commit '{}' ('{}') may drift from arc description '{}'",
                                arc.name, commit_id, event.description, arc.description
                            ));
                        }
                    }
                }
            }
        }
        issues
    }

    fn detect_orphan_entities(&self) -> Vec<String> {
        let mut issues = Vec::new();
        for sheet in &self.state.characters {
            let active = self.state.arcs.iter().any(|arc| {
                arc.status == ArcStatus::Active
                    && arc.commits.iter().any(|cid| {
                        self.state.events.iter().any(|e| &e.id == cid && e.files_changed.contains(&sheet.entity_name))
                    })
            });
            if !active && sheet.commit_count > 1 {
                issues.push(format!(
                    "entity '{}' has {} commits but no active arc references it",
                    sheet.entity_name, sheet.commit_count
                ));
            }
        }
        issues
    }

    fn detect_stale_arcs(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let stale_threshold = 7 * 24 * 3600;
        for arc in &self.state.arcs {
            if arc.status != ArcStatus::Active {
                continue;
            }
            if let Some(last_cid) = arc.commits.last() {
                if let Some(event) = self.state.events.iter().find(|e| &e.id == last_cid) {
                    if now - event.timestamp > stale_threshold {
                        issues.push(format!(
                            "arc '{}' has no commits for >7 days (last: {})",
                            arc.name, event.description
                        ));
                    }
                }
            }
        }
        issues
    }

    pub fn get_summary(&self) -> String {
        format!(
            "NarrativeConsistencyChecker: {} events, {} arcs ({} active), {} entities tracked",
            self.state.events.len(),
            self.state.arcs.len(),
            self.state.arcs.iter().filter(|a| a.status == ArcStatus::Active).count(),
            self.state.characters.len(),
        )
    }

    pub fn event_count(&self) -> usize {
        self.state.event_count()
    }

    pub fn arc_count(&self) -> usize {
        self.state.arc_count()
    }

    pub fn character_count(&self) -> usize {
        self.state.character_count()
    }
}

impl SelfTest for NarrativeConsistencyChecker {
    fn name(&self) -> &str {
        "nt_memory_narrative_checker"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.state.events.len() > MAX_EVENTS {
            failures.push(format!("event count {} exceeds max {}", self.state.events.len(), MAX_EVENTS));
        }
        for sheet in &self.state.characters {
            if sheet.first_seen > sheet.last_modified {
                failures.push(format!(
                    "entity '{}': first_seen {} > last_modified {}",
                    sheet.entity_name, sheet.first_seen, sheet.last_modified
                ));
            }
        }
        for arc in &self.state.arcs {
            if arc.name.is_empty() {
                failures.push("arc has empty name".to_string());
            }
        }
        let total_expected = self.state.characters.len();
        let total_indexed = self.entity_index.len();
        if total_expected != total_indexed {
            failures.push(format!(
                "character count {} != entity_index size {}",
                total_expected, total_indexed
            ));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_checker_is_empty() {
        let checker = NarrativeConsistencyChecker::new();
        assert_eq!(checker.event_count(), 0);
        assert_eq!(checker.arc_count(), 0);
        assert_eq!(checker.character_count(), 0);
    }

    #[test]
    fn test_record_event_creates_entity() {
        let mut checker = NarrativeConsistencyChecker::new();
        checker.record_event(
            "abc123".into(),
            "dev1".into(),
            "add user authentication".into(),
            vec!["src/auth.rs".into(), "src/lib.rs".into()],
            CommitType::Feature,
        );
        assert_eq!(checker.event_count(), 1);
        assert_eq!(checker.character_count(), 2);
    }

    #[test]
    fn test_repeated_entity_updates_not_duplicated() {
        let mut checker = NarrativeConsistencyChecker::new();
        checker.record_event(
            "c1".into(), "dev1".into(), "first commit".into(),
            vec!["src/core.rs".into()], CommitType::Feature,
        );
        checker.record_event(
            "c2".into(), "dev1".into(), "second commit".into(),
            vec!["src/core.rs".into()], CommitType::Fix,
        );
        assert_eq!(checker.character_count(), 1);
        let sheet = &checker.state.characters[0];
        assert_eq!(sheet.commit_count, 2);
    }

    #[test]
    fn test_add_arc_and_link() {
        let mut checker = NarrativeConsistencyChecker::new();
        checker.record_event(
            "c1".into(), "dev1".into(), "initial setup".into(),
            vec!["src/main.rs".into()], CommitType::Feature,
        );
        checker.add_arc("auth".into(), "authentication system".into(), ArcStatus::Active);
        assert!(checker.link_commit_to_arc("c1", "auth").is_ok());
        assert!(checker.link_commit_to_arc("c1", "nonexistent").is_err());
    }

    #[test]
    fn test_check_consistency_finds_drift() {
        let mut checker = NarrativeConsistencyChecker::new();
        checker.add_arc("core".into(), "database engine optimization".into(), ArcStatus::Active);
        checker.record_event(
            "c1".into(), "dev1".into(), "refactor UI button colors".into(),
            vec!["src/ui.rs".into()], CommitType::Refactor,
        );
        assert!(checker.link_commit_to_arc("c1", "core").is_ok());
        let issues = checker.check_consistency();
        let has_drift = issues.iter().any(|i| i.contains("drift"));
        assert!(has_drift);
    }

    #[test]
    fn test_get_summary_format() {
        let mut checker = NarrativeConsistencyChecker::new();
        checker.record_event(
            "c1".into(), "dev1".into(), "work".into(),
            vec!["src/a.rs".into()], CommitType::Feature,
        );
        checker.add_arc("main".into(), "main work".into(), ArcStatus::Active);
        let summary = checker.get_summary();
        assert!(summary.contains("1 events"));
        assert!(summary.contains("1 arcs"));
        assert!(summary.contains("1 entities"));
    }

    #[test]
    fn test_self_test_passes_on_empty() {
        let checker = NarrativeConsistencyChecker::new();
        assert!(checker.self_test().is_ok());
    }

    #[test]
    fn test_entity_tagging_by_extension() {
        let mut checker = NarrativeConsistencyChecker::new();
        checker.record_event(
            "c1".into(), "dev1".into(), "add docs".into(),
            vec!["README.md".into(), "src/lib.rs".into(), "script.py".into()],
            CommitType::Docs,
        );
        assert_eq!(checker.character_count(), 3);
        for sheet in &checker.state.characters {
            if sheet.entity_name == "README.md" {
                assert!(sheet.tags.contains("documentation"));
            }
            if sheet.entity_name == "src/lib.rs" {
                assert!(sheet.tags.contains("rust"));
            }
            if sheet.entity_name == "script.py" {
                assert!(sheet.tags.contains("python"));
            }
        }
    }
}
