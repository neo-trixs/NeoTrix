//! X25519 密钥生成与交换
//!
//! 密钥对: `PrivateKey` (32 bytes) → `PublicKey` (32 bytes)
//! ECDH 交换: `PrivateKey::diffie_hellman(&public_key) -> SharedSecret`

use rand::rngs::OsRng;

/// X25519 私钥 (32 bytes, 零化保护)
pub struct PrivateKey {
    inner: x25519_dalek::StaticSecret,
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivateKey").field("inner", &"[redacted]").finish()
    }
}

impl Clone for PrivateKey {
    fn clone(&self) -> Self {
        Self::from_bytes(self.inner.as_bytes())
    }
}

/// X25519 公钥 (32 bytes)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PublicKey {
    inner: x25519_dalek::PublicKey,
}

/// ECDH 共享秘密
pub struct SharedSecret([u8; 32]);

impl PrivateKey {
    /// 生成随机私钥
    pub fn generate() -> Self {
        Self {
            inner: x25519_dalek::StaticSecret::random_from_rng(OsRng),
        }
    }

    /// 从字节切片构造 (用于测试/持久化)
    ///
    /// # Safety
    /// 调用方需确保 bytes 长度为 32
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            inner: x25519_dalek::StaticSecret::from(*bytes),
        }
    }

    /// 导出公钥
    pub fn public(&self) -> PublicKey {
        PublicKey {
            inner: x25519_dalek::PublicKey::from(&self.inner),
        }
    }

    /// ECDH Diffie-Hellman 密钥交换
    pub fn diffie_hellman(&self, peer: &PublicKey) -> SharedSecret {
        let secret = self.inner.diffie_hellman(&peer.inner);
        SharedSecret(secret.as_bytes().clone())
    }

    /// 零化内存中的私钥材料
    ///
    /// 注意: Rust 的 `SecretVec` 或 `Zeroize` trait 可提供更强的零化保证，
    /// 这里使用手动 zeroing 以保持最小依赖。
    pub fn zeroize(&mut self) {
        // StaticSecret doesn't expose mutable reference, here we regenerate to overwrite
        // Actual deployment should use zeroize crate or SecretVec
        self.inner = x25519_dalek::StaticSecret::random_from_rng(OsRng);
    }
}

impl PublicKey {
    /// 从字节切片构造
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            inner: x25519_dalek::PublicKey::from(*bytes),
        }
    }

    /// 导出为字节切片
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.inner.as_bytes()
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }
}

impl SharedSecret {
    /// 获取共享秘密的字节引用
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_exchange() {
        let alice = PrivateKey::generate();
        let bob = PrivateKey::generate();

        let alice_pub = alice.public();
        let bob_pub = bob.public();

        let alice_shared = alice.diffie_hellman(&bob_pub);
        let bob_shared = bob.diffie_hellman(&alice_pub);

        assert_eq!(alice_shared.to_bytes(), bob_shared.to_bytes());
    }

    #[test]
    fn different_keys_differ() {
        let alice = PrivateKey::generate();
        let bob = PrivateKey::generate();

        let alice_pub = alice.public();
        let bob_pub = bob.public();

        assert_ne!(alice_pub.to_bytes(), bob_pub.to_bytes());
    }

    #[test]
    fn from_bytes_roundtrip() {
        let alice = PrivateKey::generate();
        let bytes = alice.public().to_bytes();
        let restored = PublicKey::from_bytes(&bytes);
        assert_eq!(alice.public(), restored);
    }
}
