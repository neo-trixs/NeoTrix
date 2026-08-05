/// Test: nt_world_parse module structure compiles and creates ParseGateway
#[test]
fn test_parse_gateway_creation() {
    use neotrix::neotrix::nt_world_parse::ParseGateway;
    let _gw = ParseGateway::new();
}

/// Test: ParseGateway can register backends and report status
#[test]
fn test_parse_gateway_register_backends() {
    use neotrix::neotrix::l2_world_impl::nt_world_parse::backends::pymupdf_backend::PyMuPDFBackend;
    use neotrix::neotrix::nt_world_parse::{ParseGateway, ParseTier};

    let mut gw = ParseGateway::new();
    gw.register_backend("pymupdf", Box::new(PyMuPDFBackend));

    let status = gw.provider_status();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].0, "pymupdf");
    assert_eq!(status[0].1, ParseTier::Tier0Fast);
}

/// Test: ConfidenceScorer scores clean text appropriately
#[test]
fn test_confidence_scorer() {
    use neotrix::neotrix::nt_world_parse::ConfidenceScorer;

    let clean = "# Title\n\nThis is a paragraph with proper sentences. It has structure.\n\n## Section\n\nMore content here.";
    let score = ConfidenceScorer::score(clean, 1000.0);
    assert!(score > 0.7, "clean text should score high, got {}", score);

    let empty = "";
    let score_empty = ConfidenceScorer::score(empty, 1000.0);
    assert!(score_empty < 0.5, "empty text should score low");
}

/// Test: MarkdownRenderer no longer exists (removed in Cycle 55).
/// Placeholder to remind that renderers should be re-added when real implementations exist.
#[test]
fn test_renderers_removed_during_consolidation() {
    // Renderers were removed in Cycle 55. Re-add tests when re-implemented.
}
