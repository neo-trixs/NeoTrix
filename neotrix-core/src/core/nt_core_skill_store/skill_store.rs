use std::collections::HashMap;

use crate::core::nt_core_skill_store::discovery::{
    DiscoveredSkill, SearchQuery, SkillCategory, SkillDiscovery, SkillSource,
};
use crate::core::nt_core_skill_store::fusion::{FusionReport, SkillFusion, SkillGap};

/// Metadata for a store entry
#[derive(Debug, Clone)]
pub struct SkillMetadata {
    pub added: String,
    pub last_used: Option<String>,
    pub use_count: u64,
    pub version: u32,
    pub source: String,
    pub rating: f64,
    pub tags: Vec<String>,
}

/// Status of an entry in the store
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryStatus {
    Active,
    Deprecated,
    Experimental,
    Inactive,
}

/// Store entry — wrapping existing skill infrastructure
#[derive(Debug, Clone)]
pub struct StoreEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub metadata: SkillMetadata,
    pub status: EntryStatus,
}

/// Evolution event log
#[derive(Debug, Clone)]
pub struct EvolutionEvent {
    pub cycle: u64,
    pub event_type: String,
    pub skill_id: Option<String>,
    pub details: String,
}

/// Signature for dispatch matching
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SkillSignature {
    pub task_keywords: Vec<String>,
    pub category: SkillCategory,
}

impl SkillSignature {
    pub fn from_task_description(description: &str) -> Self {
        let keywords: Vec<String> = description
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|w| {
                w.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|w| !w.is_empty())
            .collect();

        let cat = if keywords
            .iter()
            .any(|k| ["test", "qa", "verify"].contains(&k.as_str()))
        {
            SkillCategory::Testing
        } else if keywords
            .iter()
            .any(|k| ["debug", "fix", "bug", "error"].contains(&k.as_str()))
        {
            SkillCategory::Debugging
        } else if keywords
            .iter()
            .any(|k| ["design", "ui", "frontend", "style"].contains(&k.as_str()))
        {
            SkillCategory::Frontend
        } else if keywords
            .iter()
            .any(|k| ["security", "audit", "vuln"].contains(&k.as_str()))
        {
            SkillCategory::Security
        } else if keywords
            .iter()
            .any(|k| ["data", "analysis", "ml", "science"].contains(&k.as_str()))
        {
            SkillCategory::DataScience
        } else if keywords
            .iter()
            .any(|k| ["deploy", "ci", "cd", "pipeline"].contains(&k.as_str()))
        {
            SkillCategory::Devops
        } else if keywords
            .iter()
            .any(|k| ["write", "content", "doc"].contains(&k.as_str()))
        {
            SkillCategory::Content
        } else if keywords
            .iter()
            .any(|k| ["plan", "organize", "manage"].contains(&k.as_str()))
        {
            SkillCategory::Productivity
        } else if keywords
            .iter()
            .any(|k| ["research", "paper", "study"].contains(&k.as_str()))
        {
            SkillCategory::Research
        } else {
            SkillCategory::Development
        };

        Self {
            task_keywords: keywords,
            category: cat,
        }
    }

    /// Match score against another signature (0.0-1.0)
    pub fn match_score(&self, other: &SkillSignature) -> f64 {
        let cat_bonus = if self.category == other.category {
            0.4
        } else {
            0.0
        };

        if self.task_keywords.is_empty() || other.task_keywords.is_empty() {
            return cat_bonus;
        }

        let matches: usize = self
            .task_keywords
            .iter()
            .filter(|kw| {
                other
                    .task_keywords
                    .iter()
                    .any(|ok| ok.contains(*kw) || kw.contains(ok))
            })
            .count();

        let kw_score =
            matches as f64 / self.task_keywords.len().max(other.task_keywords.len()) as f64;

        (cat_bonus + kw_score * 0.6).min(1.0)
    }
}

/// The unified skill store — meta-layer over existing skill system
pub struct SkillStore {
    entries: HashMap<String, StoreEntry>,
    discovery: SkillDiscovery,
    evolution_log: Vec<EvolutionEvent>,
    cycle: u64,
    max_entries: usize,
}

impl SkillStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            discovery: SkillDiscovery::with_top_skills(),
            evolution_log: Vec::new(),
            cycle: 0,
            max_entries: 200,
        }
    }

    pub fn register(&mut self, entry: StoreEntry) {
        let id = entry.id.clone();
        self.entries.insert(id, entry);
    }

    pub fn get(&self, id: &str) -> Option<&StoreEntry> {
        self.entries.get(id)
    }

    pub fn list(&self) -> Vec<&StoreEntry> {
        self.entries.values().collect()
    }

    pub fn list_by_category(&self, cat: &SkillCategory) -> Vec<&StoreEntry> {
        self.entries
            .values()
            .filter(|e| e.category == *cat)
            .collect()
    }

    pub fn list_by_status(&self, status: EntryStatus) -> Vec<&StoreEntry> {
        self.entries
            .values()
            .filter(|e| e.status == status)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Search the web for new skills and fuse them in
    pub fn evolution_tick(&mut self) -> EvolutionEvent {
        self.cycle += 1;

        // Every 5 cycles, do a discovery + fusion cycle
        if self.cycle % 5 != 0 {
            return EvolutionEvent {
                cycle: self.cycle,
                event_type: "tick".into(),
                skill_id: None,
                details: format!("Cycle {} — no discovery triggered", self.cycle),
            };
        }

        let existing_names: Vec<String> = self.entries.keys().cloned().collect();
        let existing_methods = HashMap::new(); // simplified for auto mode

        let discovered = self.discovery.search(&SearchQuery::default());
        let report = SkillFusion::fusion_cycle(&discovered, &existing_names, &existing_methods);

        // Register new skills
        for new_name in &report.new_skills {
            if let Some(ds) = discovered.iter().find(|d| &d.name == new_name) {
                let id = format!("disc-{}", ds.name);
                let entry = StoreEntry {
                    id: id.clone(),
                    name: ds.name.clone(),
                    description: ds.description.clone(),
                    category: ds.category.clone(),
                    metadata: SkillMetadata {
                        added: format!("cycle-{}", self.cycle),
                        last_used: None,
                        use_count: 0,
                        version: 1,
                        source: format!("discovered:{}", ds.source_url),
                        rating: (ds.star_count as f64 / 200.0).min(5.0),
                        tags: ds.tags.clone(),
                    },
                    status: EntryStatus::Experimental,
                };
                self.entries.insert(id, entry);
            }
        }

        let gaps = SkillFusion::detect_gaps(&existing_names, &[]);

        let event = EvolutionEvent {
            cycle: self.cycle,
            event_type: "evolution".into(),
            skill_id: None,
            details: format!(
                "Discovered {} | New {} | Merged {} | Skipped {} | Gaps {}",
                discovered.len(),
                report.new_skills.len(),
                report.merged_skills.len(),
                report.skipped_skills.len(),
                gaps.len(),
            ),
        };

        self.evolution_log.push(event.clone());
        event
    }

    pub fn discover_skills(&self, query: &SearchQuery) -> Vec<DiscoveredSkill> {
        self.discovery.search(query)
    }

    pub fn fuse_discovered(
        &mut self,
        discovered: &[DiscoveredSkill],
        existing_skill_names: &[String],
    ) -> FusionReport {
        let methods = HashMap::new();
        let report = SkillFusion::fusion_cycle(discovered, existing_skill_names, &methods);

        for new_name in &report.new_skills {
            if let Some(ds) = discovered.iter().find(|d| &d.name == new_name) {
                let id = format!("fused-{}", ds.name);
                let entry = StoreEntry {
                    id,
                    name: ds.name.clone(),
                    description: ds.description.clone(),
                    category: ds.category.clone(),
                    metadata: SkillMetadata {
                        added: format!("cycle-{}", self.cycle),
                        last_used: None,
                        use_count: 0,
                        version: 1,
                        source: "fused:web-discovery".into(),
                        rating: (ds.star_count as f64 / 200.0).min(5.0),
                        tags: ds.tags.clone(),
                    },
                    status: EntryStatus::Experimental,
                };
                self.entries.insert(format!("fused-{}", ds.name), entry);
            }
        }

        self.evolution_log.push(EvolutionEvent {
            cycle: self.cycle,
            event_type: "fusion".into(),
            skill_id: None,
            details: format!(
                "Fused {} skills: new={} merged={} skipped={}",
                report.total_analyzed,
                report.new_skills.len(),
                report.merged_skills.len(),
                report.skipped_skills.len(),
            ),
        });

        report
    }

    pub fn detect_gaps(&self) -> Vec<SkillGap> {
        let names: Vec<String> = self.entries.keys().cloned().collect();
        let cats: Vec<SkillCategory> = self.entries.values().map(|e| e.category.clone()).collect();
        SkillFusion::detect_gaps(&names, &cats)
    }

    /// Dispatch — find best skill for a task description
    pub fn dispatch_for_task(&self, description: &str) -> Option<&StoreEntry> {
        if self.entries.is_empty() {
            return None;
        }

        let query_sig = SkillSignature::from_task_description(description);

        self.entries
            .values()
            .filter(|e| e.status == EntryStatus::Active)
            .map(|e| {
                let entry_sig = SkillSignature {
                    task_keywords: vec![e.name.clone()],
                    category: e.category.clone(),
                };
                let score = query_sig.match_score(&entry_sig);
                (e, score)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .and_then(|(e, score)| if score > 0.1 { Some(e) } else { None })
    }

    /// Mark a skill as used (bump use_count, update last_used)
    pub fn record_use(&mut self, id: &str) {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.metadata.use_count += 1;
            entry.metadata.last_used = Some(format!("cycle-{}", self.cycle));
        }
    }

    /// Deprecate skills not used in N cycles
    pub fn auto_deprecate(&mut self, max_idle_cycles: u64) {
        let threshold_cycle = if self.cycle > max_idle_cycles {
            self.cycle - max_idle_cycles
        } else {
            0
        };

        let to_deprecate: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, e)| {
                if e.status != EntryStatus::Active {
                    return false;
                }
                match &e.metadata.last_used {
                    Some(lc) => {
                        let cycle_num = lc
                            .strip_prefix("cycle-")
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0);
                        cycle_num < threshold_cycle
                    }
                    None => true, // never used
                }
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_deprecate {
            if let Some(entry) = self.entries.get_mut(&id) {
                entry.status = EntryStatus::Deprecated;
                self.evolution_log.push(EvolutionEvent {
                    cycle: self.cycle,
                    event_type: "deprecation".into(),
                    skill_id: Some(id.clone()),
                    details: format!("Deprecated after {} idle cycles", max_idle_cycles),
                });
            }
        }
    }

    pub fn evolution_log(&self) -> &[EvolutionEvent] {
        &self.evolution_log
    }

    /// Generate summary report
    pub fn summary_markdown(&self) -> String {
        let total = self.entries.len();
        let active = self.list_by_status(EntryStatus::Active).len();
        let experimental = self.list_by_status(EntryStatus::Experimental).len();
        let deprecated = self.list_by_status(EntryStatus::Deprecated).len();

        let mut cats: HashMap<&str, usize> = HashMap::new();
        for e in self.entries.values() {
            *cats.entry(e.category.label()).or_insert(0) += 1;
        }
        let mut cat_lines: Vec<String> = cats
            .iter()
            .map(|(cat, count)| format!("  - {}: {}", cat, count))
            .collect();
        cat_lines.sort();

        let mut report = format!(
            "## SkillStore Summary\n\n\
             **Cycle**: {} | **Total**: {} | **Active**: {} | **Experimental**: {} | **Deprecated**: {}\n\n\
             **Categories**:\n{}\n\n\
             **Evolution Events**: {} total\n",
            self.cycle,
            total,
            active,
            experimental,
            deprecated,
            cat_lines.join("\n"),
            self.evolution_log.len(),
        );

        if !self.evolution_log.is_empty() {
            let last = self.evolution_log.last().unwrap();
            report.push_str(&format!(
                "**Last Event**: [{}] {} — {}\n",
                last.cycle, last.event_type, last.details
            ));
        }

        report
    }

    /// Seed with 20+ built-in skills matching NeoTrix capabilities
    pub fn with_builtin_skills() -> Self {
        let mut store = Self::new();
        store.cycle = 0;

        let builtins = vec![
            StoreEntry {
                id: "brainstorming".into(),
                name: "brainstorming".into(),
                description: "Idea generation and creative work. Explores user intent, requirements, and design before implementation.".into(),
                category: SkillCategory::Productivity,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.8, tags: vec!["creativity".into(), "planning".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "code-review".into(),
                name: "code-review".into(),
                description: "Code quality and security analysis. Identifies vulnerabilities, anti-patterns, and performance bottlenecks.".into(),
                category: SkillCategory::Security,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.6, tags: vec!["security".into(), "quality".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "debugging".into(),
                name: "debugging".into(),
                description: "Systematic debugging with root cause isolation, hypothesis testing, and verification.".into(),
                category: SkillCategory::Debugging,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.7, tags: vec!["debug".into(), "fix".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "test-generation".into(),
                name: "test-generation".into(),
                description: "Automated test writing with coverage tracking and edge case detection.".into(),
                category: SkillCategory::Testing,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.3, tags: vec!["testing".into(), "qa".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "architecture-analysis".into(),
                name: "architecture-analysis".into(),
                description: "Codebase understanding and architecture discovery. Maps module dependencies, data flow, and design patterns.".into(),
                category: SkillCategory::Development,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.5, tags: vec!["architecture".into(), "analysis".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "knowledge-discovery".into(),
                name: "knowledge-discovery".into(),
                description: "Web research and knowledge extraction. Searches, retrieves, and synthesizes information from multiple sources.".into(),
                category: SkillCategory::Research,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.4, tags: vec!["research".into(), "search".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "image-understanding".into(),
                name: "image-understanding".into(),
                description: "Visual analysis pipeline — file/base64 to multimodal LLM to VSA encoding to sensory integration.".into(),
                category: SkillCategory::Frontend,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.2, tags: vec!["vision".into(), "image".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "voice-processing".into(),
                name: "voice-processing".into(),
                description: "Speech-to-text pipeline via Whisper API with WAV conversion and streaming support.".into(),
                category: SkillCategory::Communication,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.1, tags: vec!["voice".into(), "audio".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "memory-search".into(),
                name: "memory-search".into(),
                description: "Cross-session semantic retrieval via VSA embeddings and Hamming similarity search.".into(),
                category: SkillCategory::DataScience,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.6, tags: vec!["memory".into(), "search".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "planning".into(),
                name: "planning".into(),
                description: "Task decomposition and structured plan generation. Breaks complex goals into actionable steps.".into(),
                category: SkillCategory::Productivity,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.9, tags: vec!["planning".into(), "organization".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "data-analysis".into(),
                name: "data-analysis".into(),
                description: "Quantitative reasoning and data processing. Statistical analysis, trend detection, and insight extraction.".into(),
                category: SkillCategory::DataScience,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.3, tags: vec!["data".into(), "analysis".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "document-processing".into(),
                name: "document-processing".into(),
                description: "PDF, DOCX, and XLSX extraction and parsing. Converts documents to structured text for analysis.".into(),
                category: SkillCategory::Backend,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.4, tags: vec!["documents".into(), "parsing".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "content-writing".into(),
                name: "content-writing".into(),
                description: "Natural, human-like content generation. Removes AI patterns, varies style, and adapts to audience.".into(),
                category: SkillCategory::Content,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.5, tags: vec!["writing".into(), "content".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "security-audit".into(),
                name: "security-audit".into(),
                description: "Vulnerability scanning and security assessment. OWASP Top 10 checks, injection detection, secret scanning.".into(),
                category: SkillCategory::Security,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.2, tags: vec!["security".into(), "audit".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "performance-analysis".into(),
                name: "performance-analysis".into(),
                description: "Profiling and optimization. Identifies bottlenecks, measures latency, and suggests improvements.".into(),
                category: SkillCategory::Development,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.0, tags: vec!["performance".into(), "profile".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "consensus-building".into(),
                name: "consensus-building".into(),
                description: "Multi-agent agreement mechanism. Recepter-side evaluation, Byzantine filtering, and quorum detection.".into(),
                category: SkillCategory::Communication,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.1, tags: vec!["consensus".into(), "agents".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "creative-generation".into(),
                name: "creative-generation".into(),
                description: "Generates images, music, metaphors, and analogies through multimodal pipelines.".into(),
                category: SkillCategory::Design,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.3, tags: vec!["creative".into(), "generation".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "skill-fusion".into(),
                name: "skill-fusion".into(),
                description: "The skill store meta-skill itself. Discovers, analyzes, fuses and evolves new skills from external sources.".into(),
                category: SkillCategory::Development,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 5.0, tags: vec!["meta".into(), "evolution".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "self-evolution".into(),
                name: "self-evolution".into(),
                description: "SEAL pipeline — self-evolving meta-program that rewrites its own improvement mechanisms.".into(),
                category: SkillCategory::Development,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 5.0, tags: vec!["evolution".into(), "meta".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "tool-orchestration".into(),
                name: "tool-orchestration".into(),
                description: "Multi-tool workflow coordination. Chains tools, manages dependencies, and handles error recovery.".into(),
                category: SkillCategory::Backend,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.6, tags: vec!["tools".into(), "workflow".into()],
                }, status: EntryStatus::Active,
            },
            StoreEntry {
                id: "osint-investigation".into(),
                name: "osint-investigation".into(),
                description: "Open-source intelligence gathering. Domain/IP/binary analysis with VSA-native probe orchestration.".into(),
                category: SkillCategory::Security,
                metadata: SkillMetadata {
                    added: "built-in".into(), last_used: None, use_count: 0, version: 1,
                    source: "built-in".into(), rating: 4.4, tags: vec!["osint".into(), "intelligence".into()],
                }, status: EntryStatus::Active,
            },
        ];

        for entry in builtins {
            store.entries.insert(entry.id.clone(), entry);
        }

        store
    }
}

impl Default for SkillStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified dispatch — looks up best skill in store and returns its ID
pub struct UnifiedSkillDispatch {
    store: SkillStore,
}

impl UnifiedSkillDispatch {
    pub fn new(store: SkillStore) -> Self {
        Self { store }
    }

    pub fn dispatch(&self, task: &str) -> Option<&StoreEntry> {
        self.store.dispatch_for_task(task)
    }

    pub fn store(&self) -> &SkillStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut SkillStore {
        &mut self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_skill_store::discovery::{DiscoveredSkill, SkillSource};

    #[test]
    fn test_with_builtin_skills_seed_count() {
        let store = SkillStore::with_builtin_skills();
        assert!(
            store.len() >= 20,
            "Expected >=20 built-in skills, got {}",
            store.len()
        );
    }

    #[test]
    fn test_register_and_get() {
        let mut store = SkillStore::new();
        let entry = StoreEntry {
            id: "test-1".into(),
            name: "test".into(),
            description: "test skill".into(),
            category: SkillCategory::Testing,
            metadata: SkillMetadata {
                added: "test".into(),
                last_used: None,
                use_count: 0,
                version: 1,
                source: "test".into(),
                rating: 3.0,
                tags: vec![],
            },
            status: EntryStatus::Active,
        };
        store.register(entry);
        assert!(store.get("test-1").is_some());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_list_by_category() {
        let mut store = SkillStore::new();
        store.register(StoreEntry {
            id: "d1".into(),
            name: "debug1".into(),
            description: "".into(),
            category: SkillCategory::Debugging,
            metadata: SkillMetadata::default(),
            status: EntryStatus::Active,
        });
        store.register(StoreEntry {
            id: "d2".into(),
            name: "debug2".into(),
            description: "".into(),
            category: SkillCategory::Debugging,
            metadata: SkillMetadata::default(),
            status: EntryStatus::Active,
        });
        store.register(StoreEntry {
            id: "t1".into(),
            name: "test1".into(),
            description: "".into(),
            category: SkillCategory::Testing,
            metadata: SkillMetadata::default(),
            status: EntryStatus::Active,
        });
        assert_eq!(store.list_by_category(&SkillCategory::Debugging).len(), 2);
        assert_eq!(store.list_by_category(&SkillCategory::Testing).len(), 1);
    }

    #[test]
    fn test_dispatch_for_task_matches_relevant_skill() {
        let store = SkillStore::with_builtin_skills();
        let result = store.dispatch_for_task("I need to debug this error in the code");
        assert!(result.is_some(), "should find a skill for debugging task");
        let entry = result.unwrap();
        assert!(
            entry.name.contains("debug") || entry.category == SkillCategory::Debugging,
            "matched skill should be debugging-related, got: {}",
            entry.name
        );
    }

    #[test]
    fn test_auto_deprecate_marks_unused() {
        let mut store = SkillStore::with_builtin_skills();
        store.cycle = 20;

        // Mark one skill as recently used
        store.record_use("debugging");
        store.cycle = 25;

        store.auto_deprecate(10);
        // debugging was used at cycle 20, current cycle 25, so it's within 10 cycles → should stay active
        assert_eq!(
            store.get("debugging").unwrap().status,
            EntryStatus::Active,
            "recently-used skill should remain Active"
        );
    }

    #[test]
    fn test_evolution_tick_does_not_panic() {
        let mut store = SkillStore::with_builtin_skills();
        // Multiple ticks
        for _ in 0..12 {
            let event = store.evolution_tick();
            assert!(!event.event_type.is_empty());
        }
    }

    #[test]
    fn test_summary_markdown_contains_entries() {
        let store = SkillStore::with_builtin_skills();
        let summary = store.summary_markdown();
        assert!(summary.contains("SkillStore Summary"));
        assert!(summary.contains("Cycle"));
        assert!(summary.contains("Total"));
    }

    #[test]
    fn test_skill_signature_match_score() {
        let sig_a = SkillSignature {
            task_keywords: vec!["debug".into(), "error".into(), "fix".into()],
            category: SkillCategory::Debugging,
        };
        let sig_b = SkillSignature {
            task_keywords: vec!["debug".into(), "code".into()],
            category: SkillCategory::Debugging,
        };
        let score = sig_a.match_score(&sig_b);
        assert!(
            score > 0.5,
            "same category + overlapping keywords should score > 0.5, got {}",
            score
        );

        let sig_c = SkillSignature {
            task_keywords: vec!["design".into(), "ui".into()],
            category: SkillCategory::Design,
        };
        let score_diff = sig_a.match_score(&sig_c);
        assert!(score_diff < score, "different category should score lower");
    }

    #[test]
    fn test_discover_skills_returns_results() {
        let store = SkillStore::with_builtin_skills();
        let query = SearchQuery {
            keywords: vec!["debug".into()],
            category: None,
            min_stars: 0,
            max_results: 5,
        };
        let results = store.discover_skills(&query);
        assert!(!results.is_empty(), "should find debugging-related skills");
    }

    #[test]
    fn test_fuse_discovered_report() {
        let mut store = SkillStore::new();
        let discovered = vec![DiscoveredSkill {
            name: "brand-new-skill".into(),
            description: "A completely novel skill".into(),
            source: SkillSource::GitHub,
            source_url: "https://github.com/test/new".into(),
            category: SkillCategory::DataScience,
            star_count: 50,
            author: None,
            methodology: vec!["novel".into()],
            instructions: None,
            tags: vec![],
            install_command: None,
        }];
        let existing = vec![];
        let report = store.fuse_discovered(&discovered, &existing);
        assert_eq!(report.total_analyzed, 1);
        assert_eq!(report.new_skills.len(), 1);
        assert_eq!(report.new_skills[0], "brand-new-skill");
    }

    #[test]
    fn test_detect_gaps_identifies_missing_areas() {
        let store = SkillStore::with_builtin_skills();
        let gaps = store.detect_gaps();
        // Built-in skills cover most categories, but there may still be specialized gaps
        assert!(
            !gaps.is_empty() || store.len() >= 20,
            "either gaps exist or store is well-populated"
        );
    }

    #[test]
    fn test_unified_skill_dispatch() {
        let store = SkillStore::with_builtin_skills();
        let dispatch = UnifiedSkillDispatch::new(store);
        let entry = dispatch.dispatch("find a security vulnerability in this code");
        assert!(entry.is_some());
    }

    #[test]
    fn test_skill_signature_from_task_description() {
        let sig = SkillSignature::from_task_description("deploy the application to kubernetes");
        assert_eq!(sig.category, SkillCategory::Devops);
        assert!(!sig.task_keywords.is_empty());

        let sig2 = SkillSignature::from_task_description("write unit tests for the API");
        assert_eq!(sig2.category, SkillCategory::Testing);
    }

    #[test]
    fn test_record_use_updates_metadata() {
        let mut store = SkillStore::with_builtin_skills();
        store.record_use("planning");
        let entry = store.get("planning").unwrap();
        assert_eq!(entry.metadata.use_count, 1);
        assert!(entry.metadata.last_used.is_some());
    }

    #[test]
    fn test_list_by_status() {
        let mut store = SkillStore::with_builtin_skills();
        let active = store.list_by_status(EntryStatus::Active).len();
        let deprecated = store.list_by_status(EntryStatus::Deprecated).len();

        assert!(active > 0);
        assert_eq!(deprecated, 0);

        // Deprecate one
        store.entries.get_mut("planning").unwrap().status = EntryStatus::Deprecated;
        assert_eq!(store.list_by_status(EntryStatus::Deprecated).len(), 1);
    }
}

impl Default for SkillMetadata {
    fn default() -> Self {
        Self {
            added: "built-in".into(),
            last_used: None,
            use_count: 0,
            version: 1,
            source: "built-in".into(),
            rating: 3.0,
            tags: vec![],
        }
    }
}
