//! EU AI Act compliance module (2026-08-02 full applicability).
//!
//! Provides watermarking, risk classification, incident reporting,
//! transparency reports, and compliance audit trail.

use std::collections::HashMap;

/// EU AI Act risk category (Article 6-51)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiActRiskCategory {
    Prohibited,
    HighRisk,
    LimitedRisk,
    MinimalRisk,
}

/// Watermark type for AI-generated content (Article 50 disclosure)
#[derive(Debug, Clone, PartialEq)]
pub enum WatermarkType {
    Metadata,
    ContentPattern,
    Cryptographic,
    Steganographic,
}

/// Specification for AI-generated content watermark
#[derive(Debug, Clone)]
pub struct WatermarkSpec {
    pub watermark_type: WatermarkType,
    pub strength: f64,
    pub detectable: bool,
    pub machine_readable: bool,
}

impl WatermarkSpec {
    pub fn metadata() -> Self {
        Self {
            watermark_type: WatermarkType::Metadata,
            strength: 1.0,
            detectable: true,
            machine_readable: true,
        }
    }
}

/// Transparency report required by EU AI Act Article 13
#[derive(Debug, Clone)]
pub struct TransparencyReport {
    pub model_name: String,
    pub version: String,
    pub provider: String,
    pub intended_use: String,
    pub risk_category: AiActRiskCategory,
    pub capabilities_assessment: HashMap<String, f64>,
    pub limitations: Vec<String>,
    pub training_data_summary: String,
    pub safety_measures: Vec<String>,
    pub generated_at: u64,
}

/// Incident type for EU AI Act Article 73 reporting
#[derive(Debug, Clone, PartialEq)]
pub enum IncidentType {
    SafetyViolation,
    PrivacyBreach,
    BiasIncident,
    OutOfScopeUse,
    PerformanceDegradation,
    Other(String),
}

/// Incident severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl IncidentSeverity {
    pub fn weight(&self) -> u32 {
        match self {
            IncidentSeverity::Critical => 10,
            IncidentSeverity::High => 6,
            IncidentSeverity::Medium => 3,
            IncidentSeverity::Low => 1,
        }
    }
}

/// Compliance incident record (Article 73)
#[derive(Debug, Clone)]
pub struct ComplianceIncident {
    pub incident_id: String,
    pub incident_type: IncidentType,
    pub severity: IncidentSeverity,
    pub description: String,
    pub affected_components: Vec<String>,
    pub mitigation: Option<String>,
    pub reported_at: u64,
    pub resolved_at: Option<u64>,
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    pub entry_id: String,
    pub action: String,
    pub component: String,
    pub details: String,
    pub timestamp: u64,
    pub success: bool,
}

/// Main compliance auditor — manages watermarking, risk, incidents, audit trail
#[derive(Debug, Clone)]
pub struct ComplianceAuditor {
    pub version: String,
    pub incidents: Vec<ComplianceIncident>,
    pub audit_log: Vec<AuditLogEntry>,
    pub watermark_key: [u8; 32],
    incident_counter: u32,
    audit_counter: u32,
}

impl Default for ComplianceAuditor {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceAuditor {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            incidents: vec![],
            audit_log: vec![],
            watermark_key: [0u8; 32],
            incident_counter: 0,
            audit_counter: 0,
        }
    }

    /// Generate a watermark for AI-generated content (metadata-based)
    pub fn generate_watermark(&self, content: &str, spec: &WatermarkSpec) -> String {
        let tag = format!(
            "<!-- neotrix-watermark: type={:?}, strength={}, machine_readable={} -->",
            spec.watermark_type, spec.strength, spec.machine_readable
        );
        match spec.watermark_type {
            WatermarkType::Metadata => format!("{}\n{}", tag, content),
            WatermarkType::ContentPattern | WatermarkType::Cryptographic | WatermarkType::Steganographic => {
                format!("{}\n{}", tag, content)
            }
        }
    }

    /// Detect if content contains a NeoTrix watermark
    pub fn detect_watermark(&self, content: &str) -> Option<WatermarkSpec> {
        if content.starts_with("<!-- neotrix-watermark:") {
            Some(WatermarkSpec::metadata())
        } else {
            None
        }
    }

    /// Classify model risk based on capabilities assessment (Article 6)
    pub fn classify_risk(capabilities: &HashMap<String, f64>) -> AiActRiskCategory {
        let harmful_caps = ["hate_speech", "self_harm", "violence", "harassment", "discrimination"];
        for cap in &harmful_caps {
            if capabilities.get(*cap).copied().unwrap_or(0.0) > 0.3 {
                return AiActRiskCategory::HighRisk;
            }
        }
        let safety_score = capabilities.get("safety_alignment").copied().unwrap_or(0.0);
        if safety_score > 0.8 {
            AiActRiskCategory::LimitedRisk
        } else if safety_score > 0.5 {
            AiActRiskCategory::HighRisk
        } else {
            AiActRiskCategory::MinimalRisk
        }
    }

    /// Generate a transparency report (Article 13)
    pub fn generate_transparency_report(
        &self,
        model_name: &str,
        version: &str,
        provider: &str,
        capabilities: &HashMap<String, f64>,
        limitations: &[String],
    ) -> TransparencyReport {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        TransparencyReport {
            model_name: model_name.to_string(),
            version: version.to_string(),
            provider: provider.to_string(),
            intended_use: "General-purpose AI assistant and reasoning engine".to_string(),
            risk_category: Self::classify_risk(capabilities),
            capabilities_assessment: capabilities.clone(),
            limitations: limitations.to_vec(),
            training_data_summary: "NeoTrix trained on curated datasets with safety filtering".to_string(),
            safety_measures: vec![
                "Constitutional AI alignment".into(),
                "Safety Kernel execution-time guard".into(),
                "Priority-tiered ethics compliance".into(),
            ],
            generated_at: now,
        }
    }

    /// Report a compliance incident (Article 73)
    pub fn report_incident(&mut self, incident: ComplianceIncident) -> String {
        self.incident_counter += 1;
        let incident_id = format!("INC-{:04}", self.incident_counter);
        let mut recorded = incident;
        recorded.incident_id.clone_from(&incident_id);
        self.incidents.push(recorded);
        self.audit("report_incident", "compliance", &format!("Incident {} reported", incident_id));
        incident_id
    }

    /// Resolve an incident
    pub fn resolve_incident(&mut self, incident_id: &str, mitigation: &str) -> bool {
        if let Some(inc) = self.incidents.iter_mut().find(|i| i.incident_id == incident_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            inc.mitigation = Some(mitigation.to_string());
            inc.resolved_at = Some(now);
            self.audit("resolve_incident", "compliance", &format!("Incident {} resolved", incident_id));
            true
        } else {
            false
        }
    }

    /// Compute overall compliance score [0, 100]
    pub fn get_compliance_score(&self) -> f64 {
        if self.incidents.is_empty() {
            return 100.0;
        }
        let total_severity: u32 = self.incidents.iter().map(|i| i.severity.weight()).sum();
        let resolved_severity: u32 = self
            .incidents
            .iter()
            .filter(|i| i.resolved_at.is_some())
            .map(|i| i.severity.weight())
            .sum();
        if total_severity == 0 {
            return 100.0;
        }
        let resolution_rate = resolved_severity as f64 / total_severity as f64;
        let base = 100.0 - (total_severity as f64) * 2.0;
        base.max(0.0).min(100.0) * (0.5 + 0.5 * resolution_rate)
    }

    /// Get audit log entries since a timestamp
    pub fn audit_log_entries_since(&self, timestamp: u64) -> Vec<&AuditLogEntry> {
        self.audit_log.iter().filter(|e| e.timestamp >= timestamp).collect()
    }

    /// Export compliance status as JSON
    pub fn export_report_json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "compliance_score": self.get_compliance_score(),
            "incident_count": self.incidents.len(),
            "incidents": self.incidents.iter().map(|i| {
                serde_json::json!({
                    "id": i.incident_id,
                    "type": format!("{:?}", i.incident_type),
                    "severity": format!("{:?}", i.severity),
                    "resolved": i.resolved_at.is_some(),
                })
            }).collect::<Vec<_>>(),
            "audit_log_entries": self.audit_log.len(),
        })
    }

    fn audit(&mut self, action: &str, component: &str, details: &str) {
        self.audit_counter += 1;
        let entry_id = format!("AUD-{:04}", self.audit_counter);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.audit_log.push(AuditLogEntry {
            entry_id,
            action: action.to_string(),
            component: component.to_string(),
            details: details.to_string(),
            timestamp: now,
            success: true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_capabilities(harmful: bool) -> HashMap<String, f64> {
        let mut caps = HashMap::new();
        caps.insert("reasoning".into(), 0.9);
        caps.insert("coding".into(), 0.85);
        caps.insert("translation".into(), 0.7);
        if harmful {
            caps.insert("hate_speech".into(), 0.5);
        }
        caps.insert("safety_alignment".into(), if harmful { 0.3 } else { 0.9 });
        caps
    }

    #[test]
    fn test_risk_classification_high_risk() {
        let caps = sample_capabilities(true);
        assert_eq!(ComplianceAuditor::classify_risk(&caps), AiActRiskCategory::HighRisk);
    }

    #[test]
    fn test_risk_classification_limited_risk() {
        let caps = sample_capabilities(false);
        assert_eq!(ComplianceAuditor::classify_risk(&caps), AiActRiskCategory::LimitedRisk);
    }

    #[test]
    fn test_watermark_generation_and_detection() {
        let auditor = ComplianceAuditor::new();
        let content = "Hello, this is AI-generated text.";
        let spec = WatermarkSpec::metadata();
        let watermarked = auditor.generate_watermark(content, &spec);
        assert!(watermarked.starts_with("<!-- neotrix-watermark:"));
        assert!(watermarked.contains(content));

        let detected = auditor.detect_watermark(&watermarked);
        assert!(detected.is_some());
        assert_eq!(detected.unwrap().watermark_type, WatermarkType::Metadata);
    }

    #[test]
    fn test_watermark_detect_no_watermark_returns_none() {
        let auditor = ComplianceAuditor::new();
        assert!(auditor.detect_watermark("plain text without watermark").is_none());
    }

    #[test]
    fn test_transparency_report_creation() {
        let auditor = ComplianceAuditor::new();
        let caps = sample_capabilities(false);
        let limitations = vec!["May hallucinate".into(), "Knowledge cutoff 2026-07".into()];
        let report = auditor.generate_transparency_report("NeoTrix", "0.19.0", "NeoTrix Corp", &caps, &limitations);
        assert_eq!(report.model_name, "NeoTrix");
        assert_eq!(report.version, "0.19.0");
        assert_eq!(report.limitations.len(), 2);
        assert!(report.generated_at > 0);
    }

    #[test]
    fn test_incident_reporting_and_resolution() {
        let mut auditor = ComplianceAuditor::new();
        let incident = ComplianceIncident {
            incident_id: String::new(),
            incident_type: IncidentType::SafetyViolation,
            severity: IncidentSeverity::High,
            description: "Model generated harmful content".into(),
            affected_components: vec!["reasoning_engine".into()],
            mitigation: None,
            reported_at: 1000,
            resolved_at: None,
        };
        let id = auditor.report_incident(incident);
        assert!(id.starts_with("INC-"));
        assert_eq!(auditor.incidents.len(), 1);

        assert!(auditor.resolve_incident(&id, "Added safety filter"));
        let resolved = auditor.incidents.first().unwrap();
        assert!(resolved.resolved_at.is_some());
        assert_eq!(resolved.mitigation.as_deref(), Some("Added safety filter"));
    }

    #[test]
    fn test_resolve_nonexistent_incident() {
        let mut auditor = ComplianceAuditor::new();
        assert!(!auditor.resolve_incident("INC-9999", "nope"));
    }

    #[test]
    fn test_compliance_score_calculation() {
        let auditor = ComplianceAuditor::new();
        assert!((auditor.get_compliance_score() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_compliance_score_with_unresolved_incidents() {
        let mut auditor = ComplianceAuditor::new();
        auditor.report_incident(ComplianceIncident {
            incident_id: String::new(),
            incident_type: IncidentType::SafetyViolation,
            severity: IncidentSeverity::Critical,
            description: "test".into(),
            affected_components: vec![],
            mitigation: None,
            reported_at: 1000,
            resolved_at: None,
        });
        let score = auditor.get_compliance_score();
        assert!(score < 100.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_audit_log_recording() {
        let mut auditor = ComplianceAuditor::new();
        auditor.report_incident(ComplianceIncident {
            incident_id: String::new(),
            incident_type: IncidentType::BiasIncident,
            severity: IncidentSeverity::Low,
            description: "test".into(),
            affected_components: vec![],
            mitigation: None,
            reported_at: 2000,
            resolved_at: None,
        });
        assert!(auditor.audit_log.len() >= 1);
        assert!(auditor.audit_log.first().unwrap().entry_id.starts_with("AUD-"));
        assert_eq!(auditor.audit_log.first().unwrap().action, "report_incident");
    }

    #[test]
    fn test_export_report_json_structure() {
        let auditor = ComplianceAuditor::new();
        let json = auditor.export_report_json();
        assert_eq!(json["compliance_score"].as_f64(), Some(100.0));
        assert_eq!(json["incident_count"].as_u64(), Some(0));
        assert!(json["version"].is_string());
    }

    #[test]
    fn test_incident_severity_weight_ordering() {
        assert!(IncidentSeverity::Critical.weight() > IncidentSeverity::High.weight());
        assert!(IncidentSeverity::High.weight() > IncidentSeverity::Medium.weight());
        assert!(IncidentSeverity::Medium.weight() > IncidentSeverity::Low.weight());
    }

    #[test]
    fn test_audit_log_entries_since() {
        let mut auditor = ComplianceAuditor::new();
        auditor.report_incident(ComplianceIncident {
            incident_id: String::new(),
            incident_type: IncidentType::Other("test".into()),
            severity: IncidentSeverity::Low,
            description: "test".into(),
            affected_components: vec![],
            mitigation: None,
            reported_at: 3000,
            resolved_at: None,
        });
        let recent = auditor.audit_log_entries_since(0);
        assert_eq!(recent.len(), 1);
    }
}
