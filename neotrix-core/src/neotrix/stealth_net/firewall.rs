use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::rules::{OutboundAction, OutboundRule, RuleEngine, RuleCondition};

const PF_ANCHOR_PATH: &str = "neotrix";
const SYNC_INTERVAL_SECS: u64 = 9;
const DIVERT_PORT: u16 = 11081;

#[derive(Debug, Clone, PartialEq)]
pub enum FirewallType {
    Pf,
    Nftables,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub action: FirewallAction,
    pub protocol: String,
    pub dst_addr: Option<String>,
    pub dst_port: Option<u16>,
    pub label: String,
    pub priority: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FirewallAction {
    Pass,
    Block,
    DivertToProxy,
    RedirectDns,
}

impl FirewallRule {
    fn to_pf_rule(&self) -> String {
        let action_str = match &self.action {
            FirewallAction::Pass => "pass out quick",
            FirewallAction::Block => "block out quick",
            FirewallAction::DivertToProxy =>
                "pass out quick divert-to 127.0.0.1 port 11081 no-state",
            FirewallAction::RedirectDns =>
                "rdr pass on lo0 proto udp from any to any port 53 -> 127.0.0.1 port 11053",
        };
        let dst = if let Some(ref addr) = self.dst_addr {
            format!(" from any to {}", addr)
        } else {
            String::new()
        };
        format!("{} proto {} {} label \"neotrix:{}\"", action_str, self.protocol, dst, self.label)
    }
}

pub struct FirewallManager {
    enabled: AtomicBool,
    firewall_type: FirewallType,
    active_rules: RwLock<Vec<FirewallRule>>,
    sync_count: AtomicU64,
}

impl FirewallManager {
    pub fn new() -> Self {
        let fw_type = Self::detect_firewall();
        let is_supported = fw_type != FirewallType::Unsupported;
        println!("[fw] detected firewall: {:?}", fw_type);
        let mgr = Self {
            enabled: AtomicBool::new(false),
            firewall_type: fw_type,
            active_rules: RwLock::new(Vec::new()),
            sync_count: AtomicU64::new(0),
        };
        if is_supported {
            let _ = Self::ensure_pf_loaded();
        }
        mgr
    }

    fn detect_firewall() -> FirewallType {
        #[cfg(target_os = "macos")]
        {
            if std::process::Command::new("which").arg("pfctl").output().is_ok() {
                return FirewallType::Pf;
            }
        }
        #[cfg(target_os = "linux")]
        {
            if std::process::Command::new("which").arg("nft").output().is_ok() {
                return FirewallType::Nftables;
            }
        }
        FirewallType::Unsupported
    }

    pub fn is_available(&self) -> bool { self.firewall_type != FirewallType::Unsupported }

    fn ensure_pf_loaded() -> Result<(), String> {
        let _ = std::process::Command::new("pfctl").args(["-e", "-q"]).output();
        let anchor_rules = format!(
            "anchor \"{}\"\nload anchor \"{}\" from \"/tmp/neotrix_pf_anchor.conf\"\n",
            PF_ANCHOR_PATH, PF_ANCHOR_PATH
        );
        let _ = std::fs::write("/tmp/neotrix_pf_main.conf", &anchor_rules);
        let output = std::process::Command::new("pfctl")
            .args(["-f", "/tmp/neotrix_pf_main.conf", "-q"]).output();
        match output {
            Ok(o) if o.status.success() => {
                let _ = std::process::Command::new("pfctl")
                    .args(["-a", PF_ANCHOR_PATH, "-F", "all", "-q"]).output();
                Ok(())
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if stderr.contains("Permission denied") {
                    Err("pfctl: Permission denied (run with sudo or add user to _pfd_admins)".to_string())
                } else {
                    Err(format!("pfctl anchor: {}", stderr))
                }
            }
            Err(e) => Err(format!("pfctl error: {}", e)),
        }
    }

    fn write_pf_rules(rules: &[FirewallRule]) -> Result<(), String> {
        let mut pf_text = format!("# NeoTrix pf rules\n# Divert ALL TCP -> {}\n", DIVERT_PORT);
        pf_text.push_str(&format!("pass out quick proto tcp from any to any divert-to 127.0.0.1 port {} no-state\n", DIVERT_PORT));
        pf_text.push_str("pass out quick proto udp from any to any port 53 divert-to 127.0.0.1 port 11053\n");
        for r in rules {
            if r.action == FirewallAction::Block {
                pf_text.push_str(&r.to_pf_rule());
                pf_text.push('\n');
            }
        }
        let _ = std::fs::write("/tmp/neotrix_pf_anchor.conf", &pf_text);
        let output = std::process::Command::new("pfctl")
            .args(["-a", PF_ANCHOR_PATH, "-f", "/tmp/neotrix_pf_anchor.conf", "-q"]).output();
        match output {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(format!("pfctl load: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(format!("pfctl spawn: {}", e)),
        }
    }

    fn write_nftables_rules(rules: &[FirewallRule]) -> Result<(), String> {
        let mut nft = String::from("add table inet neotrix\nflush table inet neotrix\n");
        nft.push_str("add chain inet neotrix output { type filter hook output priority 0; policy accept; }\n");
        nft.push_str(&format!("add rule inet neotrix divert tcp dport != 11080 divert-to 127.0.0.1:{}\n", DIVERT_PORT));
        nft.push_str("add rule inet neotrix divert udp dport 53 divert-to 127.0.0.1:11053\n");
        for r in rules {
            if r.action == FirewallAction::Block {
                let dst = r.dst_addr.as_deref().unwrap_or("");
                nft.push_str(&format!("add rule inet neotrix output {} drop comment \"{}\"\n", dst, r.label));
            }
        }
        if let Err(e) = std::fs::write("/tmp/neotrix_nftables.conf", &nft) {
            log::warn!("[fw] failed to write nftables config: {}", e);
        }
        let child = std::process::Command::new("nft").args(["-f", "/tmp/neotrix_nftables.conf"]).output();
        match child {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(format!("nft: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(format!("nft spawn: {}", e)),
        }
    }

    pub fn sync_now(rules: &[FirewallRule], fw_type: &FirewallType) -> Result<(), String> {
        match fw_type {
            FirewallType::Pf => Self::write_pf_rules(rules),
            FirewallType::Nftables => Self::write_nftables_rules(rules),
            FirewallType::Unsupported => Err("unsupported".into()),
        }
    }

    pub fn clear_all_rules(fw_type: &FirewallType) -> Result<(), String> {
        match fw_type {
            FirewallType::Pf => {
                let _ = std::process::Command::new("pfctl")
                    .args(["-a", PF_ANCHOR_PATH, "-F", "all", "-q"]).output();
                if let Err(e) = std::fs::write("/tmp/neotrix_pf_anchor.conf", "") {
                    log::warn!("[fw] failed to clear pf anchor: {}", e);
                }
                Ok(())
            }
            FirewallType::Nftables => {
                let _ = std::process::Command::new("nft")
                    .args(["delete", "table", "inet", "neotrix"]).output();
                Ok(())
            }
            FirewallType::Unsupported => Ok(()),
        }
    }

    fn derive_firewall_rules(rules: &[OutboundRule]) -> Vec<FirewallRule> {
        let mut fw_rules = Vec::new();
        for r in rules {
            if !r.enabled { continue; }
            if let RuleCondition::Cidr(ip, prefix) = &r.condition {
                if matches!(r.action, OutboundAction::Block) {
                    fw_rules.push(FirewallRule {
                        action: FirewallAction::Block,
                        protocol: "tcp".into(),
                        dst_addr: Some(format!("{}/{}", ip, prefix)),
                        dst_port: None,
                        label: format!("block-{}", r.label),
                        priority: r.priority,
                    });
                }
            }
        }
        fw_rules.sort_by_key(|r| r.priority);
        fw_rules
    }

    pub async fn sync_rules(&self, rules: &[OutboundRule]) -> Result<(), String> {
        if !self.is_available() { return Err("No firewall available".into()); }
        let fw_rules = Self::derive_firewall_rules(rules);
        *self.active_rules.write().await = fw_rules.clone();
        let count = fw_rules.len();
        let result = Self::sync_now(&fw_rules, &self.firewall_type);
        match &result {
            Ok(_) => {
                self.sync_count.fetch_add(1, Ordering::Relaxed);
                println!("[fw] synced {} CIDR rules (total syncs: {})", count, self.sync_count.load(Ordering::Relaxed));
            }
            Err(e) => { if !e.contains("Permission denied") { log::warn!("[fw] sync failed: {}", e); } }
        }
        result
    }

    pub async fn clear_all(&self) -> Result<(), String> {
        *self.active_rules.write().await = Vec::new();
        Self::clear_all_rules(&self.firewall_type)
    }

    pub async fn enable_divert(&self) -> Result<(), String> {
        if !self.is_available() { return Err("No firewall available".into()); }
        self.enabled.store(true, Ordering::Relaxed);
        let rules = vec![
            FirewallRule { action: FirewallAction::DivertToProxy, protocol: "tcp".into(), dst_addr: None, dst_port: None, label: "divert-all".into(), priority: 0 },
            FirewallRule { action: FirewallAction::RedirectDns, protocol: "udp".into(), dst_addr: None, dst_port: None, label: "redirect-dns".into(), priority: 0 },
        ];
        Self::sync_now(&rules, &self.firewall_type)
    }

    pub async fn disable_divert(&self) -> Result<(), String> {
        self.enabled.store(false, Ordering::Relaxed);
        Self::clear_all_rules(&self.firewall_type)
    }

    pub async fn start_auto_sync(self: Arc<Self>, rule_engine: Arc<RwLock<RuleEngine>>) {
        println!("[fw] auto-sync started (interval: {}s)", SYNC_INTERVAL_SECS);
        loop {
            sleep(Duration::from_secs(SYNC_INTERVAL_SECS)).await;
            if !self.enabled.load(Ordering::Relaxed) {
                sleep(Duration::from_secs(5)).await;
                if !self.enabled.load(Ordering::Relaxed) { continue; }
            }
            let rules = rule_engine.read().await.export();
            let _ = self.sync_rules(&rules).await;
        }
    }

    pub fn enable(&self) { self.enabled.store(true, Ordering::Relaxed); }
    pub fn disable(&self) { self.enabled.store(false, Ordering::Relaxed); }
    pub fn is_enabled(&self) -> bool { self.enabled.load(Ordering::Relaxed) }

    pub async fn stats(&self) -> FirewallStats {
        let rules = self.active_rules.read().await;
        FirewallStats {
            available: self.is_available(),
            enabled: self.is_enabled(),
            firewall_type: format!("{:?}", self.firewall_type),
            rule_count: rules.len(),
            sync_count: self.sync_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FirewallStats {
    pub available: bool,
    pub enabled: bool,
    pub firewall_type: String,
    pub rule_count: usize,
    pub sync_count: u64,
}

impl Default for FirewallManager { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::stealth_net::rules::{OutboundRule, OutboundAction, RuleCondition};

    #[test]
    fn test_firewall_type_debug_clone() {
        let fw = FirewallType::Pf;
        assert_eq!(format!("{:?}", fw), "Pf");
        assert_eq!(FirewallType::Nftables, FirewallType::Nftables);
        assert_ne!(FirewallType::Pf, FirewallType::Unsupported);
    }

    #[test]
    fn test_firewall_action_partial_eq() {
        assert_eq!(FirewallAction::Pass, FirewallAction::Pass);
        assert_ne!(FirewallAction::Pass, FirewallAction::Block);
        assert_eq!(FirewallAction::DivertToProxy, FirewallAction::DivertToProxy);
        assert_eq!(FirewallAction::RedirectDns, FirewallAction::RedirectDns);
    }

    #[test]
    fn test_pf_rule_formatting() {
        let rule = FirewallRule {
            action: FirewallAction::Block,
            protocol: "tcp".into(),
            dst_addr: Some("10.0.0.0/8".into()),
            dst_port: None,
            label: "test-block".into(),
            priority: 10,
        };
        let pf = rule.to_pf_rule();
        assert!(pf.contains("block out quick"));
        assert!(pf.contains("10.0.0.0/8"));
        assert!(pf.contains("test-block"));
    }

    #[test]
    fn test_pf_rule_pass_no_dst() {
        let rule = FirewallRule {
            action: FirewallAction::Pass,
            protocol: "udp".into(),
            dst_addr: None,
            dst_port: None,
            label: "pass-all".into(),
            priority: 0,
        };
        let pf = rule.to_pf_rule();
        assert!(pf.contains("pass out quick"));
        assert!(pf.contains("proto udp"));
        assert!(!pf.contains("from any to any"));
    }

    #[test]
    fn test_pf_rule_divert() {
        let rule = FirewallRule {
            action: FirewallAction::DivertToProxy,
            protocol: "tcp".into(),
            dst_addr: None,
            dst_port: None,
            label: "divert".into(),
            priority: 0,
        };
        let pf = rule.to_pf_rule();
        assert!(pf.contains("divert-to 127.0.0.1 port 11081"));
    }

    #[test]
    fn test_pf_rule_redirect_dns() {
        let rule = FirewallRule {
            action: FirewallAction::RedirectDns,
            protocol: "udp".into(),
            dst_addr: None,
            dst_port: None,
            label: "dns-redirect".into(),
            priority: 0,
        };
        let pf = rule.to_pf_rule();
        assert!(pf.contains("rdr pass on lo0"));
        assert!(pf.contains("port 53"));
        assert!(pf.contains("port 11053"));
    }

    #[test]
    fn test_derive_firewall_rules_from_outbound() {
        let rules = vec![
            OutboundRule::new("test-block", RuleCondition::Cidr("10.0.0.0".parse().expect("value should be ok in test"), 8), OutboundAction::Block),
            OutboundRule::new("test-direct", RuleCondition::Always, OutboundAction::Direct),
        ];
        let fw_rules = FirewallManager::derive_firewall_rules(&rules);
        assert_eq!(fw_rules.len(), 1);
        assert_eq!(fw_rules[0].label, "block-test-block");
        assert_eq!(fw_rules[0].action, FirewallAction::Block);
    }

    #[test]
    fn test_derive_firewall_rules_skips_disabled() {
        let mut rule = OutboundRule::new("disabled", RuleCondition::Cidr("10.0.0.0".parse().expect("value should be ok in test"), 8), OutboundAction::Block);
        rule.enabled = false;
        let rules = vec![rule];
        let fw_rules = FirewallManager::derive_firewall_rules(&rules);
        assert_eq!(fw_rules.len(), 0);
    }

    #[test]
    fn test_derive_firewall_rules_sorts_by_priority() {
        let rules = vec![
            OutboundRule::new("low", RuleCondition::Cidr("10.0.0.0".parse().expect("value should be ok in test"), 8), OutboundAction::Block).with_priority(200),
            OutboundRule::new("high", RuleCondition::Cidr("192.168.0.0".parse().expect("value should be ok in test"), 16), OutboundAction::Block).with_priority(10),
        ];
        let fw_rules = FirewallManager::derive_firewall_rules(&rules);
        assert_eq!(fw_rules.len(), 2);
        assert_eq!(fw_rules[0].label, "block-high");
        assert_eq!(fw_rules[1].label, "block-low");
    }

    #[test]
    fn test_firewall_stats_struct() {
        let stats = FirewallStats {
            available: true,
            enabled: false,
            firewall_type: "Pf".into(),
            rule_count: 5,
            sync_count: 3,
        };
        assert!(stats.available);
        assert!(!stats.enabled);
        assert_eq!(stats.firewall_type, "Pf");
        assert_eq!(stats.rule_count, 5);
        assert_eq!(stats.sync_count, 3);
    }

    #[test]
    fn test_unsupported_firewall_not_available() {
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let mgr = FirewallManager::new();
            assert!(!mgr.is_available());
        }
        // On macos/linux it depends on whether pfctl/nft is installed,
        // so we just test the method exists and returns bool
        let mgr = FirewallManager::new();
        let _available = mgr.is_available(); // don't assert on platform
    }

    #[test]
    fn test_firewall_rule_debug_clone() {
        let rule = FirewallRule {
            action: FirewallAction::Block,
            protocol: "tcp".into(),
            dst_addr: Some("10.0.0.0/8".into()),
            dst_port: Some(443),
            label: "test".into(),
            priority: 100,
        };
        let cloned = rule.clone();
        assert_eq!(rule.label, cloned.label);
        assert_eq!(rule.priority, cloned.priority);
    }
}
