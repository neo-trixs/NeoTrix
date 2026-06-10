pub mod http_client;
pub mod blocklist;
pub mod self_iterating;
pub mod tor_client;
pub mod system_fingerprint;
pub mod proxy_chain;
pub mod network_pool;
pub mod system_proxy;
pub mod lan_router;
pub mod ip_privacy;
pub mod rules;
pub mod geo_proxy;
pub mod local_proxy;
pub mod proxy_pool;
pub mod ip_geo;
pub mod rotation_coordinator;
pub mod bandit;
pub mod stealth_browser;
pub mod config;
pub mod firewall;
pub mod rule_api;
pub mod ip_rotator;
// pub mod dns_hijack;  // removed — no standalone use case
// pub mod transparent_proxy;  // removed — depends on pf_nat
pub mod tor_crawler;
pub mod crawler_core;
pub mod crawler_parse;
pub mod pool_types;
pub mod pool_strategies;
pub mod pool_health;
pub mod ca_cert;
// pub mod pf_nat;  // removed — pf firewall rules conflict with TUN VPN, too invasive
// pub mod identity_rotator;  // removed — system-wide side effects (timezone/locale/hostname)
pub mod network_monitor;
pub mod network_diagnostics;
pub mod nt_shield_manager;
pub mod proxy_control;
pub mod proxy_heartbeat;
pub mod proxy_sourcing;
pub mod proxy_discovery;

pub use ip_geo::{IpGeoLocator, GeoResult};

pub use http_client::{StealthHttpClient, ProxyConfig, STEALTH_USER_AGENT, Response};
// pub use identity_rotator::{IdentityRotator, IdentityReport};  // removed
pub use network_monitor::NetworkMonitor;
pub use network_diagnostics::{
    NetworkDiagnosticReport, NetworkEnvironment, ApiHealth, EndpointDiagnostic,
    ConnectionFailureRootCause, VpnType, diagnose_all, scan_environment, check_endpoint,
    KNOWN_ENDPOINTS, ApiEndpoint, PredictiveNetworkMonitor, PredictiveSummary,
    FailurePrediction, TrendDir, LapStats, EwmaDetector, CusumDetector,
    MonitoredEndpoint, ErrorRateTracker, RootCauseClassifier, ClassifiedRootCause,
    ErrorPhase, ErrorClass, TimingBreakdown, HoltWinters, HealthScore, HealthGrade,
    compute_health, RemediationEngine, RemediationAction, RemediationRisk,
    PlaybookStage, REMEDIATION_PLAYBOOK,
};
pub use blocklist::is_tracker_blocked;
pub use self_iterating::{SelfIteratingStealth, FingerprintManager, Fingerprint, StealthLearning, RotationProfile, select_profile};
pub use tor_client::{TorClient, TorConfig, TorStatus};
pub use system_fingerprint::{SystemFingerprint, SystemFingerprintConfig, SystemFingerprintGenerator, Platform, Browser, TlsFingerprintHint};
pub use proxy_chain::{DynamicProxyChain, DynamicChainSummary, ProxyNode, ProxyProtocol, ProxyPool, ProxyPoolSummary, ProxyHealth};
pub use network_pool::{NetworkResourcePool, PoolSnapshot, DnsServer, DnsProtocol, RouteNode, IpResource, default_public_dns};
pub use system_proxy::{SystemProxyManager, SystemProxyConfig, SystemProxyStatus, OsType};
pub use lan_router::{LanRouter, LanRouterSummary, LocalInterface};
pub use ip_privacy::{IpPrivacyManager, IpPrivacySummary, FakeIpConfig, FakeGeoLocation, IpSubnet};
pub use rules::{RuleEngine, OutboundRule, OutboundAction, RuleCondition, RuleOrigin, china_bypass_rules};
pub use geo_proxy::{is_china_ip, domain_resolves_to_china, is_timeout_error};
pub use local_proxy::{LocalProxy, TorManager, tor_connect};
pub use rotation_coordinator::{RotationCoordinator, RotationDomain};
pub use proxy_heartbeat::{ProxyHeartbeatEngine, HeartbeatRecord, HeartbeatSummary};
pub use config::{load as load_config, config_file_path, NeoTrixConfig, reload as reload_config};
pub use proxy_control::{ProxyControl, ProxyClient, DaemonMode};
pub use proxy_pool::NodeSelectionStrategy;
pub use firewall::{FirewallManager, FirewallStats, FirewallType, FirewallAction, FirewallRule};
pub use rule_api::{RulesApiServer, RulesApiStatus, RuleRequest};
pub use proxy_sourcing::{ProxySourceDef, RawProxy, SourceHealth, PROXIFLY_ALL, GEONODE, ALL_FREE_SOURCES};
pub use proxy_discovery::ProxyDiscoveryEngine;
pub use ip_rotator::{OsIpRotator, OsIpRotatorConfig, OsIpRotatorStats};
// pub use dns_hijack::{DnsHijacker, DnsHijackerStats};  // removed
