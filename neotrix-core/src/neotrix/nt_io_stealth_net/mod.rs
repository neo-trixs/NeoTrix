pub mod bandit;
pub mod blocklist;
pub mod connectivity_checker;
pub mod geo_proxy;
pub mod http_client;
pub mod ip_geo;
pub mod ip_privacy;
pub mod lan_router;
pub mod local_proxy;
pub mod network_pool;
pub mod ohttp_gateway;
pub mod proxy_chain;
pub mod proxy_pool;
pub mod rotation_coordinator;
pub mod rule_importer;
pub mod rules;
pub mod self_iterating;
pub mod system_fingerprint;
pub mod system_proxy;
pub mod tor_client;

pub mod ca_cert;
pub mod captcha;
pub mod circuit_isolation;
pub mod config;
pub mod crawler_core;
pub mod crawler_integration;
pub mod crawler_parse;
pub mod firewall;
pub mod ip_rotator;
pub mod network_diagnostics;
pub mod network_monitor;
pub mod nt_shield_manager;
pub mod pool_health;
pub mod pool_strategies;
pub mod pool_types;
pub mod proxy_control;
pub mod proxy_discovery;
pub mod proxy_heartbeat;
pub mod proxy_sourcing;
pub mod rule_api;
pub mod tor_crawler;
pub mod transit_station;

pub use ip_geo::{GeoResult, IpGeoLocator};

pub use blocklist::is_tracker_blocked;
pub use captcha::{
    BrowserSession, BrowserSessionPool, CapsolverSolver, CaptchaDetection, CaptchaDetector,
    CaptchaOutcome, CaptchaSolutionManager, CaptchaSolver, CaptchaType, DummySolver,
    SessionPoolHealth, SessionPoolStats, SessionStatus, SolveResult, TwoCaptchaSolver,
};
pub use circuit_isolation::{
    global_circuit_manager, CircuitIsolationConfig, CircuitIsolationManager,
};
pub use config::{config_file_path, load as load_config, reload as reload_config, NeoTrixConfig};
pub use firewall::{
    global_firewall_manager, FirewallAction, FirewallManager, FirewallRule, FirewallStats,
    FirewallType,
};
pub use geo_proxy::{domain_resolves_to_china, is_china_ip, is_timeout_error};
pub use http_client::{stealth_user_agent, ProxyConfig, Response, StealthHttpClient};
pub use ip_privacy::{FakeGeoLocation, FakeIpConfig, IpPrivacyManager, IpPrivacySummary, IpSubnet};
pub use ip_rotator::{OsIpRotator, OsIpRotatorConfig, OsIpRotatorStats};
pub use lan_router::{LanRouter, LanRouterSummary, LocalInterface};
pub use local_proxy::{tor_connect, LocalProxy, TorManager};
pub use network_diagnostics::{
    check_endpoint, compute_health, diagnose_all, scan_environment, ApiEndpoint, ApiHealth,
    ClassifiedRootCause, ConnectionFailureRootCause, CusumDetector, EndpointDiagnostic, ErrorClass,
    ErrorPhase, ErrorRateTracker, EwmaDetector, FailurePrediction, HealthGrade, HealthScore,
    HoltWinters, LapStats, MonitoredEndpoint, NetworkDiagnosticReport, NetworkEnvironment,
    PlaybookStage, PredictiveNetworkMonitor, PredictiveSummary, RemediationAction,
    RemediationEngine, RemediationRisk, RootCauseClassifier, TimingBreakdown, TrendDir, VpnType,
    KNOWN_ENDPOINTS, REMEDIATION_PLAYBOOK,
};
pub use network_monitor::NetworkMonitor;
pub use network_pool::{
    default_public_dns, DnsProtocol, DnsServer, IpResource, NetworkResourcePool, PoolSnapshot,
    RouteNode,
};
pub use proxy_chain::{
    DynamicChainSummary, DynamicProxyChain, ProxyHealth, ProxyNode, ProxyPool, ProxyPoolSummary,
    ProxyProtocol,
};
pub use proxy_control::{DaemonMode, ProxyClient, ProxyControl};
pub use proxy_discovery::ProxyDiscoveryEngine;
pub use proxy_heartbeat::{HeartbeatRecord, HeartbeatSummary, ProxyHeartbeatEngine};
pub use proxy_pool::{NodeRole, NodeSelectionStrategy};
pub use proxy_sourcing::{
    ProxySourceDef, RawProxy, SourceHealth, ALL_FREE_SOURCES, GEONODE, PROXIFLY_ALL,
};
pub use rotation_coordinator::{RotationCoordinator, RotationDomain};
pub use rule_api::{RuleRequest, RulesApiServer, RulesApiStatus};
pub use rule_importer::{import_shadowrocket_rules, parse_shadowrocket_line};
pub use rules::{
    china_bypass_rules, OutboundAction, OutboundRule, RuleCondition, RuleEngine, RuleOrigin,
};
pub use self_iterating::{
    select_profile, Fingerprint, FingerprintManager, RotationProfile, SelfIteratingStealth,
    StealthLearning,
};
pub use system_fingerprint::{
    Browser, Platform, SystemFingerprint, SystemFingerprintConfig, SystemFingerprintGenerator,
    TlsFingerprintHint,
};
pub use system_proxy::{OsType, SystemProxyConfig, SystemProxyManager, SystemProxyStatus};
pub use tor_client::{TorClient, TorConfig, TorStatus};
pub use transit_station::{
    auto_start_transit, global_transit_station, stop_transit, RoutingRecord, TransitFingerprint,
    TransitMode, TransitStation, TransitStats,
};
