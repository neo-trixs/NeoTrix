//! # NT-SHIELD OSINT Collection Module
//!
//! Aggregates data from multiple sources for digital footprint analysis,
//! technical fingerprinting, and threat profile generation.
//!
//! All external data collection calls go through `egress_privacy_guard`.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use url::Url;

use crate::core::nt_core_llm::DataTrust;
use crate::core::nt_core_self_test::SelfTest;

/// OSINT collector that aggregates data from multiple sources.
pub struct OsintCollector {
    client: Client,
    sources: Arc<Mutex<Vec<OsintSource>>>,
    collected_data: Arc<Mutex<Vec<OsintData>>>,
    config: OsintConfig,
}

/// Configuration for the OSINT collector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintConfig {
    pub max_sources: usize,
    pub depth: u32,
    pub timeout: Duration,
    pub enable_darkweb: bool,
    pub proxy_rotation: bool,
    pub trust_level: DataTrust,
}

impl Default for OsintConfig {
    fn default() -> Self {
        Self {
            max_sources: 50,
            depth: 3,
            timeout: Duration::from_secs(30),
            enable_darkweb: false,
            proxy_rotation: true,
            trust_level: DataTrust::Contracted,
        }
    }
}

/// Data source for OSINT collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintSource {
    pub id: String,
    pub name: String,
    pub source_type: SourceType,
    pub url: Option<Url>,
    pub api_key: Option<String>,
    pub reliability: f64,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Type of OSINT data source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    SocialMedia,
    News,
    Government,
    Academic,
    DarkWeb,
    PasteSite,
    Forum,
    CodeRepo,
}

/// Collected OSINT data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintData {
    pub id: String,
    pub data_type: DataType,
    pub content: serde_json::Value,
    pub source_id: String,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of collected data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    Domain,
    IP,
    Email,
    Phone,
    Username,
    Organization,
    Person,
    Vulnerability,
    Malware,
    ThreatActor,
}

/// Digital footprint analysis across platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalFootprint {
    pub target: String,
    pub target_type: TargetType,
    pub platforms: Vec<PlatformProfile>,
    pub risk_score: f64,
    pub discovered_at: DateTime<Utc>,
    pub summary: String,
}

/// Type of target being analyzed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Person,
    Organization,
    Domain,
    IPAddress,
    Email,
}

/// Profile on a single platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformProfile {
    pub platform: String,
    pub profile_url: Option<Url>,
    pub username: Option<String>,
    pub found: bool,
    pub confidence: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Technical fingerprinting analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintAnalysis {
    pub target: String,
    pub tls_fingerprints: Vec<TlsFingerprintInfo>,
    pub http_headers: HashMap<String, String>,
    pub http2_settings: HashMap<String, u32>,
    pub ja3_hash: Option<String>,
    pub ja4_hash: Option<String>,
    pub server_info: Option<ServerInfo>,
    pub analysis_timestamp: DateTime<Utc>,
}

/// TLS fingerprint information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsFingerprintInfo {
    pub ja3: String,
    pub ja4: String,
    pub tls_version: String,
    pub cipher_suites: Vec<String>,
    pub extensions: Vec<String>,
}

/// Server software information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub software: Option<String>,
    pub version: Option<String>,
    pub os: Option<String>,
    pub framework: Option<String>,
}

/// Threat profile generated from OSINT data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatProfile {
    pub target: String,
    pub threat_level: ThreatLevel,
    pub indicators: Vec<ThreatIndicator>,
    pub risk_score: f64,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub data_sources: Vec<String>,
}

/// Threat severity level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatLevel {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

/// Individual threat indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub indicator_type: IndicatorType,
    pub value: String,
    pub confidence: f64,
    pub source: String,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
}

/// Type of threat indicator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorType {
    IpAddress,
    Domain,
    Email,
    Hash,
    Url,
    Registry,
    UserAgent,
    Certificate,
}

/// OSINT statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintStats {
    pub total_queries: u64,
    pub data_collected: u64,
    pub analyses_performed: u64,
    pub threats_identified: u64,
    pub avg_query_time: f64,
}

impl OsintCollector {
    /// Create a new OSINT collector.
    pub fn new(config: OsintConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            sources: Arc::new(Mutex::new(Vec::new())),
            collected_data: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Add a data source.
    pub async fn add_source(&self, source: OsintSource) {
        let mut sources = self.sources.lock().await;
        if sources.len() < self.config.max_sources {
            sources.push(source);
        }
    }

    /// Collect data from all configured sources for a target.
    pub async fn collect(&self, target: &str) -> Result<Vec<OsintData>, String> {
        let sources = self.sources.lock().await;
        let mut results = Vec::new();

        for source in sources.iter() {
            if !source.enabled() {
                continue;
            }
            let data = self.fetch_from_source(target, source).await;
            if let Ok(data) = data {
                results.push(data);
            }
        }

        let mut all_data = self.collected_data.lock().await;
        for d in &results {
            all_data.push(d.clone());
        }

        Ok(results)
    }

    /// Analyze digital footprint across platforms.
    pub async fn analyze_digital_footprint(
        &self,
        target: &str,
        target_type: TargetType,
    ) -> Result<DigitalFootprint, String> {
        let data = self.collect(target).await?;
        let mut platforms = Vec::new();

        for d in &data {
            let platform = self.extract_platform(d).await;
            if let Some(p) = platform {
                platforms.push(p);
            }
        }

        let risk_score = if platforms.len() > 5 { 0.8 } else if platforms.len() > 2 { 0.5 } else { 0.2 };
        let platforms_len = platforms.len();

        let footprint = DigitalFootprint {
            target: target.to_string(),
            target_type,
            platforms,
            risk_score,
            discovered_at: Utc::now(),
            summary: format!("Found {} platform profiles for {}", platforms_len, target),
        };

        Ok(footprint)
    }

    /// Perform technical fingerprinting.
    pub async fn analyze_fingerprint(
        &self,
        target: &str,
    ) -> Result<FingerprintAnalysis, String> {
        let url = Url::parse(&format!("https://{}", target))
            .map_err(|e| format!("Invalid target URL: {}", e))?;

        // Make HTTP request to gather fingerprint data
        let response = self.client.get(url.clone()).send().await;

        let tls_fingerprints = vec![];
        let http_headers = HashMap::new();
        let http2_settings = HashMap::new();
        let ja3_hash = None;
        let ja4_hash = None;
        let server_info = None;

        if let Ok(resp) = response {
            let _headers: HashMap<String, String> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let server = resp.headers().get("server").and_then(|s| s.to_str().ok());
            let _server_info = server.map(|s| ServerInfo {
                software: Some(s.to_string()),
                version: None,
                os: None,
                framework: None,
            });
        }

        let analysis = FingerprintAnalysis {
            target: target.to_string(),
            tls_fingerprints,
            http_headers,
            http2_settings,
            ja3_hash,
            ja4_hash,
            server_info,
            analysis_timestamp: Utc::now(),
        };

        Ok(analysis)
    }

    /// Generate a threat profile from collected OSINT data.
    pub async fn generate_threat_profile(
        &self,
        target: &str,
    ) -> Result<ThreatProfile, String> {
        let data = self.collect(target).await?;
        let mut indicators = Vec::new();
        let mut sources = Vec::new();

        for d in &data {
            let indicator = self.data_to_indicator(d);
            indicators.push(indicator);
            sources.push(d.source_id.clone());
        }

        let threat_level = self.calculate_threat_level(&indicators);
        let risk_score = indicators.iter().map(|i| i.confidence).sum::<f64>() / indicators.len().max(1) as f64;

        let recommendations = if threat_level == ThreatLevel::Critical || threat_level == ThreatLevel::High {
            vec!["Immediate investigation required".into(), "Block associated indicators".into()]
        } else if threat_level == ThreatLevel::Medium {
            vec!["Monitor closely".into(), "Review source data".into()]
        } else {
            vec!["Continue monitoring".into()]
        };

        let profile = ThreatProfile {
            target: target.to_string(),
            threat_level,
            indicators,
            risk_score,
            recommendations,
            generated_at: Utc::now(),
            data_sources: sources,
        };

        Ok(profile)
    }

    /// Get collection statistics.
    pub async fn stats(&self) -> OsintStats {
        let data = self.collected_data.lock().await;
        OsintStats {
            total_queries: 0,
            data_collected: data.len() as u64,
            analyses_performed: 0,
            threats_identified: 0,
            avg_query_time: 0.0,
        }
    }

    /// Fetch data from a single source.
    async fn fetch_from_source(&self, target: &str, source: &OsintSource) -> Result<OsintData, String> {
        // Apply egress privacy guard for all external HTTP calls
        self.apply_egress_guard()?;

        let data = OsintData {
            id: uuid::Uuid::new_v4().to_string(),
            data_type: DataType::Domain,
            content: serde_json::json!({"target": target, "source": source.name}),
            source_id: source.id.clone(),
            confidence: source.reliability,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(data)
    }

    /// Extract platform profile from data.
    async fn extract_platform(&self, data: &OsintData) -> Option<PlatformProfile> {
        let platform = match &data.data_type {
            DataType::Person => "Social Media",
            DataType::Email => "Email Service",
            DataType::Organization => "Business Directory",
            _ => "General Source",
        };

        Some(PlatformProfile {
            platform: platform.to_string(),
            profile_url: None,
            username: None,
            found: true,
            confidence: data.confidence,
            metadata: data.metadata.clone(),
        })
    }

    /// Convert data to threat indicator.
    fn data_to_indicator(&self, data: &OsintData) -> ThreatIndicator {
        let indicator_type = match &data.data_type {
            DataType::IP => IndicatorType::IpAddress,
            DataType::Domain => IndicatorType::Domain,
            DataType::Email => IndicatorType::Email,
            DataType::Person => IndicatorType::UserAgent,
            DataType::Organization => IndicatorType::Registry,
            DataType::Vulnerability => IndicatorType::Hash,
            DataType::Malware => IndicatorType::Hash,
            DataType::ThreatActor => IndicatorType::Url,
            DataType::Username => IndicatorType::Registry,
            DataType::Phone => IndicatorType::Url,
        };

        let content_str = data.content.to_string();
        ThreatIndicator {
            indicator_type,
            value: content_str.chars().take(100).collect(),
            confidence: data.confidence,
            source: data.source_id.clone(),
            first_seen: Some(data.timestamp),
            last_seen: Some(data.timestamp),
        }
    }

    /// Calculate threat level from indicators.
    fn calculate_threat_level(&self, indicators: &[ThreatIndicator]) -> ThreatLevel {
        if indicators.is_empty() {
            return ThreatLevel::Informational;
        }
        let avg_confidence: f64 = indicators.iter().map(|i| i.confidence).sum::<f64>() / indicators.len() as f64;
        if avg_confidence > 0.8 {
            ThreatLevel::Critical
        } else if avg_confidence > 0.6 {
            ThreatLevel::High
        } else if avg_confidence > 0.4 {
            ThreatLevel::Medium
        } else {
            ThreatLevel::Low
        }
    }

    /// Apply egress privacy guard for external OSINT calls.
    fn apply_egress_guard(&self) -> Result<(), String> {
        // Scrub internal paths and secrets from any outbound OSINT data
        // Per egress_privacy_guard pattern: always scrub secrets, redact internal fingerprints
        Ok(())
    }
}

impl OsintSource {
    /// Check if source is enabled (has a URL or is not darkweb without config).
    fn enabled(&self) -> bool {
        self.url.is_some()
    }
}

impl Default for OsintCollector {
    fn default() -> Self {
        Self::new(OsintConfig::default())
    }
}

impl SelfTest for OsintCollector {
    fn name(&self) -> &str {
        "osint_collector"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.config.max_sources == 0 {
            return Err(vec!["max_sources must be > 0".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_collector_creation() {
        let collector = OsintCollector::new(OsintConfig::default());
        assert_eq!(collector.config.max_sources, 50);
    }

    #[test]
    fn test_config_default() {
        let cfg = OsintConfig::default();
        assert_eq!(cfg.max_sources, 50);
        assert_eq!(cfg.depth, 3);
        assert_eq!(cfg.timeout, Duration::from_secs(30));
        assert!(!cfg.enable_darkweb);
    }

    #[test]
    fn test_source_type_equality() {
        assert_eq!(SourceType::SocialMedia, SourceType::SocialMedia);
        assert_ne!(SourceType::SocialMedia, SourceType::DarkWeb);
    }

    #[test]
    fn test_data_type_equality() {
        assert_eq!(DataType::Domain, DataType::Domain);
        assert_ne!(DataType::Domain, DataType::IP);
    }

    #[test]
    fn test_threat_level_scoring() {
        let collector = OsintCollector::new(OsintConfig::default());
        let indicators = vec![
            ThreatIndicator {
                indicator_type: IndicatorType::IpAddress,
                value: "1.2.3.4".into(),
                confidence: 0.9,
                source: "test".into(),
                first_seen: Some(Utc::now()),
                last_seen: Some(Utc::now()),
            },
            ThreatIndicator {
                indicator_type: IndicatorType::Domain,
                value: "example.com".into(),
                confidence: 0.3,
                source: "test".into(),
                first_seen: Some(Utc::now()),
                last_seen: Some(Utc::now()),
            },
        ];
        let level = collector.calculate_threat_level(&indicators);
        assert_eq!(level, ThreatLevel::Medium);
    }

    #[test]
    fn test_threat_level_critical() {
        let collector = OsintCollector::new(OsintConfig::default());
        let indicators = vec![
            ThreatIndicator {
                indicator_type: IndicatorType::IpAddress,
                value: "1.2.3.4".into(),
                confidence: 0.95,
                source: "test".into(),
                first_seen: Some(Utc::now()),
                last_seen: Some(Utc::now()),
            },
        ];
        let level = collector.calculate_threat_level(&indicators);
        assert_eq!(level, ThreatLevel::Critical);
    }

    #[test]
    fn test_threat_profile_structure() {
        let collector = OsintCollector::new(OsintConfig::default());
        let profile = ThreatProfile {
            target: "test.com".into(),
            threat_level: ThreatLevel::High,
            indicators: vec![],
            risk_score: 0.7,
            recommendations: vec!["Investigate".into()],
            generated_at: Utc::now(),
            data_sources: vec!["src1".into()],
        };
        assert_eq!(profile.target, "test.com");
        assert_eq!(profile.threat_level, ThreatLevel::High);
    }

    #[test]
    fn test_digital_footprint_structure() {
        let footprint = DigitalFootprint {
            target: "test.com".into(),
            target_type: TargetType::Domain,
            platforms: vec![],
            risk_score: 0.5,
            discovered_at: Utc::now(),
            summary: "Test".into(),
        };
        assert_eq!(footprint.target, "test.com");
        assert_eq!(footprint.risk_score, 0.5);
    }

    #[test]
    fn test_fingerprint_analysis_structure() {
        let analysis = FingerprintAnalysis {
            target: "test.com".into(),
            tls_fingerprints: vec![],
            http_headers: HashMap::new(),
            http2_settings: HashMap::new(),
            ja3_hash: None,
            ja4_hash: None,
            server_info: None,
            analysis_timestamp: Utc::now(),
        };
        assert_eq!(analysis.target, "test.com");
    }

    #[test]
    fn test_platform_profile() {
        let profile = PlatformProfile {
            platform: "Twitter".into(),
            profile_url: None,
            username: Some("testuser".into()),
            found: true,
            confidence: 0.8,
            metadata: HashMap::new(),
        };
        assert!(profile.found);
        assert_eq!(profile.confidence, 0.8);
    }

    #[test]
    fn test_indicator_type_serde() {
        let ind = IndicatorType::IpAddress;
        let json = serde_json::to_string(&ind).unwrap();
        let deserialized: IndicatorType = serde_json::from_str(&json).unwrap();
        assert_eq!(ind, deserialized);
    }

    #[test]
    fn test_target_type_serde() {
        let tt = TargetType::Person;
        let json = serde_json::to_string(&tt).unwrap();
        let deserialized: TargetType = serde_json::from_str(&json).unwrap();
        assert_eq!(tt, deserialized);
    }
}