use crate::core::nt_core_time::unix_now_secs;
use crate::neotrix::nt_world_crawl::data_connector::ExternalDataConnector;
use crate::neotrix::nt_world_search::WebSearchEngine;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

const MAX_SEARCH_CACHE: usize = 1000;

fn stable_hash(s: &str) -> u64 {
    let mut h: u64 = 0xdead_beef;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h
}

// ── Profile Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntelTargetType {
    Person,
    Organization,
    Project,
    Event,
    Technology,
    Concept,
    Location,
}

impl IntelTargetType {
    pub fn name(&self) -> &'static str {
        match self {
            IntelTargetType::Person => "person",
            IntelTargetType::Organization => "organization",
            IntelTargetType::Project => "project",
            IntelTargetType::Event => "event",
            IntelTargetType::Technology => "technology",
            IntelTargetType::Concept => "concept",
            IntelTargetType::Location => "location",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelProfile {
    pub target_name: String,
    pub target_type: IntelTargetType,
    pub aliases: Vec<String>,
    pub summary: String,
    pub timeline: Vec<IntelEvent>,
    pub relationships: Vec<IntelRelation>,
    pub sources: Vec<IntelSource>,
    pub metadata: HashMap<String, String>,
    pub confidence: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelEvent {
    pub title: String,
    pub description: String,
    pub date: String,
    pub date_precision: DatePrecision,
    pub source_urls: Vec<String>,
    pub entities: Vec<String>,
    pub event_type: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatePrecision {
    Exact(i64),
    Year(i32),
    YearMonth(i32, u32),
    Approx(String),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelRelation {
    pub relation_type: String,
    pub target: String,
    pub target_type: String,
    pub description: String,
    pub confidence: f64,
    pub source_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelSource {
    pub url: String,
    pub title: String,
    pub source_type: String,
    pub relevance_score: f64,
    pub timestamp: i64,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelQuery {
    pub keywords: Vec<String>,
    pub target_type: Option<IntelTargetType>,
    pub depth: IntelDepth,
    pub max_sources: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntelDepth {
    Quick,
    Standard,
    Deep,
}

impl Default for IntelQuery {
    fn default() -> Self {
        Self {
            keywords: vec![],
            target_type: None,
            depth: IntelDepth::Standard,
            max_sources: 50,
        }
    }
}

// ── Search Result ──

#[derive(Debug, Clone)]
pub struct RawSearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub source_label: String,
    pub timestamp: i64,
}

// ── Pipeline Phases ──

pub struct IntelPipeline {
    pub profiles: HashMap<String, IntelProfile>,
    cached_searches: HashMap<u64, Vec<RawSearchResult>>,
    max_profiles: usize,
    pending_requests: VecDeque<IntelQuery>,
}

impl IntelPipeline {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            cached_searches: HashMap::new(),
            max_profiles: 100,
            pending_requests: VecDeque::new(),
        }
    }

    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    pub fn enqueue_request(&mut self, query: IntelQuery) {
        self.pending_requests.push_back(query);
    }

    pub fn process_pending(&mut self) -> usize {
        let count = self.pending_requests.len();
        while let Some(query) = self.pending_requests.pop_front() {
            self.research(query);
        }
        count
    }

    // ── Phase 1: Multi-Source Search ──

    fn generate_queries(&self, target: &str, target_type: &Option<IntelTargetType>) -> Vec<String> {
        let mut queries = vec![
            target.to_string(),
            format!(r#""{}" biography"#, target),
            format!(r#""{}" timeline"#, target),
        ];
        if let Some(tt) = target_type {
            match tt {
                IntelTargetType::Person => {
                    queries.push(format!(r#""{}" career history"#, target));
                    queries.push(format!(r#""{}" achievements"#, target));
                    queries.push(format!(r#""{}" publications"#, target));
                }
                IntelTargetType::Organization => {
                    queries.push(format!(r#""{}" founding history"#, target));
                    queries.push(format!(r#""{}" products"#, target));
                    queries.push(format!(r#""{}" funding"#, target));
                }
                IntelTargetType::Project => {
                    queries.push(format!(r#""{}" release history"#, target));
                    queries.push(format!(r#""{}" architecture"#, target));
                }
                IntelTargetType::Event => {
                    queries.push(format!(r#""{}" timeline"#, target));
                    queries.push(format!(r#""{}" impact"#, target));
                }
                _ => {}
            }
        }
        queries
    }

    fn search_web(&self, query: &str) -> Vec<RawSearchResult> {
        let engine = WebSearchEngine::default();
        match engine.search(query, 10) {
            Ok(results) if !results.is_empty() => results
                .into_iter()
                .map(|r| RawSearchResult {
                    title: r.title,
                    snippet: r.snippet,
                    url: r.url,
                    source_label: "web".into(),
                    timestamp: unix_now_secs() as i64,
                })
                .collect(),
            _ => vec![],
        }
    }

    fn collect_data_sources(&self, target: &str) -> Vec<RawSearchResult> {
        let mut all = Vec::new();
        let sources = ExternalDataConnector::collect_all();
        for record in sources {
            let text = format!("{} {}", record.title, record.summary);
            if text.to_lowercase().contains(&target.to_lowercase()) {
                all.push(RawSearchResult {
                    title: record.title,
                    snippet: record.summary,
                    url: record.url,
                    source_label: record.source_type.name().to_string(),
                    timestamp: record.timestamp,
                });
            }
        }
        all
    }

    fn phase1_search(&mut self, target: &str, query: &IntelQuery) -> Vec<RawSearchResult> {
        let cache_key = stable_hash(&format!("search:{}:{:?}", target, query.depth));
        if let Some(cached) = self.cached_searches.get(&cache_key) {
            return cached.clone();
        }

        let mut results = Vec::new();

        let queries = self.generate_queries(target, &query.target_type);
        for q in queries.iter().take(if query.depth == IntelDepth::Quick {
            2
        } else {
            5
        }) {
            let web_results = self.search_web(q);
            results.extend(web_results);
        }

        let data_results = self.collect_data_sources(target);
        results.extend(data_results);

        if query.depth == IntelDepth::Deep || query.depth == IntelDepth::Standard {
            let deeper =
                self.search_web(&format!(r#""{}" history OR biography OR profile"#, target));
            results.extend(deeper);
        }

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.dedup_by(|a, b| a.url == b.url);
        results.truncate(query.max_sources);

        self.cached_searches.insert(cache_key, results.clone());
        if self.cached_searches.len() > MAX_SEARCH_CACHE {
            let remove = self.cached_searches.len() / 5;
            let keys: Vec<u64> = self.cached_searches.keys().take(remove).copied().collect();
            for k in keys {
                self.cached_searches.remove(&k);
            }
        }
        results
    }

    // ── Phase 2: Entity & Event Extraction ──

    fn extract_events(&self, results: &[RawSearchResult], target: &str) -> Vec<IntelEvent> {
        let mut events = Vec::new();
        let target_lower = target.to_lowercase();

        for result in results {
            let text = format!("{} {}", result.title, result.snippet);
            if !text.to_lowercase().contains(&target_lower) {
                continue;
            }

            let text_lower = text.to_lowercase();

            let year_patterns: Vec<(i32, &str)> = {
                let mut years = Vec::new();
                for year in (1950..=2030).rev() {
                    let pattern = format!("{}", year);
                    if text_lower.contains(&pattern) {
                        years.push((year, &result.url as &str));
                        if years.len() >= 3 {
                            break;
                        }
                    }
                }
                years.iter().map(|(y, _)| (*y, "")).collect()
            };

            for (year, _) in &year_patterns {
                events.push(IntelEvent {
                    title: format!("{} — {}", target, result.title),
                    description: result.snippet.clone(),
                    date: year.to_string(),
                    date_precision: DatePrecision::Year(*year),
                    source_urls: vec![result.url.clone()],
                    entities: vec![target.to_string()],
                    event_type: "milestone".into(),
                    confidence: 0.5,
                });
            }

            let trigger_keywords = [
                "founded",
                "released",
                "joined",
                "published",
                "announced",
                "acquired",
                "launched",
                "created",
                "developed",
                "graduated",
                "started",
                "appointed",
                "elected",
                "awarded",
                "died",
                "born",
                "discovered",
                "invented",
            ];
            for trigger in &trigger_keywords {
                if let Some(pos) = text_lower.find(trigger) {
                    let start = pos.saturating_sub(20);
                    let end = (pos + trigger.len() + 60).min(text.len());
                    let context = &text[start..end];

                    let year_match: Option<i32> = {
                        let ctx = context.to_lowercase();
                        (1950..=2030).rev().find(|y| ctx.contains(&y.to_string()))
                    };

                    if let Some(year) = year_match {
                        events.push(IntelEvent {
                            title: format!("{} — {}", target, trigger),
                            description: context.to_string(),
                            date: year.to_string(),
                            date_precision: DatePrecision::Year(year),
                            source_urls: vec![result.url.clone()],
                            entities: vec![target.to_string()],
                            event_type: trigger.to_string(),
                            confidence: 0.6,
                        });
                    }
                }
            }
        }

        events.sort_by(|a, b| a.date.cmp(&b.date));
        events.dedup_by(|a, b| a.title == b.title && a.date == b.date);
        events
    }

    fn extract_relationships(
        &self,
        results: &[RawSearchResult],
        target: &str,
    ) -> Vec<IntelRelation> {
        let mut relations = Vec::new();
        let target_lower = target.to_lowercase();

        let known_orgs = [
            "Google",
            "Microsoft",
            "OpenAI",
            "Apple",
            "Amazon",
            "Meta",
            "Anthropic",
            "DeepMind",
            "Nvidia",
            "IBM",
            "Intel",
            "Tesla",
            "SpaceX",
            "GitHub",
            "Stanford",
            "MIT",
            "Harvard",
            "Oxford",
            "Cambridge",
        ];

        for result in results {
            let text = format!("{} {}", result.title, result.snippet);
            let text_lower = text.to_lowercase();
            if !text_lower.contains(&target_lower) {
                continue;
            }

            for &org in &known_orgs {
                if text_lower.contains(&org.to_lowercase())
                    && !text_lower.contains(&format!(
                        "{} {}",
                        org.to_lowercase(),
                        org.to_lowercase()
                    ))
                {
                    let relation_type = if text_lower.contains("works at")
                        || text_lower.contains("employed")
                        || text_lower.contains("joins")
                        || text_lower.contains("joined")
                    {
                        "works_at"
                    } else if text_lower.contains("partnership")
                        || text_lower.contains("collaborat")
                    {
                        "collaborates_with"
                    } else if text_lower.contains("funded") || text_lower.contains("invest") {
                        "funded_by"
                    } else if text_lower.contains("acquired") {
                        "acquired_by"
                    } else {
                        "related_to"
                    };
                    relations.push(IntelRelation {
                        relation_type: relation_type.to_string(),
                        target: org.to_string(),
                        target_type: "organization".into(),
                        description: result.snippet.clone(),
                        confidence: 0.4,
                        source_urls: vec![result.url.clone()],
                    });
                }
            }
        }
        relations
    }

    // ── Phase 3: Timeline Construction ──

    fn phase3_timeline(&self, events: &[IntelEvent]) -> Vec<IntelEvent> {
        let mut sorted = events.to_vec();
        sorted.sort_by(|a, b| {
            let date_cmp = a.date.cmp(&b.date);
            if date_cmp != std::cmp::Ordering::Equal {
                return date_cmp;
            }
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut timeline = Vec::new();
        let mut seen = HashSet::new();
        for event in sorted {
            let key = format!("{}|{}", event.title, event.date);
            if seen.insert(key) {
                timeline.push(event.clone());
            }
        }
        timeline
    }

    // ── Phase 4: Dossier Generation ──

    fn generate_summary(
        &self,
        target: &str,
        target_type: &IntelTargetType,
        events: &[IntelEvent],
    ) -> String {
        let type_label = target_type.name();
        let event_count = events.len();
        let year_range = if events.is_empty() {
            "unknown".to_string()
        } else {
            let first = events.first().map(|e| e.date.as_str()).unwrap_or("");
            let last = events.last().map(|e| e.date.as_str()).unwrap_or("");
            if first == last {
                format!("{}", first)
            } else {
                format!("{} — {}", first, last)
            }
        };

        format!(
            "Intelligence profile for {} ({}). Timeline spans {} with {} documented events.",
            target, type_label, year_range, event_count
        )
    }

    fn compute_confidence(&self, sources: &[IntelSource], events: &[IntelEvent]) -> f64 {
        if sources.is_empty() {
            return 0.0;
        }
        let source_diversity: HashSet<&str> =
            sources.iter().map(|s| s.source_type.as_str()).collect();
        let diversity_score = (source_diversity.len() as f64 / 5.0).min(1.0);
        let recency = {
            let now = unix_now_secs() as i64;
            let avg_age: f64 = sources
                .iter()
                .map(|s| (now - s.timestamp).max(0) as f64)
                .sum::<f64>()
                / sources.len().max(1) as f64;
            if avg_age < 86400.0 {
                1.0
            } else if avg_age < 604800.0 {
                0.9
            } else if avg_age < 2592000.0 {
                0.7
            } else if avg_age < 31536000.0 {
                0.5
            } else {
                0.3
            }
        };
        let event_density = (events.len() as f64 / 10.0).min(1.0);
        0.4 * diversity_score
            + 0.3 * recency
            + 0.2 * event_density
            + 0.1 * (sources.len() as f64 / 30.0).min(1.0)
    }

    // ── Phase 5: Persistent Storage ──

    pub fn store_profile(&mut self, profile: IntelProfile) {
        let key = profile.target_name.clone();
        if self.profiles.len() >= self.max_profiles {
            if let Some(oldest_key) = self.profiles.keys().next().cloned() {
                self.profiles.remove(&oldest_key);
            }
        }
        self.profiles.insert(key, profile);
    }

    pub fn get_profile(&self, target: &str) -> Option<&IntelProfile> {
        self.profiles.get(target)
    }

    // ── Main Pipeline ──

    pub fn research(&mut self, query: IntelQuery) -> IntelProfile {
        let target = query.keywords.join(" ");
        let target_type = query.target_type.clone().unwrap_or(IntelTargetType::Person);

        let sources_raw = self.phase1_search(&target, &query);
        let sources: Vec<IntelSource> = sources_raw
            .iter()
            .map(|r| IntelSource {
                url: r.url.clone(),
                title: r.title.clone(),
                source_type: r.source_label.clone(),
                relevance_score: 0.5,
                timestamp: r.timestamp,
                snippet: r.snippet.clone(),
            })
            .collect();

        let events = self.extract_events(&sources_raw, &target);
        let relationships = self.extract_relationships(&sources_raw, &target);
        let timeline = self.phase3_timeline(&events);
        let summary = self.generate_summary(&target, &target_type, &timeline);
        let confidence = self.compute_confidence(&sources, &timeline);

        let now = unix_now_secs() as i64;

        let existing = self.profiles.get(&target);
        let profile = IntelProfile {
            target_name: target.clone(),
            target_type,
            aliases: query.keywords.clone(),
            summary,
            timeline,
            relationships,
            sources,
            metadata: HashMap::new(),
            confidence,
            created_at: existing.map(|p| p.created_at).unwrap_or(now),
            updated_at: now,
        };

        self.store_profile(profile.clone());
        profile
    }

    pub fn research_person(&mut self, name: &str) -> IntelProfile {
        self.research(IntelQuery {
            keywords: vec![name.to_string()],
            target_type: Some(IntelTargetType::Person),
            depth: IntelDepth::Standard,
            max_sources: 50,
        })
    }

    pub fn research_organization(&mut self, name: &str) -> IntelProfile {
        self.research(IntelQuery {
            keywords: vec![name.to_string()],
            target_type: Some(IntelTargetType::Organization),
            depth: IntelDepth::Standard,
            max_sources: 50,
        })
    }

    pub fn research_project(&mut self, name: &str) -> IntelProfile {
        self.research(IntelQuery {
            keywords: vec![name.to_string()],
            target_type: Some(IntelTargetType::Project),
            depth: IntelDepth::Standard,
            max_sources: 50,
        })
    }

    pub fn research_deep(
        &mut self,
        keywords: Vec<String>,
        target_type: Option<IntelTargetType>,
    ) -> IntelProfile {
        self.research(IntelQuery {
            keywords,
            target_type,
            depth: IntelDepth::Deep,
            max_sources: 100,
        })
    }

    /// Run the most recent profile's events through the TruthPipeline.
    /// Only events with confidence > 0.3 are checked.
    /// Returns a Vec of (event_index, TruthEstimate) for correlation.
    pub fn verify_events(
        &self,
        truth: &mut crate::core::nt_core_truth::pipeline::TruthPipeline,
    ) -> Vec<(usize, crate::core::nt_core_truth::pipeline::TruthEstimate)> {
        let profile = match self.profiles.values().max_by_key(|pr| pr.updated_at) {
            Some(p) => p,
            None => return vec![],
        };

        profile
            .timeline
            .iter()
            .enumerate()
            .filter(|(_, e)| e.confidence > 0.3)
            .map(|(i, e)| {
                let claim = format!("{}: {}", e.title, e.description);
                let source = format!("intel_profile:{}|event:{}", profile.target_name, i);
                (i, truth.quick_check(&claim, &source))
            })
            .collect()
    }

    pub fn format_dossier(&self, profile: &IntelProfile) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "# Intelligence Profile: {}\n\n",
            profile.target_name
        ));
        output.push_str(&format!("**Type:** {}\n", profile.target_type.name()));
        output.push_str(&format!(
            "**Confidence:** {:.1}%\n",
            profile.confidence * 100.0
        ));
        output.push_str(&format!("**Updated:** {}\n\n", profile.updated_at));
        output.push_str(&format!("## Summary\n\n{}\n\n", profile.summary));

        if !profile.aliases.is_empty() {
            output.push_str("## Aliases\n\n");
            for alias in &profile.aliases {
                output.push_str(&format!("- {}\n", alias));
            }
            output.push_str("\n");
        }

        if !profile.timeline.is_empty() {
            output.push_str("## Timeline\n\n");
            for event in &profile.timeline {
                output.push_str(&format!("### {} ({})\n", event.title, event.date));
                output.push_str(&format!("{}\n\n", event.description));
                output.push_str(&format!("Confidence: {:.0}%\n", event.confidence * 100.0));
                if !event.source_urls.is_empty() {
                    output.push_str("Sources:\n");
                    for url in &event.source_urls {
                        output.push_str(&format!("- {}\n", url));
                    }
                }
                output.push_str("\n");
            }
        }

        if !profile.relationships.is_empty() {
            output.push_str("## Relationships\n\n");
            for rel in &profile.relationships {
                output.push_str(&format!(
                    "- **{}** → [{}] {}: {}\n",
                    rel.relation_type, rel.target_type, rel.target, rel.description
                ));
            }
            output.push_str("\n");
        }

        if !profile.sources.is_empty() {
            output.push_str("## Sources\n\n");
            for source in &profile.sources {
                output.push_str(&format!(
                    "- [{}]({}) — {} (relevance: {:.0}%)\n",
                    source.title,
                    source.url,
                    source.source_type,
                    source.relevance_score * 100.0
                ));
            }
            output.push_str("\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── IntelPipeline core ──

    #[test]
    fn test_new_pipeline() {
        let pipe = IntelPipeline::new();
        assert_eq!(pipe.profile_count(), 0);
        assert_eq!(pipe.max_profiles, 100);
        assert!(pipe.pending_requests.is_empty());
        assert!(pipe.cached_searches.is_empty());
    }

    #[test]
    fn test_enqueue_request() {
        let mut pipe = IntelPipeline::new();
        let query = IntelQuery {
            keywords: vec!["test".into()],
            target_type: Some(IntelTargetType::Person),
            depth: IntelDepth::Quick,
            max_sources: 5,
        };
        pipe.enqueue_request(query);
        assert_eq!(pipe.pending_requests.len(), 1);
    }

    // ── IntelTargetType ──

    #[test]
    fn test_target_type_has_seven_variants() {
        let variants = [
            IntelTargetType::Person,
            IntelTargetType::Organization,
            IntelTargetType::Project,
            IntelTargetType::Event,
            IntelTargetType::Technology,
            IntelTargetType::Concept,
            IntelTargetType::Location,
        ];
        assert_eq!(variants.len(), 7);
    }

    #[test]
    fn test_target_type_name_mapping() {
        assert_eq!(IntelTargetType::Person.name(), "person");
        assert_eq!(IntelTargetType::Organization.name(), "organization");
        assert_eq!(IntelTargetType::Project.name(), "project");
        assert_eq!(IntelTargetType::Event.name(), "event");
        assert_eq!(IntelTargetType::Technology.name(), "technology");
        assert_eq!(IntelTargetType::Concept.name(), "concept");
        assert_eq!(IntelTargetType::Location.name(), "location");
    }

    // ── IntelDepth + IntelQuery default ──

    #[test]
    fn test_intel_depth_variants() {
        assert_eq!(IntelDepth::Quick, IntelDepth::Quick);
        assert_eq!(IntelDepth::Standard, IntelDepth::Standard);
        assert_eq!(IntelDepth::Deep, IntelDepth::Deep);
    }

    #[test]
    fn test_intel_query_default() {
        let q = IntelQuery::default();
        assert!(q.keywords.is_empty());
        assert!(q.target_type.is_none());
        assert_eq!(q.depth, IntelDepth::Standard);
        assert_eq!(q.max_sources, 50);
    }

    // ── generate_queries (Phase 1 logic, no network) ──

    #[test]
    fn test_generate_queries_base() {
        let pipe = IntelPipeline::new();
        let queries = pipe.generate_queries("Target", &None);
        assert_eq!(queries.len(), 3);
        assert!(queries.iter().any(|q| q.contains("Target")));
        assert!(queries.iter().any(|q| q.contains("biography")));
        assert!(queries.iter().any(|q| q.contains("timeline")));
    }

    #[test]
    fn test_generate_queries_person() {
        let pipe = IntelPipeline::new();
        let queries = pipe.generate_queries("Einstein", &Some(IntelTargetType::Person));
        assert!(queries.len() >= 6);
        assert!(queries.iter().any(|q| q.contains("career history")));
        assert!(queries.iter().any(|q| q.contains("achievements")));
        assert!(queries.iter().any(|q| q.contains("publications")));
    }

    #[test]
    fn test_generate_queries_organization() {
        let pipe = IntelPipeline::new();
        let queries = pipe.generate_queries("OpenAI", &Some(IntelTargetType::Organization));
        assert!(queries.len() >= 6);
        assert!(queries.iter().any(|q| q.contains("founding history")));
        assert!(queries.iter().any(|q| q.contains("products")));
        assert!(queries.iter().any(|q| q.contains("funding")));
    }

    #[test]
    fn test_generate_queries_project() {
        let pipe = IntelPipeline::new();
        let queries = pipe.generate_queries("NeoTrix", &Some(IntelTargetType::Project));
        assert!(queries.len() >= 5);
        assert!(queries.iter().any(|q| q.contains("release history")));
        assert!(queries.iter().any(|q| q.contains("architecture")));
    }

    #[test]
    fn test_generate_queries_event() {
        let pipe = IntelPipeline::new();
        let queries = pipe.generate_queries("WWDC", &Some(IntelTargetType::Event));
        assert!(queries.len() >= 5);
        assert!(queries.iter().any(|q| q.contains("impact")));
    }

    // ── stable_hash ──

    #[test]
    fn test_stable_hash_deterministic() {
        assert_eq!(stable_hash("hello"), stable_hash("hello"));
    }

    #[test]
    fn test_stable_hash_different_inputs_diverge() {
        assert_ne!(stable_hash("hello"), stable_hash("world"));
    }

    // ── Phase 2: Event extraction ──

    fn make_result(title: &str, snippet: &str, url: &str) -> RawSearchResult {
        RawSearchResult {
            title: title.into(),
            snippet: snippet.into(),
            url: url.into(),
            source_label: "web".into(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_extract_events_finds_year_in_snippet() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "Nobel win",
            "Albert Einstein won the Nobel Prize in Physics in 1921.",
            "http://ex.com/einstein",
        )];
        let events = pipe.extract_events(&results, "Einstein");
        assert!(
            events.iter().any(|e| e.date == "1921"),
            "should extract year 1921"
        );
    }

    #[test]
    fn test_extract_events_trigger_keyword_founded() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "OpenAI founded",
            "OpenAI was founded in 2015 by Sam Altman and Elon Musk.",
            "http://ex.com/openai",
        )];
        let events = pipe.extract_events(&results, "OpenAI");
        let founded: Vec<&IntelEvent> = events
            .iter()
            .filter(|e| e.event_type == "founded")
            .collect();
        assert!(!founded.is_empty(), "should detect 'founded' trigger");
        assert_eq!(founded[0].date, "2015");
    }

    #[test]
    fn test_extract_events_trigger_keyword_launched() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "Product launch",
            "The company launched its first product in 2020.",
            "http://ex.com/launch",
        )];
        let events = pipe.extract_events(&results, "company");
        let launched: Vec<&IntelEvent> = events
            .iter()
            .filter(|e| e.event_type == "launched")
            .collect();
        assert!(!launched.is_empty(), "should detect 'launched' trigger");
        assert_eq!(launched[0].date, "2020");
    }

    #[test]
    fn test_extract_events_skips_unrelated_target() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "Unrelated",
            "This text does not contain the target name at all.",
            "http://ex.com/x",
        )];
        assert!(pipe.extract_events(&results, "GhostTarget").is_empty());
    }

    #[test]
    fn test_extract_events_empty_results() {
        let pipe = IntelPipeline::new();
        assert!(pipe.extract_events(&[], "anything").is_empty());
    }

    // ── Phase 2: Relationship extraction ──

    #[test]
    fn test_extract_relationships_works_at() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "John joins",
            "John Smith joins Microsoft as a senior engineer in 2023.",
            "http://ex.com/john",
        )];
        let rels = pipe.extract_relationships(&results, "John");
        assert!(rels
            .iter()
            .any(|r| r.target == "Microsoft" && r.relation_type == "works_at"));
    }

    #[test]
    fn test_extract_relationships_collaborates() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "Partnership",
            "Google announced a partnership with Stanford for AI research.",
            "http://ex.com/partnership",
        )];
        let rels = pipe.extract_relationships(&results, "Google");
        assert!(rels.iter().any(|r| r.relation_type == "collaborates_with"));
    }

    #[test]
    fn test_extract_relationships_funded() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "Funding round",
            "OpenAI was funded by Microsoft with a billion-dollar investment.",
            "http://ex.com/funding",
        )];
        let rels = pipe.extract_relationships(&results, "OpenAI");
        assert!(rels.iter().any(|r| r.relation_type == "funded_by"));
    }

    #[test]
    fn test_extract_relationships_acquired() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "Acquisition",
            "Microsoft has acquired GitHub for $7.5 billion.",
            "http://ex.com/acquired",
        )];
        let rels = pipe.extract_relationships(&results, "Microsoft");
        assert!(rels.iter().any(|r| r.relation_type == "acquired_by"));
    }

    #[test]
    fn test_extract_relationships_no_known_org_returns_empty() {
        let pipe = IntelPipeline::new();
        let results = vec![make_result(
            "Small startup",
            "A tiny unknown startup raised a seed round.",
            "http://ex.com/unknown",
        )];
        assert!(pipe.extract_relationships(&results, "startup").is_empty());
    }

    // ── Phase 3: Timeline construction ──

    fn make_event(title: &str, date: &str, year: i32, confidence: f64) -> IntelEvent {
        IntelEvent {
            title: title.into(),
            description: "".into(),
            date: date.into(),
            date_precision: DatePrecision::Year(year),
            source_urls: vec![],
            entities: vec![],
            event_type: "milestone".into(),
            confidence,
        }
    }

    #[test]
    fn test_phase3_timeline_sorts_chronologically() {
        let pipe = IntelPipeline::new();
        let events = vec![
            make_event("Late", "2025", 2025, 0.5),
            make_event("Early", "2015", 2015, 0.5),
            make_event("Middle", "2020", 2020, 0.5),
        ];
        let tl = pipe.phase3_timeline(&events);
        assert_eq!(tl.len(), 3);
        assert_eq!(tl[0].date, "2015");
        assert_eq!(tl[1].date, "2020");
        assert_eq!(tl[2].date, "2025");
    }

    #[test]
    fn test_phase3_timeline_dedup_same_title_and_date() {
        let pipe = IntelPipeline::new();
        let events = vec![
            make_event("Duplicate", "2020", 2020, 0.4),
            make_event("Duplicate", "2020", 2020, 0.9),
        ];
        let tl = pipe.phase3_timeline(&events);
        assert_eq!(tl.len(), 1, "same title+date should deduplicate");
        assert!(
            (tl[0].confidence - 0.9).abs() < 1e-6,
            "higher confidence should win"
        );
    }

    #[test]
    fn test_phase3_timeline_preserves_different_dates() {
        let pipe = IntelPipeline::new();
        let events = vec![
            make_event("Same Title", "2020", 2020, 0.5),
            make_event("Same Title", "2021", 2021, 0.5),
        ];
        assert_eq!(
            pipe.phase3_timeline(&events).len(),
            2,
            "same title different date = unique"
        );
    }

    #[test]
    fn test_phase3_timeline_empty() {
        let pipe = IntelPipeline::new();
        assert!(pipe.phase3_timeline(&[]).is_empty());
    }

    // ── Phase 4 helpers ──

    #[test]
    fn test_generate_summary_with_events() {
        let pipe = IntelPipeline::new();
        let events = vec![
            make_event("Born", "1920", 1920, 1.0),
            make_event("Died", "2000", 2000, 1.0),
        ];
        let s = pipe.generate_summary("Test Person", &IntelTargetType::Person, &events);
        assert!(s.contains("Test Person"));
        assert!(s.contains("person"));
        assert!(s.contains("1920"));
        assert!(s.contains("2000"));
        assert!(s.contains("2 documented events"));
    }

    #[test]
    fn test_generate_summary_single_event_shows_same_date() {
        let pipe = IntelPipeline::new();
        let events = vec![make_event("Only", "1999", 1999, 1.0)];
        let s = pipe.generate_summary("X", &IntelTargetType::Concept, &events);
        assert!(
            s.contains("1999"),
            "single event: first==last, should show the single year"
        );
    }

    #[test]
    fn test_generate_summary_empty_events_shows_unknown_range() {
        let pipe = IntelPipeline::new();
        let s = pipe.generate_summary("Empty", &IntelTargetType::Organization, &[]);
        assert!(s.contains("unknown"));
        assert!(s.contains("0 documented events"));
    }

    #[test]
    fn test_compute_confidence_no_sources_is_zero() {
        let pipe = IntelPipeline::new();
        assert_eq!(pipe.compute_confidence(&[], &[]), 0.0);
    }

    #[test]
    fn test_compute_confidence_increases_with_diverse_sources() {
        let pipe = IntelPipeline::new();
        let make_src = |st: &str, ts: i64| IntelSource {
            url: format!("http://{}.ex", st),
            title: "".into(),
            source_type: st.into(),
            relevance_score: 0.5,
            timestamp: ts,
            snippet: "".into(),
        };
        let low = pipe.compute_confidence(&[make_src("web", 0)], &[]);
        let high = pipe.compute_confidence(
            &[
                make_src("web", 0),
                make_src("hn", 0),
                make_src("arxiv", 0),
                make_src("github", 0),
            ],
            &[make_event("E", "2020", 2020, 1.0)],
        );
        assert!(
            high > low,
            "more diverse sources + events should increase confidence"
        );
    }

    // ── Dossier generation ──

    #[test]
    fn test_format_dossier_full() {
        let pipe = IntelPipeline::new();
        let profile = IntelProfile {
            target_name: "Test".into(),
            target_type: IntelTargetType::Person,
            aliases: vec!["T".into()],
            summary: "A profile.".into(),
            timeline: vec![IntelEvent {
                title: "Event1".into(),
                description: "Desc1".into(),
                date: "2020".into(),
                date_precision: DatePrecision::Year(2020),
                source_urls: vec!["http://src.com".into()],
                entities: vec![],
                event_type: "milestone".into(),
                confidence: 0.75,
            }],
            relationships: vec![IntelRelation {
                relation_type: "works_at".into(),
                target: "ACME".into(),
                target_type: "organization".into(),
                description: "Works at ACME".into(),
                confidence: 0.5,
                source_urls: vec![],
            }],
            sources: vec![IntelSource {
                url: "http://ex.com".into(),
                title: "Source1".into(),
                source_type: "web".into(),
                relevance_score: 0.8,
                timestamp: 0,
                snippet: "".into(),
            }],
            metadata: HashMap::new(),
            confidence: 0.75,
            created_at: 0,
            updated_at: 1,
        };
        let d = pipe.format_dossier(&profile);
        assert!(d.contains("Intelligence Profile: Test"));
        assert!(d.contains("Aliases"));
        assert!(d.contains("- T"));
        assert!(d.contains("Timeline"));
        assert!(d.contains("Event1 (2020)"));
        assert!(d.contains("Relationships"));
        assert!(d.contains("works_at"));
        assert!(d.contains("Sources"));
        assert!(d.contains("Source1"));
        assert!(d.contains("Confidence: 75.0%"));
    }

    #[test]
    fn test_format_dossier_empty_sections_omitted() {
        let pipe = IntelPipeline::new();
        let profile = IntelProfile {
            target_name: "Empty".into(),
            target_type: IntelTargetType::Concept,
            aliases: vec![],
            summary: "Nothing.".into(),
            timeline: vec![],
            relationships: vec![],
            sources: vec![],
            metadata: HashMap::new(),
            confidence: 0.0,
            created_at: 0,
            updated_at: 0,
        };
        let d = pipe.format_dossier(&profile);
        assert!(d.contains("Intelligence Profile: Empty"));
        assert!(!d.contains("Aliases"), "should omit aliases section");
        assert!(!d.contains("Timeline"), "should omit timeline section");
        assert!(
            !d.contains("Relationships"),
            "should omit relationships section"
        );
        assert!(!d.contains("Sources"), "should omit sources section");
    }

    // ── Profile storage & boundedness ──

    fn make_profile(name: &str, ts: i64) -> IntelProfile {
        IntelProfile {
            target_name: name.into(),
            target_type: IntelTargetType::Person,
            aliases: vec![],
            summary: "".into(),
            timeline: vec![],
            relationships: vec![],
            sources: vec![],
            metadata: HashMap::new(),
            confidence: 0.5,
            created_at: ts,
            updated_at: ts,
        }
    }

    #[test]
    fn test_store_and_retrieve_profile() {
        let mut pipe = IntelPipeline::new();
        pipe.store_profile(make_profile("Alice", 100));
        assert_eq!(pipe.profile_count(), 1);
        assert!(pipe.get_profile("Alice").is_some());
        assert!(pipe.get_profile("Bob").is_none());
    }

    #[test]
    fn test_store_overwrite_same_key() {
        let mut pipe = IntelPipeline::new();
        pipe.store_profile(make_profile("X", 10));
        pipe.store_profile(make_profile("X", 20));
        assert_eq!(pipe.profile_count(), 1, "same key overwrites");
        assert_eq!(pipe.get_profile("X").unwrap().updated_at, 20);
    }

    #[test]
    fn test_store_profile_evicts_oldest_when_at_capacity() {
        let mut pipe = IntelPipeline::new();
        pipe.max_profiles = 3;
        for i in 0..5 {
            pipe.store_profile(make_profile(&format!("P{}", i), i));
        }
        assert_eq!(pipe.profile_count(), 3);
        assert!(pipe.get_profile("P0").is_none(), "oldest should be evicted");
        assert!(
            pipe.get_profile("P1").is_none(),
            "second oldest should be evicted"
        );
        assert!(pipe.get_profile("P2").is_some());
        assert!(pipe.get_profile("P3").is_some());
        assert!(pipe.get_profile("P4").is_some());
    }

    #[test]
    fn test_store_profile_within_capacity_keeps_all() {
        let mut pipe = IntelPipeline::new();
        pipe.max_profiles = 3;
        pipe.store_profile(make_profile("A", 0));
        pipe.store_profile(make_profile("B", 1));
        assert_eq!(pipe.profile_count(), 2);
    }

    // ── Cache boundedness ──

    #[test]
    fn test_phase1_search_cache_hit_returns_cached() {
        let mut pipe = IntelPipeline::new();
        let key = stable_hash("search:target:Quick");
        let cached = vec![make_result("Cached", "", "http://cached.com")];
        pipe.cached_searches.insert(key, cached.clone());

        let query = IntelQuery {
            keywords: vec!["target".into()],
            target_type: None,
            depth: IntelDepth::Quick,
            max_sources: 5,
        };
        let results = pipe.phase1_search("target", &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Cached");
    }

    #[test]
    fn test_cache_eviction_on_too_many_entries() {
        let mut pipe = IntelPipeline::new();
        // Fill cache to MAX + 5 so the next insert triggers eviction
        for i in 0..(MAX_SEARCH_CACHE + 5) {
            let k = stable_hash(&format!("miss:{}", i));
            pipe.cached_searches.insert(k, vec![]);
        }
        assert!(
            pipe.cached_searches.len() > MAX_SEARCH_CACHE,
            "should be over limit before a new search"
        );

        // Trigger a cache miss via phase1_search for a new target
        let query = IntelQuery {
            keywords: vec!["evictiontest".into()],
            target_type: None,
            depth: IntelDepth::Quick,
            max_sources: 5,
        };
        // This will be a cache miss (different hash), calls search_web which returns []
        // then inserts [] and triggers eviction
        let _ = pipe.phase1_search("evictiontest", &query);
        assert!(
            pipe.cached_searches.len() <= MAX_SEARCH_CACHE + 5,
            "eviction should keep cache near MAX"
        );
    }

    // ── DatePrecision variants ──

    #[test]
    fn test_date_precision_all_variants_constructible() {
        let _ = DatePrecision::Exact(1_700_000_000);
        let _ = DatePrecision::Year(2024);
        let _ = DatePrecision::YearMonth(2024, 6);
        let _ = DatePrecision::Approx("circa 2020".into());
        let _ = DatePrecision::Unknown;
    }

    // ─── research convenience methods (no-network smoke) ───

    #[test]
    fn test_research_person_constructs_profile() {
        let mut pipe = IntelPipeline::new();
        let profile = pipe.research_person("NonExistentTestTarget42");
        assert_eq!(profile.target_name, "NonExistentTestTarget42");
        assert!(matches!(profile.target_type, IntelTargetType::Person));
        // Confidence may be 0 (no sources found via network), that's fine
    }

    #[test]
    fn test_research_organization_constructs_profile() {
        let mut pipe = IntelPipeline::new();
        let profile = pipe.research_organization("NonExistentOrg42");
        assert_eq!(profile.target_name, "NonExistentOrg42");
        assert!(matches!(profile.target_type, IntelTargetType::Organization));
    }

    #[test]
    fn test_research_project_constructs_profile() {
        let mut pipe = IntelPipeline::new();
        let profile = pipe.research_project("NonExistentProject42");
        assert!(matches!(profile.target_type, IntelTargetType::Project));
    }

    #[test]
    fn test_research_deep_uses_deep_depth() {
        let mut pipe = IntelPipeline::new();
        let profile = pipe.research_deep(
            vec!["deep".into(), "test".into()],
            Some(IntelTargetType::Technology),
        );
        assert!(matches!(profile.target_type, IntelTargetType::Technology));
        assert!(profile.target_name.contains("deep"));
    }

    // ── Edge cases ──

    #[test]
    fn test_profile_with_all_empty_collections() {
        let pipe = IntelPipeline::new();
        let profile = make_profile("EmptyTest", 0);
        let d = pipe.format_dossier(&profile);
        assert!(d.contains("EmptyTest"));
        assert!(!d.contains("Aliases"));
        assert!(!d.contains("Timeline"));
    }

    #[test]
    fn test_pipeline_new_uses_default_max_profiles() {
        let pipe = IntelPipeline::new();
        assert_eq!(pipe.max_profiles, 100);
    }
}
