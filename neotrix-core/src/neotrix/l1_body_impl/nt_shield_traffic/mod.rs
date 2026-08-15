pub mod analyzer;
pub mod api_proxy;
pub mod fingerprint;

#[cfg(feature = "stealth-net")]
pub mod mitm;

pub use analyzer::{
    CapturedRequest, CapturedResponse, DefaultSensitivityDetector, HostSummary, SensitivityDetector,
    SensitiveFinding, Severity, TrafficAnalyzer, TrafficCategory, TrafficReport, TrafficSession,
};

pub use api_proxy::{ApiProxy, ApiProxyConfig};

pub use fingerprint::{BrowserHeaders, FingerprintStore, TlsFingerprint};

#[cfg(feature = "stealth-net")]
pub use mitm::{MitmProxy, MitmProxyConfig};

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TrafficWatch {
    #[cfg(feature = "stealth-net")]
    pub proxy: Option<MitmProxy>,
    pub analyzer: Arc<Mutex<TrafficAnalyzer>>,
}

impl TrafficWatch {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "stealth-net")]
            proxy: None,
            analyzer: Arc::new(Mutex::new(TrafficAnalyzer::new())),
        }
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_proxy(mut self, config: MitmProxyConfig) -> Self {
        let proxy = MitmProxy::new(config);
        self.analyzer = proxy.analyzer();
        self.proxy = Some(proxy);
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn enable_mitm(mut self, enabled: bool) -> Self {
        if let Some(ref proxy) = self.proxy {
            self.analyzer = proxy.analyzer();
        } else {
            let proxy = MitmProxy::new(MitmProxyConfig::default()).with_mitm(enabled);
            self.analyzer = proxy.analyzer();
            self.proxy = Some(proxy);
        }
        self
    }

    pub async fn start(&self) -> Result<(), String> {
        #[cfg(feature = "stealth-net")]
        if let Some(ref proxy) = self.proxy {
            return proxy.start().await;
        }
        Err("MITM proxy requires stealth-net feature".into())
    }

    pub async fn report(&self) -> TrafficReport {
        let a = self.analyzer.lock().await;
        a.generate_report()
    }

    pub async fn host_summary(&self) -> Vec<HostSummary> {
        let a = self.analyzer.lock().await;
        a.host_summary()
    }

    pub async fn findings(&self) -> Vec<(String, Vec<SensitiveFinding>)> {
        let a = self.analyzer.lock().await;
        a.findings_by_host()
    }

    pub async fn clear(&self) {
        let mut a = self.analyzer.lock().await;
        *a = TrafficAnalyzer::new();
    }
}

impl Default for TrafficWatch {
    fn default() -> Self {
        Self::new()
    }
}
