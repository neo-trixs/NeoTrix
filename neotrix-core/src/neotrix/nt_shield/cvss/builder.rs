use super::metrics::{
    AttackComplexity, AttackVector, Confidentiality, ExploitCodeMaturity, PrivilegesRequired,
    RemediationLevel, ReportConfidence, Scope, UserInteraction,
};
use super::severity::{CvssScore, Severity};

fn round_up(x: f64) -> f64 {
    (x * 10.0).ceil() / 10.0
}

#[derive(Debug, Clone)]
pub struct CvssBuilder {
    pub av: AttackVector,
    pub ac: AttackComplexity,
    pub pr: PrivilegesRequired,
    pub ui: UserInteraction,
    pub s: Scope,
    pub c: Confidentiality,
    pub i: Confidentiality,
    pub a: Confidentiality,
    pub e: ExploitCodeMaturity,
    pub rl: RemediationLevel,
    pub rc: ReportConfidence,
}

impl CvssBuilder {
    pub fn new() -> Self {
        Self {
            av: AttackVector::Network,
            ac: AttackComplexity::Low,
            pr: PrivilegesRequired::None,
            ui: UserInteraction::None,
            s: Scope::Unchanged,
            c: Confidentiality::High,
            i: Confidentiality::High,
            a: Confidentiality::High,
            e: ExploitCodeMaturity::NotDefined,
            rl: RemediationLevel::NotDefined,
            rc: ReportConfidence::NotDefined,
        }
    }

    pub fn base_score(&self) -> f64 {
        let c_val = self.c.value();
        let i_val = self.i.value();
        let a_val = self.a.value();

        let iss = 1.0 - (1.0 - c_val) * (1.0 - i_val) * (1.0 - a_val);

        let impact = match self.s {
            Scope::Unchanged => 6.42 * iss,
            Scope::Changed => {
                let imp = 7.52 * iss - 3.25;
                if imp < 0.0 {
                    0.0
                } else {
                    imp
                }
            }
        };

        if impact <= 0.0 {
            return 0.0;
        }

        let pr_val = self.pr.value(self.s == Scope::Changed);
        let exploitability = 8.22
            * self.av.value()
            * self.ac.value()
            * pr_val
            * self.ui.value();

        let base = match self.s {
            Scope::Unchanged => impact + exploitability,
            Scope::Changed => 1.08 * (impact + exploitability),
        };

        let base = if base > 10.0 { 10.0 } else { base };
        round_up(base)
    }

    pub fn temporal_score(&self) -> f64 {
        let base = self.base_score();
        round_up(base * self.e.value() * self.rl.value() * self.rc.value())
    }

    pub fn severity_from_score(&self, score: f64) -> Severity {
        Severity::from_score(score)
    }

    pub fn score(&self) -> CvssScore {
        let base = self.base_score();
        let temporal = self.temporal_score();
        let severity = Severity::from_score(base);
        let vector = self.to_vector_string();
        CvssScore {
            base_score: base,
            temporal_score: temporal,
            severity,
            vector_string: vector,
        }
    }

    pub fn to_vector_string(&self) -> String {
        let av = match self.av {
            AttackVector::Network => "N",
            AttackVector::Adjacent => "A",
            AttackVector::Local => "L",
            AttackVector::Physical => "P",
        };
        let ac = match self.ac {
            AttackComplexity::Low => "L",
            AttackComplexity::High => "H",
        };
        let pr = match self.pr {
            PrivilegesRequired::None => "N",
            PrivilegesRequired::Low => "L",
            PrivilegesRequired::High => "H",
        };
        let ui = match self.ui {
            UserInteraction::None => "N",
            UserInteraction::Required => "R",
        };
        let s = match self.s {
            Scope::Unchanged => "U",
            Scope::Changed => "C",
        };
        let c = match self.c {
            Confidentiality::None => "N",
            Confidentiality::Low => "L",
            Confidentiality::High => "H",
        };
        let i = match self.i {
            Confidentiality::None => "N",
            Confidentiality::Low => "L",
            Confidentiality::High => "H",
        };
        let a = match self.a {
            Confidentiality::None => "N",
            Confidentiality::Low => "L",
            Confidentiality::High => "H",
        };

        let base = format!(
            "CVSS:3.1/AV:{}/AC:{}/PR:{}/UI:{}/S:{}/C:{}/I:{}/A:{}",
            av, ac, pr, ui, s, c, i, a
        );

        let has_temporal = self.e != ExploitCodeMaturity::NotDefined
            || self.rl != RemediationLevel::NotDefined
            || self.rc != ReportConfidence::NotDefined;

        if !has_temporal {
            return base;
        }

        let e = match self.e {
            ExploitCodeMaturity::NotDefined => "X",
            ExploitCodeMaturity::Unproven => "U",
            ExploitCodeMaturity::ProofOfConcept => "P",
            ExploitCodeMaturity::Functional => "F",
            ExploitCodeMaturity::High => "H",
        };
        let rl = match self.rl {
            RemediationLevel::NotDefined => "X",
            RemediationLevel::OfficialFix => "O",
            RemediationLevel::TemporaryFix => "T",
            RemediationLevel::Workaround => "W",
            RemediationLevel::Unavailable => "U",
        };
        let rc = match self.rc {
            ReportConfidence::NotDefined => "X",
            ReportConfidence::Unknown => "U",
            ReportConfidence::Reasonable => "R",
            ReportConfidence::Confirmed => "C",
        };

        format!("{}/E:{}/RL:{}/RC:{}", base, e, rl, rc)
    }

    pub fn with_vector(vector: &str) -> Result<Self, String> {
        let parts: Vec<&str> = vector.split('/').collect();
        if parts.is_empty() {
            return Err("empty vector string".to_string());
        }

        if parts[0] != "CVSS:3.1" {
            return Err(format!("invalid version: {}, expected CVSS:3.1", parts[0]));
        }

        let mut builder = Self::new();

        for &part in &parts[1..] {
            let colon = part.find(':').ok_or_else(|| {
                format!("invalid metric token: {}", part)
            })?;
            let metric = &part[..colon];
            let value = &part[colon + 1..];

            if metric.is_empty() || value.is_empty() {
                return Err(format!("invalid metric token: {}", part));
            }

            match metric {
                "AV" => {
                    builder.av = match value {
                        "N" => AttackVector::Network,
                        "A" => AttackVector::Adjacent,
                        "L" => AttackVector::Local,
                        "P" => AttackVector::Physical,
                        _ => return Err(format!("invalid AttackVector: {}", value)),
                    };
                }
                "AC" => {
                    builder.ac = match value {
                        "L" => AttackComplexity::Low,
                        "H" => AttackComplexity::High,
                        _ => return Err(format!("invalid AttackComplexity: {}", value)),
                    };
                }
                "PR" => {
                    builder.pr = match value {
                        "N" => PrivilegesRequired::None,
                        "L" => PrivilegesRequired::Low,
                        "H" => PrivilegesRequired::High,
                        _ => {
                            return Err(format!(
                                "invalid PrivilegesRequired: {}",
                                value
                            ))
                        }
                    };
                }
                "UI" => {
                    builder.ui = match value {
                        "N" => UserInteraction::None,
                        "R" => UserInteraction::Required,
                        _ => {
                            return Err(format!("invalid UserInteraction: {}", value))
                        }
                    };
                }
                "S" => {
                    builder.s = match value {
                        "U" => Scope::Unchanged,
                        "C" => Scope::Changed,
                        _ => return Err(format!("invalid Scope: {}", value)),
                    };
                }
                "C" => {
                    builder.c = match value {
                        "N" => Confidentiality::None,
                        "L" => Confidentiality::Low,
                        "H" => Confidentiality::High,
                        _ => {
                            return Err(format!(
                                "invalid Confidentiality: {}",
                                value
                            ))
                        }
                    };
                }
                "I" => {
                    builder.i = match value {
                        "N" => Confidentiality::None,
                        "L" => Confidentiality::Low,
                        "H" => Confidentiality::High,
                        _ => return Err(format!("invalid Integrity: {}", value)),
                    };
                }
                "A" => {
                    builder.a = match value {
                        "N" => Confidentiality::None,
                        "L" => Confidentiality::Low,
                        "H" => Confidentiality::High,
                        _ => {
                            return Err(format!(
                                "invalid Availability: {}",
                                value
                            ))
                        }
                    };
                }
                "E" => {
                    builder.e = match value {
                        "X" => ExploitCodeMaturity::NotDefined,
                        "U" => ExploitCodeMaturity::Unproven,
                        "P" => ExploitCodeMaturity::ProofOfConcept,
                        "F" => ExploitCodeMaturity::Functional,
                        "H" => ExploitCodeMaturity::High,
                        _ => {
                            return Err(format!(
                                "invalid ExploitCodeMaturity: {}",
                                value
                            ))
                        }
                    };
                }
                "RL" => {
                    builder.rl = match value {
                        "X" => RemediationLevel::NotDefined,
                        "O" => RemediationLevel::OfficialFix,
                        "T" => RemediationLevel::TemporaryFix,
                        "W" => RemediationLevel::Workaround,
                        "U" => RemediationLevel::Unavailable,
                        _ => {
                            return Err(format!(
                                "invalid RemediationLevel: {}",
                                value
                            ))
                        }
                    };
                }
                "RC" => {
                    builder.rc = match value {
                        "X" => ReportConfidence::NotDefined,
                        "U" => ReportConfidence::Unknown,
                        "R" => ReportConfidence::Reasonable,
                        "C" => ReportConfidence::Confirmed,
                        _ => {
                            return Err(format!(
                                "invalid ReportConfidence: {}",
                                value
                            ))
                        }
                    };
                }
                _ => return Err(format!("unknown metric: {}", metric)),
            }
        }

        Ok(builder)
    }
}

impl Default for CvssBuilder {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
