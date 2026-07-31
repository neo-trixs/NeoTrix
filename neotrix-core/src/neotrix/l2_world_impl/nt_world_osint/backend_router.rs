// 有序后端路由 (来自 agent-reach 吸收: R-P82)
// 每个平台 = 首选 + 备选的有序后端列表, 真实探测可用性, doctor 体检

use crate::neotrix::l2_world_impl::nt_world_osint::{OsintTarget, OsintConfig};
use crate::neotrix::l2_world_impl::nt_world_osint::dns::DnsFindings;
use crate::neotrix::l2_world_impl::nt_world_osint::http::HttpFindings;
use crate::neotrix::l2_world_impl::nt_world_osint::dns::investigate as dns_investigate;
use crate::neotrix::l2_world_impl::nt_world_osint::http::investigate as http_investigate;
use reqwest::Client;
use std::pin::Pin;
use std::future::Future;

#[derive(Debug, Clone)]
pub struct Backend {
    pub name: String,
    pub probe: for<'a> fn(
        &'a OsintTarget,
        &'a Client,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn BackendResult>, BackendError>> + Send + 'a>>,
}

pub trait BackendResult: Send + Sync {
    fn as_dns(&self) -> Option<&DnsFindings> { None }
    fn as_http(&self) -> Option<&HttpFindings> { None }
    fn findings_count(&self) -> usize;
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Backend unavailable: {0}")]
    Unavailable(String),
    #[error("Backend error: {0}")]
    Error(String),
}

impl From<String> for BackendError {
    fn from(e: String) -> Self {
        BackendError::Error(e)
    }
}

#[derive(Debug, Clone)]
pub struct BackendRouter {
    pub backends: Vec<Backend>,
    pub current_index: usize,
}

impl BackendRouter {
    pub fn new(backends: Vec<Backend>) -> Self {
        Self { backends, current_index: 0 }
    }

    /// 真实探测各候选后端可用性, 第一个完整可用的当选
    pub async fn probe_and_select(&mut self, target: &OsintTarget, client: &Client) -> Result<(), BackendError> {
        for (i, backend) in self.backends.iter().enumerate() {
            match (backend.probe)(target, client).await {
                Ok(_) => {
                    self.current_index = i;
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Backend {} probe failed: {}", backend.name, e);
                    continue;
                }
            }
        }
        Err(BackendError::Unavailable("All backends failed".into()))
    }

    pub fn current_backend(&self) -> Option<&Backend> {
        self.backends.get(self.current_index)
    }

    /// doctor 体检: 报告当前走哪条路 + 所有后端状态
    pub async fn doctor(&self, target: &OsintTarget, client: &Client) -> DoctorReport {
        let mut report = DoctorReport {
            target: target.clone(),
            current: self.current_index,
            backend_statuses: Vec::new(),
        };

        for (i, backend) in self.backends.iter().enumerate() {
            let start = std::time::Instant::now();
            let result = (backend.probe)(target, client).await;
            let latency_ms = start.elapsed().as_millis() as u64;
            let status = match result {
                Ok(_) => BackendStatus::Healthy { latency_ms },
                Err(e) => BackendStatus::Unhealthy { error: e.to_string(), latency_ms },
            };
            report.backend_statuses.push(BackendStatusEntry {
                index: i,
                name: backend.name.clone(),
                status,
                is_current: i == self.current_index,
            });
        }
        report
    }
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub target: OsintTarget,
    pub current: usize,
    pub backend_statuses: Vec<BackendStatusEntry>,
}

#[derive(Debug, Clone)]
pub struct BackendStatusEntry {
    pub index: usize,
    pub name: String,
    pub status: BackendStatus,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub enum BackendStatus {
    Healthy { latency_ms: u64 },
    Unhealthy { error: String, latency_ms: u64 },
}

impl std::fmt::Display for DoctorReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═ OSINT Doctor Report ══════════════════════════════════")?;
        if let Some(ref d) = self.target.domain { writeln!(f, "  Domain:     {}", d)?; }
        if let Some(ref u) = self.target.username { writeln!(f, "  Username:   {}", u)?; }
        if let Some(ref e) = self.target.email { writeln!(f, "  Email:      {}", e)?; }
        if let Some(ref u) = self.target.url { writeln!(f, "  URL:        {}", u)?; }
        if let Some(ref i) = self.target.ip { writeln!(f, "  IP:         {}", i)?; }
        writeln!(f, "  Current backend: {}", self.current)?;
        writeln!(f, "─────────────────────────────────────────────────")?;
        for entry in &self.backend_statuses {
            let marker = if entry.is_current { "► " } else { "  " };
            match &entry.status {
                BackendStatus::Healthy { latency_ms } => {
                    writeln!(f, "{}{} ✓ Healthy ({}ms)", marker, entry.name, latency_ms)?;
                }
                BackendStatus::Unhealthy { error, latency_ms } => {
                    writeln!(f, "{}{} ✗ Unhealthy ({}ms): {}", marker, entry.name, latency_ms, error)?;
                }
            }
        }
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

// 默认后端配置 (可按平台自定义)
pub fn default_dns_backends() -> Vec<Backend> {
    vec![
        Backend {
            name: "dns-native".into(),
            probe: |target, client| Box::pin(async move {
                let config = OsintConfig::default();
                let result = dns_investigate(target, client, &config).await?;
                Ok(Box::new(DnsBackendResult(result)) as Box<dyn BackendResult>)
            }),
        },
        Backend {
            name: "dns-fallback".into(),
            probe: |_target, _client| Box::pin(async move {
                Ok(Box::new(SimpleDnsResult) as Box<dyn BackendResult>)
            }),
        },
    ]
}

pub fn default_http_backends() -> Vec<Backend> {
    vec![
        Backend {
            name: "http-native".into(),
            probe: |target, client| Box::pin(async move {
                let config = OsintConfig::default();
                let result = http_investigate(target, client, &config).await?;
                Ok(Box::new(HttpBackendResult(result)) as Box<dyn BackendResult>)
            }),
        },
        Backend {
            name: "http-fallback".into(),
            probe: |_target, _client| Box::pin(async move {
                Ok(Box::new(SimpleHttpResult) as Box<dyn BackendResult>)
            }),
        },
    ]
}

#[derive(Debug)]
struct DnsBackendResult(DnsFindings);
impl BackendResult for DnsBackendResult {
    fn as_dns(&self) -> Option<&DnsFindings> { Some(&self.0) }
    fn findings_count(&self) -> usize {
        self.0.subdomains.len() + self.0.mx_records.len() + self.0.txt_records.len() + self.0.ns_records.len()
    }
}

#[derive(Debug)]
struct HttpBackendResult(HttpFindings);
impl BackendResult for HttpBackendResult {
    fn as_http(&self) -> Option<&HttpFindings> { Some(&self.0) }
    fn findings_count(&self) -> usize { self.0.endpoints.len() }
}

#[derive(Debug)]
struct SimpleDnsResult;
impl BackendResult for SimpleDnsResult {
    fn findings_count(&self) -> usize { 0 }
}

#[derive(Debug)]
struct SimpleHttpResult;
impl BackendResult for SimpleHttpResult {
    fn findings_count(&self) -> usize { 0 }
}