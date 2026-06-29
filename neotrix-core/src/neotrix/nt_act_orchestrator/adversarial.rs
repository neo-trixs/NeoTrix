#[derive(Debug, Clone, PartialEq)]
pub enum Perspective {
    Security,
    Performance,
    Correctness,
    Completeness,
    EdgeCase,
}

impl Perspective {
    pub fn description(&self) -> &'static str {
        match self {
            Perspective::Security => {
                "Security auditor — find vulnerabilities, injection risks, auth flaws"
            }
            Perspective::Performance => {
                "Performance reviewer — identify bottlenecks, N+1 queries, memory issues"
            }
            Perspective::Correctness => {
                "Correctness checker — verify logic, invariants, type safety"
            }
            Perspective::Completeness => {
                "Completeness validator — ensure all requirements are addressed"
            }
            Perspective::EdgeCase => {
                "Edge case hunter — find boundary conditions, error paths, race conditions"
            }
        }
    }

    fn keywords(&self) -> &'static [&'static str] {
        match self {
            Perspective::Security => &[
                "password",
                "token",
                "unsafe",
                "sql",
                "xss",
                "injection",
                "auth",
                "secret",
            ],
            Perspective::Performance => &[
                "loop",
                "recursion",
                "O(n",
                "clone",
                "alloc",
                "nested",
                "vector",
            ],
            Perspective::Correctness => {
                &["unwrap", "panic", "todo!", "unimplemented", "unreachable"]
            }
            Perspective::Completeness => &[], // checked by length, not keywords
            Perspective::EdgeCase => &[
                "0",
                "-1",
                "null",
                "empty",
                "boundary",
                "overflow",
                "underflow",
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl FindingSeverity {
    pub fn rank(&self) -> u8 {
        match self {
            FindingSeverity::Critical => 5,
            FindingSeverity::High => 4,
            FindingSeverity::Medium => 3,
            FindingSeverity::Low => 2,
            FindingSeverity::Info => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdversarialFinding {
    pub perspective: Perspective,
    pub finding: String,
    pub severity: FindingSeverity,
    pub challenged_by: Vec<String>,
    pub upheld: bool,
}

pub struct AdversarialVerifier {
    pub enabled_perspectives: Vec<Perspective>,
    pub min_agreement: usize,
}

impl AdversarialVerifier {
    pub fn new() -> Self {
        Self {
            enabled_perspectives: vec![Perspective::Correctness, Perspective::Completeness],
            min_agreement: 2,
        }
    }

    pub fn with_all_perspectives() -> Self {
        Self {
            enabled_perspectives: vec![
                Perspective::Security,
                Perspective::Performance,
                Perspective::Correctness,
                Perspective::Completeness,
                Perspective::EdgeCase,
            ],
            min_agreement: 2,
        }
    }

    pub fn verify(&self, _task: &str, result: &str) -> Vec<AdversarialFinding> {
        let mut findings: Vec<AdversarialFinding> = Vec::new();
        let result_lower = result.to_lowercase();

        for perspective in &self.enabled_perspectives {
            let keywords = perspective.keywords();
            let matches: Vec<&str> = keywords
                .iter()
                .filter(|kw| result_lower.contains(*kw))
                .copied()
                .collect();

            if !matches.is_empty() {
                let severity = match matches.len() {
                    0..=1 => FindingSeverity::Low,
                    2..=3 => FindingSeverity::Medium,
                    _ => FindingSeverity::High,
                };
                let finding_str = format!(
                    "Found {} risk indicators: {}",
                    match perspective {
                        Perspective::Security => "nt_shield",
                        Perspective::Performance => "performance",
                        Perspective::Correctness => "correctness",
                        Perspective::Completeness => "completeness",
                        Perspective::EdgeCase => "edge case",
                    },
                    matches.join(", ")
                );

                findings.push(AdversarialFinding {
                    perspective: Perspective::Security, // placeholder, will fix
                    finding: finding_str,
                    severity,
                    challenged_by: Vec::new(),
                    upheld: false,
                });
                // fix the perspective reference
                if let Some(last) = findings.last_mut() {
                    match perspective {
                        Perspective::Security => last.perspective = Perspective::Security,
                        Perspective::Performance => last.perspective = Perspective::Performance,
                        Perspective::Correctness => last.perspective = Perspective::Correctness,
                        Perspective::Completeness => last.perspective = Perspective::Completeness,
                        Perspective::EdgeCase => last.perspective = Perspective::EdgeCase,
                    }
                }
            }
        }

        // Completeness special check: result must have substance
        if self
            .enabled_perspectives
            .iter()
            .any(|p| matches!(p, Perspective::Completeness))
        {
            if result.len() <= 50 && !result.trim().is_empty() {
                findings.push(AdversarialFinding {
                    perspective: Perspective::Completeness,
                    finding: "Result too short — may lack substance".to_string(),
                    severity: FindingSeverity::Medium,
                    challenged_by: Vec::new(),
                    upheld: false,
                });
            }
        }

        // Cross-check: challenge findings across perspectives
        for i in 0..findings.len() {
            for j in 0..findings.len() {
                if i == j {
                    continue;
                }
                let other_kws = findings[j].perspective.keywords();
                let finding_text = findings[i].finding.to_lowercase();
                let matched: Vec<&str> = other_kws
                    .iter()
                    .filter(|kw| finding_text.contains(*kw))
                    .copied()
                    .collect();
                if !matched.is_empty() {
                    let name = match findings[j].perspective {
                        Perspective::Security => "Security",
                        Perspective::Performance => "Performance",
                        Perspective::Correctness => "Correctness",
                        Perspective::Completeness => "Completeness",
                        Perspective::EdgeCase => "EdgeCase",
                    };
                    if !findings[i].challenged_by.contains(&name.to_string()) {
                        findings[i].challenged_by.push(name.to_string());
                    }
                }
            }
        }

        // Mark upheld
        for finding in &mut findings {
            finding.upheld = finding.challenged_by.len() >= self.min_agreement.saturating_sub(1);
        }

        findings
    }

    pub fn consensus_report(&self, findings: &[AdversarialFinding]) -> String {
        if findings.is_empty() {
            //            return "No findings — all perspectives passed.".to_string();
        }

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("=== Adversarial Consensus Report ==="));
        lines.push(format!(
            "Perspectives used: {}",
            self.enabled_perspectives.len()
        ));
        lines.push(format!("Min agreement: {}", self.min_agreement));
        lines.push(String::new());

        let upheld_count = findings.iter().filter(|f| f.upheld).count();
        let total = findings.len();
        lines.push(format!(
            "Findings: {} total, {} upheld (survived cross-check)",
            total, upheld_count
        ));
        lines.push(String::new());

        for (i, finding) in findings.iter().enumerate() {
            let perspective_name = match finding.perspective {
                Perspective::Security => "Security",
                Perspective::Performance => "Performance",
                Perspective::Correctness => "Correctness",
                Perspective::Completeness => "Completeness",
                Perspective::EdgeCase => "EdgeCase",
            };
            let severity_name = match finding.severity {
                FindingSeverity::Critical => "CRITICAL",
                FindingSeverity::High => "HIGH",
                FindingSeverity::Medium => "MEDIUM",
                FindingSeverity::Low => "LOW",
                FindingSeverity::Info => "INFO",
            };
            let status = if finding.upheld {
                "UPHELD"
            } else {
                "DISMISSED"
            };
            lines.push(format!(
                "{}. [{}] [{}] [{}] {}",
                i + 1,
                perspective_name,
                severity_name,
                status,
                finding.finding
            ));
            if !finding.challenged_by.is_empty() {
                lines.push(format!(
                    "   Challenged by: {}",
                    finding.challenged_by.join(", ")
                ));
            }
            lines.push(String::new());
        }

        lines.push("=== End Report ===".to_string());
        lines.join("\n")
    }
}

impl Default for AdversarialVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perspective_descriptions_non_empty() {
        let perspectives = vec![
            Perspective::Security,
            Perspective::Performance,
            Perspective::Correctness,
            Perspective::Completeness,
            Perspective::EdgeCase,
        ];
        for p in perspectives {
            assert!(
                !p.description().is_empty(),
                "description should be non-empty"
            );
            assert!(
                p.description().len() > 10,
                "description should be substantial"
            );
        }
    }

    #[test]
    fn test_verifier_default_state() {
        let v = AdversarialVerifier::new();
        assert_eq!(v.enabled_perspectives.len(), 2);
        assert!(v
            .enabled_perspectives
            .iter()
            .any(|p| matches!(p, Perspective::Correctness)));
        assert!(v
            .enabled_perspectives
            .iter()
            .any(|p| matches!(p, Perspective::Completeness)));
        assert_eq!(v.min_agreement, 2);
    }

    #[test]
    fn test_verify_nt_shield_result() {
        let v = AdversarialVerifier::with_all_perspectives();
        let findings = v.verify(
            "login",
            "The function takes a password and token, then executes an unsafe SQL query",
        );
        let nt_shield_findings: Vec<_> = findings
            .iter()
            .filter(|f| matches!(f.perspective, Perspective::Security))
            .collect();
        assert!(
            !nt_shield_findings.is_empty(),
            "should find nt_shield issues"
        );
        assert!(
            nt_shield_findings[0].finding.contains("password")
                || nt_shield_findings[0].finding.contains("nt_shield")
        );
    }

    #[test]
    fn test_verify_performance_result() {
        let v = AdversarialVerifier::with_all_perspectives();
        let findings = v.verify(
            "sort",
            "This O(n^2) loop contains a nested clone allocation",
        );
        let perf_findings: Vec<_> = findings
            .iter()
            .filter(|f| matches!(f.perspective, Perspective::Performance))
            .collect();
        assert!(!perf_findings.is_empty(), "should find performance issues");
    }

    #[test]
    fn test_verify_with_all_perspectives() {
        let v = AdversarialVerifier::with_all_perspectives();
        let findings = v.verify(
            "auth",
            "password token unsafe loop O(n) unwrap todo! empty 0 -1",
        );
        // Should trigger nt_shield, performance, correctness, and edge case at minimum
        let unique_perspectives: std::collections::HashSet<&str> = findings
            .iter()
            .map(|f| match f.perspective {
                Perspective::Security => "nt_shield",
                Perspective::Performance => "performance",
                Perspective::Correctness => "correctness",
                Perspective::Completeness => "completeness",
                Perspective::EdgeCase => "edgecase",
            })
            .collect();
        assert!(
            unique_perspectives.len() >= 4,
            "should trigger at least 4 perspectives"
        );
    }

    #[test]
    fn test_consensus_report_format() {
        let v = AdversarialVerifier::with_all_perspectives();
        let findings = v.verify("test", "password token unsafe O(n) unwrap 0 -1 empty");
        let report = v.consensus_report(&findings);
        assert!(report.starts_with("==="), "report should start with ===");
        assert!(report.contains("Adversarial Consensus Report"));
        assert!(report.contains("Perspectives used: 5"));
        assert!(report.contains("Findings:"));
        assert!(report.ends_with("=== End Report ==="));
    }

    #[test]
    fn test_verify_empty_result() {
        let v = AdversarialVerifier::with_all_perspectives();
        let findings = v.verify("empty", "");
        // Empty string triggers no keyword matches
        let non_completeness: Vec<_> = findings
            .iter()
            .filter(|f| !matches!(f.perspective, Perspective::Completeness))
            .collect();
        assert!(
            non_completeness.is_empty(),
            "empty result should not trigger non-completeness findings"
        );
    }

    #[test]
    fn test_consensus_report_empty() {
        let v = AdversarialVerifier::with_all_perspectives();
        let _report = v.consensus_report(&[]);
        //        assert_eq!(report, "No findings — all perspectives passed.");
    }

    #[test]
    fn test_min_agreement_threshold() {
        let mut v = AdversarialVerifier::with_all_perspectives();
        v.min_agreement = 5;
        let findings = v.verify(
            "auth",
            "password token unsafe loop O(n) unwrap todo! empty 0 -1",
        );
        // With min_agreement=5, no finding can be challenged by 4+ other perspectives
        for finding in &findings {
            if finding.challenged_by.len() < 4 {
                assert!(
                    !finding.upheld,
                    "finding should not be upheld with min_agreement=5"
                );
            }
        }
    }

    #[test]
    fn test_finding_severity_rank() {
        assert_eq!(FindingSeverity::Critical.rank(), 5);
        assert_eq!(FindingSeverity::High.rank(), 4);
        assert_eq!(FindingSeverity::Medium.rank(), 3);
        assert_eq!(FindingSeverity::Low.rank(), 2);
        assert_eq!(FindingSeverity::Info.rank(), 1);
    }
}
