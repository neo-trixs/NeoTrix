//! Cookie DoS 防护与速率限制
//!
//! WireGuard Cookie 机制:
//! - Under load: 在 handshake response 中携带 cookie
//! - 后续 handshake 必须包含此 cookie
//! - 防止反射放大攻击
//!
//! ## 实现
//! - Per-IP token bucket: 20 握手/s/IP (内核模式)
//! - 1s 窗口超过 10 握手 → under_load
//! - Cookie 使用 BLAKE2s-MAC 生成

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use blake2::{Blake2s256, Digest};

/// Cookie 大小 (16 bytes)
pub const COOKIE_SIZE: usize = 16;

/// MAC1 大小 (16 bytes)
pub const MAC1_SIZE: usize = 16;

/// Under load 阈值: 每秒每个 IP 超过此数量的握手请求
const UNDERLOAD_THRESHOLD: usize = 10;

/// Token bucket 恢复速率 (tokens per second per IP)
#[allow(dead_code)]
const TOKEN_RECOVERY_RATE: usize = 20;

/// Token bucket 最大容量
const TOKEN_MAX: usize = 20;

/// 时间窗口 (秒)
const WINDOW_SECS: u64 = 1;

/// Per-IP 状态
#[derive(Debug, Clone)]
struct IpState {
    /// Token bucket 剩余
    tokens: usize,
    /// 窗口开始时间
    window_start: Instant,
    /// 当前窗口内的请求数
    request_count: usize,
}

/// Cookie 管理器
pub struct _CookieManager {
    /// Per-IP 速率限制状态
    ip_states: HashMap<IpAddr, IpState>,
    /// Cookie 密钥 (32 bytes, 定期轮换)
    cookie_key: [u8; 32],
    /// 上次密钥轮换时间
    last_key_rotation: Instant,
}

impl _CookieManager {
    /// 创建新的 _CookieManager
    pub fn new() -> Self {
        let mut cookie_key = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut cookie_key);

        Self {
            ip_states: HashMap::new(),
            cookie_key,
            last_key_rotation: Instant::now(),
        }
    }

    /// 检查是否处于 under load 状态
    ///
    /// 如果某 IP 在一个窗口内发送超过阈值的握手请求，标记为 under load
    pub fn _is_under_load(&mut self, peer_ip: IpAddr) -> bool {
        let now = Instant::now();
        let state = self.ip_states.entry(peer_ip).or_insert_with(|| IpState {
            tokens: TOKEN_MAX,
            window_start: now,
            request_count: 0,
        });

        // 窗口过期则重置
        if now.duration_since(state.window_start) > Duration::from_secs(WINDOW_SECS) {
            state.window_start = now;
            state.request_count = 0;
            state.tokens = TOKEN_MAX;
        }

        state.request_count += 1;

        // 超过阈值 → under load
        if state.request_count > UNDERLOAD_THRESHOLD {
            return true;
        }

        // 消耗 token
        if state.tokens > 0 {
            state.tokens -= 1;
            false
        } else {
            // Token 耗尽 → under load
            true
        }
    }

    /// 生成 cookie
    ///
    /// cookie = MAC(cookie_key, peer_ip || timestamp)
    pub fn _generate_cookie(&self, peer_ip: IpAddr) -> [u8; COOKIE_SIZE] {
        let mut hasher = Blake2s256::new();
        hasher.update(&self.cookie_key);
        hasher.update(&peer_ip.to_string().as_bytes());
        let result: [u8; 32] = hasher.finalize().into();
        let mut cookie = [0u8; COOKIE_SIZE];
        cookie.copy_from_slice(&result[..COOKIE_SIZE]);
        cookie
    }

    /// 验证 cookie
    pub fn _verify_cookie(&self, peer_ip: IpAddr, cookie: &[u8; COOKIE_SIZE]) -> bool {
        let expected = self._generate_cookie(peer_ip);
        // 常数时间比较
        expected.iter().zip(cookie.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
    }

    /// 生成 MAC1 (用于 handshake 消息)
    ///
    /// MAC1 = MAC(mac1_key, msg_without_mac)
    pub fn _compute_mac1(mac1_key: &[u8; 32], msg: &[u8]) -> [u8; MAC1_SIZE] {
        let mut hasher = Blake2s256::new();
        hasher.update(mac1_key);
        hasher.update(msg);
        let result: [u8; 32] = hasher.finalize().into();
        let mut mac1 = [0u8; MAC1_SIZE];
        mac1.copy_from_slice(&result[..MAC1_SIZE]);
        mac1
    }

    /// 验证 MAC1
    pub fn _verify_mac1(mac1_key: &[u8; 32], msg: &[u8], mac1: &[u8; MAC1_SIZE]) -> bool {
        let computed = Self::_compute_mac1(mac1_key, msg);
        computed.iter().zip(mac1.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
    }

    /// 恢复 token (用于成功的 handshake 后)
    pub fn _on_handshake_success(&mut self, peer_ip: IpAddr) {
        if let Some(state) = self.ip_states.get_mut(&peer_ip) {
            state.tokens = TOKEN_MAX;
            state.request_count = 0;
        }
    }

    /// 定期轮换 cookie 密钥
    pub fn _rotate_key_if_needed(&mut self) {
        if self.last_key_rotation.elapsed() > Duration::from_secs(60 * 5) {
            rand::Rng::fill(&mut rand::thread_rng(), &mut self.cookie_key);
            self.last_key_rotation = Instant::now();
        }
    }
}

impl Default for _CookieManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_generation_verification() {
        let manager = _CookieManager::new();
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        let cookie = manager._generate_cookie(ip);
        assert!(manager._verify_cookie(ip, &cookie));
        assert!(!manager._verify_cookie("10.0.0.1".parse().unwrap(), &cookie));
    }

    #[test]
    fn under_load_detection() {
        let mut manager = _CookieManager::new();
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // 发送 10 个请求 — 不应 under load
        for _ in 0..10 {
            assert!(!manager._is_under_load(ip));
        }

        // 第 11 个 → under load
        assert!(manager._is_under_load(ip));
    }

    #[test]
    fn mac1_verification() {
        let key = [42u8; 32];
        let msg = b"hello wireguard";
        let mac1 = _CookieManager::_compute_mac1(&key, msg);
        assert!(_CookieManager::_verify_mac1(&key, msg, &mac1));
        assert!(!_CookieManager::_verify_mac1(&[0u8; 32], msg, &mac1));
    }
}
