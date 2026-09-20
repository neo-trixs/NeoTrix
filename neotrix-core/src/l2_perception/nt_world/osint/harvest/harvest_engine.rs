use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::cert_transparency::{CertEntry, CertTransparency};
use super::email_harvester::{EmailHarvester, EmailResult};
use super::subdomain_harvester::{SubdomainHarvester, SubdomainResult};
use super::super::OsintConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestReport {
    pub domain: String,
    pub emails: Vec<EmailResult>,
    pub subdomains: Vec<SubdomainResult>,
    pub cert_entries: Vec<CertEntry>,
    pub total_findings: usize,
    pub risk_score: f64,
}

impl HarvestReport {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            emails: Vec::new(),
            subdomains: Vec::new(),
            cert_entries: Vec::new(),
            total_findings: 0,
            risk_score: 0.0,
        }
    }

    pub fn compute_totals(&mut self) {
        self.total_findings = self.emails.len() + self.subdomains.len() + self.cert_entries.len();
        self.risk_score = compute_risk_score(&self.emails, &self.subdomains, &self.cert_entries);
    }
}

fn compute_risk_score(
    emails: &[EmailResult],
    subdomains: &[SubdomainResult],
    certs: &[CertEntry],
) -> f64 {
    let email_score = (emails.len() as f64 * 0.15).min(1.0);
    let sub_score = (subdomains.len() as f64 * 0.05).min(1.0);
    let cert_score = (certs.len() as f64 * 0.03).min(1.0);
    let total = email_score + sub_score + cert_score;
    (total / 3.0).min(1.0)
}

impl std::fmt::Display for HarvestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══ Harvest Report ═══════════════════════════════════")?;
        writeln!(f, "  Domain:          {}", self.domain)?;
        writeln!(f, "  Emails found:    {}", self.emails.len())?;
        writeln!(f, "  Subdomains:      {}", self.subdomains.len())?;
        writeln!(f, "  Cert entries:    {}", self.cert_entries.len())?;
        writeln!(f, "  Total findings:  {}", self.total_findings)?;
        writeln!(f, "  Risk score:      {:.2}", self.risk_score)?;
        writeln!(f, "─────────────────────────────────────────────────────")?;
        for e in self.emails.iter().take(10) {
            writeln!(f, "    email: {} ({}, {:.0}%)", e.email, e.source, e.confidence * 100.0)?;
        }
        if self.emails.len() > 10 {
            writeln!(f, "    ... and {} more emails", self.emails.len() - 10)?;
        }
        for s in self.subdomains.iter().take(10) {
            writeln!(f, "    sub: {} [{}] ({})", s.subdomain, s.ip_addresses.join(","), s.source)?;
        }
        if self.subdomains.len() > 10 {
            writeln!(f, "    ... and {} more subdomains", self.subdomains.len() - 10)?;
        }
        for c in self.cert_entries.iter().take(5) {
            writeln!(f, "    cert: {} ({})", c.subdomain, c.issuer)?;
        }
        if self.cert_entries.len() > 5 {
            writeln!(f, "    ... and {} more cert entries", self.cert_entries.len() - 5)?;
        }
        writeln!(f, "══════════════════════════════════════════════════════")
    }
}

pub struct HarvestEngine {
    client: Client,
    email_harvester: EmailHarvester,
    subdomain_harvester: SubdomainHarvester,
    cert_transparency: CertTransparency,
}

impl HarvestEngine {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .https_only(true)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self::with_client(client)
    }

    pub fn with_client(client: Client) -> Self {
        let email_harvester = EmailHarvester::new(client.clone());
        let subdomain_harvester = SubdomainHarvester::new(client.clone());
        let cert_transparency = CertTransparency::new(client.clone());
        Self { client, email_harvester, subdomain_harvester, cert_transparency }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn harvest_all(&self, domain: &str) -> HarvestReport {
        self.harvest_all_with_config(domain, &OsintConfig::default()).await
    }

    pub async fn harvest_all_with_config(&self, domain: &str, config: &OsintConfig) -> HarvestReport {
        let mut report = HarvestReport::new(domain.to_string());

        let emails = self.email_harvester.harvest(domain, config).await;
        report.emails = emails;

        let subdomains = self.subdomain_harvester.harvest(domain, config).await;
        report.subdomains = subdomains;

        let cert_entries = self.cert_transparency.query(domain).await;
        report.cert_entries = cert_entries;

        report.compute_totals();
        report
    }
}

impl Default for HarvestEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harvest_report_new() {
        let r = HarvestReport::new("example.com".into());
        assert_eq!(r.domain, "example.com");
        assert!(r.emails.is_empty());
        assert!(r.subdomains.is_empty());
        assert!(r.cert_entries.is_empty());
        assert_eq!(r.total_findings, 0);
        assert!((r.risk_score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_risk_score_empty() {
        let score = compute_risk_score(&[], &[], &[]);
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_risk_score_high() {
        use super::super::email_harvester::{EmailResult, EmailSource};
        use super::super::subdomain_harvester::{SubdomainResult, SubdomainSource};
        let emails: Vec<EmailResult> = (0..20)
            .map(|i| EmailResult::new(format!("u{i}@x.com"), EmailSource::Dns, 0.9))
            .collect();
        let subs: Vec<SubdomainResult> = (0..50)
            .map(|i| SubdomainResult {
                subdomain: format!("s{i}.x.com"),
                ip_addresses: vec!["1.2.3.4".into()],
                source: SubdomainSource::DnsBrute,
            })
            .collect();
        let certs: Vec<CertEntry> = (0..30)
            .map(|_| CertEntry {
                subdomain: "x.com".into(),
                issuer: "ISRG".into(),
                not_before: Utc::now(),
                not_after: Utc::now(),
            })
            .collect();
        let score = compute_risk_score(&emails, &subs, &certs);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_harvest_report_compute_totals() {
        let mut r = HarvestReport::new("test.com".into());
        r.emails = vec![
            EmailResult::new("a@test.com".into(), EmailSource::Dns, 0.9),
            EmailResult::new("b@test.com".into(), EmailSource::WebSearch, 0.7),
        ];
        r.subdomains = vec![
            SubdomainResult {
                subdomain: "api.test.com".into(),
                ip_addresses: vec!["1.1.1.1".into()],
                source: super::super::subdomain_harvester::SubdomainSource::CrtSh,
            },
        ];
        r.compute_totals();
        assert_eq!(r.total_findings, 3);
        assert!(r.risk_score > 0.0);
    }

    #[test]
    fn test_harvest_report_display() {
        let r = HarvestReport::new("example.com".into());
        let display = format!("{r}");
        assert!(display.contains("Harvest Report"));
        assert!(display.contains("example.com"));
    }

    #[test]
    fn test_harvest_engine_default() {
        let _engine = HarvestEngine::new();
    }

    #[test]
    fn test_harvest_engine_with_client() {
        let client = Client::new();
        let _engine = HarvestEngine::with_client(client);
    }

    #[tokio::test]
    async fn test_harvest_all_nonexistent_domain() {
        let engine = HarvestEngine::new();
        let report = engine.harvest_all("nonexistent-domain-zzz.invalid").await;
        assert_eq!(report.domain, "nonexistent-domain-zzz.invalid");
        assert!(report.emails.is_empty());
    }

    #[test]
    fn test_risk_score_clamped() {
        let score = compute_risk_score(&[], &[], &[]);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }
}
