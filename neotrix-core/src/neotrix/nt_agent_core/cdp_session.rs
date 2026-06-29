#![forbid(unsafe_code)]

use super::error::AgentError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CDPSession {
    pub session_id: String,
    pub ws_endpoint: String,
    pub tab_id: String,
    pub connected_at: u64,
    pub last_activity: u64,
    command_count: u64,
    failed_count: u64,
    total_latency_ms: u64,
}

impl CDPSession {
    fn new(session_id: &str, ws_endpoint: &str, tab_id: &str) -> Self {
        let now = now_secs();
        Self {
            session_id: session_id.to_string(),
            ws_endpoint: ws_endpoint.to_string(),
            tab_id: tab_id.to_string(),
            connected_at: now,
            last_activity: now,
            command_count: 0,
            failed_count: 0,
            total_latency_ms: 0,
        }
    }

    fn record_command(&mut self, latency_ms: u64) {
        self.command_count += 1;
        self.total_latency_ms += latency_ms;
        self.last_activity = now_secs();
    }

    fn avg_latency(&self) -> f64 {
        if self.command_count == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.command_count as f64
        }
    }

    fn is_stale(&self, timeout_secs: u64) -> bool {
        now_secs().saturating_sub(self.last_activity) > timeout_secs
    }
}

#[derive(Debug, Clone)]
pub struct TabInfo {
    pub tab_id: String,
    pub title: String,
    pub url: String,
    pub attached: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CdpStats {
    pub active_sessions: usize,
    pub total_commands: u64,
    pub failed_commands: u64,
    pub avg_latency_ms: f64,
}

const MAX_COMMAND_HISTORY: usize = 1000;

pub struct CDPSessionManager {
    sessions: HashMap<String, CDPSession>,
    max_sessions: usize,
    default_timeout: u64,
    command_history: Vec<(String, String, bool)>,
}

impl CDPSessionManager {
    fn prune_command_history(&mut self) {
        if self.command_history.len() > MAX_COMMAND_HISTORY {
            let overflow = self.command_history.len() - MAX_COMMAND_HISTORY;
            self.command_history.drain(0..overflow);
        }
    }
}

impl Default for CDPSessionManager {
    fn default() -> Self {
        Self::with_capacity(10, 60)
    }
}

impl CDPSessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(max_sessions: usize, default_timeout: u64) -> Self {
        Self {
            sessions: HashMap::new(),
            max_sessions,
            default_timeout,
            command_history: Vec::new(),
        }
    }

    pub fn connect(&mut self, ws_endpoint: &str) -> Result<String, AgentError> {
        if self.sessions.len() >= self.max_sessions {
            return Err(AgentError::InvalidState(format!(
                "Max sessions ({}) reached",
                self.max_sessions
            )));
        }
        let session_id = format!("cdp_{}", now_secs());
        let tab_id = format!("tab_{}", now_secs());
        let session = CDPSession::new(&session_id, ws_endpoint, &tab_id);
        self.sessions.insert(session_id.clone(), session);
        self.command_history
            .push((session_id.clone(), "connect".into(), true));
        self.prune_command_history();
        Ok(session_id)
    }

    pub fn disconnect(&mut self, session_id: &str) -> Result<(), AgentError> {
        if self.sessions.remove(session_id).is_none() {
            return Err(AgentError::NotFound(format!(
                "Session '{}' not found",
                session_id
            )));
        }
        self.command_history
            .push((session_id.to_string(), "disconnect".into(), true));
        self.prune_command_history();
        Ok(())
    }

    pub fn execute_command(
        &mut self,
        session_id: &str,
        method: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, AgentError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;

        let start = std::time::Instant::now();
        let simulated = serde_json::json!({
            "result": {
                "method": method,
                "sessionId": session_id,
                "status": "ok"
            }
        });
        let latency = start.elapsed().as_millis() as u64;
        session.record_command(latency);
        self.command_history
            .push((session_id.to_string(), format!("cmd:{}", method), true));
        self.prune_command_history();
        Ok(simulated)
    }

    pub fn list_tabs(&mut self, session_id: &str) -> Result<Vec<TabInfo>, AgentError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;

        let tabs = vec![TabInfo {
            tab_id: session.tab_id.clone(),
            title: format!("Tab for {}", session.ws_endpoint),
            url: "about:blank".into(),
            attached: true,
        }];
        Ok(tabs)
    }

    pub fn navigate(&mut self, session_id: &str, url: &str) -> Result<(), AgentError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;

        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        session.record_command(latency);
        self.command_history
            .push((session_id.to_string(), format!("navigate:{}", url), true));
        self.prune_command_history();
        Ok(())
    }

    pub fn take_screenshot(&mut self, session_id: &str) -> Result<Vec<u8>, AgentError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;

        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        session.record_command(latency);
        self.command_history
            .push((session_id.to_string(), "screenshot".into(), true));
        self.prune_command_history();
        let png_header: Vec<u8> = vec![137, 80, 78, 71, 13, 10, 26, 10];
        let mock_png: Vec<u8> = png_header
            .into_iter()
            .chain(std::iter::repeat(0u8).take(128))
            .collect();
        Ok(mock_png)
    }

    pub fn cleanup_stale(&mut self, timeout_secs: u64) -> usize {
        let timeout_secs = if timeout_secs == 0 {
            self.default_timeout
        } else {
            timeout_secs
        };
        let stale_ids: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_stale(timeout_secs))
            .map(|(id, _)| id.clone())
            .collect();
        let count = stale_ids.len();
        for id in stale_ids {
            self.sessions.remove(&id);
            self.command_history
                .push((id, "cleanup_stale".into(), false));
        }
        self.prune_command_history();
        count
    }

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn stats(&self) -> CdpStats {
        let mut total_cmds = 0u64;
        let mut failed_cmds = 0u64;
        let mut sum_lat = 0.0f64;
        for s in self.sessions.values() {
            total_cmds += s.command_count;
            failed_cmds += s.failed_count;
            sum_lat += s.avg_latency();
        }
        let avg = if self.sessions.is_empty() {
            0.0
        } else {
            sum_lat / self.sessions.len() as f64
        };
        CdpStats {
            active_sessions: self.sessions.len(),
            total_commands: total_cmds,
            failed_commands: failed_cmds,
            avg_latency_ms: avg,
        }
    }

    pub fn get_session(&self, session_id: &str) -> Option<&CDPSession> {
        self.sessions.get(session_id)
    }
}

fn now_secs() -> u64 {
    crate::core::nt_core_time::unix_now_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_creates_session() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222/devtools/page/1").unwrap();
        assert!(id.starts_with("cdp_"));
        assert_eq!(mgr.active_count(), 1);
    }

    #[test]
    fn test_connect_exceeds_max_sessions() {
        let mut mgr = CDPSessionManager::with_capacity(2, 60);
        mgr.connect("ws://a").unwrap();
        mgr.connect("ws://b").unwrap();
        let result = mgr.connect("ws://c");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Max sessions"));
    }

    #[test]
    fn test_disconnect_removes_session() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222").unwrap();
        mgr.disconnect(&id).unwrap();
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_disconnect_nonexistent_fails() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let result = mgr.disconnect("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_command_returns_value() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222").unwrap();
        let params = serde_json::json!({});
        let result = mgr.execute_command(&id, "Page.enable", &params).unwrap();
        assert_eq!(result["result"]["method"], "Page.enable");
    }

    #[test]
    fn test_execute_command_invalid_session_fails() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let result = mgr.execute_command("bad_id", "Page.enable", &serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_navigate_updates_tab() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222").unwrap();
        mgr.navigate(&id, "https://example.com").unwrap();
        let session = mgr.get_session(&id).unwrap();
        assert!(session.command_count > 0);
    }

    #[test]
    fn test_take_screenshot_returns_png_bytes() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222").unwrap();
        let png = mgr.take_screenshot(&id).unwrap();
        assert_eq!(png[..4], [137, 80, 78, 71]);
        assert!(png.len() > 8);
    }

    #[test]
    fn test_list_tabs_returns_tab_info() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222").unwrap();
        let tabs = mgr.list_tabs(&id).unwrap();
        assert_eq!(tabs.len(), 1);
        assert!(tabs[0].tab_id.starts_with("tab_"));
        assert!(tabs[0].attached);
    }

    #[test]
    fn test_cleanup_stale_removes_old_sessions() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        mgr.connect("ws://a").unwrap();
        mgr.connect("ws://b").unwrap();
        let removed = mgr.cleanup_stale(0);
        assert_eq!(removed, 2);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_cleanup_stale_keeps_recent_sessions() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        mgr.connect("ws://a").unwrap();
        let removed = mgr.cleanup_stale(86_400);
        assert_eq!(removed, 0);
        assert_eq!(mgr.active_count(), 1);
    }

    #[test]
    fn test_stats_tracks_command_metrics() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222").unwrap();
        let params = serde_json::json!({});
        mgr.execute_command(&id, "Page.enable", &params).unwrap();
        mgr.execute_command(&id, "DOM.getDocument", &params)
            .unwrap();
        let stats = mgr.stats();
        assert_eq!(stats.active_sessions, 1);
        assert_eq!(stats.total_commands, 2);
    }

    #[test]
    fn test_connect_disconnect_lifecycle() {
        let mut mgr = CDPSessionManager::with_capacity(5, 30);
        let id = mgr.connect("ws://h").unwrap();
        assert!(mgr.get_session(&id).is_some());
        mgr.disconnect(&id).unwrap();
        assert!(mgr.get_session(&id).is_none());
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_screenshot_invalid_session_fails() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let result = mgr.take_screenshot("no_such_session");
        assert!(result.is_err());
    }

    #[test]
    fn test_navigate_invalid_session_fails() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let result = mgr.navigate("bad", "https://x.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_empty_when_no_sessions() {
        let mgr = CDPSessionManager::with_capacity(10, 60);
        let stats = mgr.stats();
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.total_commands, 0);
        assert_eq!(stats.avg_latency_ms, 0.0);
    }

    #[test]
    fn test_get_session_returns_none_for_missing() {
        let mgr = CDPSessionManager::with_capacity(10, 60);
        assert!(mgr.get_session("nonexistent").is_none());
    }

    #[test]
    fn test_connect_updates_last_activity() {
        let mut mgr = CDPSessionManager::with_capacity(10, 60);
        let id = mgr.connect("ws://localhost:9222").unwrap();
        let session = mgr.get_session(&id).unwrap();
        assert!(session.connected_at > 0);
        assert_eq!(session.connected_at, session.last_activity);
    }
}
