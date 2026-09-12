//! 密钥轮换策略与预计算安全
//!
//! WireGuard 密钥轮换:
//! - REKEY_AFTER_MESSAGES = 2^64 - 2^16 - 1
//! - REKEY_AFTER_TIME = 120s
//! - REJECT_AFTER_MESSAGES = 2^64 - 1
//! - REJECT_AFTER_TIME = 180s
//!
//! ## NDSS 2024 修正: 延迟预计算 + 零化
//! - 预计算的 ECDH 产物不立即存入持久存储
//! - 轮换后立即零化旧密钥材料
//! - 使用内存屏障防止编译器优化掉 zeroize

use std::time::{Duration, Instant};

use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::keys::PrivateKey;

/// 密钥轮换常量
pub const REKEY_AFTER_MESSAGES: u64 = u64::MAX - (1 << 16) - 1;
pub const REKEY_AFTER_TIME: Duration = Duration::from_secs(120);
pub const REJECT_AFTER_MESSAGES: u64 = u64::MAX - 1;
pub const REJECT_AFTER_TIME: Duration = Duration::from_secs(180);

/// Cookie 轮换间隔
pub const COOKIE_ROTATION_INTERVAL: Duration = Duration::from_secs(120);

/// 密钥轮换状态
#[derive(Debug, Clone)]
pub struct _RekeyState {
    /// 当前发送密钥
    send_private: Option<PrivateKey>,
    /// 当前发送密钥创建时间
    send_created: Option<Instant>,
    /// 当前发送密钥使用次数
    send_count: u64,
    /// 当前接收密钥
    recv_private: Option<PrivateKey>,
    /// 当前接收密钥创建时间
    recv_created: Option<Instant>,
    /// 当前接收密钥使用次数
    recv_count: u64,
}

impl _RekeyState {
    /// 创建初始状态
    pub fn new() -> Self {
        Self {
            send_private: None,
            send_created: None,
            send_count: 0,
            recv_private: None,
            recv_created: None,
            recv_count: 0,
        }
    }

    /// 检查发送密钥是否需要轮换
    pub fn _needs_send_rekey(&self) -> bool {
        // 检查消息计数
        if self.send_count >= REKEY_AFTER_MESSAGES {
            return true;
        }

        // 检查时间
        if let Some(created) = self.send_created {
            if created.elapsed() >= REKEY_AFTER_TIME {
                return true;
            }
        }

        false
    }

    /// 检查接收密钥是否需要轮换
    pub fn _needs_recv_rekey(&self) -> bool {
        if self.recv_count >= REKEY_AFTER_MESSAGES {
            return true;
        }

        if let Some(created) = self.recv_created {
            if created.elapsed() >= REKEY_AFTER_TIME {
                return true;
            }
        }

        false
    }

    /// 检查密钥是否已过期 (应拒绝)
    pub fn is_expired(&self) -> bool {
        // 发送密钥过期检查
        if let Some(created) = self.send_created {
            if created.elapsed() >= REJECT_AFTER_TIME {
                return true;
            }
        }

        // 接收密钥过期检查
        if let Some(created) = self.recv_created {
            if created.elapsed() >= REJECT_AFTER_TIME {
                return true;
            }
        }

        false
    }

    /// 轮换发送密钥 (延迟预计算 + 零化旧密钥)
    pub fn _rotate_send(&mut self) {
        // 1. 零化旧密钥
        if let Some(ref mut old_key) = self.send_private {
            old_key.zeroize();
        }

        // 2. 生成新密钥
        self.send_private = Some(PrivateKey::generate());
        self.send_created = Some(Instant::now());
        self.send_count = 0;
    }

    /// 轮换接收密钥
    pub fn _rotate_recv(&mut self) {
        if let Some(ref mut old_key) = self.recv_private {
            old_key.zeroize();
        }

        self.recv_private = Some(PrivateKey::generate());
        self.recv_created = Some(Instant::now());
        self.recv_count = 0;
    }

    /// 记录发送消息
    pub fn _on_send(&mut self) {
        self.send_count += 1;
    }

    /// 记录接收消息
    pub fn _on_recv(&mut self) {
        self.recv_count += 1;
    }

    /// 获取发送密钥引用
    pub fn send_key(&self) -> Option<&PrivateKey> {
        self.send_private.as_ref()
    }

    /// 获取接收密钥引用
    pub fn recv_key(&self) -> Option<&PrivateKey> {
        self.recv_private.as_ref()
    }
}

impl Default for _RekeyState {
    fn default() -> Self {
        Self::new()
    }
}

/// 密钥轮换决策
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _RekeyAction {
    /// 无需轮换
    None,
    /// 需要轮换发送密钥
    RotateSend,
    /// 需要轮换接收密钥
    RotateRecv,
    /// 需要轮换双方密钥
    RotateBoth,
    /// 密钥已过期，应断开连接
    Expired,
}

/// 决策函数: 根据当前状态决定操作
pub fn _decide_rekey(state: &_RekeyState) -> _RekeyAction {
    if state.is_expired() {
        return _RekeyAction::Expired;
    }

    let needs_send = state._needs_send_rekey();
    let needs_recv = state._needs_recv_rekey();

    match (needs_send, needs_recv) {
        (true, true) => _RekeyAction::RotateBoth,
        (true, false) => _RekeyAction::RotateSend,
        (false, true) => _RekeyAction::RotateRecv,
        (false, false) => _RekeyAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_no_rekey() {
        let state = _RekeyState::new();
        assert_eq!(_decide_rekey(&state), _RekeyAction::None);
    }

    #[test]
    fn expired_after_reject_timeout() {
        let mut state = _RekeyState::new();
        state.send_private = Some(PrivateKey::generate());
        state.send_created = Some(Instant::now() - REJECT_AFTER_TIME - Duration::from_secs(1));
        assert!(state.is_expired());
        assert_eq!(_decide_rekey(&state), _RekeyAction::Expired);
    }

    #[test]
    fn rotate_after_rekey_timeout() {
        let mut state = _RekeyState::new();
        state.send_private = Some(PrivateKey::generate());
        state.send_created = Some(Instant::now() - REKEY_AFTER_TIME - Duration::from_secs(1));
        assert!(state._needs_send_rekey());
        assert!(!state.is_expired());
    }
}
