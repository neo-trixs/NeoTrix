use crate::neotrix::nt_core_signal::ops::cosine_similarity;

pub use crate::core::nt_core_embed::TextEmbedder;

pub fn recall_similar(query: &str, memories: &[crate::neotrix::nt_mind::memory::ReasoningMemory], top_k: usize) -> Vec<(usize, f64)> {
    if memories.is_empty() {
        return Vec::new();
    }

    let mut embedder = TextEmbedder::new();

    let qv = embedder.embed(query);

    let mut scored: Vec<(usize, f64)> = memories
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mv = embedder.embed(&m.task_description);
            let sim = cosine_similarity(&qv, &mv);
            let reward_bonus = m.reward * 0.3;
            (i, sim + reward_bonus)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_embed::EMBEDDING_DIM;

    #[test]
    fn test_embedding_creation() {
        let mut embedder = TextEmbedder::new();
        let vec = embedder.embed("hello world test");
        assert_eq!(vec.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_similar_texts() {
        let mut embedder = TextEmbedder::new();
        let sim = embedder.similarity(
            "implement user authentication with JWT tokens",
            "add JWT based user authentication system",
        );
        assert!(sim > 0.1);
    }

    #[test]
    fn test_dissimilar_texts() {
        let mut embedder = TextEmbedder::new();
        let sim = embedder.similarity(
            "implement user authentication",
            "color scheme for dark mode UI design",
        );
        assert!(sim < 0.8);
    }

    #[test]
    fn test_find_most_similar() {
        let mut embedder = TextEmbedder::new();
        let candidates = [
            "fix database connection pool leak",
            "design landing page with tailwind",
            "add unit tests for api endpoints",
        ];
        let result = embedder.find_most_similar("database connection issue debug", &candidates);
        assert!(result.is_some());
        let (idx, _sim, text) = result.expect("result should be ok in test");
        assert_eq!(idx, 0);
        assert!(text.contains("database"));
    }

    #[test]
    fn test_recall_similar_basic() {
        use crate::neotrix::nt_mind::memory::ReasoningMemory;
        use crate::neotrix::nt_world_model::TaskType;

        let mems = vec![
            ReasoningMemory::new("fix database connection pool", TaskType::CodeReview, &[], 0.8),
            ReasoningMemory::new("design responsive layout", TaskType::UIDesign, &[], 0.6),
        ];

        let results = recall_similar("database error handling", &mems, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0);
    }

    #[test]
    fn test_embedder_vocab_growth() {
        let mut embedder = TextEmbedder::new();
        embedder.embed("rust compiler optimization techniques");
        assert!(embedder.vocab_size() >= 3);
    }
}
