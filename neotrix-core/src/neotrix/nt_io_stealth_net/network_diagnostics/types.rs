use std::collections::{HashMap, VecDeque};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorPhase {
    Dns,
    Tcp,
    Tls,
    Http,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorClass {
    Dns {
        subtype: DnsErrorSubtype,
    },
    Tcp {
        subtype: TcpErrorSubtype,
    },
    Tls {
        subtype: TlsErrorSubtype,
    },
    Http {
        subtype: HttpErrorSubtype,
        status: Option<u16>,
    },
    Unknown(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsErrorSubtype {
    NameNotResolved,
    NoAddress,
    Timeout,
    FakeIp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpErrorSubtype {
    TimedOut,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    HostUnreachable,
    NetworkUnreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsErrorSubtype {
    CertDateInvalid,
    CertAuthorityInvalid,
    CertNameInvalid,
    HandshakeFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpErrorSubtype {
    ServiceUnavailable,
    TooManyRequests,
    InternalError,
    BadRequest,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TimingBreakdown {
    pub dns_lookup: Duration,
    pub tcp_connect: Duration,
    pub tls_handshake: Duration,
    pub ttfb: Duration,
    pub total: Duration,
}

#[derive(Debug, Clone)]
pub struct LapStats {
    pub z_score: f64,
    pub trend: TrendDir,
    pub prediction: FailurePrediction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDir {
    Stable,
    Rising,
    Falling,
    Spike,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FailurePrediction {
    None,
    Watch,
    Warning,
    Imminent,
}

#[derive(Debug, Clone)]
pub struct HealthScore {
    pub overall: f64,
    pub latency_score: f64,
    pub error_score: f64,
    pub dns_score: f64,
    pub grade: HealthGrade,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthGrade {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybookStage {
    Idle,
    Detecting,
    Diagnosing,
    Approving,
    Executing,
    Verifying,
    RollingBack,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RemediationRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct RemediationAction {
    pub name: &'static str,
    pub description: &'static str,
    pub risk: RemediationRisk,
    pub reversible: bool,
    pub execute: fn() -> Result<String, String>,
    pub verify: fn() -> bool,
    pub rollback: Option<fn() -> Result<String, String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiHealth {
    Healthy {
        latency: Duration,
        ttfb: Duration,
    },
    Degraded {
        latency: Duration,
        ttfb: Duration,
    },
    Unreachable {
        reason: String,
    },
    DnsFailure {
        reason: String,
    },
    TlsFailure {
        reason: String,
    },
    HttpError {
        status_code: u16,
        body_snippet: String,
    },
}

#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub name: &'static str,
    pub url: &'static str,
    pub expected_status: u16,
}

pub static KNOWN_ENDPOINTS: &[ApiEndpoint] = &[
    ApiEndpoint {
        name: "opencode",
        url: "https://api.opencode.ai/v1/models",
        expected_status: 200,
    },
    ApiEndpoint {
        name: "opencode-chat",
        url: "https://api.opencode.ai/v1/chat/completions",
        expected_status: 200,
    },
    ApiEndpoint {
        name: "openai",
        url: "https://api.openai.com/v1/models",
        expected_status: 401,
    },
    ApiEndpoint {
        name: "anthropic",
        url: "https://api.anthropic.com/v1/messages",
        expected_status: 401,
    },
];

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionFailureRootCause {
    FakeIpDns,
    ConnectionTimeout,
    TlsHandshakeFailed,
    ConnectionReset,
    ConnectionRefused,
    DnsResolutionFailed,
    HttpServiceUnavailable,
    ProviderTooSlow { ttfb: Duration },
    VpnRoutingIssue,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct EndpointDiagnostic {
    pub endpoint: &'static str,
    pub health: ApiHealth,
    pub timing: Option<TimingBreakdown>,
    pub root_cause: Option<ConnectionFailureRootCause>,
}

#[derive(Debug, Clone)]
pub struct NetworkEnvironment {
    pub has_tun_interfaces: Vec<String>,
    pub fake_ip_dns: Option<String>,
    pub physical_ip: Option<String>,
    pub default_interface: Option<String>,
    pub vpn_type: Option<VpnType>,
    pub egress_country: Option<String>,
    pub egress_org: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VpnType {
    Shadowrocket,
    Surge,
    Clash,
    UnknownTun,
}

#[derive(Debug, Clone)]
pub struct NetworkDiagnosticReport {
    pub environment: NetworkEnvironment,
    pub endpoints: Vec<EndpointDiagnostic>,
    pub has_issue: bool,
    pub primary_root_cause: Option<ConnectionFailureRootCause>,
    pub recommendation: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ClassifiedRootCause {
    pub cause: ConnectionFailureRootCause,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

pub struct ErrorRateTracker {
    pub(crate) window: VecDeque<bool>,
    pub(crate) capacity: usize,
}

pub struct MonitoredEndpoint {
    pub name: &'static str,
    pub ttfb_ewma: super::protocol::EwmaDetector,
    pub ttfb_cusum: super::protocol::CusumDetector,
    pub tcp_ewma: super::protocol::EwmaDetector,
    pub ttfb_hw: super::protocol::HoltWinters,
    pub tcp_hw: super::protocol::HoltWinters,
    pub error_rate: ErrorRateTracker,
    pub history: VecDeque<(std::time::Instant, f64, u16)>,
    pub consecutive_failures: u32,
    pub last_prediction: FailurePrediction,
    pub trend: TrendDir,
    pub last_health_score: Option<HealthScore>,
    pub predicted_ttfb: f64,
    pub prediction_confidence: f64,
}

pub struct PredictiveNetworkMonitor {
    pub endpoints: HashMap<String, MonitoredEndpoint>,
    pub environment: NetworkEnvironment,
    pub last_scan: Option<std::time::Instant>,
    pub global_prediction: FailurePrediction,
    pub global_root_cause: Option<ClassifiedRootCause>,
    pub remediation: super::remediation::RemediationEngine,
    pub(crate) scan_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct PredictiveSummary {
    pub global_prediction: FailurePrediction,
    pub worst_endpoint: Option<String>,
    pub worst_z_score: f64,
    pub root_cause: Option<ClassifiedRootCause>,
    pub endpoint_predictions: Vec<(String, FailurePrediction, f64)>,
}
