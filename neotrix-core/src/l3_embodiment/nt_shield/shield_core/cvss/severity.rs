pub use neotrix_types::shared::Severity;

pub fn severity_from_score(score: f64) -> Severity {
    if score >= 9.0 {
        Severity::Critical
    } else if score >= 7.0 {
        Severity::High
    } else if score >= 4.0 {
        Severity::Medium
    } else if score >= 0.1 {
        Severity::Low
    } else {
        Severity::Informational
    }
}

#[derive(Debug, Clone)]
pub struct CvssScore {
    pub base_score: f64,
    pub temporal_score: f64,
    pub severity: Severity,
    pub vector_string: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_informational_for_zero() {
        assert_eq!(severity_from_score(0.0), Severity::Informational);
    }

    #[test]
    fn test_severity_low_boundary() {
        assert_eq!(severity_from_score(0.1), Severity::Low);
        assert_eq!(severity_from_score(3.9), Severity::Low);
    }

    #[test]
    fn test_severity_medium_boundary() {
        assert_eq!(severity_from_score(4.0), Severity::Medium);
        assert_eq!(severity_from_score(6.9), Severity::Medium);
    }

    #[test]
    fn test_severity_high_boundary() {
        assert_eq!(severity_from_score(7.0), Severity::High);
        assert_eq!(severity_from_score(8.9), Severity::High);
    }

    #[test]
    fn test_severity_critical() {
        assert_eq!(severity_from_score(9.0), Severity::Critical);
        assert_eq!(severity_from_score(10.0), Severity::Critical);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Informational), "Informational");
        assert_eq!(format!("{}", Severity::Critical), "Critical");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Informational < Severity::Low);
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_cvss_score_creation() {
        let s = CvssScore {
            base_score: 7.5,
            temporal_score: 6.8,
            severity: Severity::High,
            vector_string: "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H".into(),
        };
        assert!((s.base_score - 7.5).abs() < 1e-9);
        assert_eq!(s.severity, Severity::High);
    }
}
