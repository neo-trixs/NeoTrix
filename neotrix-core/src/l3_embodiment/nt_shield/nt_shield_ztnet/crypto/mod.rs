//! C0: Cryptographic Primitives — 密码学原语层
//!
//! 零依赖纯函数库。所有密码学操作均无 IO。
//!
//! ## 安全修正 (来自 NDSS 2024 研究)
//! - `identity_hiding`: 会话级密钥混淆 + 随机填充，防止跨会话静态密钥链接
//! - `rekey`: 延迟预计算 + 零化策略，防止内存中 ECDH 产物暴露

pub mod aead;
pub mod cookie;
pub mod identity_hiding;
pub mod kdf;
pub mod keys;
pub mod noise_handshake;
pub mod rekey;
