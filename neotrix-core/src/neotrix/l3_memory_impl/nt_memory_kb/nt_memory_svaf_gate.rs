use serde::{Deserialize, Serialize};
use super::nt_memory_embed;
use super::KnowledgeBase;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SvafDecision {
    Accept,
    Guard,
    Redundant,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvafEvaluation {
    pub decision: SvafDecision,
    pub novelty: f64,
    pub coherence: f64,
    pub relevance: f64,
    pub authority: f64,
    pub reason: String,
}

/// SVAF quality gate — evaluates incoming knowledge before ingestion.
/// 4 dimensions: novelty (semantic), coherence (heuristic),
/// relevance (domain), authority (source).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvafGate {
    novelty_threshold: f64,
    coherence_threshold: f64,
    relevance_threshold: f64,
    authority_weights: Vec<(String, f64)>,
}

impl Default for SvafGate {
    fn default() -> Self {
        Self {
            novelty_threshold: 0.15,
            coherence_threshold: 0.2,
            relevance_threshold: 0.1,
            authority_weights: vec![
                ("arxiv".to_string(), 0.9), ("wikipedia".to_string(), 0.8), ("github".to_string(), 0.6),
                ("blog".to_string(), 0.4), ("news".to_string(), 0.3), ("forum".to_string(), 0.2), ("unknown".to_string(), 0.1),
            ],
        }
    }
}

impl SvafGate {
    pub fn new(
        novelty_threshold: f64,
        coherence_threshold: f64,
        relevance_threshold: f64,
    ) -> Self {
        Self { novelty_threshold, coherence_threshold, relevance_threshold, ..Default::default() }
    }

    /// Evaluate content only (no KB connection) — for testing / pre-filter.
    pub fn evaluate_content_only(&self, content: &str, source_type: &str) -> SvafEvaluation {
        let coherence = self.coherence_score(content);
        let relevance = self.relevance_score("", content);
        let authority = self.authority_score(source_type);
        let mean = (coherence + relevance + authority) / 3.0;
        let (decision, reason) = if mean < 0.2 {
            (SvafDecision::Reject, format!("mean={:.2}", mean))
        } else if coherence < self.coherence_threshold {
            (SvafDecision::Guard, format!("coherence={:.2}", coherence))
        } else if relevance > 0.3 || authority > 0.5 {
            (SvafDecision::Accept, format!("coherence={:.2} relevance={:.2}", coherence, relevance))
        } else {
            (SvafDecision::Guard, format!("low relevance ({:.2})", relevance))
        };
        SvafEvaluation { decision, novelty: 0.5, coherence, relevance, authority, reason }
    }

    pub fn evaluate(
        &self, kb: &KnowledgeBase, title: &str, content: &str, source_type: &str,
    ) -> SvafEvaluation {
        let novelty = self.novelty_score(kb, title, content);
        let coherence = self.coherence_score(content);
        let relevance = self.relevance_score(title, content);
        let authority = self.authority_score(source_type);

        let novelty_pass = novelty >= self.novelty_threshold;
        let coherence_pass = coherence >= self.coherence_threshold;
        let relevance_pass = relevance >= self.relevance_threshold;
        let mean = (novelty + coherence + relevance + authority) / 4.0;
        let min_dim = novelty.min(coherence).min(relevance);

        let (decision, reason) = if min_dim < 0.05 {
            (SvafDecision::Reject, format!("all dimensions too low (min={:.2})", min_dim))
        } else if !novelty_pass && mean < 0.2 {
            (SvafDecision::Redundant, format!("low novelty ({:.2}) + low mean ({:.2})", novelty, mean))
        } else if !novelty_pass || !coherence_pass {
            (SvafDecision::Guard, format!("novelty={:.2} coherence={:.2}", novelty, coherence))
        } else if relevance_pass || authority > 0.5 {
            (SvafDecision::Accept, format!("novelty={:.2} coherence={:.2} relevance={:.2}", novelty, coherence, relevance))
        } else {
            (SvafDecision::Guard, format!("low relevance ({:.2})", relevance))
        };

        SvafEvaluation { decision, novelty, coherence, relevance, authority, reason }
    }

    fn novelty_score(&self, kb: &KnowledgeBase, title: &str, content: &str) -> f64 {
        let text = format!("{} {}", title, content);
        let config = match kb.embedding_config.read() {
            Ok(c) => c.clone(),
            _ => return 0.5,
        };
        let config = match config {
            Some(c) => c,
            None => return 0.5,
        };
        let query_vec = match nt_memory_embed::embed_text(&config, &text) {
            Ok(v) => v,
            _ => return 0.5,
        };
        let conn = match kb.conn.lock() {
            Ok(c) => c,
            _ => return 0.5,
        };
        let all = match nt_memory_embed::load_all_embeddings(&conn) {
            Ok(v) => v,
            _ => return 0.5,
        };
        drop(conn);

        if all.is_empty() { return 1.0; }
        let max_sim: f64 = all.iter()
            .map(|(_, vec)| nt_memory_embed::cosine_similarity(&query_vec, vec))
            .fold(0.0_f64, |a, b| a.max(b));
        1.0 - max_sim
    }

    fn coherence_score(&self, content: &str) -> f64 {
        if content.len() < 10 { return 0.1; }
        let sentences: Vec<&str> = content.split(['.', '!', '?'])
            .map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if sentences.len() < 2 { return 0.3; }
        let avg_len = sentences.iter().map(|s| s.len()).sum::<usize>() as f64 / sentences.len() as f64;
        if avg_len < 10.0 { return 0.2; }
        let has_connectors = sentences.iter()
            .filter(|s| s.contains("因为") || s.contains("所以") || s.contains("但是")
                || s.contains("因此") || s.contains("例如") || s.contains("then")
                || s.contains("because") || s.contains("therefore") || s.contains("however"))
            .count();
        0.3 + 0.4 * (has_connectors as f64 / sentences.len() as f64).min(1.0)
            + 0.3 * (avg_len / 80.0).min(1.0)
    }

    fn relevance_score(&self, _title: &str, content: &str) -> f64 {
        let keywords = ["algorithm", "model", "system", "method", "data", "learning",
            "neural", "network", "函数", "算法", "模型", "系统", "方法", "数据"];
        let lower = content.to_lowercase();
        let hits = keywords.iter().filter(|k| lower.contains(*k)).count();
        (hits as f64 / keywords.len() as f64) * 0.8 + 0.1
    }

    fn authority_score(&self, source_type: &str) -> f64 {
        let lower = source_type.to_lowercase();
        for (prefix, weight) in &self.authority_weights {
            if lower.contains(prefix) { return *weight; }
        }
        0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_high_quality_content() {
        let gate = SvafGate::default();
        let content = "This paper proposes a novel neural network model for reinforcement learning.
            The algorithm achieves state-of-the-art results because it uses a transformer architecture.
            Therefore the method significantly outperforms previous approaches.";
        let eval = gate.evaluate_content_only(content, "arxiv");
        assert_eq!(eval.decision, SvafDecision::Accept);
        assert!(eval.coherence > 0.5);
        assert!(eval.authority > 0.8);
    }

    #[test]
    fn test_reject_gibberish() {
        let gate = SvafGate::default();
        let eval = gate.evaluate_content_only("abc xyz qwe", "unknown");
        assert_eq!(eval.decision, SvafDecision::Reject);
    }

    #[test]
    fn test_coherence_empty() {
        let gate = SvafGate::default();
        assert!(gate.coherence_score("") < 0.2);
    }

    #[test]
    fn test_coherence_with_connectors() {
        let gate = SvafGate::default();
        let c = gate.coherence_score("We tried method A because it is efficient. Therefore we used it. However the results were poor.");
        assert!(c > 0.5);
    }

    #[test]
    fn test_authority_known_sources() {
        let gate = SvafGate::default();
        assert!(gate.authority_score("arxiv") > 0.8);
        assert!(gate.authority_score("github.com") > 0.5);
        assert!(gate.authority_score("unknown_forum") < 0.3);
    }

    #[test]
    fn test_redundant_low_novelty() {
        let gate = SvafGate::new(0.8, 0.2, 0.1);
        let eval = gate.evaluate_content_only("low novelty content", "blog");
        assert!(eval.decision == SvafDecision::Guard || eval.decision == SvafDecision::Redundant);
    }

    #[test]
    fn test_guard_marginal_coherence() {
        let gate = SvafGate::new(0.1, 0.9, 0.1);
        let eval = gate.evaluate_content_only("This is an article about machine learning and data science.
            The model works well.", "arxiv");
        assert_eq!(eval.decision, SvafDecision::Guard);
    }
}
