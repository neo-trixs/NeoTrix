use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Duration;

use crate::neotrix::l1_body_impl::nt_l1_error::{L1Result, from_string_result};

/// Magic bytes sent as a scan probe — receivers should respond with their AgentInfo.
const PROBE_MAGIC: &[u8] = b"NEOTRIX_DISCOVER_PROBE";

/// UDP discovery for external agents (V3-P1)
pub struct AgentDiscovery {
    socket: UdpSocket,
    pub known_agents: HashMap<String, AgentInfo>,
    _running: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn new(id: impl Into<String>, name: impl Into<String>, host: impl Into<String>, port: u16) -> Self {
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
    pub fn new(port: u16) -> L1Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", port))
            .map_err(|e| format!("Failed to bind UDP: {}", e))?;
        let socket = from_string_result(Ok(socket))?;
        if let Err(e) = socket.set_read_timeout(Some(Duration::from_secs(1))) {
            log::warn!("[discovery] set read timeout: {}", e);
        }
        if let Err(e) = socket.set_broadcast(true) {
            log::warn!("[discovery] set broadcast: {}", e);
        }
        Ok(Self { socket, known_agents: HashMap::new(), _running: false })
    }

    /// Broadcast presence to the network
    pub fn broadcast(&self, info: &AgentInfo, broadcast_addr: &str) -> L1Result<()> {
        let data = serde_json::to_vec(info).map_err(|e| format!("Serialize: {}", e))?;
        self.socket.send_to(&data, broadcast_addr).map_err(|e| format!("Broadcast: {}", e))?;
        Ok(())
    }

    /// Listen for agent broadcasts — 排空 socket 缓冲区所有待处理数据报 (D17)。
    ///
    /// 背景: background loop 以固定间隔调用本方法 (默认 60s)。UDP 数据报在
    /// socket buffer 排队, 若每次只 recv 一个, 积压会被逐个吞掉 — 一次轮询只
    /// 消费一个, 其余在下次轮询才处理 (延迟放大), 且高频广播下持续积压。
    /// 改为循环读到 WouldBlock/TimedOut 为止, 一次调用排空全部。
    /// 返回新发现数 (0 = 无新 agent, 供上层判断是否触发事件)。
    pub fn listen(&mut self) -> L1Result<usize> {
        let mut discovered: usize = 0;
        loop {
            let mut buf = [0u8; 4096];
            match self.socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    if size == PROBE_MAGIC.len() && &buf[..size] == PROBE_MAGIC {
                        continue;
                    }
                    if let Ok(info) = serde_json::from_slice::<AgentInfo>(&buf[..size]) {
                        self.known_agents.insert(info.id.clone(), info);
                        discovered += 1;
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        break;
                    }
                    return from_string_result(Err(format!("Recv: {}", e)));
                }
            }
        }
        Ok(discovered)
    }

    /// Active scan: broadcast a probe and listen for responses for `duration_ms`.
    /// Returns the number of newly discovered agents.
    pub fn scan(&mut self, duration_ms: u64) -> L1Result<usize> {
        let before = self.known_agents.len();

        // Broadcast probe
        if let Err(e) = self.socket.send_to(PROBE_MAGIC, "255.255.255.255:42069") {
            log::warn!("[discovery] probe send: {}", e);
        }

        // Listen for responses
        let deadline = std::time::Instant::now() + Duration::from_millis(duration_ms);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            self.socket.set_read_timeout(Some(remaining.min(Duration::from_millis(200))))
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
                    let err = from_string_result(Err(format!("Scan recv: {}", e)));
                    return err;
                }
            }
        }

        // Restore original timeout
        let _ = self.socket.set_read_timeout(Some(Duration::from_secs(1)));

        Ok(self.known_agents.len() - before)
    }

    /// Convenience: scan and return all discovered agents.
    pub fn discover(&mut self, duration_ms: u64) -> L1Result<Vec<AgentInfo>> {
        self.scan(duration_ms)?;
        Ok(self.known_agents.values().cloned().collect())
    }

    /// mDNS-like service advertisement: broadcast AgentInfo with service_type/instance_name.
    pub fn advertise(&self, info: &AgentInfo, broadcast_addr: &str) -> L1Result<()> {
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
        let info = AgentInfo::new("a1", "alpha", "192.168.1.10", 42070)
            .with_service_type("_neotrix._udp")
            .with_instance_name("NeoTrix Alpha");
        assert_eq!(info.id, "a1");
        assert_eq!(info.service_type, "_neotrix._udp");
        assert_eq!(info.instance_name, "NeoTrix Alpha");
        assert_eq!(info.port, 42070);
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

    #[test]
    fn test_listen_drains_multiple_datagrams() {
        // D17: listen() 必须排空 socket buffer 全部积压数据报, 而非单次只读一个。
        // 用本地 UDP 对: 发送 2 个 agent info, 一次 listen 应记录 2 个。
        let d = AgentDiscovery::new(42110).expect("bind");
        let port = 42110;

        // 从另一个 socket 广播两个 agent 到本 socket
        let sender = UdpSocket::bind("127.0.0.1:0").expect("bind sender");
        let a = AgentInfo::new("one", "one", "127.0.0.1", port);
        let b = AgentInfo::new("two", "two", "127.0.0.1", port);
        sender.send_to(&serde_json::to_vec(&a).unwrap(), format!("127.0.0.1:{}", port)).unwrap();
        sender.send_to(&serde_json::to_vec(&b).unwrap(), format!("127.0.0.1:{}", port)).unwrap();

        // 给数据报到达 socket buffer 的窗口
        std::thread::sleep(Duration::from_millis(50));
        let mut d = d;
        let n = d.listen().expect("listen");
        assert_eq!(n, 2, "single listen() call must drain both queued datagrams");
        assert_eq!(d.agent_count(), 2, "both agents recorded");
    }
}
