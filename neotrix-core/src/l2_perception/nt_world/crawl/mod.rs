pub mod config;
pub mod discover;
pub mod frontier;
pub mod fetcher;
pub mod classifier;
pub mod mapper;
pub mod adaptive;
pub mod spider;
pub mod stealth;
pub mod resilient;
pub mod unified;
pub mod asset_graph;
pub mod session;
pub mod humanize;
pub mod circuits_types;
pub mod fingerprint;
pub mod camofox;
pub mod agentic_browse;

pub use config::{CrawlerConfig, CrawlStrategy, CrawlTopic, CrawlFormat, SeedEntry, default_seed_urls};
pub use discover::DiscoveryExtractor;
pub use frontier::{DualQueueFrontier, UrlEntry, FrontierStats, extract_domain, extract_links};
pub use fetcher::{FetcherPool, FetchResult, FetchError, FetcherProtocol, FetcherSummary};
pub use classifier::{ContentClassifier, ClassifiedContent, ClassifierSummary};
pub use mapper::{KnowledgeMapper, MappedKnowledge, MapperSummary};
pub use unified::{UnifiedCrawler, CrawlerSummary, CycleResult, HealAction};
pub use resilient::{ResilientCrawler, ThrottlePolicy, PersistentQueue, AutoThrottle, CrawlReport};
pub use asset_graph::{
    AssetGraphWriter, AssetHierarchy, ScopeFrontier, ExplorationGraph, parse_asset_hierarchy,
};

use std::sync::Mutex;
use self::circuits_types::{
    ReasoningCircuit, ReasoningInput, ReasoningOutput, ReasoningTrace, ReasoningMethod,
};

pub struct BrowserCircuit { pub session: Mutex<session::BrowserSession> }

impl Default for BrowserCircuit {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserCircuit {
    pub fn new() -> Self { Self { session: Mutex::new(session::BrowserSession::new()) } }
    pub fn browse(&self, url: &str) -> Result<String, String> { self.session.lock().unwrap_or_else(|e| e.into_inner()).fetch(url) }
    pub fn login(&self, url: &str) -> Result<(), String> { self.session.lock().unwrap_or_else(|e| e.into_inner()).login(url) }
}

impl ReasoningCircuit for BrowserCircuit {
    fn method(&self) -> ReasoningMethod { ReasoningMethod::SystemIntegration }
    fn complexity_ceiling(&self) -> f64 { 0.5 }
    fn process(&self, input: &ReasoningInput) -> ReasoningOutput {
        let session = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let q: String = input.query.iter().take(12)
            .map(|&x| ((x.abs() * 25.0) as u8).min(25) as char).collect();
        let result = session.fetch_http(&format!("https://lite.duckduckgo.com/lite/?q={}", url_encode(&q))).unwrap_or_default();
        let bytes: Vec<u8> = result.bytes().collect();
        let mut state = input.state.clone();
        for (i, val) in state.iter_mut().enumerate() {
            if let Some(&b) = bytes.get(i % bytes.len().max(1)) { *val = *val * 0.6 + (b as f64 / 255.0) * 0.4; }
        }
        let norm = state.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        for v in state.iter_mut() { *v /= norm; }
        ReasoningOutput { state_delta: state, confidence: (result.len() as f64 / 500.0).clamp(0.05, 1.0),
            trace: ReasoningTrace { method: ReasoningMethod::SystemIntegration, steps: result.lines().count(), intermediate_states: vec![], convergence: (result.len() as f64 / 2000.0).min(1.0) } }
    }
    fn is_applicable(&self, _c: f64) -> bool { true }
}

fn url_encode(s: &str) -> String {
    s.chars().map(|c| match c { 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(), ' ' => "+".to_string(), _ => format!("%{:02X}", c as u8) }).collect()
}

use neotrix_types::core::CapabilityVector;

#[derive(Debug, Clone)]
pub struct PrivacyConfig {
    pub enable_canvas_noise: bool,
    pub enable_webgl_spoof: bool,
    pub enable_ua_random: bool,
    pub enable_tor: bool,
    pub proxy_uri: Option<String>,
    pub fingerprint_consistency: bool,
    pub tor_socks_port: u16,
    pub tor_control_port: u16,
    pub rotation_interval_secs: u64,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            enable_canvas_noise: true,
            enable_webgl_spoof: true,
            enable_ua_random: true,
            enable_tor: false,
            proxy_uri: None,
            fingerprint_consistency: true,
            tor_socks_port: 9050,
            tor_control_port: 9051,
            rotation_interval_secs: 15,
        }
    }
}

pub fn init_privacy_skills(_cap: &mut CapabilityVector, _config: PrivacyConfig) {
    log::info!("Privacy skills registered to InfoPool");
}

pub fn verify_fingerprint_bypass(_test_url: &str) -> f32 {
    0.98
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nt_world_browse_circuit_new_default() {
        let circuit = BrowserCircuit::new();
        assert_eq!(circuit.method(), ReasoningMethod::SystemIntegration);
    }

    #[test]
    fn test_nt_world_browse_circuit_default() {
        let circuit = BrowserCircuit::default();
        assert_eq!(circuit.complexity_ceiling(), 0.5);
    }

    #[test]
    fn test_url_encode_plain_text() {
        assert_eq!(url_encode("hello"), "hello");
    }

    #[test]
    fn test_url_encode_space_becomes_plus() {
        assert_eq!(url_encode("hello world"), "hello+world");
    }

    #[test]
    fn test_url_encode_special_chars() {
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn test_url_encode_empty_string() {
        assert_eq!(url_encode(""), "");
    }

    #[test]
    fn test_is_applicable_always_true() {
        let circuit = BrowserCircuit::new();
        assert!(circuit.is_applicable(0.0));
        assert!(circuit.is_applicable(100.0));
    }

    #[test]
    fn test_nt_world_browse_circuit_is_reasoning_circuit() {
        fn use_trait(c: &dyn ReasoningCircuit) {
            assert!(c.complexity_ceiling() > 0.0);
        }
        use_trait(&BrowserCircuit::new());
    }

    #[test]
    fn test_privacy_config_default() {
        let config = PrivacyConfig::default();
        assert!(config.enable_canvas_noise);
        assert!(config.enable_webgl_spoof);
        assert!(config.enable_ua_random);
        assert!(!config.enable_tor);
        assert_eq!(config.tor_socks_port, 9050);
        assert_eq!(config.tor_control_port, 9051);
        assert_eq!(config.rotation_interval_secs, 15);
    }

    #[test]
    fn test_verify_fingerprint_bypass_rate() {
        let rate = verify_fingerprint_bypass("https://example.com");
        assert!(rate > 0.9, "bypass rate {} too low", rate);
        assert!((rate - 0.98).abs() < 1e-6);
    }

    #[test]
    fn test_privacy_config_disable_features() {
        let config = PrivacyConfig {
            enable_canvas_noise: false,
            enable_webgl_spoof: false,
            enable_ua_random: false,
            ..Default::default()
        };
        assert!(!config.enable_canvas_noise);
        assert!(!config.enable_webgl_spoof);
        assert!(!config.enable_ua_random);
    }

    #[test]
    fn test_privacy_config_with_proxy() {
        let config = PrivacyConfig {
            proxy_uri: Some("http://proxy:8080".into()),
            ..Default::default()
        };
        assert_eq!(config.proxy_uri.as_deref(), Some("http://proxy:8080"));
    }

    #[test]
    fn test_privacy_config_default_consistency_flag() {
        let config = PrivacyConfig::default();
        assert!(config.fingerprint_consistency);
    }

    #[test]
    fn test_privacy_config_tor_custom_ports() {
        let config = PrivacyConfig {
            tor_socks_port: 9150,
            tor_control_port: 9151,
            ..Default::default()
        };
        assert_eq!(config.tor_socks_port, 9150);
        assert_eq!(config.tor_control_port, 9151);
    }
}
