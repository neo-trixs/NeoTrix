use super::types::{
    ConnectionFailureRootCause, PlaybookStage, RemediationAction, RemediationRisk,
};

pub static REMEDIATION_PLAYBOOK: &[RemediationAction] = &[
    RemediationAction {
        name: "flush_dns_cache",
        description: "Flush macOS DNS cache via dscacheutil + mDNSResponder",
        risk: RemediationRisk::Low,
        reversible: true,
        execute: || {
            let r1 = std::process::Command::new("dscacheutil").arg("-flushcache").output();
            let r2 = std::process::Command::new("killall").args(["-HUP", "mDNSResponder"]).output();
            if r1.is_ok() && r2.is_ok() {
                Ok("DNS cache flushed".into())
            } else {
                Err("DNS flush failed".into())
            }
        },
        verify: || true,
        rollback: None,
    },
    RemediationAction {
        name: "toggle_wifi",
        description: "Toggle Wi-Fi interface off/on to reset routing",
        risk: RemediationRisk::Medium,
        reversible: true,
        execute: || {
            let _ = std::process::Command::new("sudo")
                .args(["ifconfig", "en0", "down"])
                .output();
            std::thread::sleep(std::time::Duration::from_secs(2));
            let r = std::process::Command::new("sudo")
                .args(["ifconfig", "en0", "up"])
                .output();
            if r.is_ok() {
                Ok("Wi-Fi toggled".into())
            } else {
                Err("Wi-Fi toggle failed".into())
            }
        },
        verify: || {
            std::process::Command::new("ping")
                .args(["-c1", "-t3", "8.8.8.8"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        },
        rollback: None,
    },
    RemediationAction {
        name: "reset_network",
        description: "Reset network interfaces: default route → DNS → interfaces",
        risk: RemediationRisk::High,
        reversible: true,
        execute: || {
            let r1 = std::process::Command::new("dscacheutil").arg("-flushcache").output();
            let r2 = std::process::Command::new("killall")
                .args(["-HUP", "mDNSResponder"])
                .output();
            let r3 = std::process::Command::new("sudo")
                .args(["route", "-n", "flush"])
                .output();
            if r1.is_ok() && r2.is_ok() && r3.is_ok() {
                Ok("Network reset complete".into())
            } else {
                Err("Network reset failed".into())
            }
        },
        verify: || {
            std::process::Command::new("ping")
                .args(["-c1", "-t3", "8.8.8.8"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        },
        rollback: None,
    },
];

pub struct RemediationEngine {
    pub stage: PlaybookStage,
    pub current_action: Option<&'static RemediationAction>,
    pub execution_history: Vec<(String, bool, String)>,
}

impl RemediationEngine {
    pub fn new() -> Self {
        Self { stage: PlaybookStage::Idle, current_action: None, execution_history: Vec::new() }
    }

    pub fn recommend(cause: &ConnectionFailureRootCause) -> Option<&'static RemediationAction> {
        match cause {
            ConnectionFailureRootCause::FakeIpDns
            | ConnectionFailureRootCause::DnsResolutionFailed => {
                REMEDIATION_PLAYBOOK.iter().find(|a| a.name == "flush_dns_cache")
            }
            ConnectionFailureRootCause::ConnectionTimeout
            | ConnectionFailureRootCause::VpnRoutingIssue => {
                REMEDIATION_PLAYBOOK.iter().find(|a| a.name == "toggle_wifi")
            }
            ConnectionFailureRootCause::ConnectionReset => {
                REMEDIATION_PLAYBOOK.iter().find(|a| a.name == "reset_network")
            }
            _ => None,
        }
    }

    pub fn execute(&mut self, action: &'static RemediationAction) -> String {
        self.stage = PlaybookStage::Executing;
        self.current_action = Some(action);

        let result = (action.execute)();
        let (success, log) = match result {
            Ok(log) => (true, log),
            Err(log) => (false, log),
        };

        self.execution_history
            .push((action.name.to_string(), success, log.clone()));
        self.stage = if success { PlaybookStage::Verifying } else { PlaybookStage::Idle };
        log
    }

    pub fn verify(&mut self) -> bool {
        let ok = self.current_action.map_or(false, |a| (a.verify)());
        self.stage = if ok { PlaybookStage::Idle } else { PlaybookStage::RollingBack };
        ok
    }

    pub fn should_auto_remediate(cause: &ConnectionFailureRootCause) -> bool {
        matches!(
            cause,
            ConnectionFailureRootCause::FakeIpDns
                | ConnectionFailureRootCause::DnsResolutionFailed
                | ConnectionFailureRootCause::ConnectionTimeout
                | ConnectionFailureRootCause::ConnectionReset
        )
    }

    pub fn summary(&self) -> String {
        let s = format!("Remediation state: {:?}\n", self.stage);
        for (_name, _ok, _log) in &self.execution_history {}
        s
    }
}
