use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn from_score(score: f64) -> Self {
        if score >= 9.0 {
            Self::Critical
        } else if score >= 7.0 {
            Self::High
        } else if score >= 4.0 {
            Self::Medium
        } else if score >= 0.1 {
            Self::Low
        } else {
            Self::None
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
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
    fn test_severity_none_for_zero() {
        assert_eq!(Severity::from_score(0.0), Severity::None);
    }

    #[test]
    fn test_severity_low_boundary() {
        assert_eq!(Severity::from_score(0.1), Severity::Low);
        assert_eq!(Severity::from_score(3.9), Severity::Low);
    }

    #[test]
    fn test_severity_medium_boundary() {
        assert_eq!(Severity::from_score(4.0), Severity::Medium);
        assert_eq!(Severity::from_score(6.9), Severity::Medium);
    }

    #[test]
    fn test_severity_high_boundary() {
        assert_eq!(Severity::from_score(7.0), Severity::High);
        assert_eq!(Severity::from_score(8.9), Severity::High);
    }

    #[test]
    fn test_severity_critical() {
        assert_eq!(Severity::from_score(9.0), Severity::Critical);
        assert_eq!(Severity::from_score(10.0), Severity::Critical);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::None), "None");
        assert_eq!(format!("{}", Severity::Critical), "Critical");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::None < Severity::Low);
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
