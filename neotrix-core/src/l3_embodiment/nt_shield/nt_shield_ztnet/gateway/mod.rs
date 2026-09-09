//! C4: Gateway & Peer Management — 网关与对等管理层
//!
//! PeerStore (双索引 + 惰性配置), 会话密钥轮换, DNS 解析, 流日志。

pub mod dns_resolver;
pub mod flow_logger;
pub mod peer_store;
pub mod session;
pub mod tunnel;
