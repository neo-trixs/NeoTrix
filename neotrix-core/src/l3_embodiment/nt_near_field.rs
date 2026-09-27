//! nt_near_field — EVO-09 近场协作平面（localsend＋authentik 思想）。
//!
//! 去中心近场通道的类型与纯逻辑：节点发现注册（上限 [`MAX_PEERS`]）、
//! 通道策略（mTLS 要求＋allowlist）、帧长门。真实传输与密码学在外层，
//! 本文件只做同步纯逻辑，无 IO / 全局状态。

use serde::{Deserialize, Serialize};

/// 发现注册上限。
pub const MAX_PEERS: usize = 16;
/// 默认发现/传输端口（localsend 同值，防火墙放行位）。
pub const DEFAULT_PORT: u16 = 53317;
/// 单帧上限（目录递归保结构在外层分片）。
pub const MAX_FRAME: usize = 65536;

/// 近场错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NearFieldError {
    EmptyId,
    EmptyAddr,
    InvalidPort,
    Duplicate,
    RegistryFull,
    NotAllowed,
    FrameTooLarge,
}

/// 近场节点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearFieldPeer {
    pub id: String,
    pub alias: String,
    pub addr: String,
    pub port: u16,
}

impl NearFieldPeer {
    pub fn validate(&self) -> Result<(), NearFieldError> {
        if self.id.is_empty() || self.id.len() > 64 {
            return Err(NearFieldError::EmptyId);
        }
        if self.addr.is_empty() {
            return Err(NearFieldError::EmptyAddr);
        }
        if self.port == 0 {
            return Err(NearFieldError::InvalidPort);
        }
        Ok(())
    }
}

/// 发现注册表（内存）。
#[derive(Debug, Default, Clone)]
pub struct DiscoveryRegistry {
    peers: Vec<NearFieldPeer>,
}

impl DiscoveryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// 宣布节点（校验＋去重＋上限）。
    pub fn announce(&mut self, peer: NearFieldPeer) -> Result<(), NearFieldError> {
        peer.validate()?;
        if self.peers.iter().any(|p| p.id == peer.id) {
            return Err(NearFieldError::Duplicate);
        }
        if self.peers.len() >= MAX_PEERS {
            return Err(NearFieldError::RegistryFull);
        }
        self.peers.push(peer);
        Ok(())
    }

    /// 撤下节点（不存在返回 false）。
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.peers.iter().position(|p| p.id == id) {
            self.peers.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn list_ids(&self) -> Vec<&str> {
        self.peers.iter().map(|p| p.id.as_str()).collect()
    }
}

/// 通道策略。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelPolicy {
    /// 要求 mTLS（标志位，握手在外层）。
    pub require_mtls: bool,
    /// allowlist（空＝全放行）。
    pub allowlist: Vec<String>,
}

impl Default for ChannelPolicy {
    fn default() -> Self {
        Self { require_mtls: true, allowlist: Vec::new() }
    }
}

impl ChannelPolicy {
    /// 授权节点（allowlist 非空时必须在列）。
    pub fn authorize(&self, peer_id: &str) -> Result<(), NearFieldError> {
        if !self.allowlist.is_empty() && !self.allowlist.iter().any(|a| a == peer_id) {
            return Err(NearFieldError::NotAllowed);
        }
        Ok(())
    }
}

/// 帧长门。
pub fn check_frame(len: usize) -> Result<(), NearFieldError> {
    if len > MAX_FRAME {
        return Err(NearFieldError::FrameTooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(id: &str) -> NearFieldPeer {
        NearFieldPeer {
            id: id.to_owned(),
            alias: "neo".to_owned(),
            addr: "192.168.1.2".to_owned(),
            port: DEFAULT_PORT,
        }
    }

    #[test]
    fn validate_edges() {
        assert!(peer("a").validate().is_ok());
        let mut bad = peer("a");
        bad.id.clear();
        assert_eq!(bad.validate(), Err(NearFieldError::EmptyId));
        let mut bad = peer("a");
        bad.port = 0;
        assert_eq!(bad.validate(), Err(NearFieldError::InvalidPort));
    }

    #[test]
    fn registry_dup_remove_full() {
        let mut r = DiscoveryRegistry::new();
        assert!(r.announce(peer("a")).is_ok());
        assert_eq!(r.announce(peer("a")), Err(NearFieldError::Duplicate));
        for i in 0..15 {
            r.announce(peer(&format!("n{i}"))).ok();
        }
        assert_eq!(r.len(), MAX_PEERS);
        assert_eq!(r.announce(peer("over")), Err(NearFieldError::RegistryFull));
        assert!(r.remove("a"));
        assert!(!r.remove("a"));
    }

    #[test]
    fn policy_allowlist() {
        let open = ChannelPolicy::default();
        assert!(open.authorize("any").is_ok());
        let strict = ChannelPolicy { require_mtls: true, allowlist: vec!["a".to_owned()] };
        assert!(strict.authorize("a").is_ok());
        assert_eq!(strict.authorize("b"), Err(NearFieldError::NotAllowed));
    }

    #[test]
    fn frame_gate() {
        assert!(check_frame(MAX_FRAME).is_ok());
        assert_eq!(check_frame(MAX_FRAME + 1), Err(NearFieldError::FrameTooLarge));
    }
}
