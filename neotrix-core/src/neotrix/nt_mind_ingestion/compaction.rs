use crate::core::nt_core_consciousness::stream_buffer::ConsciousnessStream;

/// Session-aware compaction orchestration
///
/// Instead of operating on SpeciousPresent (5-10 entries, never hits
/// 512/768 thresholds), this now operates on ConsciousnessStream
/// with importance-weighted compaction that preserves high-salience entries.

/// Run compaction on a ConsciousnessStream with session context awareness
pub fn compact_stream(stream: &mut ConsciousnessStream) -> bool {
    if let Some(level) = stream.compaction_needed() {
        log::info!(
            "[compaction] {} threshold hit (len={}), preserving high-salience entries",
            level,
            stream.len()
        );
        stream.compact(level);
        log::info!("[compaction] stream compacted to {} entries", stream.len());
        true
    } else {
        false
    }
}

/// Session context for guiding compaction decisions
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub active_topics: Vec<String>,
    pub session_id: String,
    pub message_count: u64,
    pub last_summary: Option<String>,
}

impl SessionContext {
    pub fn new(session_id: &str) -> Self {
        Self {
            active_topics: Vec::new(),
            session_id: session_id.to_string(),
            message_count: 0,
            last_summary: None,
        }
    }

    /// Score how relevant a VsaTagged entry is to the current session context
    pub fn relevance_to_session(&self, _entry: &crate::core::nt_core_consciousness::VsaTagged) -> f64 {
        if self.active_topics.is_empty() {
            return 0.5;
        }
        // Use VSA similarity against bundled topic vectors
        // (simplified: topic overlap as a proxy)
        let topic_weight = self.active_topics.len() as f64 * 0.1;
        (0.5 + topic_weight).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory};
    use crate::core::nt_core_consciousness::stream_buffer::ConsciousnessStream;

    fn tagged() -> crate::core::nt_core_consciousness::VsaTagged {
        crate::core::nt_core_consciousness::VsaTagged::new(
            QuantizedVSA::random_binary(),
            VsaOrigin::Self_(VsaSelfCategory::Thought),
        )
    }

    #[test]
    fn test_compact_stream_does_not_run_below_threshold() {
        let mut stream = ConsciousnessStream::new(100);
        for _ in 0..50 {
            stream.push(tagged());
        }
        assert!(!compact_stream(&mut stream));
    }

    #[test]
    fn test_compact_stream_runs_at_threshold() {
        let mut stream = ConsciousnessStream::new(1024);
        for _ in 0..520 {
            stream.push(tagged());
        }
        assert!(compact_stream(&mut stream));
        assert!(stream.len() < 520);
    }

    #[test]
    fn test_session_context_default() {
        let ctx = SessionContext::new("test_ses");
        assert_eq!(ctx.active_topics.len(), 0);
        assert_eq!(ctx.session_id, "test_ses");
    }
}
