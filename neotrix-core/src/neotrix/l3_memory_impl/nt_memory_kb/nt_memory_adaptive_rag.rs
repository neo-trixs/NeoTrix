use std::sync::RwLock;

use super::nt_memory_types::*;

/// Query complexity categories for adaptive retrieval routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryComplexity {
    Simple,
    Medium,
    Hard,
    Reject,
}

/// Result of the retrieval evaluator (CRAG-style relevance grading)
#[derive(Debug, Clone)]
pub struct GradedDocument {
    pub node_id: String,
    pub relevance: RelevanceGrade,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelevanceGrade {
    Relevant,
    Partial,
    Irrelevant,
}

/// Configuration for the adaptive RAG pipeline
pub struct AdaptiveRagConfig {
    pub fts_limit_multiplier: usize,
    pub rerank_weight_fts: f64,
    pub rerank_weight_embed: f64,
    pub fuse_weights: [f64; 4],
    pub cache_ttl_secs: u64,
    pub max_iterations: usize,
    pub web_search_fallback: bool,
}

impl Default for AdaptiveRagConfig {
    fn default() -> Self {
        Self {
            fts_limit_multiplier: 3,
            rerank_weight_fts: 0.3,
            rerank_weight_embed: 0.7,
            fuse_weights: [0.25, 0.15, 0.40, 0.20],
            cache_ttl_secs: 30,
            max_iterations: 3,
            web_search_fallback: true,
        }
    }
}

/// Adaptive RAG controller attached to KnowledgeBase
pub struct AdaptiveRetrieval {
    pub config: AdaptiveRagConfig,
    complexity_cache: RwLock<lru::LruCache<String, (QueryComplexity, f64)>>,
}

impl AdaptiveRetrieval {
    pub fn new(config: AdaptiveRagConfig) -> Self {
        Self {
            config,
            complexity_cache: RwLock::new(
                lru::LruCache::new(std::num::NonZeroUsize::new(200).expect("non-zero cache capacity")),
            ),
        }
    }

    /// Classify query complexity using lightweight heuristics.
    /// Returns (complexity, confidence).
    pub fn classify_query(&self, query: &str) -> (QueryComplexity, f64) {
        if let Ok(mut cache) = self.complexity_cache.write() {
            if let Some(cached) = cache.get(query) {
                return *cached;
            }
        }

        let complexity = self.heuristic_classify(query);
        let confidence = match complexity {
            QueryComplexity::Simple => 0.85,
            QueryComplexity::Medium => 0.70,
            QueryComplexity::Hard => 0.60,
            QueryComplexity::Reject => 0.50,
        };

        if let Ok(mut cache) = self.complexity_cache.write() {
            cache.put(query.to_string(), (complexity, confidence));
        }

        (complexity, confidence)
    }

    /// Heuristic classifier: entity count + comparison markers + question depth
    fn heuristic_classify(&self, query: &str) -> QueryComplexity {
        let q = query.to_lowercase();

        // Reject: gibberish, too short, or obviously unanswerable
        if q.len() < 5 || q.chars().filter(|c| c.is_alphabetic()).count() < 3 {
            return QueryComplexity::Reject;
        }

        let entity_count = self.count_entities(query);
        let has_comparison = [" vs ", " versus ", " compare ", " difference "]
            .iter().any(|w| q.contains(w) || q.starts_with(w.trim()));
        let has_analysis = [" why ", " how does ", " what causes ", " explain "]
            .iter().any(|w| q.contains(w));
        let has_multi_hop = [" and ", " then ", " subsequently ", " after that "]
            .iter().any(|w| q.contains(w));

        match (entity_count, has_comparison, has_analysis, has_multi_hop) {
            (e, _, _, _) if e >= 4 => QueryComplexity::Hard,
            (e, true, _, _) if e >= 2 => QueryComplexity::Hard,
            (e, _, true, true) if e >= 2 => QueryComplexity::Hard,
            (e, _, _, _) if e >= 3 => QueryComplexity::Medium,
            (_, true, _, _) => QueryComplexity::Medium,
            (_, _, true, _) => QueryComplexity::Medium,
            (e, _, _, _) if e >= 2 => QueryComplexity::Medium,
            _ => QueryComplexity::Simple,
        }
    }

    /// Naïve entity counter: count capitalized words (含短字母数字混合词如 E8/GPT-4)
    fn count_entities(&self, query: &str) -> usize {
        query.split_whitespace()
            .filter(|w| {
                w.starts_with(|c: char| c.is_uppercase())
                    && !["The", "This", "That", "What", "Why", "How", "When", "Where"]
                        .contains(w)
                    && (w.len() > 2 || w.chars().any(|c| c.is_ascii_digit()))
            })
            .count()
    }

    /// CRAG-style retrieval evaluator.
    /// Grades each retrieved document for relevance to the query.
    pub fn grade_documents(
        &self,
        query: &str,
        results: &[SearchResult],
    ) -> Vec<GradedDocument> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        results
            .iter()
            .map(|r| {
                let text = format!(
                    "{} {} {}",
                    r.node.title,
                    r.node.summary.as_deref().unwrap_or(""),
                    r.node.content.as_deref().unwrap_or("")
                )
                .to_lowercase();

                let term_match_ratio = if query_terms.is_empty() {
                    0.0
                } else {
                    let matched = query_terms
                        .iter()
                        .filter(|t| text.contains(*t))
                        .count();
                    matched as f64 / query_terms.len() as f64
                };

                let score_boost = r.score;

                let relevance = if term_match_ratio > 0.6 || score_boost > 0.7 {
                    RelevanceGrade::Relevant
                } else if term_match_ratio > 0.3 || score_boost > 0.4 {
                    RelevanceGrade::Partial
                } else {
                    RelevanceGrade::Irrelevant
                };

                GradedDocument {
                    node_id: r.node.id.clone(),
                    relevance,
                    confidence: (term_match_ratio * 0.5 + score_boost * 0.5)
                        .max(0.0)
                        .min(1.0),
                }
            })
            .collect()
    }

    /// Route decision based on grading results.
    /// Returns Action::Generate (all relevant), Action::Refine (partial),
    /// Action::WebSearch (all irrelevant), or Action::Skip (no docs).
    /// Execute the full adaptive RAG pipeline:
    /// 1. Classify query complexity
    /// 2. Retrieve + grade documents
    /// 3. Route decision → Generate / Refine (with rewrite) / WebSearch
    /// 4. Return the final result with all metadata
    pub fn execute_pipeline(
        &self,
        kb: &super::KnowledgeBase,
        query: &str,
    ) -> AdaptiveRagResult {
        let (complexity, _) = self.classify_query(query);

        // Initial retrieval
        let initial_results = match complexity {
            QueryComplexity::Hard => {
                // C2: map-reduce 分解 — 对比/枚举 → 并行子检索 merge; 顺序 → 链式; 原子 → 保留迭代
                let mut results = Vec::new();
                match super::nt_memory_decompose::decompose_query(query) {
                    super::nt_memory_decompose::Decomposition::Flat(subs)
                    | super::nt_memory_decompose::Decomposition::Sequential(subs) => {
                        for sub in subs {
                            let r = kb.hybrid_rerank_search(&sub, 5).unwrap_or_default();
                            results = super::nt_memory_decompose::merge_results(results, r, 10);
                        }
                        results
                    }
                    super::nt_memory_decompose::Decomposition::Atomic(_) => {
                        let _ = iterative_retrieval(kb, query, &self.config, &mut results);
                        results
                    }
                }
            }
            _ => kb.hybrid_rerank_search(query, 5).unwrap_or_default(),
        };

        let mut graded = self.grade_documents(query, &initial_results);
        let mut action = self.route_decision(&graded);
        let mut iteration = 0;
        let max_iter = self.config.max_iterations;
        let mut current_query = query.to_string();
        let mut rewritten: Option<String> = None;
        let mut all_results = initial_results;

        // Iterative refinement loop
        while action == RetrievalAction::Refine && iteration < max_iter {
            let new_query = rewrite_query(&current_query, &graded, &all_results);
            if new_query == current_query {
                break; // no improvement possible
            }
            rewritten = Some(new_query.clone());
            current_query = new_query;

            let more_results = kb.hybrid_rerank_search(&current_query, 5).unwrap_or_default();
            for r in more_results {
                if !all_results.iter().any(|existing| existing.node.id == r.node.id) {
                    all_results.push(r);
                }
            }
            all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            all_results.truncate(10);

            graded = self.grade_documents(&current_query, &all_results);
            action = self.route_decision(&graded);
            iteration += 1;
        }

        AdaptiveRagResult {
            query: query.to_string(),
            rewritten_query: rewritten,
            complexity,
            action,
            graded,
            results: all_results,
            iteration_count: iteration,
        }
    }

    pub fn route_decision(
        &self,
        graded: &[GradedDocument],
    ) -> RetrievalAction {
        if graded.is_empty() {
            return RetrievalAction::Skip;
        }

        let relevant_count = graded.iter().filter(|g| g.relevance == RelevanceGrade::Relevant).count();
        let _partial_count = graded.iter().filter(|g| g.relevance == RelevanceGrade::Partial).count();
        let irrelevant_count = graded.iter().filter(|g| g.relevance == RelevanceGrade::Irrelevant).count();
        let total = graded.len();

        if relevant_count == total {
            RetrievalAction::Generate
        } else if irrelevant_count == total {
            if self.config.web_search_fallback {
                RetrievalAction::WebSearch
            } else {
                RetrievalAction::Skip
            }
        } else {
            RetrievalAction::Refine
        }
    }

    /// Signal a web search fallback when KB retrieval yields sparse or irrelevant results.
    /// Returns the WebSearch action for the caller to dispatch.
    pub fn web_search(&self, _query: &str) -> RetrievalAction {
        RetrievalAction::WebSearch
    }
}

/// Result of the full adaptive RAG pipeline
#[derive(Debug, Clone)]
pub struct AdaptiveRagResult {
    pub query: String,
    pub rewritten_query: Option<String>,
    pub complexity: QueryComplexity,
    pub action: RetrievalAction,
    pub graded: Vec<GradedDocument>,
    pub results: Vec<SearchResult>,
    pub iteration_count: usize,
}

/// Rewrite a query for refined retrieval when initial results are partial.
/// Extracts distinctive terms from original query and removes stop words.
pub fn rewrite_query(original: &str, graded: &[GradedDocument], results: &[SearchResult]) -> String {
    let stop_words: std::collections::HashSet<&str> = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been",
        "being", "have", "has", "had", "do", "does", "did", "will",
        "would", "could", "should", "may", "might", "shall", "can",
        "to", "of", "in", "for", "on", "with", "at", "by", "from",
        "as", "into", "through", "during", "before", "after", "about",
        "between", "under", "over", "and", "or", "but", "not", "so",
        "if", "than", "that", "this", "these", "those", "it", "its",
        "what", "which", "who", "whom", "how", "why", "where",
    ].iter().copied().collect();

    let mut terms: Vec<&str> = original
        .split_whitespace()
        .filter(|w| w.len() > 2 && !stop_words.contains(w.to_lowercase().as_str()))
        .collect();

    // Add terms from irrelevant results that might be misleading
    for g in graded {
        if g.relevance == RelevanceGrade::Irrelevant {
            if let Some(r) = results.iter().find(|r| r.node.id == g.node_id) {
                let title_terms: Vec<&str> = r.node.title
                    .split_whitespace()
                    .filter(|w| w.len() > 3 && !stop_words.contains(w.to_lowercase().as_str()))
                    .collect();
                // Remove misleading terms: if an irrelevant doc's title terms overlap
                // with the query, those terms are being misinterpreted
                terms.retain(|t| !title_terms.iter().any(|tt| tt.eq_ignore_ascii_case(t)));
            }
        }
    }

    if terms.is_empty() {
        original.to_string()
    } else {
        terms.join(" ")
    }
}

/// Actions the adaptive pipeline can take after retrieval evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrievalAction {
    Generate,    // all docs relevant → proceed to generation
    Refine,      // partial → rewrite query + retrieve again
    WebSearch,   // all docs irrelevant → fallback to web
    Skip,        // no docs or confidence too low → answer from parametric knowledge
}

/// Multi-step iterative retrieval for Hard queries.
/// Appends new results to existing set, deduplicating by node_id.
pub fn iterative_retrieval(
    kb: &super::KnowledgeBase,
    query: &str,
    config: &AdaptiveRagConfig,
    existing_results: &mut Vec<SearchResult>,
) -> Result<Vec<SearchResult>, String> {
    let mut current_query = query.to_string();
    let existing_ids: std::collections::HashSet<String> =
        existing_results.iter().map(|r| r.node.id.clone()).collect();

    for iteration in 0..config.max_iterations {
        let new_results = kb.hybrid_rerank_search(&current_query, 5)?;
        for r in new_results {
            if !existing_ids.contains(&r.node.id) {
                existing_results.push(r);
            }
        }

        if iteration < config.max_iterations - 1 {
            current_query = format!("{} {}", current_query, query);
        }
    }

    existing_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    existing_results.truncate(10);
    Ok(existing_results.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_simple() {
        let ar = AdaptiveRetrieval::new(AdaptiveRagConfig::default());
        let (comp, _) = ar.classify_query("what is 2+2?");
        assert_eq!(comp, QueryComplexity::Simple);
    }

    #[test]
    fn test_classify_medium() {
        let ar = AdaptiveRetrieval::new(AdaptiveRagConfig::default());
        // 小写对比查询 (无大写实体) → Medium; 带实体对比 (E8 vs GWT) 升级为 Hard 走 decompose
        let (comp, _) = ar.classify_query("what is the difference between functional and object-oriented programming");
        assert_eq!(comp, QueryComplexity::Medium);
    }

    #[test]
    fn test_classify_hard() {
        let ar = AdaptiveRetrieval::new(AdaptiveRagConfig::default());
        let (comp, _) = ar.classify_query(
            "compare the SEAL self-iteration pipeline with the PRM reward model and E8 hex state transitions"
        );
        assert_eq!(comp, QueryComplexity::Hard);
    }

    #[test]
    fn test_grade_relevant() {
        let results = vec![
            SearchResult {
                node: KnowledgeNode {
                    id: "1".into(),
                    node_type: NodeType::Concept,
                    title: "E8 Reasoning Engine".into(),
                    summary: Some("E8 state-space reasoning for consciousness".into()),
                    content: None,
                    url: None,
                    domain: None,
                    language: "en".into(),
                    confidence: 1.0,
                    importance: 0.5,
                    created_at: 0,
                    updated_at: 0,
                    access_count: 0,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                },
                score: 0.9,
                matched_on: vec![],
                signals: None,
            },
        ];
        let ar = AdaptiveRetrieval::new(AdaptiveRagConfig::default());
        let graded = ar.grade_documents("E8 reasoning", &results);
        assert_eq!(graded[0].relevance, RelevanceGrade::Relevant);
    }

    #[test]
    fn test_route_generate() {
        let graded = vec![
            GradedDocument { node_id: "1".into(), relevance: RelevanceGrade::Relevant, confidence: 0.9 },
            GradedDocument { node_id: "2".into(), relevance: RelevanceGrade::Relevant, confidence: 0.8 },
        ];
        let ar = AdaptiveRetrieval::new(AdaptiveRagConfig::default());
        assert_eq!(ar.route_decision(&graded), RetrievalAction::Generate);
    }

    #[test]
    fn test_route_websearch() {
        let graded = vec![
            GradedDocument { node_id: "1".into(), relevance: RelevanceGrade::Irrelevant, confidence: 0.1 },
        ];
        let ar = AdaptiveRetrieval::new(AdaptiveRagConfig::default());
        assert_eq!(ar.route_decision(&graded), RetrievalAction::WebSearch);
    }

    #[test]
    fn test_route_refine() {
        let graded = vec![
            GradedDocument { node_id: "1".into(), relevance: RelevanceGrade::Relevant, confidence: 0.9 },
            GradedDocument { node_id: "2".into(), relevance: RelevanceGrade::Irrelevant, confidence: 0.2 },
        ];
        let ar = AdaptiveRetrieval::new(AdaptiveRagConfig::default());
        assert_eq!(ar.route_decision(&graded), RetrievalAction::Refine);
    }
}
