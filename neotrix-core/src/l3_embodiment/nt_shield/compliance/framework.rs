//! Compliance framework definitions

/// A single compliance requirement
#[derive(Debug, Clone)]
pub struct Requirement {
    /// Unique identifier for the requirement (e.g., "OWASP-A01", "ASVS-1.1.1")
    pub id: String,
    /// Short title of the requirement
    pub title: String,
    /// Detailed description of what must be satisfied
    pub description: String,
    /// Severity level: "critical", "high", "medium", "low"
    pub severity: String,
}

/// A compliance framework containing a set of requirements
#[derive(Debug, Clone)]
pub struct ComplianceFramework {
    /// Name of the framework (e.g., "OWASP Top 10", "ASVS")
    pub name: String,
    /// Version of the framework
    pub version: String,
    /// List of requirements in this framework
    pub requirements: Vec<Requirement>,
}

impl ComplianceFramework {
    /// Creates an OWASP Top 10 (2021) compliance framework
    pub fn owasp_top10() -> Self {
        Self {
            name: "OWASP Top 10".to_string(),
            version: "2021".to_string(),
            requirements: vec![
                Requirement {
                    id: "OWASP-A01".to_string(),
                    title: "Broken Access Control".to_string(),
                    description: "Restrictions on what authenticated users are allowed to do are often not properly enforced.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "OWASP-A02".to_string(),
                    title: "Cryptographic Failures".to_string(),
                    description: "Failures related to cryptography which often leads to sensitive data exposure.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "OWASP-A03".to_string(),
                    title: "Injection".to_string(),
                    description: "User-supplied data is not validated, filtered, or sanitized by the application.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "OWASP-A04".to_string(),
                    title: "Insecure Design".to_string(),
                    description: "Risks related to design flaws, missing or ineffective security controls.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "OWASP-A05".to_string(),
                    title: "Security Misconfiguration".to_string(),
                    description: "Missing appropriate security hardening across any part of the application stack.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "OWASP-A06".to_string(),
                    title: "Vulnerable and Outdated Components".to_string(),
                    description: "Using components with known vulnerabilities.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "OWASP-A07".to_string(),
                    title: "Identification and Authentication Failures".to_string(),
                    description: "Confirmation of the user's identity, authentication, and session management is critical.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "OWASP-A08".to_string(),
                    title: "Software and Data Integrity Failures".to_string(),
                    description: "Failures related to code and infrastructure that does not protect against integrity violations.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "OWASP-A09".to_string(),
                    title: "Security Logging and Monitoring Failures".to_string(),
                    description: "Insufficient logging, detection, monitoring, and active response.".to_string(),
                    severity: "medium".to_string(),
                },
                Requirement {
                    id: "OWASP-A10".to_string(),
                    title: "Server-Side Request Forgery (SSRF)".to_string(),
                    description: "SSRF flaws occur when a web application fetches a remote resource without validating the user-supplied URL.".to_string(),
                    severity: "medium".to_string(),
                },
            ],
        }
    }

    /// Creates an Application Security Verification Standard (ASVS) 4.0 compliance framework
    pub fn asvs() -> Self {
        Self {
            name: "ASVS".to_string(),
            version: "4.0".to_string(),
            requirements: vec![
                Requirement {
                    id: "ASVS-1.1.1".to_string(),
                    title: "Secure Development Lifecycle".to_string(),
                    description: "Verify the application has a secure development lifecycle, including security requirements, threat modeling, and secure coding practices.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "ASVS-1.2.1".to_string(),
                    title: "Security Architecture".to_string(),
                    description: "Verify that a security architecture is created, maintained, and reviewed.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "ASVS-2.1.1".to_string(),
                    title: "Authentication Mechanisms".to_string(),
                    description: "Verify that user authentication mechanisms are implemented with defense in depth.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "ASVS-2.2.1".to_string(),
                    title: "Password Security".to_string(),
                    description: "Verify that passwords, keys, and secrets are not hard-coded in source code.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "ASVS-3.1.1".to_string(),
                    title: "Access Control".to_string(),
                    description: "Verify that the application enforces access control at the function, method, or service level.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "ASVS-3.2.1".to_string(),
                    title: "Server-Side Access Control".to_string(),
                    description: "Verify that the application enforces access control on the server side.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "ASVS-4.1.1".to_string(),
                    title: "Input Validation".to_string(),
                    description: "Verify that input validation is enforced on the server side for all untrusted data.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "ASVS-4.2.1".to_string(),
                    title: "Output Encoding".to_string(),
                    description: "Verify that output encoding is used to prevent injection attacks.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "ASVS-5.1.1".to_string(),
                    title: "Cryptography Management".to_string(),
                    description: "Verify that a cryptographic inventory is maintained and cryptographic failures are addressed.".to_string(),
                    severity: "high".to_string(),
                },
                Requirement {
                    id: "ASVS-5.2.1".to_string(),
                    title: "Secure Communication".to_string(),
                    description: "Verify that all communication between services uses TLS or an equivalent transport security mechanism.".to_string(),
                    severity: "critical".to_string(),
                },
                Requirement {
                    id: "ASVS-6.1.1".to_string(),
                    title: "Error Handling".to_string(),
                    description: "Verify that error handling is implemented securely and does not leak sensitive information.".to_string(),
                    severity: "medium".to_string(),
                },
                Requirement {
                    id: "ASVS-6.2.1".to_string(),
                    title: "Logging Security".to_string(),
                    description: "Verify that security events are logged securely and monitored for suspicious activity.".to_string(),
                    severity: "medium".to_string(),
                },
                Requirement {
                    id: "ASVS-7.1.1".to_string(),
                    title: "Secure Configuration".to_string(),
                    description: "Verify that the application is hardened by default and uses secure configuration settings.".to_string(),
                    severity: "high".to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owasp_top10_has_ten_requirements() {
        let framework = ComplianceFramework::owasp_top10();
        assert_eq!(framework.requirements.len(), 10);
        assert_eq!(framework.name, "OWASP Top 10");
        assert_eq!(framework.version, "2021");
    }

    #[test]
    fn test_owasp_top10_ids_are_unique() {
        let framework = ComplianceFramework::owasp_top10();
        let ids: Vec<&str> = framework.requirements.iter().map(|r| r.id.as_str()).collect();
        let unique_ids: Vec<&str> = ids.iter().copied().collect::<std::collections::HashSet<_>>().into_iter().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_asvs_has_thirteen_requirements() {
        let framework = ComplianceFramework::asvs();
        assert_eq!(framework.requirements.len(), 13);
        assert_eq!(framework.name, "ASVS");
        assert_eq!(framework.version, "4.0");
    }

    #[test]
    fn test_asvs_ids_are_unique() {
        let framework = ComplianceFramework::asvs();
        let ids: Vec<&str> = framework.requirements.iter().map(|r| r.id.as_str()).collect();
        let unique_ids: Vec<&str> = ids.iter().copied().collect::<std::collections::HashSet<_>>().into_iter().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_all_requirements_have_valid_severity() {
        let valid_severities = ["critical", "high", "medium", "low"];
        let owasp = ComplianceFramework::owasp_top10();
        let asvs = ComplianceFramework::asvs();

        for req in owasp.requirements.iter().chain(asvs.requirements.iter()) {
            assert!(
                valid_severities.contains(&req.severity.as_str()),
                "Invalid severity '{}' for requirement {}",
                req.severity,
                req.id
            );
        }
    }

    #[test]
    fn test_frameworks_are_cloneable() {
        let owasp = ComplianceFramework::owasp_top10();
        let owasp_clone = owasp.clone();
        assert_eq!(owasp.name, owasp_clone.name);
        assert_eq!(owasp.requirements.len(), owasp_clone.requirements.len());
    }
}