use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum ProvenanceLevel {
    Verified,
    SelfHosted,
    Mirrored,
    External,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DependencyRecord {
    pub name: String,
    pub version: String,
    pub provenance: ProvenanceLevel,
    pub source_url: String,
    pub is_direct: bool,
    pub known_vulnerabilities: Vec<String>,
    pub last_audited: u64,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub total_deps: usize,
    pub verified: usize,
    pub external: usize,
    pub unknown: usize,
    pub vulnerabilities: Vec<String>,
    pub risk_score: f64,
    pub recommendations: Vec<String>,
}

#[derive(Clone)]
pub struct SupplyChainGuard {
    pub dependencies: HashMap<String, DependencyRecord>,
    pub audit_history: Vec<AuditReport>,
    pub blocked_packages: HashSet<String>,
    pub max_deps: usize,
}

impl SupplyChainGuard {
    pub fn new() -> Self {
        let mut blocked = HashSet::new();
        blocked.insert("malicious-pkg".to_string());
        blocked.insert("typosquatting".to_string());
        blocked.insert("dependency-confusion-test".to_string());

        SupplyChainGuard {
            dependencies: HashMap::new(),
            audit_history: Vec::new(),
            blocked_packages: blocked,
            max_deps: 1000,
        }
    }

    pub fn register_dependency(
        &mut self,
        name: &str,
        version: &str,
        source_url: &str,
        is_direct: bool,
    ) {
        let provenance = if name == "neotrix" || name == "nt-core" {
            ProvenanceLevel::SelfHosted
        } else if source_url.contains("crates.io") || source_url.contains("github.com") {
            ProvenanceLevel::External
        } else if source_url.starts_with("https://") {
            ProvenanceLevel::Mirrored
        } else {
            ProvenanceLevel::Unknown
        };

        let record = DependencyRecord {
            name: name.to_string(),
            version: version.to_string(),
            provenance,
            source_url: source_url.to_string(),
            is_direct,
            known_vulnerabilities: Vec::new(),
            last_audited: 0,
        };

        self.dependencies.insert(name.to_string(), record);
        if self.dependencies.len() > self.max_deps {
            let oldest = self.dependencies.keys().next().cloned();
            if let Some(k) = oldest {
                self.dependencies.remove(&k);
            }
        }
    }

    pub fn check_blocked(&self, name: &str) -> bool {
        self.blocked_packages.contains(name)
            || self.blocked_packages.iter().any(|b| name.contains(b))
    }

    pub fn audit(&mut self) -> AuditReport {
        let total = self.dependencies.len();
        let mut verified = 0;
        let mut external = 0;
        let mut unknown = 0;
        let mut vulnerabilities: Vec<String> = Vec::new();
        let mut recommendations: Vec<String> = Vec::new();

        for dep in self.dependencies.values() {
            match dep.provenance {
                ProvenanceLevel::Verified | ProvenanceLevel::SelfHosted => verified += 1,
                ProvenanceLevel::External => external += 1,
                ProvenanceLevel::Unknown => {
                    unknown += 1;
                    recommendations.push(format!(
                        "unknown provenance: {} ({})",
                        dep.name, dep.version
                    ));
                }
                ProvenanceLevel::Mirrored => {
                    if dep.known_vulnerabilities.is_empty() {
                        verified += 1;
                    } else {
                        external += 1;
                    }
                }
            }
            for vuln in &dep.known_vulnerabilities {
                vulnerabilities.push(format!("{}@{}: {}", dep.name, dep.version, vuln));
            }
            if self.check_blocked(&dep.name) {
                recommendations.push(format!("BLOCKED PACKAGE DETECTED: {}", dep.name));
            }
        }

        let risk_score = if total == 0 {
            0.0
        } else {
            let unknown_ratio = unknown as f64 / total as f64;
            let vuln_penalty = (vulnerabilities.len() as f64 * 0.3).min(1.0);
            (unknown_ratio * 0.6 + vuln_penalty * 0.4).min(1.0)
        };

        let report = AuditReport {
            total_deps: total,
            verified,
            external,
            unknown,
            vulnerabilities,
            risk_score,
            recommendations,
        };

        self.audit_history.push(report.clone());
        if self.audit_history.len() > 50 {
            self.audit_history.remove(0);
        }

        report
    }

    pub fn register_cargo_dependency(&mut self, name: &str, version: &str) {
        let source = format!("https://crates.io/crates/{}", name);
        self.register_dependency(name, version, &source, true);
    }

    pub fn risk_trend(&self) -> f64 {
        if self.audit_history.len() < 2 {
            return 0.0;
        }
        let recent = &self.audit_history[self.audit_history.len() - 1];
        let previous = &self.audit_history[self.audit_history.len() - 2];
        recent.risk_score - previous.risk_score
    }
}
