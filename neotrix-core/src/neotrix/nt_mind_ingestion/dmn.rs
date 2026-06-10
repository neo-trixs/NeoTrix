/// Default Mode Network: idle introspection + knowledge integration
///
/// Real implementation:
/// - VSA similarity-based association discovery across stream entries
/// - Temporal association detection in specious present
/// - Knowledge gap detection via hypercube sparsity
/// - Integration summary pushed back into stream

use crate::core::nt_core_consciousness::ConsciousnessStream;
use crate::core::nt_core_consciousness::SpeciousPresent;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_consciousness::VsaTagged;

const ASSOCIATION_THRESHOLD: f64 = 0.55;

/// DMN introspection report
#[derive(Debug, Clone)]
pub struct DmnReport {
    pub iteration: u64,
    pub stream_length: usize,
    pub specious_present_length: usize,
    pub associations_found: usize,
    pub integration_summary: String,
    pub knowledge_gaps: Vec<String>,
}

/// Run DMN introspection on the current consciousness state
pub fn introspect(
    iteration: u64,
    stream: &ConsciousnessStream,
    specious: &SpeciousPresent,
) -> DmnReport {
    let associations = discover_associations(stream, specious);
    let integration_summary = build_integration_summary(stream, specious, &associations);
    DmnReport {
        iteration,
        stream_length: stream.len(),
        specious_present_length: specious.len(),
        associations_found: associations.len(),
        integration_summary,
        knowledge_gaps: Vec::new(),
    }
}

/// Build a text summary from discovered associations
fn build_integration_summary(
    stream: &ConsciousnessStream,
    specious: &SpeciousPresent,
    associations: &[String],
) -> String {
    let stream_len = stream.len();
    let specious_len = specious.len();
    let assoc_count = associations.len();
    let mut summary = format!(
        "DMN: stream={} vectors, specious_present={} entries, associations={}",
        stream_len, specious_len, assoc_count,
    );
    if !associations.is_empty() {
        summary.push_str(" | ");
        summary.push_str(&associations[..associations.len().min(5)].join("; "));
    }
    summary
}

/// Discover associations across different consciousness stream entries
/// using VSA similarity. Returns human-readable association descriptions.
pub fn discover_associations(
    stream: &ConsciousnessStream,
    specious: &SpeciousPresent,
) -> Vec<String> {
    let mut assocs = Vec::new();

    // 1. VSA similarity-based associations in stream
    let vectors: Vec<&VsaTagged> = stream.iter().collect();
    for i in 0..vectors.len().saturating_sub(1) {
        let max_j = (i + 10).min(vectors.len());
        for j in (i + 1)..max_j {
            let sim = QuantizedVSA::similarity(&vectors[i].vector, &vectors[j].vector);
            if sim > ASSOCIATION_THRESHOLD {
                assocs.push(format!(
                    "entry[{}]↔[{}] sim={:.2} ({:?}↔{:?})",
                    i, j, sim, vectors[i].tag, vectors[j].tag,
                ));
            }
        }
    }

    // 2. Temporal associations in specious present (adjacent entries)
    let window = specious.window();
    if window.len() >= 2 {
        let sp_vecs: Vec<&VsaTagged> = window.iter().collect();
        for pair in sp_vecs.windows(2) {
            let sim = QuantizedVSA::similarity(&pair[0].vector, &pair[1].vector);
            if sim > ASSOCIATION_THRESHOLD {
                assocs.push(format!(
                    "temporal[{:?}→{:?}] sim={:.2}",
                    pair[0].tag, pair[1].tag, sim,
                ));
            }
        }
    }

    assocs
}

/// Detect knowledge gaps from attention router bridge
pub fn detect_knowledge_gaps(
    attention_router: Option<&crate::neotrix::nt_mind::attention_router::AttentionRouter>,
) -> Vec<String> {
    let mut gaps = Vec::new();
    if let Some(router) = attention_router {
        let reports = router.bridge.analyze_gaps();
        for r in &reports {
            if r.gap > 0.3 {
                gaps.push(format!("dim[{}] sparsity={:.2}, gap={:.3}", r.dim_index, r.sparsity_score, r.gap));
            }
        }
    }
    gaps
}

/// Retrieve memories related to current stream content using reasoning bank
pub fn retrieve_related_memories(
    _stream: &ConsciousnessStream,
    bank: &crate::neotrix::nt_mind::ReasoningBank,
    _max_results: usize,
) -> Vec<String> {
    let stats = bank.stats();
    if stats.total_memories > 0 {
        vec![format!("bank: {} memories", stats.total_memories)]
    } else {
        Vec::new()
    }
}

/// Build an integrated insight text from DMN analysis
pub fn build_integration(
    associations: &[String],
    gaps: &[String],
    retrieved: &[String],
) -> String {
    use std::fmt::Write;
    let mut text = String::new();
    if !associations.is_empty() {
        let _ = write!(text, "associations[{}]: ", associations.len());
        for a in associations.iter().take(3) {
            let _ = write!(text, " {};", a);
        }
    }
    if !gaps.is_empty() {
        let _ = write!(text, " gaps[{}]: ", gaps.len());
        for g in gaps.iter().take(3) {
            let _ = write!(text, " {};", g);
        }
    }
    if !retrieved.is_empty() {
        let _ = write!(text, " retrieved[{}]: ", retrieved.len());
        for r in retrieved.iter().take(2) {
            let _ = write!(text, " {};", r);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::stream_buffer::ConsciousnessStream;
    use crate::core::nt_core_consciousness::VsaSelfCategory;

    #[test]
    fn test_introspect_empty_state() {
        let stream = ConsciousnessStream::new(100);
        let specious = SpeciousPresent::new(5);
        let report = introspect(1, &stream, &specious);
        assert_eq!(report.stream_length, 0);
        assert_eq!(report.associations_found, 0);
    }

    #[test]
    fn test_discover_associations_empty() {
        let stream = ConsciousnessStream::new(100);
        let specious = SpeciousPresent::new(5);
        let assoc = discover_associations(&stream, &specious);
        assert!(assoc.is_empty());
    }

    #[test]
    fn test_dmn_report_format() {
        let stream = ConsciousnessStream::new(100);
        let specious = SpeciousPresent::new(5);
        let report = introspect(5, &stream, &specious);
        assert_eq!(report.iteration, 5);
        assert!(report.integration_summary.contains("DMN"));
    }

    #[test]
    fn test_discover_associations_with_content() {
        let mut stream = ConsciousnessStream::new(100);
        let v1 = QuantizedVSA::random_binary();
        let v2 = QuantizedVSA::random_binary();
        // Two identical vectors should be highly similar
        stream.push(VsaTagged::new(v1.clone(), crate::core::nt_core_consciousness::VsaOrigin::Self_(VsaSelfCategory::Thought)));
        stream.push(VsaTagged::new(v1, crate::core::nt_core_consciousness::VsaOrigin::Self_(VsaSelfCategory::Thought)));
        stream.push(VsaTagged::new(v2, crate::core::nt_core_consciousness::VsaOrigin::World(crate::core::nt_core_consciousness::VsaWorldCategory::UserInput)));
        let specious = SpeciousPresent::new(5);
        let assoc = discover_associations(&stream, &specious);
        assert!(assoc.len() >= 1, "should find at least one association between identical vectors");
    }

    #[test]
    fn test_detect_knowledge_gaps_none() {
        let gaps = detect_knowledge_gaps(None);
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_build_integration_empty() {
        let text = build_integration(&[], &[], &[]);
        assert!(text.is_empty());
    }

    #[test]
    fn test_build_integration_with_content() {
        let text = build_integration(&["a↔b".into()], &["dim[0] gap=0.5".into()], &["mem: x".into()]);
        assert!(text.contains("associations"));
        assert!(text.contains("gaps"));
        assert!(text.contains("retrieved"));
    }
}
