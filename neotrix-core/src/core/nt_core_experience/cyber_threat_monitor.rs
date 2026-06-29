#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl ThreatSeverity {
    pub fn numeric_value(&self) -> f64 {
        match self {
            ThreatSeverity::Critical => 9.0,
            ThreatSeverity::High => 7.0,
            ThreatSeverity::Medium => 5.0,
            ThreatSeverity::Low => 3.0,
            ThreatSeverity::Info => 1.0,
        }
    }

    pub fn from_cvss(score: f64) -> Self {
        if score >= 9.0 {
            ThreatSeverity::Critical
        } else if score >= 7.0 {
            ThreatSeverity::High
        } else if score >= 5.0 {
            ThreatSeverity::Medium
        } else if score >= 3.0 {
            ThreatSeverity::Low
        } else {
            ThreatSeverity::Info
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttackType {
    Ransomware,
    Phishing,
    SupplyChain,
    ZeroDay,
    APT,
    DDoS,
    Malware,
    DataBreach,
    SocialEngineering,
    Vulnerability,
    Unknown,
}

impl AttackType {
    pub fn name(&self) -> &str {
        match self {
            AttackType::Ransomware => "Ransomware",
            AttackType::Phishing => "Phishing",
            AttackType::SupplyChain => "Supply Chain",
            AttackType::ZeroDay => "Zero-Day",
            AttackType::APT => "APT",
            AttackType::DDoS => "DDoS",
            AttackType::Malware => "Malware",
            AttackType::DataBreach => "Data Breach",
            AttackType::SocialEngineering => "Social Engineering",
            AttackType::Vulnerability => "Vulnerability",
            AttackType::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThreatSource {
    CVE,
    HackerNews,
    TwitterSecurity,
    VendorAdvisory { vendor: String },
    ThreatIntelFeed { name: String, url: String },
    DarkWeb,
    Custom { name: String },
}

impl ThreatSource {
    pub fn name(&self) -> String {
        match self {
            ThreatSource::CVE => "CVE".into(),
            ThreatSource::HackerNews => "The Hacker News".into(),
            ThreatSource::TwitterSecurity => "Twitter Security".into(),
            ThreatSource::VendorAdvisory { vendor } => format!("Vendor Advisory ({})", vendor),
            ThreatSource::ThreatIntelFeed { name, .. } => name.clone(),
            ThreatSource::DarkWeb => "Dark Web Intel".into(),
            ThreatSource::Custom { name } => name.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThreatStatus {
    Active,
    Monitoring,
    Mitigated,
    Resolved,
    FalsePositive,
}

impl ThreatStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, ThreatStatus::Active | ThreatStatus::Monitoring)
    }
}

#[derive(Debug, Clone)]
pub struct CveEntry {
    pub cve_id: String,
    pub description: String,
    pub cvss_score: f64,
    pub exploit_exists: bool,
    pub patch_available: bool,
    pub affected_versions: Vec<String>,
    pub published: u64,
    pub last_modified: u64,
}

#[derive(Debug, Clone)]
pub struct ThreatIntel {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub severity: ThreatSeverity,
    pub cve_id: Option<String>,
    pub affected_systems: Vec<String>,
    pub attack_type: AttackType,
    pub source: ThreatSource,
    pub published_at: u64,
    pub detected_at: u64,
    pub remediation: Option<String>,
    pub iocs: Vec<String>,
    pub tags: Vec<String>,
    pub status: ThreatStatus,
}

impl ThreatIntel {
    pub fn needs_remediation(&self) -> bool {
        matches!(self.status, ThreatStatus::Active | ThreatStatus::Monitoring)
            && self.remediation.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct ThreatBriefing {
    pub critical_count: usize,
    pub high_count: usize,
    pub new_today: usize,
    pub top_threats: Vec<ThreatIntel>,
    pub cve_updates: Vec<CveEntry>,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct CyberBriefing {
    pub alerts: Vec<String>,
}

pub struct CyberThreatMonitor {
    pub threats: Vec<ThreatIntel>,
    pub cve_cache: HashMap<String, CveEntry>,
    pub sources: Vec<ThreatSource>,
    pub max_threats: usize,
    pub update_timestamp: u64,
    next_id: u64,
}

impl Default for CyberThreatMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl CyberThreatMonitor {
    pub fn new() -> Self {
        let now = now_unix_secs();

        let mut cve_cache = HashMap::new();
        for entry in Self::seed_cves() {
            cve_cache.insert(entry.cve_id.clone(), entry);
        }

        let sources = vec![
            ThreatSource::CVE,
            ThreatSource::HackerNews,
            ThreatSource::TwitterSecurity,
            ThreatSource::VendorAdvisory {
                vendor: "Palo Alto Networks".into(),
            },
            ThreatSource::ThreatIntelFeed {
                name: "VulnCheck".into(),
                url: "https://vulncheck.com/feed".into(),
            },
        ];

        let seed_threats = vec![
            ThreatIntel {
                id: 1,
                title: "CVE-2026-11645 — Chrome V8 Remote Code Execution".into(),
                description: "A critical type confusion vulnerability in V8's Turbofan JIT compiler allows remote code execution via crafted JavaScript. Exploited in the wild as a 0-day.".into(),
                severity: ThreatSeverity::Critical,
                cve_id: Some("CVE-2026-11645".into()),
                affected_systems: vec!["Chrome < 126.0.6478".into(), "Chromium < 126.0.6478".into()],
                attack_type: AttackType::ZeroDay,
                source: ThreatSource::HackerNews,
                published_at: now - 7200,
                detected_at: now - 3600,
                remediation: Some("Update Chrome to 126.0.6478.56+. Enable site isolation and disable JIT via --jitless for high-risk users.".into()),
                iocs: vec!["js/exploit_v8_2026.js".into(), "45.33.32.156".into(), "malicious-chrome-extension-id-a7f3".into()],
                tags: vec!["chrome".into(), "v8".into(), "0-day".into(), "JIT".into(), "rce".into()],
                status: ThreatStatus::Active,
            },
            ThreatIntel {
                id: 2,
                title: "CVE-2026-42824 — Microsoft 365 SearchLeak Data Exposure".into(),
                description: "An improper access control vulnerability in Microsoft 365 search indexing exposes private email attachments to unauthorized tenant users via search queries.".into(),
                severity: ThreatSeverity::High,
                cve_id: Some("CVE-2026-42824".into()),
                affected_systems: vec!["Microsoft 365 E3/E5".into(), "Exchange Online".into(), "SharePoint Online".into()],
                attack_type: AttackType::DataBreach,
                source: ThreatSource::HackerNews,
                published_at: now - 86400,
                detected_at: now - 43200,
                remediation: Some("Apply Microsoft security update KB5021137. Review search permission boundaries and audit cross-tenant access logs.".into()),
                iocs: vec!["search-leak-poc.py".into(), "api.office.com/search".into()],
                tags: vec!["microsoft-365".into(), "data-leak".into(), "search".into(), "oauth".into()],
                status: ThreatStatus::Active,
            },
            ThreatIntel {
                id: 3,
                title: "CVE-2026-0257 — PAN-OS VPN Buffer Overflow".into(),
                description: "A stack-based buffer overflow in GlobalProtect portal allows unauthenticated remote code execution on PAN-OS firewalls. PoC published on GitHub.".into(),
                severity: ThreatSeverity::Critical,
                cve_id: Some("CVE-2026-0257".into()),
                affected_systems: vec!["PAN-OS < 11.1.3".into(), "GlobalProtect Portal".into(), "PA-Series".into()],
                attack_type: AttackType::Vulnerability,
                source: ThreatSource::VendorAdvisory { vendor: "Palo Alto Networks".into() },
                published_at: now - 36000,
                detected_at: now - 10800,
                remediation: Some("Upgrade PAN-OS to 11.1.3-h2+. Restrict GlobalProtect portal access to trusted IPs.".into()),
                iocs: vec!["exploit_pan_vpn.sh".into(), "185.130.5.204".into(), "malicious.globalprotect-config.xml".into()],
                tags: vec!["pan-os".into(), "vpn".into(), "rce".into(), "buffer-overflow".into(), "globalprotect".into()],
                status: ThreatStatus::Active,
            },
        ];

        CyberThreatMonitor {
            threats: seed_threats,
            cve_cache,
            sources,
            max_threats: 200,
            update_timestamp: now,
            next_id: 4,
        }
    }

    pub fn seed_cves() -> Vec<CveEntry> {
        vec![
            CveEntry {
                cve_id: "CVE-2026-11645".into(),
                description: "Type confusion in V8 Turbofan JIT compiler leading to out-of-bounds memory access".into(),
                cvss_score: 9.8,
                exploit_exists: true,
                patch_available: true,
                affected_versions: vec!["Chrome < 126.0.6478.56".into(), "Chromium < 126.0.6478.56".into()],
                published: 1749168000,
                last_modified: 1749254400,
            },
            CveEntry {
                cve_id: "CVE-2026-42824".into(),
                description: "Improper access control in Microsoft 365 search indexing exposes cross-tenant data".into(),
                cvss_score: 7.5,
                exploit_exists: false,
                patch_available: true,
                affected_versions: vec!["Microsoft 365 E3 (all builds before May 2026)".into(), "Microsoft 365 E5 (all builds before May 2026)".into()],
                published: 1749081600,
                last_modified: 1749168000,
            },
            CveEntry {
                cve_id: "CVE-2026-0257".into(),
                description: "Stack-based buffer overflow in GlobalProtect portal of PAN-OS".into(),
                cvss_score: 9.3,
                exploit_exists: true,
                patch_available: true,
                affected_versions: vec!["PAN-OS < 11.1.3".into(), "PAN-OS < 10.2.8".into()],
                published: 1748822400,
                last_modified: 1749254400,
            },
            CveEntry {
                cve_id: "CVE-2026-20253".into(),
                description: "Pre-auth remote code execution via Java deserialization in Splunk Enterprise".into(),
                cvss_score: 8.8,
                exploit_exists: true,
                patch_available: false,
                affected_versions: vec!["Splunk Enterprise < 9.3.2".into(), "Splunk Universal Forwarder < 9.3.2".into()],
                published: 1748649600,
                last_modified: 1749081600,
            },
            CveEntry {
                cve_id: "CVE-2026-47101".into(),
                description: "Server-side request forgery in LiteLLM proxy allows internal network scanning".into(),
                cvss_score: 6.5,
                exploit_exists: true,
                patch_available: true,
                affected_versions: vec!["LiteLLM < 1.45.0".into()],
                published: 1748736000,
                last_modified: 1749168000,
            },
        ]
    }

    pub fn ingest_threat(&mut self, threat: ThreatIntel) -> bool {
        if let Some(existing) = self.threats.iter_mut().find(|t| t.id == threat.id) {
            let updated = existing.title != threat.title
                || existing.severity != threat.severity
                || existing.status != threat.status;
            if updated {
                *existing = threat;
            }
            return updated;
        }
        if self.threats.len() >= self.max_threats {
            self.threats
                .retain(|t| t.status.is_active() || t.status == ThreatStatus::Mitigated);
        }
        self.threats.push(threat);
        true
    }

    pub fn threats_by_severity(&self, min: ThreatSeverity) -> Vec<&ThreatIntel> {
        let min_val = min.numeric_value();
        self.threats
            .iter()
            .filter(|t| t.severity.numeric_value() >= min_val)
            .collect()
    }

    pub fn threats_by_type(&self, t: AttackType) -> Vec<&ThreatIntel> {
        self.threats
            .iter()
            .filter(|threat| threat.attack_type == t)
            .collect()
    }

    pub fn threats_by_cve(&self, cve: &str) -> Option<&ThreatIntel> {
        self.threats
            .iter()
            .find(|t| t.cve_id.as_deref() == Some(cve))
    }

    pub fn active_threats(&self) -> Vec<&ThreatIntel> {
        self.threats
            .iter()
            .filter(|t| t.status.is_active())
            .collect()
    }

    pub fn briefing(&self) -> ThreatBriefing {
        let critical_count = self
            .threats
            .iter()
            .filter(|t| t.severity == ThreatSeverity::Critical)
            .count();
        let high_count = self
            .threats
            .iter()
            .filter(|t| t.severity == ThreatSeverity::High)
            .count();
        let now = now_unix_secs();
        let new_today = self
            .threats
            .iter()
            .filter(|t| now - t.detected_at < 86400)
            .count();

        let mut top: Vec<ThreatIntel> = self.threats.clone();
        top.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        top.truncate(5);

        let cve_updates: Vec<CveEntry> = self.cve_cache.values().cloned().collect();

        let summary = format!(
            "Threat Briefing: {} critical, {} high — {} new today. {} active threats tracked.",
            critical_count,
            high_count,
            new_today,
            self.threats.iter().filter(|t| t.status.is_active()).count(),
        );

        ThreatBriefing {
            critical_count,
            high_count,
            new_today,
            top_threats: top,
            cve_updates,
            summary,
        }
    }

    pub fn generate_briefing(&self) -> CyberBriefing {
        let brief = self.briefing();
        let alerts: Vec<String> = brief.top_threats.iter().map(|t| t.title.clone()).collect();
        CyberBriefing { alerts }
    }

    pub fn cve_lookup(cve_id: &str) -> Option<CveEntry> {
        let cves = Self::seed_cves();
        cves.into_iter().find(|c| c.cve_id == cve_id)
    }

    pub fn needs_immediate_action(&self) -> bool {
        self.threats
            .iter()
            .any(|t| t.severity == ThreatSeverity::Critical && t.status.is_active())
    }

    pub fn remediation_suggestion(&self, threat: &ThreatIntel) -> Option<String> {
        threat.remediation.clone().or_else(|| {
            let cve = threat.cve_id.as_ref().and_then(|id| self.cve_cache.get(id))?;
            if cve.patch_available {
                Some(format!(
                    "Apply vendor patch for {}. Check {} for details.",
                    cve.cve_id,
                    cve.cve_id
                ))
            } else {
                Some(format!(
                    "No patch available for {}. Implement mitigation controls: network segmentation, \
                     WAF rules, and monitor for exploitation attempts.",
                    cve.cve_id
                ))
            }
        })
    }

    pub fn to_report(&self) -> String {
        let now = now_unix_secs();
        let mut report = String::new();

        report.push_str("# Cybersecurity Threat Report\n\n");
        report.push_str(&format!(
            "**Generated**: {} (UNIX: {})\n\n",
            format_unix_timestamp(now),
            now
        ));
        report.push_str(&format!(
            "**Active Threats**: {} | **Tracked CVEs**: {}\n\n",
            self.threats.iter().filter(|t| t.status.is_active()).count(),
            self.cve_cache.len()
        ));
        report.push_str("---\n\n");

        let mut critical: Vec<&ThreatIntel> = self
            .threats
            .iter()
            .filter(|t| t.severity == ThreatSeverity::Critical)
            .collect();
        critical.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));

        if !critical.is_empty() {
            report.push_str("## 🔴 Critical Threats\n\n");
            for t in &critical {
                report.push_str(&format_threat_report(t));
            }
        }

        let high: Vec<&ThreatIntel> = self
            .threats
            .iter()
            .filter(|t| t.severity == ThreatSeverity::High)
            .collect();
        if !high.is_empty() {
            report.push_str("## 🟠 High ThreatSeverity\n\n");
            for t in &high {
                report.push_str(&format_threat_report(t));
            }
        }

        report.push_str("## CVE Database\n\n");
        report.push_str("| CVE ID | CVSS | Exploit | Patch |\n");
        report.push_str("|--------|------|---------|-------|\n");
        for entry in self.cve_cache.values() {
            report.push_str(&format!(
                "| {} | {:.1} | {} | {} |\n",
                entry.cve_id,
                entry.cvss_score,
                if entry.exploit_exists {
                    "⚠️ Yes"
                } else {
                    "No"
                },
                if entry.patch_available { "✅" } else { "❌" },
            ));
        }

        report
    }

    pub fn simulate_feed_poll(&mut self) -> usize {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(1..=3);
        let now = now_unix_secs();

        let severities = [
            ThreatSeverity::Critical,
            ThreatSeverity::High,
            ThreatSeverity::Medium,
            ThreatSeverity::Low,
            ThreatSeverity::Info,
        ];
        let attack_types = [
            AttackType::Ransomware,
            AttackType::Phishing,
            AttackType::SupplyChain,
            AttackType::ZeroDay,
            AttackType::APT,
            AttackType::DDoS,
            AttackType::Malware,
            AttackType::DataBreach,
            AttackType::SocialEngineering,
            AttackType::Vulnerability,
        ];

        for _ in 0..count {
            let sev = severities[rng.gen_range(0..severities.len())];
            let atk = attack_types[rng.gen_range(0..attack_types.len())];
            let id = self.next_id;
            self.next_id += 1;

            let threat = ThreatIntel {
                id,
                title: format!("Simulated {} threat #{}", atk.name(), id),
                description: format!("Automated feed poll generated this {} threat for testing and simulation purposes.", sev.numeric_value()),
                severity: sev,
                cve_id: None,
                affected_systems: vec!["generic-system".into()],
                attack_type: atk,
                source: ThreatSource::ThreatIntelFeed {
                    name: "Simulation Feed".into(),
                    url: "https://sim.feeds.local".into(),
                },
                published_at: now - rng.gen_range(0..3600),
                detected_at: now,
                remediation: None,
                iocs: vec![],
                tags: vec!["simulated".into(), "feed-poll".into()],
                status: ThreatStatus::Active,
            };

            self.ingest_threat(threat);
        }

        count
    }

    pub fn prune_old_threats(&mut self, older_than_days: u64) {
        let cutoff = now_unix_secs() - (older_than_days * 86400);
        self.threats.retain(|t| {
            !matches!(
                t.status,
                ThreatStatus::Resolved | ThreatStatus::FalsePositive
            ) || t.detected_at >= cutoff
        });
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn format_unix_timestamp(ts: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let datetime = UNIX_EPOCH + Duration::from_secs(ts);
    let secs = datetime
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

fn format_threat_report(t: &ThreatIntel) -> String {
    let mut s = String::new();
    s.push_str(&format!("### {}\n\n", t.title));
    s.push_str(&format!("- **ThreatSeverity**: {:?}\n", t.severity));
    s.push_str(&format!("- **Type**: {}\n", t.attack_type.name()));
    s.push_str(&format!("- **Source**: {}\n", t.source.name()));
    s.push_str(&format!("- **Status**: {:?}\n", t.status));
    if let Some(cve) = &t.cve_id {
        s.push_str(&format!("- **CVE**: {}\n", cve));
    }
    s.push_str(&format!("- **Description**: {}\n", t.description));
    if !t.affected_systems.is_empty() {
        s.push_str(&format!(
            "- **Affected**: {}\n",
            t.affected_systems.join(", ")
        ));
    }
    if let Some(rem) = &t.remediation {
        s.push_str(&format!("- **Remediation**: {}\n", rem));
    }
    if !t.iocs.is_empty() {
        s.push_str(&format!("- **IOCs**: `{}`\n", t.iocs.join("`, `")));
    }
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_creation() {
        let t = ThreatIntel {
            id: 42,
            title: "Test Threat".into(),
            description: "Description".into(),
            severity: ThreatSeverity::Critical,
            cve_id: Some("CVE-2026-99999".into()),
            affected_systems: vec!["test-system".into()],
            attack_type: AttackType::Ransomware,
            source: ThreatSource::Custom {
                name: "Test Source".into(),
            },
            published_at: 1000,
            detected_at: 2000,
            remediation: None,
            iocs: vec!["ioc1".into()],
            tags: vec!["test".into()],
            status: ThreatStatus::Active,
        };
        assert_eq!(t.id, 42);
        assert!(t.needs_remediation());
        assert!(t.status.is_active());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ThreatSeverity::Critical > ThreatSeverity::High);
        assert!(ThreatSeverity::High > ThreatSeverity::Medium);
        assert!(ThreatSeverity::Medium > ThreatSeverity::Low);
        assert!(ThreatSeverity::Low > ThreatSeverity::Info);
        assert_eq!(ThreatSeverity::Critical.numeric_value(), 9.0);
        assert_eq!(ThreatSeverity::Info.numeric_value(), 1.0);
        assert_eq!(ThreatSeverity::from_cvss(9.5), ThreatSeverity::Critical);
        assert_eq!(ThreatSeverity::from_cvss(7.5), ThreatSeverity::High);
        assert_eq!(ThreatSeverity::from_cvss(2.0), ThreatSeverity::Low);
    }

    #[test]
    fn test_type_filtering() {
        let monitor = CyberThreatMonitor::new();
        let zero_days = monitor.threats_by_type(AttackType::ZeroDay);
        assert_eq!(zero_days.len(), 1);
        assert_eq!(zero_days[0].cve_id.as_deref(), Some("CVE-2026-11645"));

        let vulnerabilities = monitor.threats_by_type(AttackType::Vulnerability);
        assert_eq!(vulnerabilities.len(), 1);
    }

    #[test]
    fn test_active_threats() {
        let monitor = CyberThreatMonitor::new();
        let active = monitor.active_threats();
        assert_eq!(active.len(), 3);
        for t in &active {
            assert!(t.status.is_active());
        }
    }

    #[test]
    fn test_briefing_generation() {
        let monitor = CyberThreatMonitor::new();
        let briefing = monitor.briefing();
        assert_eq!(briefing.critical_count, 2);
        assert_eq!(briefing.high_count, 1);
        assert!(!briefing.summary.is_empty());
        assert!(!briefing.top_threats.is_empty());
        assert!(!briefing.cve_updates.is_empty());
    }

    #[test]
    fn test_cve_lookup() {
        let cve = CyberThreatMonitor::cve_lookup("CVE-2026-11645");
        assert!(cve.is_some());
        assert_eq!(cve.as_ref().unwrap().cvss_score, 9.8);
        assert!(cve.as_ref().unwrap().exploit_exists);

        let missing = CyberThreatMonitor::cve_lookup("CVE-2026-00000");
        assert!(missing.is_none());
    }

    #[test]
    fn test_immediate_action() {
        let monitor = CyberThreatMonitor::new();
        assert!(monitor.needs_immediate_action());

        let mut no_critical = CyberThreatMonitor::new();
        for t in &mut no_critical.threats {
            if t.severity == ThreatSeverity::Critical {
                t.severity = ThreatSeverity::Medium;
            }
        }
        assert!(!no_critical.needs_immediate_action());
    }

    #[test]
    fn test_feed_poll_simulation() {
        let mut monitor = CyberThreatMonitor::new();
        let initial = monitor.threats.len();
        let added = monitor.simulate_feed_poll();
        assert!(added >= 1 && added <= 3);
        assert_eq!(monitor.threats.len(), initial + added);
    }

    #[test]
    fn test_prune_old_threats() {
        let mut monitor = CyberThreatMonitor::new();
        let now = now_unix_secs();

        monitor.threats.push(ThreatIntel {
            id: 100,
            title: "Old resolved threat".into(),
            description: "Already dealt with".into(),
            severity: ThreatSeverity::Low,
            cve_id: None,
            affected_systems: vec!["legacy".into()],
            attack_type: AttackType::Unknown,
            source: ThreatSource::CVE,
            published_at: now - 86400 * 30,
            detected_at: now - 86400 * 30,
            remediation: Some("Done".into()),
            iocs: vec![],
            tags: vec![],
            status: ThreatStatus::Resolved,
        });

        monitor.threats.push(ThreatIntel {
            id: 101,
            title: "Recent resolved threat".into(),
            description: "Just resolved".into(),
            severity: ThreatSeverity::Info,
            cve_id: None,
            affected_systems: vec!["recent".into()],
            attack_type: AttackType::Phishing,
            source: ThreatSource::DarkWeb,
            published_at: now - 3600,
            detected_at: now - 3600,
            remediation: Some("Done".into()),
            iocs: vec![],
            tags: vec![],
            status: ThreatStatus::Resolved,
        });

        let before = monitor.threats.len();
        monitor.prune_old_threats(7);
        assert_eq!(monitor.threats.len(), before - 1);
        assert!(!monitor.threats.iter().any(|t| t.id == 100));
        assert!(monitor.threats.iter().any(|t| t.id == 101));
    }

    #[test]
    fn test_duplicate_ingestion() {
        let mut monitor = CyberThreatMonitor::new();
        let original_len = monitor.threats.len();

        let original = monitor.threats[0].clone();
        let dup = ThreatIntel { ..original };

        let first_insert = monitor.ingest_threat(dup);
        assert!(!first_insert);
        assert_eq!(monitor.threats.len(), original_len);
    }

    #[test]
    fn test_ingestion_updates_existing() {
        let mut monitor = CyberThreatMonitor::new();
        let original_len = monitor.threats.len();

        let original = monitor.threats[0].clone();
        let updated = ThreatIntel {
            status: ThreatStatus::Mitigated,
            remediation: Some("Patched".into()),
            ..original
        };

        let result = monitor.ingest_threat(updated);
        assert!(result);
        assert_eq!(monitor.threats.len(), original_len);
        assert_eq!(monitor.threats[0].status, ThreatStatus::Mitigated);
    }

    #[test]
    fn test_report_generation() {
        let monitor = CyberThreatMonitor::new();
        let report = monitor.to_report();
        assert!(report.contains("Cybersecurity Threat Report"));
        assert!(report.contains("CVE-2026-11645"));
        assert!(report.contains("CVE-2026-0257"));
        assert!(report.contains("Critical"));
        assert!(report.contains("CVE Database"));
    }

    #[test]
    fn test_remediation_check() {
        let monitor = CyberThreatMonitor::new();
        let threat = monitor.threats_by_cve("CVE-2026-11645").unwrap();
        let suggestion = monitor.remediation_suggestion(threat);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("Update Chrome"));

        let monitor = CyberThreatMonitor::new();
        let threat = monitor.threats_by_cve("CVE-2026-20253").unwrap();
        let suggestion = monitor.remediation_suggestion(threat);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("No patch available"));
    }

    #[test]
    fn test_severity_threshold_filtering() {
        let monitor = CyberThreatMonitor::new();
        let high_and_above = monitor.threats_by_severity(ThreatSeverity::High);
        assert_eq!(high_and_above.len(), 3);
        for t in &high_and_above {
            assert!(t.severity == ThreatSeverity::Critical || t.severity == ThreatSeverity::High);
        }

        let all = monitor.threats_by_severity(ThreatSeverity::Info);
        assert_eq!(all.len(), monitor.threats.len());
    }

    #[test]
    fn test_threat_status_transitions() {
        let mut t = ThreatIntel {
            id: 99,
            title: "Transitions".into(),
            description: "test".into(),
            severity: ThreatSeverity::Medium,
            cve_id: None,
            affected_systems: vec![],
            attack_type: AttackType::Unknown,
            source: ThreatSource::CVE,
            published_at: 0,
            detected_at: 0,
            remediation: Some("fix".into()),
            iocs: vec![],
            tags: vec![],
            status: ThreatStatus::Active,
        };
        assert!(t.status.is_active());
        assert!(!t.needs_remediation());

        t.status = ThreatStatus::Monitoring;
        assert!(t.status.is_active());

        t.status = ThreatStatus::Mitigated;
        assert!(!t.status.is_active());

        t.status = ThreatStatus::Resolved;
        assert!(!t.status.is_active());
    }
}
