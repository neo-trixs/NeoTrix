use crate::core::nt_core_hcube::trigram_index::TrigramInvertedIndex;
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

// ── Search Strategy Enum ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStrategy {
    /// Trigram/grep exact search
    Grep,
    /// VSA semantic search
    Vector,
    /// RRF-fused hybrid
    Hybrid,
    /// Agentic: plan → retrieve → evaluate → iterate
    Agentic,
}

impl SearchStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            SearchStrategy::Grep => "grep",
            SearchStrategy::Vector => "vector",
            SearchStrategy::Hybrid => "hybrid",
            SearchStrategy::Agentic => "agentic",
        }
    }
}

// ── Search Result ──

#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub doc_id: usize,
    pub text: String,
    pub score: f64,
    pub strategy: SearchStrategy,
}

// ── RRF Fuser ──

#[derive(Debug, Clone)]
pub struct RRFFuser {
    pub k: f64,
}

impl Default for RRFFuser {
    fn default() -> Self {
        Self { k: 60.0 }
    }
}

impl RRFFuser {
    pub fn new(k: f64) -> Self {
        Self { k }
    }

    /// Reciprocal Rank Fusion: merge ranked lists from multi-strategy search.
    pub fn fuse(&self, lists: &[&[SearchResultItem]]) -> Vec<SearchResultItem> {
        use std::collections::HashMap;
        let mut rrf_scores: HashMap<usize, (f64, &str, f64)> = HashMap::new();
        for list in lists {
            for (rank, item) in list.iter().enumerate() {
                let entry = rrf_scores.entry(item.doc_id).or_insert((0.0, "", 0.0));
                entry.0 += 1.0 / (self.k + rank as f64);
                entry.1 = &item.text;
                entry.2 = entry.2.max(item.score);
            }
        }
        let mut results: Vec<SearchResultItem> = rrf_scores
            .into_iter()
            .map(|(doc_id, (rrf, text, _scoree))| SearchResultItem {
                doc_id,
                text: text.to_string(),
                score: rrf,
                strategy: SearchStrategy::Hybrid,
            })
            .collect();
        results.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

// ── Hybrid Retriever ──

pub struct HybridRetriever {
    pub trigram: TrigramInvertedIndex,
    pub vsa_vectors: Vec<Vec<u8>>,
    pub rrf: RRFFuser,
    pub top_k: usize,
}

impl HybridRetriever {
    pub fn new(top_k: usize) -> Self {
        Self {
            trigram: TrigramInvertedIndex::new(),
            vsa_vectors: Vec::new(),
            rrf: RRFFuser::default(),
            top_k,
        }
    }

    pub fn len(&self) -> usize {
        self.trigram.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trigram.is_empty()
    }

    pub fn insert(&mut self, text: &str) -> usize {
        let doc_id = self.trigram.insert(text);
        let seed: u64 = text.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
        while self.vsa_vectors.len() <= doc_id {
            self.vsa_vectors.push(vec![0; VSA_DIM]);
        }
        self.vsa_vectors[doc_id] = vsa;
        doc_id
    }

    /// Grep search via trigram index
    pub fn grep_search(&self, query: &str, top_k: usize) -> Vec<SearchResultItem> {
        let ids = self.trigram.like_search(query);
        let top = ids.into_iter().take(top_k);
        top.map(|doc_id| SearchResultItem {
            doc_id,
            text: self
                .trigram
                .doc(doc_id)
                .map(|d| d.text.clone())
                .unwrap_or_default(),
            score: 1.0,
            strategy: SearchStrategy::Grep,
        })
        .collect()
    }

    /// VSA vector search via brute-force cosine
    pub fn vector_search(&self, query: &str, top_k: usize) -> Vec<SearchResultItem> {
        let seed: u64 = query.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let query_vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
        let mut scores: Vec<(usize, f64)> = self
            .vsa_vectors
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let sim = QuantizedVSA::cosine(&query_vsa, v);
                (i, sim)
            })
            .collect();
        scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
            .into_iter()
            .take(top_k)
            .map(|(doc_id, score)| SearchResultItem {
                doc_id,
                text: self
                    .trigram
                    .doc(doc_id)
                    .map(|d| d.text.clone())
                    .unwrap_or_default(),
                score,
                strategy: SearchStrategy::Vector,
            })
            .collect()
    }

    /// Hybrid search: RRF fuse grep + vector results
    pub fn hybrid_search(&self, query: &str, top_k: usize) -> Vec<SearchResultItem> {
        let grep_results = self.grep_search(query, top_k * 2);
        let vector_results = self.vector_search(query, top_k * 2);
        let fused = self.rrf.fuse(&[&grep_results, &vector_results]);
        fused.into_iter().take(top_k).collect()
    }

    /// Search using the best available strategy
    pub fn search(&self, query: &str, strategy: SearchStrategy) -> Vec<SearchResultItem> {
        match strategy {
            SearchStrategy::Grep => self.grep_search(query, self.top_k),
            SearchStrategy::Vector => self.vector_search(query, self.top_k),
            SearchStrategy::Hybrid => self.hybrid_search(query, self.top_k),
            SearchStrategy::Agentic => self.agentic_search(query, self.top_k),
        }
    }

    /// Agentic search with Plan-Retrieve-Evaluate loop up to max_iterations.
    pub fn agentic_search(&self, query: &str, top_k: usize) -> Vec<SearchResultItem> {
        let max_iterations = 3;
        let mut all_results: Vec<SearchResultItem> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut current_query = query.to_string();

        for iteration in 0..max_iterations {
            // Plan: choose strategy based on query characteristics and iteration
            let strategy = if iteration == 0 {
                // First pass: use hybrid
                SearchStrategy::Hybrid
            } else if current_query.len() < 10 {
                // Short queries: use grep for precision
                SearchStrategy::Grep
            } else {
                // Long/refined queries: use vector for semantic breadth
                SearchStrategy::Vector
            };

            // Retrieve
            let results = match strategy {
                SearchStrategy::Grep => self.grep_search(&current_query, top_k),
                SearchStrategy::Vector => self.vector_search(&current_query, top_k),
                SearchStrategy::Hybrid => self.hybrid_search(&current_query, top_k),
                SearchStrategy::Agentic => {
                    log::warn!("agentic_searcher: Agentic strategy not fully implemented, falling back to Hybrid");
                    self.hybrid_search(&current_query, top_k)
                }
            };

            // Evaluate: add unseen results
            for r in results {
                if seen.insert(r.doc_id) {
                    all_results.push(r);
                }
            }

            // Early termination if we have enough results
            if all_results.len() >= top_k * 2 || iteration >= max_iterations - 1 {
                break;
            }

            // Refine: use top result as context for next iteration
            if let Some(best) = all_results.first() {
                let words: Vec<&str> = best.text.split_whitespace().take(5).collect();
                if !words.is_empty() {
                    current_query = format!("{} {}", current_query, words.join(" "));
                }
            }
        }

        all_results.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_results.truncate(top_k);
        all_results
    }
}

// ── SearchedDocument ──

#[derive(Debug, Clone)]
pub struct SearchedDocument {
    pub doc_id: usize,
    pub title: String,
    pub text: String,
    pub vsa_vector: Vec<u8>,
}

/// A managed hybrid search collection that keeps indexed documents alive.
pub struct SearchedDocumentCollection {
    pub docs: Vec<SearchedDocument>,
}

impl Default for SearchedDocumentCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchedDocumentCollection {
    pub fn new() -> Self {
        Self { docs: Vec::new() }
    }

    pub fn add(&mut self, title: String, text: String) {
        let seed: u64 = text.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let vsa_vector = QuantizedVSA::seeded_random(seed, VSA_DIM);
        let doc_id = self.docs.len();
        self.docs.push(SearchedDocument {
            doc_id,
            title,
            text,
            vsa_vector,
        });
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<&SearchedDocument> {
        let seed: u64 = query.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let query_vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
        let mut scores: Vec<(usize, f64)> = self
            .docs
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let sim = QuantizedVSA::cosine(&query_vsa, &d.vsa_vector);
                (i, sim)
            })
            .collect();
        scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
            .into_iter()
            .take(top_k)
            .map(|(i, _)| &self.docs[i])
            .collect()
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

// ── P1.4: Agentic Search Loop ──

/// Search budget: controls iteration depth and resource usage
#[derive(Debug, Clone)]
pub struct SearchBudget {
    /// Maximum iterations
    pub max_iterations: usize,
    /// Maximum results to collect per iteration
    pub max_results_per_iter: usize,
    /// Score threshold for early termination (0-1)
    pub early_termination_threshold: f64,
}

impl Default for SearchBudget {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            max_results_per_iter: 10,
            early_termination_threshold: 0.85,
        }
    }
}

impl SearchBudget {
    pub fn new(
        max_iterations: usize,
        max_results_per_iter: usize,
        early_termination_threshold: f64,
    ) -> Self {
        Self {
            max_iterations: max_iterations.max(1),
            max_results_per_iter: max_results_per_iter.max(1),
            early_termination_threshold: early_termination_threshold.clamp(0.0, 1.0),
        }
    }
}

/// Search plan: strategy selection and query refinement
#[derive(Debug, Clone)]
pub struct SearchPlan {
    /// Primary strategy
    pub primary: SearchStrategy,
    /// Fallback strategy
    pub fallback: SearchStrategy,
    /// Refined query string
    pub refined_query: String,
    /// Expected result count
    pub expected_count: usize,
    /// Query difficulty (0 = trivial, 1 = very hard)
    pub difficulty: f64,
}

/// Search planner: analyzes query and selects optimal strategy
#[derive(Debug, Clone)]
pub struct SearchPlanner {
    /// Short query threshold (chars): queries shorter than this use Grep
    pub short_query_threshold: usize,
    /// Long query threshold (chars): queries longer than this use Vector
    pub long_query_threshold: usize,
}

impl Default for SearchPlanner {
    fn default() -> Self {
        Self {
            short_query_threshold: 10,
            long_query_threshold: 50,
        }
    }
}

impl SearchPlanner {
    pub fn new(short_query_threshold: usize, long_query_threshold: usize) -> Self {
        Self {
            short_query_threshold,
            long_query_threshold,
        }
    }

    /// Analyze a query and produce a search plan
    pub fn plan(&self, query: &str) -> SearchPlan {
        let len = query.len();
        let has_special_chars = query.chars().any(|c| c.is_ascii_punctuation() && c != '\'');
        let word_count = query.split_whitespace().count();
        let difficulty = if word_count <= 2 {
            0.2
        } else if word_count <= 5 {
            0.5
        } else {
            0.8
        };

        let (primary, fallback) = if len < self.short_query_threshold && !has_special_chars {
            (SearchStrategy::Grep, SearchStrategy::Hybrid)
        } else if len > self.long_query_threshold {
            (SearchStrategy::Vector, SearchStrategy::Hybrid)
        } else if has_special_chars {
            (SearchStrategy::Grep, SearchStrategy::Vector)
        } else {
            (SearchStrategy::Hybrid, SearchStrategy::Vector)
        };

        SearchPlan {
            primary,
            fallback,
            refined_query: query.to_string(),
            expected_count: (10.0 * (1.0 - difficulty)) as usize + 2,
            difficulty,
        }
    }

    /// Refine query based on previous results
    pub fn refine(&self, query: &str, results: &[SearchResultItem], iteration: usize) -> String {
        if results.is_empty() {
            return format!("{query} alternative search");
        }
        let keywords: Vec<&str> = results
            .iter()
            .take(3)
            .flat_map(|r| r.text.split_whitespace())
            .filter(|w| w.len() > 3)
            .take(5)
            .collect();
        let suffix = keywords.join(" ");
        if iteration == 0 {
            format!("{query} {suffix}")
        } else {
            format!("{query} more {suffix}")
        }
    }
}

/// Search evaluator: judges whether results are sufficient
#[derive(Debug, Clone)]
pub struct SearchEvaluator {
    /// Minimum results required
    pub min_results: usize,
    /// Minimum average score to consider sufficient
    pub min_avg_score: f64,
}

impl Default for SearchEvaluator {
    fn default() -> Self {
        Self {
            min_results: 2,
            min_avg_score: 0.3,
        }
    }
}

impl SearchEvaluator {
    pub fn new(min_results: usize, min_avg_score: f64) -> Self {
        Self {
            min_results: min_results.max(1),
            min_avg_score: min_avg_score.clamp(0.0, 1.0),
        }
    }

    /// Evaluate whether search results are sufficient
    pub fn evaluate(&self, results: &[SearchResultItem], plan: &SearchPlan) -> SearchVerdict {
        if results.len() < self.min_results {
            return SearchVerdict::Insufficient("too few results".into());
        }
        let avg_score: f64 = results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64;
        if avg_score < self.min_avg_score {
            return SearchVerdict::Insufficient(format!("low avg score: {:.2}", avg_score));
        }
        let max_score = results.iter().map(|r| r.score).fold(0.0_f64, f64::max);
        if max_score > plan.difficulty * 0.8 {
            SearchVerdict::Sufficient
        } else {
            SearchVerdict::Partial("results found but quality could improve".into())
        }
    }

    /// Early termination check
    pub fn should_terminate_early(
        &self,
        results: &[SearchResultItem],
        budget: &SearchBudget,
    ) -> bool {
        if results.is_empty() {
            return false;
        }
        let max_score = results.iter().map(|r| r.score).fold(0.0_f64, f64::max);
        max_score >= budget.early_termination_threshold
    }
}

/// Search verdict
#[derive(Debug, Clone, PartialEq)]
pub enum SearchVerdict {
    Sufficient,
    Partial(String),
    Insufficient(String),
}

/// Full Agentic Searcher: Plan → Retrieve → Evaluate → Iterate loop
#[derive(Debug, Clone)]
pub struct AgenticSearcher {
    pub planner: SearchPlanner,
    pub evaluator: SearchEvaluator,
    pub budget: SearchBudget,
}

impl Default for AgenticSearcher {
    fn default() -> Self {
        Self {
            planner: SearchPlanner::default(),
            evaluator: SearchEvaluator::default(),
            budget: SearchBudget::default(),
        }
    }
}

impl AgenticSearcher {
    pub fn new(planner: SearchPlanner, evaluator: SearchEvaluator, budget: SearchBudget) -> Self {
        Self {
            planner,
            evaluator,
            budget,
        }
    }

    /// Run a full agentic search loop against a HybridRetriever
    pub fn search(&self, retriever: &HybridRetriever, query: &str) -> Vec<SearchResultItem> {
        let plan = self.planner.plan(query);
        let mut all_results: Vec<SearchResultItem> = Vec::new();
        let mut seen_ids: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut current_query = query.to_string();

        for iteration in 0..self.budget.max_iterations {
            let strategy = if iteration == 0 {
                plan.primary
            } else {
                plan.fallback
            };
            let iter_results = retriever.search(&current_query, strategy);
            for r in iter_results {
                if seen_ids.insert(r.doc_id) {
                    all_results.push(r);
                }
            }
            if self
                .evaluator
                .should_terminate_early(&all_results, &self.budget)
            {
                break;
            }
            let verdict = self.evaluator.evaluate(&all_results, &plan);
            match verdict {
                SearchVerdict::Sufficient => break,
                SearchVerdict::Partial(_) => {
                    current_query = self.planner.refine(&current_query, &all_results, iteration);
                }
                SearchVerdict::Insufficient(_) => {
                    current_query = self.planner.refine(&current_query, &all_results, iteration);
                }
            }
            if all_results.len() >= self.budget.max_results_per_iter * 2 {
                break;
            }
        }

        all_results.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_results.truncate(self.budget.max_results_per_iter);
        all_results
    }

    /// Analyze query without running search
    pub fn analyze_query(&self, query: &str) -> SearchPlan {
        self.planner.plan(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_retriever() -> HybridRetriever {
        let mut r = HybridRetriever::new(5);
        r.insert("hello world this is a test document");
        r.insert("rust programming language for systems programming");
        r.insert("machine learning and artificial intelligence research");
        r.insert("neotrix consciousness engine with VSA hypercube");
        r.insert("quantum computing and topological quantum field theory");
        r
    }

    #[test]
    fn test_hybrid_retriever_basic() {
        let r = make_retriever();
        assert_eq!(r.len(), 5);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_grep_search() {
        let r = make_retriever();
        let results = r.grep_search("world", 3);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.text.contains("world")));
    }

    #[test]
    fn test_grep_search_no_results() {
        let r = make_retriever();
        let results = r.grep_search("xyznonexistent", 3);
        assert!(results.is_empty());
    }

    #[test]
    fn test_vector_search() {
        let r = make_retriever();
        let results = r.vector_search("programming language", 3);
        assert!(!results.is_empty());
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_hybrid_search() {
        let r = make_retriever();
        let results = r.hybrid_search("consciousness", 3);
        assert!(!results.is_empty());
        for r in &results {
            assert!(!r.text.is_empty());
        }
    }

    #[test]
    fn test_rrf_fuser() {
        let fuser = RRFFuser::default();
        let list1 = vec![
            SearchResultItem {
                doc_id: 0,
                text: "a".into(),
                score: 0.9,
                strategy: SearchStrategy::Grep,
            },
            SearchResultItem {
                doc_id: 1,
                text: "b".into(),
                score: 0.8,
                strategy: SearchStrategy::Grep,
            },
        ];
        let list2 = vec![
            SearchResultItem {
                doc_id: 1,
                text: "b".into(),
                score: 0.95,
                strategy: SearchStrategy::Vector,
            },
            SearchResultItem {
                doc_id: 2,
                text: "c".into(),
                score: 0.7,
                strategy: SearchStrategy::Vector,
            },
        ];
        let fused = fuser.fuse(&[&list1, &list2]);
        assert_eq!(fused.len(), 3);
        // doc 1 appears in both lists → highest RRF score
        assert_eq!(fused[0].doc_id, 1);
    }

    #[test]
    fn test_searched_document_collection() {
        let mut col = SearchedDocumentCollection::new();
        assert!(col.is_empty());
        col.add("doc1".into(), "hello world".into());
        col.add("doc2".into(), "rust programming".into());
        assert_eq!(col.len(), 2);
        let results = col.search("world", 5);
        assert!(!results.is_empty());
        assert!(results.iter().any(|d| d.title == "doc1"));
    }

    #[test]
    fn test_agentic_search() {
        let r = make_retriever();
        let results = r.agentic_search("consciousness", 3);
        assert!(!results.is_empty());
        // Agentic search should find the neotrix document
        let has_neotrix = results.iter().any(|r| r.text.contains("neotrix"));
        assert!(has_neotrix);
    }

    #[test]
    fn test_strategy_names() {
        assert_eq!(SearchStrategy::Grep.name(), "grep");
        assert_eq!(SearchStrategy::Vector.name(), "vector");
        assert_eq!(SearchStrategy::Hybrid.name(), "hybrid");
        assert_eq!(SearchStrategy::Agentic.name(), "agentic");
    }

    #[test]
    fn test_empty_retriever() {
        let r: HybridRetriever = HybridRetriever::new(5);
        assert!(r.is_empty());
        let results = r.grep_search("test", 5);
        assert!(results.is_empty());
    }
}
