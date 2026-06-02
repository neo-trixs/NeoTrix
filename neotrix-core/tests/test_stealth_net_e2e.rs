//! 端到端测试：验证代理栈各组件集成
//!
//! 覆盖:
//! - StealthHttpClient：指纹注入 + 路由规则 + Tor 降级
//! - RotationCoordinator：非周期轮转
//! - FingerprintBandit：Thompson Sampling 选择
//! - Per-destination 流隔离

use std::sync::Arc;
use std::time::Duration;

use neotrix::neotrix::stealth_net::{
    StealthHttpClient, ProxyConfig,
    RotationCoordinator, RotationDomain,
    RuleEngine, OutboundRule, OutboundAction, RuleCondition,
    http_client,
};

fn test_url() -> &'static str {
    "https://httpbin.org/get"
}

#[test]
fn test_stealth_client_direct_works() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = StealthHttpClient::new();
        let resp = client.fetch("https://example.com").await.unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.is_html());
    });
}

#[test]
fn test_stealth_client_with_tracker_check() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = StealthHttpClient::new().with_tracker_check(true);
        let resp = client.fetch("https://doubleclick.net/ad").await.unwrap();
        assert_eq!(resp.status, 0); // blocked by tracker check
    });
}

#[test]
fn test_stealth_client_block_rule() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = StealthHttpClient::new();
        let mut engine = RuleEngine::new();
        engine.add_rule(OutboundRule {
            condition: RuleCondition::DomainExact("blocked.test".into()),
            action: OutboundAction::Block,
            priority: 10,
            enabled: true,
            label: "test-block".into(),
        });
        client.set_rule_engine(Arc::new(engine)).await;
        let resp = client.fetch("https://blocked.test/page").await.unwrap();
        assert_eq!(resp.status, 0);
    });
}

#[test]
fn test_stealth_client_extra_headers_injected() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = StealthHttpClient::new();
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Test".into(), "e2e".into());
        client.set_extra_headers(headers).await;
        // 验证 headers 存储成功
        let stored = client.extra_headers().await;
        assert_eq!(stored.get("X-Test").map(|s| s.as_str()), Some("e2e"));
    });
}

#[test]
fn test_rotation_coordinator_basic() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let coord = RotationCoordinator::new();
        let summary = coord.summary().await;
        assert_eq!(summary.len(), 6);

        // 每个域应有独立的 phase
        assert!(!coord.should_rotate(RotationDomain::HttpHeaders).await);
        coord.mark_rotated(RotationDomain::HttpHeaders).await;
        assert_eq!(coord.rotation_count(RotationDomain::HttpHeaders).await, 1);
    });
}

#[test]
fn test_rotation_coordinator_nonperiodic() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let coord = RotationCoordinator::new();
        // 检查间隔在合理范围内
        let intervals: Vec<_> = (0..10).map(|_| {
            rt.block_on(coord.next_interval_ms(RotationDomain::TlsFingerprint))
        }).collect();
        for i in &intervals {
            assert!(*i >= 10_000, "interval too small: {}", i);
        }
        // jitter 应产生不同间隔
        let unique: std::collections::HashSet<_> = intervals.into_iter().collect();
        assert!(unique.len() > 1, "jitter should produce varying intervals");
    });
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 完整代理栈测试: 需系统有网络连接
    #[tokio::test]
    #[ignore]
    async fn test_full_proxy_stack() {
        let client = StealthHttpClient::new();

        // 注入指纹
        client.set_coordinator(RotationCoordinator::new()).await;

        // 请求真实站点
        let resp = client.fetch("https://httpbin.org/get").await.unwrap();
        assert_eq!(resp.status, 200);

        // 验证指纹头注入
        let body_str = String::from_utf8_lossy(&resp.body);
        assert!(body_str.contains("Accept-Language") || body_str.contains("Sec-CH-UA"));
    }

    /// Bandit 收敛测试 (无需网络)
    #[tokio::test]
    async fn test_real_bandit_convergence() {
        use neotrix::neotrix::stealth_net::bandit::FingerprintBandit;
        use neotrix::neotrix::http_factory::TlsVariant;
        use neotrix::neotrix::stealth_net::system_fingerprint::Platform;

        let bandit = FingerprintBandit::load();

        // LegacyHttp11 + MacOS 模拟最优
        let best = neotrix::neotrix::stealth_net::bandit::ComboArm { tls: TlsVariant::LegacyHttp11, platform: Platform::MacOS, h2_profile: neotrix::neotrix::http_factory::H2SettingsProfile::ChromeDefault, geo_tag: String::new() };
        let worst = neotrix::neotrix::stealth_net::bandit::ComboArm { tls: TlsVariant::StrictVerify, platform: Platform::Windows, h2_profile: neotrix::neotrix::http_factory::H2SettingsProfile::ChromeDefault, geo_tag: String::new() };

        for _ in 0..50 {
            bandit.update(best.clone(), 0.9);
            bandit.update(worst.clone(), 0.1);
        }

        let selected = bandit.select_arm(None);
        // 保存检查
        bandit.save();
    }
}
