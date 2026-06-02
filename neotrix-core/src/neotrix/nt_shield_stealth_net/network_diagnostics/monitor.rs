use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use super::classifier::{recommendation_for, RootCauseClassifier};
use super::health::compute_health;
use super::remediation::RemediationEngine;
use super::types::{
    ApiHealth, ClassifiedRootCause, ConnectionFailureRootCause, ErrorRateTracker,
    FailurePrediction, HealthGrade, LapStats, MonitoredEndpoint, NetworkDiagnosticReport,
    NetworkEnvironment, PredictiveNetworkMonitor, PredictiveSummary, TrendDir,
};
use super::KNOWN_ENDPOINTS;

impl ErrorRateTracker {
    pub fn new(capacity: usize) -> Self {
        Self { window: VecDeque::with_capacity(capacity), capacity }
    }
    pub fn record(&mut self, success: bool) {
        self.window.push_back(success);
        if self.window.len() > self.capacity {
            self.window.pop_front();
        }
    }
    pub fn error_rate(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }
        let errors = self.window.iter().filter(|&&s| !s).count();
        errors as f64 / self.window.len() as f64
    }
    pub fn is_full(&self) -> bool {
        self.window.len() >= self.capacity
    }
}

impl MonitoredEndpoint {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            ttfb_ewma: super::protocol::EwmaDetector::new(0.15, 10),
            ttfb_cusum: super::protocol::CusumDetector::new(0.5, 0.5, 5.0, 10),
            tcp_ewma: super::protocol::EwmaDetector::new(0.2, 5),
            ttfb_hw: super::protocol::HoltWinters::defaults(24),
            tcp_hw: super::protocol::HoltWinters::defaults(24),
            error_rate: ErrorRateTracker::new(20),
            history: VecDeque::new(),
            consecutive_failures: 0,
            last_prediction: FailurePrediction::None,
            trend: TrendDir::Stable,
            last_health_score: None,
            predicted_ttfb: 0.0,
            prediction_confidence: 0.0,
        }
    }

    pub fn record_probe(&mut self, ttfb_secs: f64, tcp_secs: f64, status: u16) -> LapStats {
        self.history.push_back((Instant::now(), ttfb_secs, status));
        if self.history.len() > 50 {
            self.history.pop_front();
        }

        let is_success = status < 500 || status == 401 || status == 422;
        self.error_rate.record(is_success);
        if !is_success {
            self.consecutive_failures += 1;
        } else {
            self.consecutive_failures = 0;
        }

        let ttfb_z = self.ttfb_ewma.observe(ttfb_secs);
        let tcp_z = self.tcp_ewma.observe(tcp_secs);
        let (cusum_above, _) = self.ttfb_cusum.observe(ttfb_secs);

        let (hw_forecast, hw_z) = self.ttfb_hw.step(ttfb_secs);
        let (_, _) = self.tcp_hw.step(tcp_secs);
        self.predicted_ttfb = hw_forecast;
        self.prediction_confidence = (1.0 - (hw_z.abs() / 5.0)).clamp(0.0, 1.0);

        let hw_z_abs = hw_z.abs();
        let p95_latency = self.history.iter().map(|(_, l, _)| *l).fold(0.0f64, |a, b| a.max(b));
        let error_rate = self.error_rate.error_rate();
        let health = compute_health(
            p95_latency * 1000.0,
            5000.0,
            error_rate,
            0.1,
            if hw_z_abs > 3.0 { 0.5 } else { 0.0 },
        );
        self.last_health_score = Some(health.clone());

        let trend = if cusum_above || ttfb_z > 4.0 || hw_z_abs > 4.0 {
            TrendDir::Spike
        } else if ttfb_z > 2.0 || hw_z_abs > 2.5 {
            TrendDir::Rising
        } else if ttfb_z < -2.0 {
            TrendDir::Falling
        } else {
            TrendDir::Stable
        };
        self.trend = trend;

        let z_max = ttfb_z.max(tcp_z).max(hw_z);
        let prediction = if self.consecutive_failures >= 3 || health.grade == HealthGrade::Red {
            FailurePrediction::Imminent
        } else if cusum_above
            || (self.error_rate.is_full() && error_rate > 0.3)
            || health.grade == HealthGrade::Yellow
        {
            FailurePrediction::Warning
        } else if z_max > 2.0
            || ttfb_secs > 3.0
            || self.consecutive_failures >= 1
            || self.predicted_ttfb > 5.0
        {
            FailurePrediction::Watch
        } else {
            FailurePrediction::None
        };
        self.last_prediction = prediction;

        LapStats { z_score: ttfb_z, trend, prediction }
    }

    pub fn summary(&self) -> LapStats {
        LapStats {
            z_score: self.ttfb_ewma.mean(),
            trend: self.trend,
            prediction: self.last_prediction,
        }
    }
}

impl PredictiveNetworkMonitor {
    pub fn new() -> Self {
        let mut endpoints = HashMap::new();
        for ep in KNOWN_ENDPOINTS {
            endpoints.insert(ep.name.to_string(), MonitoredEndpoint::new(ep.name));
        }
        Self {
            endpoints,
            environment: NetworkEnvironment {
                has_tun_interfaces: Vec::new(),
                fake_ip_dns: None,
                physical_ip: None,
                default_interface: None,
                vpn_type: None,
                egress_country: None,
                egress_org: None,
            },
            last_scan: None,
            global_prediction: FailurePrediction::None,
            global_root_cause: None,
            remediation: RemediationEngine::new(),
            scan_interval: Duration::from_secs(300),
        }
    }

    pub fn should_scan(&self) -> bool {
        self.last_scan.map_or(true, |t| t.elapsed() >= self.scan_interval)
    }

    pub async fn scan(&mut self) -> NetworkDiagnosticReport {
        let env = super::protocol::scan_environment().await;
        self.environment = env.clone();
        self.last_scan = Some(Instant::now());

        let mut has_issue = false;
        let mut primary_cause: Option<ConnectionFailureRootCause> = None;
        let mut endpoints_diag = Vec::new();

        for ep in KNOWN_ENDPOINTS {
            let diag = super::protocol::check_endpoint(ep).await;
            let ttfb_secs = match &diag.health {
                ApiHealth::Healthy { latency, .. } => latency.as_secs_f64(),
                ApiHealth::Degraded { latency, .. } => latency.as_secs_f64(),
                _ => 10.0,
            };
            let status = match &diag.health {
                ApiHealth::HttpError { status_code, .. } => *status_code,
                ApiHealth::Unreachable { .. } => 503,
                _ => 200,
            };

            if let Some(mon) = self.endpoints.get_mut(ep.name) {
                let stats = mon.record_probe(ttfb_secs, ttfb_secs * 0.1, status);
                if matches!(stats.prediction, FailurePrediction::Warning | FailurePrediction::Imminent) {
                    has_issue = true;
                }
            }

            if !matches!(diag.health, ApiHealth::Healthy { .. }) {
                has_issue = true;
                let cause = RootCauseClassifier::classify_deterministic(&diag.health, &env);
                if primary_cause.is_none() {
                    primary_cause = cause.map(|c| c.cause);
                }
            }
            endpoints_diag.push(diag);
        }

        let mut worst: FailurePrediction = FailurePrediction::None;
        let mut worst_cause: Option<ClassifiedRootCause> = None;
        for (name, mon) in &self.endpoints {
            let p = mon.last_prediction as u8;
            if (p as u8) > (worst as u8) {
                worst = mon.last_prediction;
                if let Some(diag) = endpoints_diag.iter().find(|d| d.endpoint == name.as_str()) {
                    worst_cause = RootCauseClassifier::classify_deterministic(&diag.health, &env);
                }
            }
        }
        self.global_prediction = worst;
        self.global_root_cause = worst_cause.clone();

        let recommendation = worst_cause.as_ref().and_then(|c| recommendation_for(&c.cause));

        NetworkDiagnosticReport {
            environment: env,
            endpoints: endpoints_diag,
            has_issue,
            primary_root_cause: primary_cause,
            recommendation,
        }
    }

    pub fn tick(&mut self) -> Option<&ClassifiedRootCause> {
        let mut worst: FailurePrediction = FailurePrediction::None;
        for mon in self.endpoints.values() {
            let p = mon.last_prediction;
            if (p as u8) > (worst as u8) {
                worst = p;
            }
        }
        self.global_prediction = worst;
        self.global_root_cause.as_ref()
    }

    pub fn global_lap_stats(&self) -> HashMap<String, LapStats> {
        let mut m = HashMap::new();
        for (name, mon) in &self.endpoints {
            m.insert(name.clone(), mon.summary());
        }
        m
    }

    pub fn auto_remediate(&mut self) -> Option<(String, bool, String)> {
        let cause = self.global_root_cause.as_ref()?;
        if !RemediationEngine::should_auto_remediate(&cause.cause) {
            return None;
        }
        let action = RemediationEngine::recommend(&cause.cause)?;
        let log = self.remediation.execute(action);
        let ok = self.remediation.verify();
        Some((action.name.to_string(), ok, log))
    }

    pub fn predictive_summary(&self) -> PredictiveSummary {
        let mut worst: (String, FailurePrediction, f64) =
            ("none".into(), FailurePrediction::None, 0.0);
        let mut endpoint_predictions = Vec::new();

        for (name, mon) in &self.endpoints {
            let p = mon.last_prediction;
            let z = mon.ttfb_ewma.mean();
            endpoint_predictions.push((name.clone(), p, z));
            if (p as u8) > (worst.1 as u8)
                || ((p as u8) == (worst.1 as u8) && z > worst.2)
            {
                worst = (name.clone(), p, z);
            }
        }

        endpoint_predictions.sort_by(|a, b| (b.1 as u8).cmp(&(a.1 as u8)));

        PredictiveSummary {
            global_prediction: worst.1,
            worst_endpoint: Some(worst.0),
            worst_z_score: worst.2,
            root_cause: self.global_root_cause.clone(),
            endpoint_predictions,
        }
    }
}

pub async fn diagnose_all() -> NetworkDiagnosticReport {
    let env = super::protocol::scan_environment().await;
    let mut has_issue = false;
    let mut primary_cause: Option<ConnectionFailureRootCause> = None;
    let mut endpoints = Vec::new();

    for ep in KNOWN_ENDPOINTS {
        let diag = super::protocol::check_endpoint(ep).await;
        let classified = RootCauseClassifier::classify_deterministic(&diag.health, &env);
        if !matches!(diag.health, ApiHealth::Healthy { .. }) {
            has_issue = true;
            if primary_cause.is_none() {
                primary_cause = classified.map(|c| c.cause);
            }
        }
        endpoints.push(diag);
    }

    let recommendation = primary_cause.as_ref().and_then(recommendation_for);

    NetworkDiagnosticReport {
        environment: env,
        endpoints,
        has_issue,
        primary_root_cause: primary_cause,
        recommendation,
    }
}

impl NetworkDiagnosticReport {
    pub fn summary_text(&self) -> String {
        let mut s = String::new();
        s.push_str("Network Diagnostic Report\n");
        s.push_str(&format!("{:─^60}\n", ""));
        s.push_str("Environment:\n");
        s.push_str(&format!(
            "  Default interface: {}\n",
            self.environment.default_interface.as_deref().unwrap_or("unknown")
        ));
        s.push_str(&format!(
            "  Physical IP: {}\n",
            self.environment.physical_ip.as_deref().unwrap_or("unknown")
        ));
        s.push_str(&format!(
            "  TUN interfaces: {}\n",
            self.environment.has_tun_interfaces.join(", ")
        ));
        s.push_str(&format!(
            "  Fake-IP DNS: {}\n",
            self.environment.fake_ip_dns.as_deref().unwrap_or("none")
        ));
        s.push_str(&format!("  VPN type: {:?}\n", self.environment.vpn_type));
        s.push_str(&format!(
            "  Egress: {:?} ({:?})\n",
            self.environment.egress_country, self.environment.egress_org
        ));
        s.push_str("\nEndpoints:\n");

        for ep in &self.endpoints {
            let status = match &ep.health {
                ApiHealth::Healthy { latency, .. } => {
                    format!("OK ({:.1}s)", latency.as_secs_f64())
                }
                ApiHealth::Degraded { latency, .. } => {
                    format!("DEGRADED ({:.1}s)", latency.as_secs_f64())
                }
                ApiHealth::Unreachable { reason } => format!("UNREACHABLE: {}", reason),
                ApiHealth::DnsFailure { reason } => format!("DNS FAIL: {}", reason),
                ApiHealth::TlsFailure { reason } => format!("TLS FAIL: {}", reason),
                ApiHealth::HttpError { status_code, .. } => format!("HTTP {}", status_code),
            };
            s.push_str(&format!("  {:<25} {}\n", ep.endpoint, status));
        }

        if let Some(cause) = &self.primary_root_cause {
            s.push_str(&format!("\nRoot Cause: {:?}\n", cause));
        }
        if let Some(rec) = self.recommendation {
            s.push_str(&format!("Recommendation: {}\n", rec));
        }
        s
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
