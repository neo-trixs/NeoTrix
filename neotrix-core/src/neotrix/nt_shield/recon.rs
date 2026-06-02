use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReconType {
    SubdomainEnumeration,
    PortScanning,
    TechnologyFingerprinting,
    EndpointDiscovery,
    EmailHarvesting,
}

#[derive(Debug, Clone)]
pub struct ReconFinding {
    pub recon_type: ReconType,
    pub target: String,
    pub value: String,
    pub confidence: f64,
    pub source: String,
    pub tags: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct ReconConfig {
    pub domain: String,
    pub recon_types: Vec<ReconType>,
    pub max_depth: u8,
    pub timeout_seconds: u64,
    pub use_passive_sources: bool,
}

#[derive(Debug, Clone)]
pub struct SubdomainInfo {
    pub domain: String,
    pub subdomain: String,
    pub resolved_ips: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct TechFingerprint {
    pub url: String,
    pub technologies: Vec<String>,
    pub version: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct AttackSurface {
    pub total_endpoints: usize,
    pub tech_stack: Vec<String>,
    pub exposed_services: Vec<String>,
    pub potential_risks: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ReconReport {
    pub domain: String,
    pub subdomain_count: usize,
    pub tech_count: usize,
    pub finding_count: usize,
    pub findings: Vec<ReconFinding>,
    pub subdomains: Vec<SubdomainInfo>,
    pub technologies: Vec<TechFingerprint>,
    pub attack_surface: AttackSurface,
    pub duration_ms: u64,
}

const COMMON_SUBDOMAINS: &[&str] = &[
    "www", "api", "admin", "mail", "dev", "staging", "app", "blog", "cdn",
    "docs", "support", "status", "test", "beta", "vpn", "gitlab", "jenkins",
    "grafana", "prometheus", "kibana",
];

fn simulate_dns(fqdn: &str) -> Vec<String> {
    let hash: u64 = fqdn.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as u64)
    });
    vec![format!(
        "{}.{}.{}.{}",
        (hash >> 24) & 0xFF,
        (hash >> 16) & 0xFF,
        (hash >> 8) & 0xFF,
        hash & 0xFF,
    )]
}

fn detect_technologies(url: &str) -> Vec<TechFingerprint> {
    let mut techs = Vec::new();
    let lower = url.to_lowercase();

    if lower.contains(".php") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["PHP".to_string()],
            version: None,
            confidence: 0.65,
        });
    }
    if lower.contains(".asp") || lower.contains(".aspx") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["ASP.NET".to_string()],
            version: None,
            confidence: 0.65,
        });
    }
    if lower.contains(".jsp") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["Java".to_string()],
            version: None,
            confidence: 0.65,
        });
    }
    if lower.contains("wp-content") || lower.contains("wp-admin") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["WordPress".to_string()],
            version: None,
            confidence: 0.85,
        });
    }
    if lower.contains("react") && !lower.contains("reactive") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["React".to_string()],
            version: None,
            confidence: 0.70,
        });
    }
    if lower.contains("graphql") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["GraphQL".to_string()],
            version: None,
            confidence: 0.90,
        });
    }
    if lower.contains("cloudflare") || lower.contains("cloudfront") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["Cloudflare".to_string()],
            version: None,
            confidence: 0.88,
        });
    }
    if lower.contains("nginx") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["nginx".to_string()],
            version: None,
            confidence: 0.82,
        });
    }
    if lower.contains("django") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["Django".to_string()],
            version: None,
            confidence: 0.75,
        });
    }
    if lower.contains("rails") || lower.contains("ruby") {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["Ruby on Rails".to_string()],
            version: None,
            confidence: 0.70,
        });
    }

    if techs.is_empty() {
        techs.push(TechFingerprint {
            url: url.to_string(),
            technologies: vec!["HTTP Server".to_string()],
            version: None,
            confidence: 0.30,
        });
    }

    techs
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug, Clone)]
pub struct ReconEngine {
    pub config: ReconConfig,
    pub findings: Vec<ReconFinding>,
    pub subdomains: Vec<SubdomainInfo>,
    pub technologies: Vec<TechFingerprint>,
    last_duration_ms: u64,
}

impl ReconEngine {
    pub fn new(config: ReconConfig) -> Self {
        Self {
            config,
            findings: Vec::new(),
            subdomains: Vec::new(),
            technologies: Vec::new(),
            last_duration_ms: 0,
        }
    }

    pub fn run(&mut self) -> Vec<ReconFinding> {
        self.findings.clear();
        self.subdomains.clear();
        self.technologies.clear();
        let start = Instant::now();
        let timestamp = current_timestamp();

        let recon_types = self.config.recon_types.clone();
        for recon_type in &recon_types {
            match recon_type {
                ReconType::SubdomainEnumeration => {
                    let subs = self.enumerate_subdomains();
                    for sub in &subs {
                        self.findings.push(ReconFinding {
                            recon_type: ReconType::SubdomainEnumeration,
                            target: self.config.domain.clone(),
                            value: format!("{}.{}", sub.subdomain, sub.domain),
                            confidence: 0.75,
                            source: sub.source.clone(),
                            tags: vec!["subdomain".to_string(), "dns".to_string()],
                            timestamp,
                        });
                    }
                }
                ReconType::PortScanning => {
                    let common_ports = [80, 443, 22, 8080, 8443, 3306, 5432, 6379, 27017, 3389];
                    for port in common_ports {
                        self.findings.push(ReconFinding {
                            recon_type: ReconType::PortScanning,
                            target: self.config.domain.clone(),
                            value: format!("{}:{}", self.config.domain, port),
                            confidence: 0.50,
                            source: "port_scan_simulation".to_string(),
                            tags: vec!["port".to_string(), format!("port_{}", port)],
                            timestamp,
                        });
                    }
                }
                ReconType::TechnologyFingerprinting => {
                    let target_url = format!("https://www.{}", self.config.domain);
                    let techs = self.fingerprint_tech(&target_url);
                    for tech in &techs {
                        self.findings.push(ReconFinding {
                            recon_type: ReconType::TechnologyFingerprinting,
                            target: tech.url.clone(),
                            value: tech.technologies.join(", "),
                            confidence: tech.confidence,
                            source: "header_analysis".to_string(),
                            tags: vec!["technology".to_string()],
                            timestamp,
                        });
                    }
                    self.technologies.extend(techs);
                }
                ReconType::EndpointDiscovery => {
                    let common_endpoints = [
                        "/robots.txt",
                        "/sitemap.xml",
                        "/.well-known/",
                        "/api/",
                        "/admin/",
                        "/login",
                        "/wp-admin",
                        "/graphql",
                    ];
                    for endpoint in common_endpoints {
                        self.findings.push(ReconFinding {
                            recon_type: ReconType::EndpointDiscovery,
                            target: format!("https://{}{}", self.config.domain, endpoint),
                            value: endpoint.to_string(),
                            confidence: 0.40,
                            source: "common_endpoints".to_string(),
                            tags: vec!["endpoint".to_string(), "discovery".to_string()],
                            timestamp,
                        });
                    }
                }
                ReconType::EmailHarvesting => {
                    let prefixes = ["admin@", "info@", "support@", "contact@", "webmaster@"];
                    for prefix in prefixes {
                        self.findings.push(ReconFinding {
                            recon_type: ReconType::EmailHarvesting,
                            target: self.config.domain.clone(),
                            value: format!("{}{}", prefix, self.config.domain),
                            confidence: 0.30,
                            source: "common_patterns".to_string(),
                            tags: vec!["email".to_string(), "harvesting".to_string()],
                            timestamp,
                        });
                    }
                }
            }
        }

        self.last_duration_ms = start.elapsed().as_millis() as u64;
        self.findings.clone()
    }

    pub fn enumerate_subdomains(&mut self) -> Vec<SubdomainInfo> {
        let max = self.config.max_depth as usize;
        let subdomains_to_check: Vec<&str> = if max == 0 || max >= COMMON_SUBDOMAINS.len() {
            COMMON_SUBDOMAINS.to_vec()
        } else {
            COMMON_SUBDOMAINS[..max].to_vec()
        };

        let mut results = Vec::new();
        let domain = &self.config.domain;

        for &sub in &subdomains_to_check {
            let fqdn = format!("{}.{}", sub, domain);
            let ips = simulate_dns(&fqdn);
            let source = if self.config.use_passive_sources {
                "certificate_transparency".to_string()
            } else {
                "dns_resolution".to_string()
            };

            let info = SubdomainInfo {
                domain: domain.clone(),
                subdomain: sub.to_string(),
                resolved_ips: ips,
                source,
            };
            results.push(info);
        }

        self.subdomains.extend(results.clone());
        results
    }

    pub fn fingerprint_tech(&self, url: &str) -> Vec<TechFingerprint> {
        detect_technologies(url)
    }

    pub fn report(&self) -> ReconReport {
        let attack_surface = self.attack_surface();
        ReconReport {
            domain: self.config.domain.clone(),
            subdomain_count: self.subdomains.len(),
            tech_count: self.technologies.len(),
            finding_count: self.findings.len(),
            findings: self.findings.clone(),
            subdomains: self.subdomains.clone(),
            technologies: self.technologies.clone(),
            attack_surface,
            duration_ms: self.last_duration_ms,
        }
    }

    pub fn attack_surface(&self) -> AttackSurface {
        let mut tech_stack: Vec<String> = Vec::new();
        for t in &self.technologies {
            for tech in &t.technologies {
                if !tech_stack.contains(tech) {
                    tech_stack.push(tech.clone());
                }
            }
        }

        let exposed_services: Vec<String> = self
            .subdomains
            .iter()
            .map(|s| format!("{}.{}", s.subdomain, s.domain))
            .collect();

        let mut potential_risks: Vec<String> = Vec::new();
        if self
            .subdomains
            .iter()
            .any(|s| s.subdomain == "admin")
        {
            potential_risks.push("Exposed admin interface".to_string());
        }
        if self
            .technologies
            .iter()
            .any(|t| t.technologies.contains(&"WordPress".to_string()))
        {
            potential_risks.push("Known CMS vulnerabilities".to_string());
        }
        if potential_risks.is_empty() {
            potential_risks.push("Limited attack surface detected".to_string());
        }

        let recommended_actions = vec![
            "Review exposed subdomains and services".to_string(),
            "Ensure all services are properly authenticated".to_string(),
            "Update and patch identified technology stack".to_string(),
            "Implement Web Application Firewall".to_string(),
        ];

        AttackSurface {
            total_endpoints: self.subdomains.len() + self.findings.len(),
            tech_stack,
            exposed_services,
            potential_risks,
            recommended_actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_types_config(domain: &str) -> ReconConfig {
        ReconConfig {
            domain: domain.to_string(),
            recon_types: vec![
                ReconType::SubdomainEnumeration,
                ReconType::PortScanning,
                ReconType::TechnologyFingerprinting,
                ReconType::EndpointDiscovery,
                ReconType::EmailHarvesting,
            ],
            max_depth: 5,
            timeout_seconds: 30,
            use_passive_sources: false,
        }
    }

    #[test]
    fn test_create_config_with_all_types() {
        let config = all_types_config("example.com");
        assert_eq!(config.recon_types.len(), 5);
        assert_eq!(config.domain, "example.com");
        assert_eq!(config.max_depth, 5);
        assert!(!config.use_passive_sources);
    }

    #[test]
    fn test_enumerate_subdomains_returns_expected_entries() {
        let config = ReconConfig {
            domain: "example.com".to_string(),
            recon_types: vec![ReconType::SubdomainEnumeration],
            max_depth: 20,
            timeout_seconds: 30,
            use_passive_sources: false,
        };
        let mut engine = ReconEngine::new(config);
        let results = engine.enumerate_subdomains();

        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.subdomain == "www"));
        assert!(results.iter().any(|s| s.subdomain == "api"));
        assert!(results.iter().any(|s| s.subdomain == "admin"));
        assert!(results.iter().any(|s| s.subdomain == "mail"));
        assert!(results.iter().any(|s| s.subdomain == "dev"));
        assert_eq!(results[0].domain, "example.com");
    }

    #[test]
    fn test_enumerate_subdomains_respects_max_depth() {
        let config = ReconConfig {
            domain: "test.org".to_string(),
            recon_types: vec![ReconType::SubdomainEnumeration],
            max_depth: 5,
            timeout_seconds: 30,
            use_passive_sources: false,
        };
        let mut engine = ReconEngine::new(config);
        let results = engine.enumerate_subdomains();
        assert_eq!(results.len(), 5);
        assert_eq!(results[0].subdomain, "www");
        assert_eq!(results[4].subdomain, "dev");
    }

    #[test]
    fn test_technology_fingerprinting_detects_common_tech() {
        let config = all_types_config("wordpress-blog.com");
        let engine = ReconEngine::new(config);

        let techs =
            engine.fingerprint_tech("https://www.wordpress-blog.com/wp-admin/index.php");
        assert!(!techs.is_empty());
        let tech_names: Vec<&str> =
            techs.iter().flat_map(|t| t.technologies.iter().map(|s| s.as_str())).collect();
        assert!(
            tech_names.contains(&"WordPress"),
            "WordPress should be detected: {:?}",
            tech_names
        );
    }

    #[test]
    fn test_attack_surface_format() {
        let config = all_types_config("example.com");
        let mut engine = ReconEngine::new(config);
        engine.run();

        let surface = engine.attack_surface();
        assert!(!surface.exposed_services.is_empty());
        assert!(!surface.potential_risks.is_empty());
        assert!(!surface.recommended_actions.is_empty());
        assert!(surface.total_endpoints > 0);
    }

    #[test]
    fn test_report_includes_all_findings() {
        let config = all_types_config("example.com");
        let mut engine = ReconEngine::new(config);
        engine.run();

        let report = engine.report();
        assert_eq!(report.domain, "example.com");
        assert!(report.finding_count > 0);
        assert!(report.subdomain_count > 0);
        let all_subdomains: Vec<&str> =
            report.subdomains.iter().map(|s| s.subdomain.as_str()).collect();
        assert!(all_subdomains.contains(&"www"));
    }

    #[test]
    fn test_passive_mode_skips_active_techniques() {
        let config = ReconConfig {
            domain: "example.com".to_string(),
            recon_types: vec![ReconType::SubdomainEnumeration],
            max_depth: 3,
            timeout_seconds: 30,
            use_passive_sources: true,
        };
        let mut engine = ReconEngine::new(config);
        let results = engine.enumerate_subdomains();

        assert!(results.iter().all(|s| s.source == "certificate_transparency"));
    }

    #[test]
    fn test_subdomain_info_has_correct_fields() {
        let config = ReconConfig {
            domain: "example.com".to_string(),
            recon_types: vec![ReconType::SubdomainEnumeration],
            max_depth: 1,
            timeout_seconds: 30,
            use_passive_sources: false,
        };
        let mut engine = ReconEngine::new(config);
        let results = engine.enumerate_subdomains();

        assert_eq!(results.len(), 1);
        let info = &results[0];
        assert_eq!(info.domain, "example.com");
        assert_eq!(info.subdomain, "www");
        assert!(!info.resolved_ips.is_empty());
        assert_eq!(info.source, "dns_resolution");
    }

    #[test]
    fn test_tech_fingerprint_confidence_scoring() {
        let config = all_types_config("cloudflare-graphql.com");
        let engine = ReconEngine::new(config);
        let techs =
            engine.fingerprint_tech("https://api.cloudflare-graphql.com/graphql");

        assert!(!techs.is_empty());
        for t in &techs {
            assert!(
                t.confidence >= 0.0 && t.confidence <= 1.0,
                "Confidence out of range: {}",
                t.confidence
            );
        }

        let graphql = techs.iter().find(|t| t.technologies.contains(&"GraphQL".to_string()));
        assert!(graphql.is_some());
        if let Some(g) = graphql {
            assert!(g.confidence >= 0.8);
        }
    }

    #[test]
    fn test_empty_domain_handling() {
        let config = ReconConfig {
            domain: String::new(),
            recon_types: vec![ReconType::SubdomainEnumeration],
            max_depth: 5,
            timeout_seconds: 30,
            use_passive_sources: false,
        };
        let mut engine = ReconEngine::new(config);
        let results = engine.enumerate_subdomains();
        assert!(!results.is_empty());
        for r in &results {
            assert!(!r.subdomain.is_empty());
        }
    }

    #[test]
    fn test_finding_deduplication() {
        let config = all_types_config("example.com");
        let mut engine = ReconEngine::new(config);

        let first = engine.run();
        let first_count = first.len();
        let engine_findings_count = engine.findings.len();
        assert_eq!(first_count, engine_findings_count);

        let second = engine.run();
        assert_eq!(second.len(), first_count);
    }

    #[test]
    fn test_multiple_recon_types_produce_different_findings() {
        let config = all_types_config("example.com");
        let mut engine = ReconEngine::new(config);
        let findings = engine.run();

        let subdomain_findings: Vec<&ReconFinding> =
            findings.iter().filter(|f| f.recon_type == ReconType::SubdomainEnumeration).collect();
        let port_findings: Vec<&ReconFinding> =
            findings.iter().filter(|f| f.recon_type == ReconType::PortScanning).collect();
        let tech_findings: Vec<&ReconFinding> = findings
            .iter()
            .filter(|f| f.recon_type == ReconType::TechnologyFingerprinting)
            .collect();
        let endpoint_findings: Vec<&ReconFinding> =
            findings.iter().filter(|f| f.recon_type == ReconType::EndpointDiscovery).collect();
        let email_findings: Vec<&ReconFinding> =
            findings.iter().filter(|f| f.recon_type == ReconType::EmailHarvesting).collect();

        assert!(!subdomain_findings.is_empty());
        assert!(!port_findings.is_empty());
        assert!(!tech_findings.is_empty());
        assert!(!endpoint_findings.is_empty());
        assert!(!email_findings.is_empty());

        assert_eq!(
            subdomain_findings.len() + port_findings.len() + tech_findings.len()
                + endpoint_findings.len() + email_findings.len(),
            findings.len()
        );
    }

    #[test]
    fn test_recon_engine_new_default_state() {
        let config = all_types_config("example.com");
        let engine = ReconEngine::new(config);
        assert!(engine.findings.is_empty());
        assert!(engine.subdomains.is_empty());
        assert!(engine.technologies.is_empty());
    }

    #[test]
    fn test_attack_surface_admin_detected() {
        let config = all_types_config("example.com");
        let mut engine = ReconEngine::new(config);

        let sub = SubdomainInfo {
            domain: "example.com".to_string(),
            subdomain: "admin".to_string(),
            resolved_ips: vec!["10.0.0.1".to_string()],
            source: "dns".to_string(),
        };
        engine.subdomains.push(sub);

        let surface = engine.attack_surface();
        assert!(surface.potential_risks.iter().any(|r| r.contains("admin")));
    }
}
