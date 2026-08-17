use std::collections::{HashMap, HashSet, VecDeque};
use std::net::UdpSocket;
use std::time::{Duration, Instant};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

use crate::neotrix::l1_body_impl::nt_l1_error::{L1Result, from_string_result};

/// Magic bytes sent as a scan probe — receivers should respond with their AgentInfo.
const PROBE_MAGIC: &[u8] = b"NEOTRIX_DISCOVER_PROBE";

/// Prefix of an encrypted agent-discovery frame (EasyTier absorption: AES-GCM payload
/// encryption + network-secret identity gate, cf. easytier-core tunnel/encrypt/aes_gcm.rs
/// and peers/peer_manager.rs add_new_peer_conn).
const SECURE_MAGIC: &[u8] = b"NEOTRIX_DISCOVER_SECURE";
/// AES-GCM nonce length (12 bytes, like EasyTier's `StandardAeadTail` 12-byte nonce).
const NONCE_LEN: usize = 12;
/// AES-256-GCM authentication tag length.
const TAG_LEN: usize = 16;
/// Bounded replay window: how many distinct nonces we remember per process.
const REPLAY_WINDOW_CAP: usize = 4096;
/// Fixed KDF label — same secret + label must yield the same key on both ends.
const KDF_LABEL: &[u8] = b"neotrix-secure-discovery-v1";

/// UDP discovery for external agents (V3-P1)
pub struct AgentDiscovery {
    socket: UdpSocket,
    pub known_agents: HashMap<String, AgentInfo>,
    _running: bool,
    /// Shared-secret derived AES-256-GCM key; `Some` enables secure discovery mode.
    secure_key: Option<[u8; 32]>,
    /// Replay window for previously-seen nonces (EasyTier ReplayWindow256 pattern).
    replay: NonceReplayWindow,
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
        Ok(Self {
            socket,
            known_agents: HashMap::new(),
            _running: false,
            secure_key: None,
            replay: NonceReplayWindow::new(REPLAY_WINDOW_CAP),
        })
    }

    /// Bind for *secure* discovery: same as [`Self::new`] but enables encrypted +
    /// authenticated frames keyed off a shared network secret (EasyTier identity-gate
    /// absorption). Only agents sharing the secret are discovered; everything else is
    /// dropped at the decrypt boundary.
    pub fn new_secure(port: u16, network_secret: &[u8]) -> L1Result<Self> {
        let mut d = Self::new(port)?;
        d.secure_key = Some(derive_key_256(network_secret));
        Ok(d)
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
        self.recv_loop(None)
    }

    /// Listen for *encrypted* agent broadcasts only. Frames that fail AES-GCM
    /// authentication (foreign secret / tamper / plaintext) are rejected at the
    /// decrypt boundary (identity gate). Requires a secure instance.
    pub fn listen_secure(&mut self) -> L1Result<usize> {
        self.require_secure()?;
        self.recv_loop(None)
    }

    /// Active scan: broadcast a probe and listen for responses for `duration_ms`.
    /// Returns the number of newly discovered agents.
    pub fn scan(&mut self, duration_ms: u64) -> L1Result<usize> {
        let before = self.known_agents.len();

        // Broadcast probe
        if let Err(e) = self.socket.send_to(PROBE_MAGIC, "255.255.255.255:42069") {
            log::warn!("[discovery] probe send: {}", e);
        }

        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        self.recv_loop(Some(deadline))?;

        // Restore original timeout
        let _ = self.socket.set_read_timeout(Some(Duration::from_secs(1)));

        Ok(self.known_agents.len() - before)
    }

    /// Active *encrypted* scan: broadcast the probe then accept only authenticated
    /// secure frames for `duration_ms`. Requires a secure instance.
    pub fn scan_secure(&mut self, duration_ms: u64) -> L1Result<usize> {
        self.require_secure()?;
        let before = self.known_agents.len();

        if let Err(e) = self.socket.send_to(PROBE_MAGIC, "255.255.255.255:42069") {
            log::warn!("[discovery] secure probe send: {}", e);
        }

        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        self.recv_loop(Some(deadline))?;

        let _ = self.socket.set_read_timeout(Some(Duration::from_secs(1)));
        Ok(self.known_agents.len() - before)
    }

    /// Broadcast an *encrypted* AgentInfo to the network. Only secure listeners
    /// holding the same secret can decrypt it (AES-256-GCM, random nonce per frame).
    pub fn broadcast_secure(&self, info: &AgentInfo, broadcast_addr: &str) -> L1Result<()> {
        let key = self.secure_key.ok_or_else(|| "secure mode not enabled".to_string())?;
        let plaintext = serde_json::to_vec(info).map_err(|e| format!("Serialize: {}", e))?;

        let mut nonce = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "AES key init".to_string())?;
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|_| "Encrypt".to_string())?;

        let mut frame = Vec::with_capacity(SECURE_MAGIC.len() + NONCE_LEN + ciphertext.len());
        frame.extend_from_slice(SECURE_MAGIC);
        frame.extend_from_slice(&nonce);
        frame.extend_from_slice(&ciphertext);
        self.socket.send_to(&frame, broadcast_addr).map_err(|e| format!("Broadcast: {}", e))?;
        Ok(())
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

    /// Shared recv loop: drains the socket buffer, dispatching each datagram to
    /// [`Self::handle_datagram`] (plaintext or secure depending on mode). When
    /// `deadline` is set (active scan), keep probing with 200ms slices until it
    /// elapses; otherwise stop at the first WouldBlock/TimedOut (drain semantics).
    fn recv_loop(&mut self, deadline: Option<Instant>) -> L1Result<usize> {
        let mut discovered: usize = 0;
        loop {
            if let Some(deadline) = deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                self.socket
                    .set_read_timeout(Some(remaining.min(Duration::from_millis(200))))
                    .unwrap_or(());
            }
            let mut buf = [0u8; 4096];
            match self.socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    if let Some(info) = self.handle_datagram(&buf[..size]) {
                        self.known_agents.insert(info.id.clone(), info);
                        discovered += 1;
                    }
                }
                Err(e) => {
                    let timeout_like = e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut;
                    if timeout_like && deadline.is_none() {
                        break;
                    }
                    if timeout_like {
                        continue;
                    }
                    return from_string_result(Err(format!("Recv: {}", e)));
                }
            }
        }
        Ok(discovered)
    }

    /// Dispatch a datagram: secure mode accepts only authenticated secure frames,
    /// plaintext mode accepts only `AgentInfo` JSON (probe magic ignored).
    fn handle_datagram(&mut self, buf: &[u8]) -> Option<AgentInfo> {
        if self.secure_key.is_some() {
            self.decrypt_secure(buf)
        } else {
            if buf == PROBE_MAGIC {
                return None;
            }
            serde_json::from_slice::<AgentInfo>(buf).ok()
        }
    }

    /// Decrypt + authenticate a secure frame. Returns `None` unless the frame:
    /// 1. carries the secure magic prefix,
    /// 2. has a fresh (unreplayed) nonce,
    /// 3. authenticates under the shared key (identity gate) — wrong secret,
    ///    tampered ciphertext, or plaintext frames are dropped.
    fn decrypt_secure(&mut self, buf: &[u8]) -> Option<AgentInfo> {
        if buf.len() < SECURE_MAGIC.len() + NONCE_LEN + TAG_LEN || !buf.starts_with(SECURE_MAGIC) {
            return None;
        }
        let nonce: [u8; NONCE_LEN] = buf[SECURE_MAGIC.len()..SECURE_MAGIC.len() + NONCE_LEN].try_into().ok()?;
        if !self.replay.check_and_record(nonce) {
            return None;
        }
        let key = self.secure_key?;
        let cipher = Aes256Gcm::new_from_slice(&key).ok()?;
        let plaintext = cipher.decrypt(Nonce::from_slice(&nonce), &buf[SECURE_MAGIC.len() + NONCE_LEN..]).ok()?;
        serde_json::from_slice::<AgentInfo>(&plaintext).ok()
    }

    fn require_secure(&self) -> L1Result<()> {
        if self.secure_key.is_some() {
            Ok(())
        } else {
            Err("secure mode not enabled — construct with AgentDiscovery::new_secure".into())
        }
    }
}

/// Derive a 32-byte AES-GCM key from the shared network secret (EasyTier absorption:
/// static key derived from network secret, cf. easytier-core tunnel/encrypt/mod.rs
/// `derive_key_256`). Uses HMAC-SHA256 with a fixed label so both ends agree.
fn derive_key_256(secret: &[u8]) -> [u8; 32] {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(KDF_LABEL).expect("hmac key from fixed label");
    mac.update(secret);
    mac.finalize().into_bytes().into()
}

/// Bounded replay window: remembers the most recent `cap` nonces and rejects
/// replays of any nonce already recorded (EasyTier `ReplayWindow256` pattern,
/// cf. easytier-core tunnel/secure_datagram.rs). Stateless-broadcast trade-off:
/// a captured frame replayed after it fell out of the window is accepted.
struct NonceReplayWindow {
    seen: HashSet<[u8; NONCE_LEN]>,
    order: VecDeque<[u8; NONCE_LEN]>,
    cap: usize,
}

impl NonceReplayWindow {
    fn new(cap: usize) -> Self {
        Self { seen: HashSet::new(), order: VecDeque::new(), cap }
    }

    /// `true` if the nonce is fresh and was recorded; `false` on replay.
    fn check_and_record(&mut self, nonce: [u8; NONCE_LEN]) -> bool {
        if !self.seen.insert(nonce) {
            return false;
        }
        self.order.push_back(nonce);
        while self.order.len() > self.cap {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }
        true
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

    // ====== secure mode (EasyTier absorption) ======

    #[test]
    fn test_derive_key_stable_and_unique() {
        let k1 = derive_key_256(b"abc");
        let k2 = derive_key_256(b"abc");
        let k3 = derive_key_256(b"abd");
        assert_eq!(k1, k2, "same secret -> same key");
        assert_ne!(k1, k3, "different secret -> different key");
        assert_eq!(k1.len(), 32);
    }

    #[test]
    fn test_secure_roundtrip_over_udp() {
        let receiver = AgentDiscovery::new_secure(42120, b"shared-secret").expect("bind receiver");
        let port = 42120;
        let sender = AgentDiscovery::new_secure(0, b"shared-secret").expect("bind sender");

        let info = AgentInfo::new("sec-1", "secure-agent", "127.0.0.1", port);
        sender.broadcast_secure(&info, format!("127.0.0.1:{}", port).as_str()).expect("broadcast secure");

        std::thread::sleep(Duration::from_millis(50));
        let mut receiver = receiver;
        let n = receiver.listen_secure().expect("listen secure");
        assert_eq!(n, 1, "one encrypted agent frame accepted");
        assert!(receiver.known_agents.contains_key("sec-1"));
    }

    #[test]
    fn test_secure_rejects_wrong_secret() {
        let receiver = AgentDiscovery::new_secure(42121, b"right-secret").expect("bind receiver");
        let port = 42121;
        let sender = AgentDiscovery::new_secure(0, b"wrong-secret").expect("bind sender");

        let info = AgentInfo::new("bad", "bad-agent", "127.0.0.1", port);
        sender.broadcast_secure(&info, format!("127.0.0.1:{}", port).as_str()).expect("broadcast secure");

        std::thread::sleep(Duration::from_millis(50));
        let mut receiver = receiver;
        let n = receiver.listen_secure().expect("listen secure");
        assert_eq!(n, 0, "wrong secret fails AES-GCM auth -> identity gate drops it");
    }

    #[test]
    fn test_secure_ignores_plaintext_in_secure_mode() {
        let receiver = AgentDiscovery::new_secure(42122, b"s").expect("bind receiver");
        let port = 42122;
        let sender = UdpSocket::bind("127.0.0.1:0").expect("bind sender");

        let info = AgentInfo::new("plain", "p", "127.0.0.1", port);
        sender.send_to(serde_json::to_vec(&info).unwrap().as_slice(), format!("127.0.0.1:{}", port)).unwrap();

        std::thread::sleep(Duration::from_millis(50));
        let mut receiver = receiver;
        let n = receiver.listen_secure().expect("listen secure");
        assert_eq!(n, 0, "plaintext AgentInfo is not accepted in secure mode");
    }

    #[test]
    fn test_secure_tampered_frame_rejected() {
        let receiver = AgentDiscovery::new_secure(42123, b"s").expect("bind receiver");
        let port = 42123;
        let sender = AgentDiscovery::new_secure(0, b"s").expect("bind sender");

        let info = AgentInfo::new("tam", "tam", "127.0.0.1", port);
        let frame = {
            let key = sender.secure_key.expect("key");
            let mut nonce = [0u8; NONCE_LEN];
            rand::rngs::OsRng.fill_bytes(&mut nonce);
            let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
            let ct = cipher.encrypt(Nonce::from_slice(&nonce), serde_json::to_vec(&info).unwrap().as_slice()).unwrap();
            let mut f = Vec::new();
            f.extend_from_slice(SECURE_MAGIC);
            f.extend_from_slice(&nonce);
            f.extend_from_slice(&ct);
            f
        };
        // flip one ciphertext byte -> tag mismatch
        let last = frame.len() - 1;
        let mut tampered = frame.clone();
        tampered[last] ^= 0x01;
        let s = UdpSocket::bind("127.0.0.1:0").unwrap();
        s.send_to(&tampered, format!("127.0.0.1:{}", port)).unwrap();

        std::thread::sleep(Duration::from_millis(50));
        let mut receiver = receiver;
        let n = receiver.listen_secure().expect("listen secure");
        assert_eq!(n, 0, "tampered ciphertext fails authentication");
    }

    #[test]
    fn test_secure_replay_rejected() {
        let receiver = AgentDiscovery::new_secure(42124, b"s").expect("bind receiver");
        let port = 42124;
        let sender = AgentDiscovery::new_secure(0, b"s").expect("bind sender");

        let info = AgentInfo::new("replay", "r", "127.0.0.1", port);
        let frame = {
            let key = sender.secure_key.expect("key");
            let mut nonce = [0u8; NONCE_LEN];
            rand::rngs::OsRng.fill_bytes(&mut nonce);
            let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
            let ct = cipher.encrypt(Nonce::from_slice(&nonce), serde_json::to_vec(&info).unwrap().as_slice()).unwrap();
            let mut f = Vec::new();
            f.extend_from_slice(SECURE_MAGIC);
            f.extend_from_slice(&nonce);
            f.extend_from_slice(&ct);
            f
        };
        let s = UdpSocket::bind("127.0.0.1:0").unwrap();
        s.send_to(&frame, format!("127.0.0.1:{}", port)).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        s.send_to(&frame, format!("127.0.0.1:{}", port)).unwrap();

        std::thread::sleep(Duration::from_millis(50));
        let mut receiver = receiver;
        let n = receiver.listen_secure().expect("listen secure");
        assert_eq!(n, 1, "same nonce replayed -> only first accepted");
    }
}
