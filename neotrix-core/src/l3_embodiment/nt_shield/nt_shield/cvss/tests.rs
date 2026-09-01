use super::*;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

fn round_up(x: f64) -> f64 {
    (x * 10.0).ceil() / 10.0
}

#[test]
fn test_critical_base_score() {
    let cvss = CvssBuilder {
        av: AttackVector::Network,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Unchanged,
        c: Confidentiality::High,
        i: Confidentiality::High,
        a: Confidentiality::High,
        ..Default::default()
    };
    let score = cvss.score();
    assert!(approx_eq(score.base_score, 9.8));
    assert_eq!(score.severity, Severity::Critical);
}

#[test]
fn test_low_severity_base_score() {
    let cvss = CvssBuilder {
        av: AttackVector::Physical,
        ac: AttackComplexity::High,
        pr: PrivilegesRequired::High,
        ui: UserInteraction::Required,
        s: Scope::Unchanged,
        c: Confidentiality::Low,
        i: Confidentiality::Low,
        a: Confidentiality::Low,
        ..Default::default()
    };
    let score = cvss.score();
    assert!(score.base_score < 4.0);
    assert_eq!(score.severity, Severity::Low);
}

#[test]
fn test_scope_changed_base_score() {
    let cvss = CvssBuilder {
        av: AttackVector::Network,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Changed,
        c: Confidentiality::High,
        i: Confidentiality::High,
        a: Confidentiality::High,
        ..Default::default()
    };
    let base = cvss.base_score();
    assert!(base > 0.0);
    assert!(base <= 10.0);
}

#[test]
fn test_temporal_score_reduces_base() {
    let cvss = CvssBuilder {
        av: AttackVector::Network,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Unchanged,
        c: Confidentiality::High,
        i: Confidentiality::High,
        a: Confidentiality::High,
        e: ExploitCodeMaturity::ProofOfConcept,
        rl: RemediationLevel::OfficialFix,
        rc: ReportConfidence::Reasonable,
        ..Default::default()
    };
    let ts = cvss.temporal_score();
    assert!(ts < cvss.base_score() - 0.1);
}

#[test]
fn test_parse_vector_string() {
    let vector = "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H";
    let cvss = CvssBuilder::with_vector(vector).expect("result");
    assert_eq!(cvss.av, AttackVector::Network);
    assert_eq!(cvss.ac, AttackComplexity::Low);
    assert_eq!(cvss.pr, PrivilegesRequired::None);
    assert_eq!(cvss.ui, UserInteraction::None);
    assert_eq!(cvss.s, Scope::Unchanged);
    assert_eq!(cvss.c, Confidentiality::High);
    assert_eq!(cvss.i, Confidentiality::High);
    assert_eq!(cvss.a, Confidentiality::High);
}

#[test]
fn test_parse_vector_string_with_temporal() {
    let vector =
        "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H/E:P/RL:O/RC:C";
    let cvss = CvssBuilder::with_vector(vector).expect("result");
    assert_eq!(cvss.e, ExploitCodeMaturity::ProofOfConcept);
    assert_eq!(cvss.rl, RemediationLevel::OfficialFix);
    assert_eq!(cvss.rc, ReportConfidence::Confirmed);
}

#[test]
fn test_vector_string_round_trip() {
    let vectors = vec![
        "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
        "CVSS:3.1/AV:A/AC:H/PR:L/UI:R/S:C/C:L/I:L/A:N",
        "CVSS:3.1/AV:P/AC:H/PR:H/UI:R/S:U/C:L/I:L/A:L",
    ];
    for &v in &vectors {
        let cvss = CvssBuilder::with_vector(v).expect("result");
        assert_eq!(cvss.to_vector_string(), v);
    }
}

#[test]
fn test_score_with_exploit_high() {
    let cvss = CvssBuilder {
        av: AttackVector::Network,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Unchanged,
        c: Confidentiality::High,
        i: Confidentiality::High,
        a: Confidentiality::High,
        e: ExploitCodeMaturity::High,
        ..Default::default()
    };
    let ts = cvss.temporal_score();
    assert!(approx_eq(ts, 9.8));
}

#[test]
fn test_score_with_official_fix() {
    let cvss = CvssBuilder {
        av: AttackVector::Network,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Unchanged,
        c: Confidentiality::High,
        i: Confidentiality::High,
        a: Confidentiality::High,
        e: ExploitCodeMaturity::ProofOfConcept,
        rl: RemediationLevel::OfficialFix,
        ..Default::default()
    };
    let ts = cvss.temporal_score();
    let expected = round_up(9.8 * 0.94 * 0.95 * 1.0);
    assert!(approx_eq(ts, expected));
}

#[test]
fn test_severity_boundaries() {
    assert_eq!(Severity::from_score(0.0), Severity::None);
    assert_eq!(Severity::from_score(0.09), Severity::None);
    assert_eq!(Severity::from_score(0.1), Severity::Low);
    assert_eq!(Severity::from_score(3.9), Severity::Low);
    assert_eq!(Severity::from_score(4.0), Severity::Medium);
    assert_eq!(Severity::from_score(6.9), Severity::Medium);
    assert_eq!(Severity::from_score(7.0), Severity::High);
    assert_eq!(Severity::from_score(8.9), Severity::High);
    assert_eq!(Severity::from_score(9.0), Severity::Critical);
    assert_eq!(Severity::from_score(10.0), Severity::Critical);
}

#[test]
fn test_zero_impact_zero_score() {
    let cvss = CvssBuilder {
        av: AttackVector::Network,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Unchanged,
        c: Confidentiality::None,
        i: Confidentiality::None,
        a: Confidentiality::None,
        ..Default::default()
    };
    assert_eq!(cvss.base_score(), 0.0);
    assert_eq!(cvss.score().severity, Severity::None);
}

#[test]
fn test_invalid_vector_string() {
    assert!(CvssBuilder::with_vector("CVSS:2.0/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H").is_err());
    assert!(CvssBuilder::with_vector("garbage").is_err());
    assert!(CvssBuilder::with_vector("").is_err());
    assert!(CvssBuilder::with_vector("CVSS:3.1/AV:X/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H").is_err());
}

#[test]
fn test_builder_defaults() {
    let cvss = CvssBuilder::new();
    assert_eq!(cvss.av, AttackVector::Network);
    assert_eq!(cvss.ac, AttackComplexity::Low);
    assert_eq!(cvss.pr, PrivilegesRequired::None);
    assert_eq!(cvss.ui, UserInteraction::None);
    assert_eq!(cvss.s, Scope::Unchanged);
    assert_eq!(cvss.c, Confidentiality::High);
    assert_eq!(cvss.i, Confidentiality::High);
    assert_eq!(cvss.a, Confidentiality::High);
    assert_eq!(cvss.e, ExploitCodeMaturity::NotDefined);
    assert_eq!(cvss.rl, RemediationLevel::NotDefined);
    assert_eq!(cvss.rc, ReportConfidence::NotDefined);
}

#[test]
fn test_all_impact_none() {
    let cvss = CvssBuilder {
        c: Confidentiality::None,
        i: Confidentiality::None,
        a: Confidentiality::None,
        ..Default::default()
    };
    assert_eq!(cvss.base_score(), 0.0);
}

#[test]
fn test_rounding_exact_one_decimal() {
    let cvss = CvssBuilder {
        av: AttackVector::Network,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Unchanged,
        c: Confidentiality::High,
        i: Confidentiality::High,
        a: Availability::High,
        ..Default::default()
    };
    let score = cvss.score().base_score;
    let rounded = (score * 10.0).round();
    assert!((score * 10.0 - rounded).abs() < 0.0001);
}

#[test]
fn test_adjacent_network_score() {
    let cvss = CvssBuilder {
        av: AttackVector::Adjacent,
        ac: AttackComplexity::Low,
        pr: PrivilegesRequired::None,
        ui: UserInteraction::None,
        s: Scope::Unchanged,
        c: Confidentiality::High,
        i: Confidentiality::High,
        a: Confidentiality::High,
        ..Default::default()
    };
    let base = cvss.base_score();
    assert!(base > 0.0 && base < 10.0);
}

#[test]
fn test_severity_display() {
    assert_eq!(Severity::None.to_string(), "None");
    assert_eq!(Severity::Low.to_string(), "Low");
    assert_eq!(Severity::Medium.to_string(), "Medium");
    assert_eq!(Severity::High.to_string(), "High");
    assert_eq!(Severity::Critical.to_string(), "Critical");
}
