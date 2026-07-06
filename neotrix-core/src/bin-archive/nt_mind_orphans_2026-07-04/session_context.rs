use crate::core::nt_core_consciousness::VsaTagged;

/// Session-level context manager
///
/// Tracks active conversation topics and guides memory management decisions.
/// Ensures that compaction/eviction preserves context-critical entries.
#[derive(Debug, Clone)]
pub struct SessionContextManager {
    /// Active topics being discussed in the current session
    active_topics: Vec<String>,
    /// Session identifier from NarrativeSelf
    session_id: String,
    /// Running message count
    pub message_count: u64,
    /// Last extracted topic summary
    last_summary: Option<String>,
    /// Decay rate for topic relevance over time (reserved for future use)
    _topic_decay: f64,
}

impl Default for SessionContextManager {
    fn default() -> Self {
        Self {
            active_topics: Vec::new(),
            session_id: "ses_init".into(),
            message_count: 0,
            last_summary: None,
            _topic_decay: 0.05,
        }
    }
}

impl SessionContextManager {
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            ..Default::default()
        }
    }

    /// Update session context with a new user input or assistant response
    pub fn observe_message(&mut self, input: &str, is_user: bool) {
        self.message_count += 1;
        if is_user {
            self.extract_topics(input);
        }
        self.decay_topics();
    }

    /// Extract topic keywords from text input
    fn extract_topics(&mut self, input: &str) {
        let words: Vec<&str> = input.split_whitespace()
            .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()))
            .filter(|w| w.len() > 3)
            .collect();

        // Simple frequency-based extraction of significant words
        let mut freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for word in words {
            if !Self::is_stop_word(word) {
                *freq.entry(word).or_insert(0) += 1;
            }
        }

        let mut topic_words: Vec<(&str, usize)> = freq.into_iter().collect();
        topic_words.sort_by(|a, b| b.1.cmp(&a.1));

        // Keep top 10 topics
        for (word, _) in topic_words.iter().take(5) {
            let topic = word.to_lowercase();
            if !self.active_topics.contains(&topic) {
                self.active_topics.push(topic);
            }
        }

        if self.active_topics.len() > 20 {
            self.active_topics = self.active_topics.split_off(self.active_topics.len() - 20);
        }
    }

    /// Decay older topic relevance
    fn decay_topics(&mut self) {
        // Simple approach: periodically rotate topics
        if self.message_count % 10 == 0 && self.active_topics.len() > 5 {
            let keep = self.active_topics.len() / 2;
            self.active_topics = self.active_topics.split_off(self.active_topics.len() - keep);
        }
    }

    /// Check if a stop word (skip for topic extraction)
    fn is_stop_word(word: &str) -> bool {
        matches!(word, "this" | "that" | "with" | "from" | "what" | "when"
            | "where" | "which" | "there" | "their" | "about" | "would"
            | "could" | "should" | "have" | "been" | "were" | "being"
            | "the" | "and" | "for" | "are" | "not" | "but" | "you"
            | "all" | "can" | "had" | "her" | "was" | "one" | "our"
            | "out" | "has" | "how" | "its" | "just" | "know" | "like"
            | "make" | "more" | "over" | "some" | "than" | "them"
            | "then" | "very" | "way" | "who" | "will" | "your")
    }

    /// Score a VsaTagged entry's relevance to current session context
    pub fn score_relevance(&self, entry: &VsaTagged) -> f64 {
        if self.active_topics.is_empty() {
            return 0.5;
        }
        // Use VSA similarity as a proxy for topic relevance
        // (In production, compare against bundled topic VSA vectors)
        let topic_density = self.active_topics.len() as f64 / 20.0;
        0.3 + topic_density * 0.3 + entry.confidence * 0.4
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn active_topics(&self) -> &[String] {
        &self.active_topics
    }

    pub fn set_session_id(&mut self, session_id: &str) {
        self.session_id = session_id.to_string();
    }

    pub fn set_summary(&mut self, summary: &str) {
        self.last_summary = Some(summary.to_string());
    }

    pub fn last_summary(&self) -> Option<&str> {
        self.last_summary.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session_context() {
        let ctx = SessionContextManager::new("ses_test");
        assert_eq!(ctx.session_id(), "ses_test");
        assert_eq!(ctx.message_count, 0);
        assert!(ctx.active_topics().is_empty());
    }

    #[test]
    fn test_message_count_increments() {
        let mut ctx = SessionContextManager::new("ses_test");
        ctx.observe_message("hello, how are you?", false);
        assert_eq!(ctx.message_count, 1);
    }

    #[test]
    fn test_user_message_extracts_topics() {
        let mut ctx = SessionContextManager::new("ses_test");
        ctx.observe_message("I want to talk about Rust compiler optimization techniques", true);
        assert!(!ctx.active_topics().is_empty());
        assert!(ctx.active_topics().iter().any(|t| t == "optimization"));
    }

    #[test]
    fn test_stop_words_not_extracted() {
        let mut ctx = SessionContextManager::new("ses_test");
        ctx.observe_message("this that and with", true);
        assert!(ctx.active_topics().is_empty());
    }

    #[test]
    fn test_relevance_scoring_basic() {
        let ctx = SessionContextManager::new("ses_test");
        let entry = VsaTagged::new(vec![0; 32], crate::core::nt_core_consciousness::VsaOrigin::Self_(
            crate::core::nt_core_consciousness::VsaSelfCategory::Thought
        )).with_confidence(0.8);
        let score = ctx.score_relevance(&entry);
        assert!(score >= 0.0 && score <= 1.0);
    }
}
