#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttackVector {
    Network,
    Adjacent,
    Local,
    Physical,
}

impl AttackVector {
    pub fn value(&self) -> f64 {
        match self {
            Self::Network => 0.85,
            Self::Adjacent => 0.62,
            Self::Local => 0.55,
            Self::Physical => 0.20,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttackComplexity {
    Low,
    High,
}

impl AttackComplexity {
    pub fn value(&self) -> f64 {
        match self {
            Self::Low => 0.77,
            Self::High => 0.44,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrivilegesRequired {
    None,
    Low,
    High,
}

impl PrivilegesRequired {
    pub fn value(&self, scope_changed: bool) -> f64 {
        match (self, scope_changed) {
            (Self::None, _) => 0.85,
            (Self::Low, false) => 0.62,
            (Self::Low, true) => 0.68,
            (Self::High, false) => 0.27,
            (Self::High, true) => 0.50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UserInteraction {
    None,
    Required,
}

impl UserInteraction {
    pub fn value(&self) -> f64 {
        match self {
            Self::None => 0.85,
            Self::Required => 0.62,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scope {
    Unchanged,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Confidentiality {
    None,
    Low,
    High,
}

impl Confidentiality {
    pub fn value(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Low => 0.22,
            Self::High => 0.56,
        }
    }
}

pub type Integrity = Confidentiality;
pub type Availability = Confidentiality;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExploitCodeMaturity {
    NotDefined,
    Unproven,
    ProofOfConcept,
    Functional,
    High,
}

impl ExploitCodeMaturity {
    pub fn value(&self) -> f64 {
        match self {
            Self::NotDefined => 1.0,
            Self::Unproven => 0.91,
            Self::ProofOfConcept => 0.94,
            Self::Functional => 0.97,
            Self::High => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RemediationLevel {
    NotDefined,
    OfficialFix,
    TemporaryFix,
    Workaround,
    Unavailable,
}

impl RemediationLevel {
    pub fn value(&self) -> f64 {
        match self {
            Self::NotDefined => 1.0,
            Self::OfficialFix => 0.95,
            Self::TemporaryFix => 0.96,
            Self::Workaround => 0.97,
            Self::Unavailable => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReportConfidence {
    NotDefined,
    Unknown,
    Reasonable,
    Confirmed,
}

impl ReportConfidence {
    pub fn value(&self) -> f64 {
        match self {
            Self::NotDefined => 1.0,
            Self::Unknown => 0.92,
            Self::Reasonable => 0.96,
            Self::Confirmed => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_vector_network_highest() {
        assert!((AttackVector::Network.value() - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_attack_vector_physical_lowest() {
        assert!((AttackVector::Physical.value() - 0.20).abs() < 1e-9);
    }

    #[test]
    fn test_attack_complexity_low_higher_than_high() {
        assert!(AttackComplexity::Low.value() > AttackComplexity::High.value());
    }

    #[test]
    fn test_privileges_required_none_highest() {
        assert!((PrivilegesRequired::None.value(false) - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_privileges_required_scope_changed_effect() {
        let low_no_scope = PrivilegesRequired::Low.value(false);
        let low_scope = PrivilegesRequired::Low.value(true);
        assert!(low_scope > low_no_scope);
    }

    #[test]
    fn test_user_interaction_none_higher() {
        assert!(UserInteraction::None.value() > UserInteraction::Required.value());
    }

    #[test]
    fn test_confidentiality_high_highest() {
        assert!((Confidentiality::High.value() - 0.56).abs() < 1e-9);
    }

    #[test]
    fn test_confidentiality_none_zero() {
        assert!((Confidentiality::None.value() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_exploit_code_maturity_high_max() {
        assert!((ExploitCodeMaturity::High.value() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_exploit_code_maturity_unproven_min() {
        assert!((ExploitCodeMaturity::Unproven.value() - 0.91).abs() < 1e-9);
    }

    #[test]
    fn test_remediation_level_official_fix_lowest() {
        assert!((RemediationLevel::OfficialFix.value() - 0.95).abs() < 1e-9);
    }

    #[test]
    fn test_remediation_level_unavailable_max() {
        assert!((RemediationLevel::Unavailable.value() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_report_confidence_confirmed_max() {
        assert!((ReportConfidence::Confirmed.value() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_report_confidence_unknown_min() {
        assert!((ReportConfidence::Unknown.value() - 0.92).abs() < 1e-9);
    }

    #[test]
    fn test_integrity_type_alias() {
        let i: Integrity = Confidentiality::High;
        assert!((i.value() - 0.56).abs() < 1e-9);
    }
}
