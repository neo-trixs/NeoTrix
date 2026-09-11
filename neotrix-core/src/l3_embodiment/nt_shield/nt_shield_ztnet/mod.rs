//! NT-SHIELD ZT-Net — Zero Trust Network Capability Cluster
//!
//! 六层能力簇架构 (自下而上单向依赖):
//!
//! - `crypto` (C0): 密码学原语 — Noise handshake, AEAD, KDF, X25519, DoS 防护
//! - `protocol` (C1): SANS-IO 协议引擎 — 零 IO 纯状态机
//! - `packet` (C2): 包处理与设备 — IP 解析, TUN 设备, 路由表, 过滤器
//! - `connectivity` (C3): 连通性与穿透 — ICE, STUN, TURN, 路径评分, Relay 降级
//! - `gateway` (C4): 网关与对等管理 — PeerStore, 会话密钥, DNS, 流日志
//! - `policy` (C5): 策略与控制 — NIST ZTA PE/PA/PEP, Batcher, WebSocket

pub mod crypto;
pub mod protocol;
pub mod packet;
pub mod connectivity;
pub mod gateway;
pub mod policy;
pub mod ztnet_capability;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZtnetError {
    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("packet error: {0}")]
    Packet(String),

    #[error("connectivity error: {0}")]
    Connectivity(String),

    #[error("gateway error: {0}")]
    Gateway(String),

    #[error("policy error: {0}")]
    Policy(String),
}

pub type Result<T> = std::result::Result<T, ZtnetError>;
