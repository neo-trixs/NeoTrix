use super::chain::parse_url_to_node;
use super::{DynamicProxyChain, ProxyHealth, ProxyNode, ProxyPool, ProxyProtocol};
use crate::core::nt_core_util::TOR_SOCKS_PORT;
use std::sync::Arc;

#[test]
fn test_proxy_node_display_url() {
    let node = ProxyNode::new(ProxyProtocol::Http, "192.168.1.1", 8080);
    assert_eq!(node.display_url(), "http://192.168.1.1:8080");

    let node = ProxyNode::new(ProxyProtocol::Socks5, "127.0.0.1", TOR_SOCKS_PORT);
    assert_eq!(
        node.display_url(),
        format!("socks5://127.0.0.1:{}", TOR_SOCKS_PORT)
    );
}

#[test]
fn test_proxy_node_secret_url_no_auth() {
    let node = ProxyNode::new(ProxyProtocol::Http, "proxy.example.com", 3128);
    assert_eq!(node.secret_url(), "http://proxy.example.com:3128");
}

#[test]
fn test_proxy_node_secret_url_with_auth() {
    let node =
        ProxyNode::new(ProxyProtocol::Http, "proxy.example.com", 3128).with_auth("user", "pass");
    assert_eq!(node.secret_url(), "http://user:pass@proxy.example.com:3128");
}

#[test]
fn test_proxy_node_geo_tag() {
    let node = ProxyNode::new(ProxyProtocol::Socks5, "tor-exit.us", TOR_SOCKS_PORT).with_geo("US");
    assert_eq!(node.geo_tag.as_deref(), Some("US"));
}

#[test]
fn test_dynamic_chain_creation() {
    let mut chain = DynamicProxyChain::new("test-chain");
    chain.add_layer(vec![
        ProxyNode::new(ProxyProtocol::Http, "entry1", 8080),
        ProxyNode::new(ProxyProtocol::Http, "entry2", 8080),
    ]);
    chain.add_layer(vec![ProxyNode::new(ProxyProtocol::Socks5, "middle1", 1080)]);
    chain.add_layer(vec![
        ProxyNode::new(ProxyProtocol::Https, "exit1", 8443),
        ProxyNode::new(ProxyProtocol::Https, "exit2", 8443),
        ProxyNode::new(ProxyProtocol::Https, "exit3", 8443),
    ]);
    assert_eq!(chain.layer_count(), 3);
}

#[tokio::test]
async fn test_dynamic_chain_rotation() {
    let mut chain = DynamicProxyChain::new("rotation-test");
    chain.add_layer(vec![
        ProxyNode::new(ProxyProtocol::Http, "p1", 8080),
        ProxyNode::new(ProxyProtocol::Http, "p2", 8080),
        ProxyNode::new(ProxyProtocol::Http, "p3", 8080),
    ]);

    chain.rotate_all().await;
    let urls = chain.current_chain_urls().await;
    assert_eq!(urls.len(), 1);
}

#[tokio::test]
async fn test_multi_layer_rotation() {
    let mut chain = DynamicProxyChain::new("multi-layer");
    chain.add_layer(vec![
        ProxyNode::new(ProxyProtocol::Http, "entry-a", 8080),
        ProxyNode::new(ProxyProtocol::Http, "entry-b", 8080),
    ]);
    chain.add_layer(vec![
        ProxyNode::new(ProxyProtocol::Socks5, "middle-x", 1080),
        ProxyNode::new(ProxyProtocol::Socks5, "middle-y", 1080),
    ]);
    chain.add_layer(vec![
        ProxyNode::new(ProxyProtocol::Https, "exit-1", 8443),
        ProxyNode::new(ProxyProtocol::Https, "exit-2", 8443),
    ]);

    chain.rotate_all().await;
    let urls1 = chain.current_chain_urls().await;
    assert_eq!(urls1.len(), 3);

    chain.rotate_all().await;
    let urls2 = chain.current_chain_urls().await;
    assert_eq!(urls2.len(), 3);
}

#[tokio::test]
async fn test_rotation_interval_default() {
    let chain = DynamicProxyChain::new("default-interval");
    assert_eq!(chain.rotation_interval(), 9);
}

#[tokio::test]
async fn test_custom_rotation_interval() {
    let chain = DynamicProxyChain::new("custom-interval").with_rotation_interval(30);
    assert_eq!(chain.rotation_interval(), 30);
}

#[test]
fn test_parse_url_to_node() {
    let node = parse_url_to_node("http://proxy.example.com:3128")
        .expect("parse_url_to_node should succeed for valid http URL");
    assert_eq!(node.protocol, ProxyProtocol::Http);
    assert_eq!(node.host, "proxy.example.com");
    assert_eq!(node.port, 3128);

    let node = parse_url_to_node(&format!("socks5://127.0.0.1:{}", TOR_SOCKS_PORT))
        .expect("parse_url_to_node should succeed for valid socks5 URL");
    assert_eq!(node.protocol, ProxyProtocol::Socks5);
}

#[test]
fn test_proxy_health_success_rate() {
    let health = ProxyHealth {
        node_label: "test".into(),
        last_check: None,
        success_count: 90,
        fail_count: 10,
        avg_latency_ms: 100.0,
    };
    assert!((health.success_rate() - 0.9).abs() < 0.01);
    assert!(health.is_healthy(0.8));
    assert!(!health.is_healthy(0.95));
}

#[tokio::test]
async fn test_proxy_pool() {
    let pool = ProxyPool::new();
    let mut chain = DynamicProxyChain::new("pool-chain");
    chain.add_layer(vec![ProxyNode::new(ProxyProtocol::Http, "10.0.0.1", 8080)]);
    pool.add_chain(Arc::new(chain)).await;
    assert_eq!(pool.chains_len().await, 1);
}

#[test]
fn test_proxy_protocol_default_ports() {
    assert_eq!(ProxyProtocol::Http.default_port(), 8080);
    assert_eq!(ProxyProtocol::Socks5.default_port(), 1080);
}
