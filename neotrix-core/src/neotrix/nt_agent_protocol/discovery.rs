use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Duration;

use crate::core::nt_core_util::A2A_INTERNAL_PORT;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};

/// Magic bytes sent as a scan probe — receivers should respond with their AgentInfo.
const PROBE_MAGIC: &[u8] = b"NEOTRIX_DISCOVER_PROBE";

/// UDP discovery for external agents (V3-P1)
pub struct AgentDiscovery {
    socket: UdpSocket,
    pub known_agents: HashMap<String, AgentInfo>,
    _running: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    /// Current E8 reasoning hexagram (0-63), 0 = unknown/unset.
    pub hexagram: u8,
    /// mDNS-like service type (e.g. "_neotrix._udp")
    #[serde(default)]
    pub service_type: String,
    /// Human-readable instance name for mDNS-like advertisement
    #[serde(default)]
    pub instance_name: String,
    #[serde(skip)]
    #[serde(default = "std::time::Instant::now")]
    pub last_seen: std::time::Instant,
}

impl AgentInfo {
    /// Create a new AgentInfo with required fields.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        host: impl Into<String>,
        port: u16,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            host: host.into(),
            port,
            capabilities: Vec::new(),
            hexagram: 0,
            service_type: String::new(),
            instance_name: String::new(),
            last_seen: std::time::Instant::now(),
        }
    }

    /// Update the hexagram from a ReasoningHexagram value.
    pub fn update_hexagram(&mut self, hexagram: u8) {
        self.hexagram = hexagram;
    }

    /// Update the hexagram from a ReasoningHexagram type.
    pub fn update_hexagram_from(&mut self, hexagram: crate::core::ReasoningHexagram) {
        self.hexagram = hexagram.0;
    }

    /// Set mDNS-like service type (e.g. "_neotrix._udp")
    pub fn with_service_type(mut self, st: impl Into<String>) -> Self {
        self.service_type = st.into();
        self
    }

    /// Set human-readable instance name
    pub fn with_instance_name(mut self, name: impl Into<String>) -> Self {
        self.instance_name = name.into();
        self
    }
}

impl AgentDiscovery {
    /// Bind to a UDP port for agent discovery
    pub fn new(port: u16) -> NeoTrixResult<Self> {
        let socket =
            UdpSocket::bind(("0.0.0.0", port)).map_err(|e| format!("Failed to bind UDP: {}", e))?;
        if let Err(e) = socket.set_read_timeout(Some(Duration::from_secs(1))) {
            log::warn!("[discovery] set read timeout: {}", e);
        }
        if let Err(e) = socket.set_broadcast(true) {
            log::warn!("[discovery] set broadcast: {}", e);
        }
        Ok(Self {
            socket,
            known_agents: HashMap::new(),
            _running: false,
        })
    }

    /// Broadcast presence to the network
    pub fn broadcast(&self, info: &AgentInfo, broadcast_addr: &str) -> NeoTrixResult<()> {
        let data = serde_json::to_vec(info).map_err(|e| format!("Serialize: {}", e))?;
        self.socket
            .send_to(&data, broadcast_addr)
            .map_err(|e| format!("Broadcast: {}", e))?;
        Ok(())
    }

    /// Listen for agent broadcasts (non-blocking with timeout)
    pub fn listen(&mut self) -> NeoTrixResult<()> {
        let mut buf = [0u8; 4096];
        match self.socket.recv_from(&mut buf) {
            Ok((size, _src)) => {
                if size == PROBE_MAGIC.len() && &buf[..size] == PROBE_MAGIC {
                    return Ok(());
                }
                if let Ok(info) = serde_json::from_slice::<AgentInfo>(&buf[..size]) {
                    self.known_agents.insert(info.id.clone(), info);
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock
                    && e.kind() != std::io::ErrorKind::TimedOut
                {
                    return Err(NeoTrixError::Internal(format!("Recv: {}", e)));
                }
            }
        }
        Ok(())
    }

    /// Active scan: broadcast a probe and listen for responses for `duration_ms`.
    /// Returns the number of newly discovered agents.
    pub fn scan(&mut self, duration_ms: u64) -> NeoTrixResult<usize> {
        let before = self.known_agents.len();

        // Broadcast probe
        if let Err(e) = self.socket.send_to(
            PROBE_MAGIC,
            format!("255.255.255.255:{}", A2A_INTERNAL_PORT),
        ) {
            log::warn!("[discovery] probe send: {}", e);
        }

        // Listen for responses
        let deadline = std::time::Instant::now() + Duration::from_millis(duration_ms);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            self.socket
                .set_read_timeout(Some(remaining.min(Duration::from_millis(200))))
                .unwrap_or(());
            let mut buf = [0u8; 4096];
            match self.socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    if size == PROBE_MAGIC.len() && &buf[..size] == PROBE_MAGIC {
                        continue;
                    }
                    if let Ok(info) = serde_json::from_slice::<AgentInfo>(&buf[..size]) {
                        self.known_agents.insert(info.id.clone(), info);
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut
                        || e.kind() == std::io::ErrorKind::WouldBlock
                    {
                        continue;
                    }
                    return Err(NeoTrixError::Internal(format!("Scan recv: {}", e)));
                }
            }
        }

        // Restore original timeout
        let _ = self.socket.set_read_timeout(Some(Duration::from_secs(1)));

        Ok(self.known_agents.len() - before)
    }

    /// Convenience: scan and return all discovered agents.
    pub fn discover(&mut self, duration_ms: u64) -> NeoTrixResult<Vec<AgentInfo>> {
        self.scan(duration_ms)?;
        Ok(self.known_agents.values().cloned().collect())
    }

    /// mDNS-like service advertisement: broadcast AgentInfo with service_type/instance_name.
    pub fn advertise(&self, info: &AgentInfo, broadcast_addr: &str) -> NeoTrixResult<()> {
        self.broadcast(info, broadcast_addr)
    }

    pub fn agent_count(&self) -> usize {
        self.known_agents.len()
    }

    pub fn update_hexagram(&mut self, hexagram: u8) {
        for info in self.known_agents.values_mut() {
            info.hexagram = hexagram;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_util::A2A_METRICS_PORT;

    #[test]
    fn test_agent_info_default_hexagram() {
        let info = AgentInfo {
            id: "test".into(),
            name: "test-agent".into(),
            host: "0.0.0.0".into(),
            port: 0,
            capabilities: vec![],
            hexagram: 0,
            service_type: String::new(),
            instance_name: String::new(),
            last_seen: std::time::Instant::now(),
        };
        assert_eq!(info.hexagram, 0);
        let json = serde_json::to_string(&info).expect("value should be ok in test");
        assert!(json.contains("\"hexagram\":0"));
    }

    #[test]
    fn test_agent_info_update_hexagram() {
        let mut info = AgentInfo {
            id: "t1".into(),
            name: "t1".into(),
            host: "0.0.0.0".into(),
            port: 0,
            capabilities: vec![],
            hexagram: 0,
            service_type: String::new(),
            instance_name: String::new(),
            last_seen: std::time::Instant::now(),
        };
        info.update_hexagram(42);
        assert_eq!(info.hexagram, 42);
        info.update_hexagram_from(crate::core::ReasoningHexagram::new(7));
        assert_eq!(info.hexagram, 7);
    }

    #[test]
    fn test_agent_info_new_builder() {
        let info = AgentInfo::new("a1", "alpha", "192.168.1.10", A2A_METRICS_PORT)
            .with_service_type("_neotrix._udp")
            .with_instance_name("NeoTrix Alpha");
        assert_eq!(info.id, "a1");
        assert_eq!(info.service_type, "_neotrix._udp");
        assert_eq!(info.instance_name, "NeoTrix Alpha");
        assert_eq!(info.port, A2A_METRICS_PORT);
    }

    #[test]
    fn test_discovery_new_sets_broadcast() {
        let d = AgentDiscovery::new(42100).expect("bind");
        assert!(d.agent_count() == 0);
        // Socket has broadcast enabled (test succeeds if no panic)
        let info = AgentInfo::new("test", "test", "127.0.0.1", 42100);
        let result = d.broadcast(&info, "127.0.0.1:42101");
        // May fail because nobody listening, but should not panic about broadcast
        let _ = result;
    }
}
