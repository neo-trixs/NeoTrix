#[cfg(test)]
mod tests {
    use super::super::*;
    use std::time::Duration;

    #[test]
    fn test_deterministic_classifier_fake_ip() {
        let env = NetworkEnvironment {
            has_tun_interfaces: vec!["utun6".into()],
            fake_ip_dns: Some("198.18.0.2".into()),
            physical_ip: Some("192.168.1.2".into()),
            default_interface: Some("en4".into()),
            vpn_type: Some(VpnType::Shadowrocket),
            egress_country: None,
            egress_org: None,
        };
        let health = ApiHealth::DnsFailure {
            reason: "DNS failure".into(),
        };
        let result = RootCauseClassifier::classify_deterministic(&health, &env);
        assert!(result.is_some());
        assert_eq!(result.unwrap().cause, ConnectionFailureRootCause::FakeIpDns);
    }

    #[test]
    fn test_deterministic_classifier_503() {
        let env = NetworkEnvironment {
            has_tun_interfaces: vec![],
            fake_ip_dns: None,
            physical_ip: None,
            default_interface: None,
            vpn_type: None,
            egress_country: None,
            egress_org: None,
        };
        let health = ApiHealth::HttpError {
            status_code: 503,
            body_snippet: "Service Unavailable".into(),
        };
        let result = RootCauseClassifier::classify_deterministic(&health, &env);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().cause,
            ConnectionFailureRootCause::HttpServiceUnavailable
        );
    }

    #[test]
    fn test_error_rate_tracker() {
        let mut t = ErrorRateTracker::new(10);
        assert_eq!(t.error_rate(), 0.0);
        for _ in 0..8 {
            t.record(true);
        }
        assert_eq!(t.error_rate(), 0.0);
        t.record(false);
        t.record(false);
        assert!((t.error_rate() - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_monitored_endpoint() {
        let mut m = MonitoredEndpoint::new("test");
        for _ in 0..10 {
            m.record_probe(0.5, 0.05, 200);
        }
        let s = m.record_probe(0.5, 0.05, 200);
        assert_eq!(s.prediction, FailurePrediction::None);
        let s = m.record_probe(5.0, 0.5, 200);
        assert!(
            matches!(s.trend, TrendDir::Rising | TrendDir::Spike),
            "expected rising/spike, got {:?}",
            s.trend
        );
        let s = m.record_probe(20.0, 1.0, 503);
        assert_eq!(s.trend, TrendDir::Spike);
    }

    #[test]
    fn test_predictive_monitor() {
        let pm = PredictiveNetworkMonitor::new();
        assert!(pm.should_scan());
        let s = pm.predictive_summary();
        assert_eq!(s.global_prediction, FailurePrediction::None);
    }

    #[test]
    fn test_health_score_green() {
        let h = compute_health(100.0, 5000.0, 0.01, 0.1, 0.0);
        assert_eq!(h.grade, HealthGrade::Green);
        assert!(h.overall > 0.8);
    }

    #[test]
    fn test_health_score_red() {
        let h = compute_health(4500.0, 5000.0, 0.5, 0.1, 0.3);
        assert_eq!(h.grade, HealthGrade::Red);
        assert!(h.overall < 0.5);
    }

    #[test]
    fn test_remediation_engine() {
        let action = RemediationEngine::recommend(&ConnectionFailureRootCause::DnsResolutionFailed);
        assert!(action.is_some());
        assert_eq!(action.unwrap().name, "flush_dns_cache");

        let action2 =
            RemediationEngine::recommend(&ConnectionFailureRootCause::HttpServiceUnavailable);
        assert!(
            action2.is_none(),
            "server-side issues should not get network remediation"
        );
    }

    #[test]
    fn test_should_auto_remediate() {
        assert!(RemediationEngine::should_auto_remediate(
            &ConnectionFailureRootCause::FakeIpDns
        ));
        assert!(!RemediationEngine::should_auto_remediate(
            &ConnectionFailureRootCause::HttpServiceUnavailable
        ));
        assert!(!RemediationEngine::should_auto_remediate(
            &ConnectionFailureRootCause::ProviderTooSlow {
                ttfb: Duration::from_secs(20)
            }
        ));
    }
}
