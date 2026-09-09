//! ChaCha20-Poly1305 AEAD 加解密
//!
//! 用于 WireGuard 数据包加密。零拷贝设计。
//!
//! WireGuard 使用 ChaCha20-Poly1305:
//! - Nonce: 8 bytes (Counter || Random)
//! - AAD: 可选 (WireGuard 不使用 AAD)
//! - Tag: 16 bytes

use ring::aead;

/// AEAD Nonce (ChaCha20-Poly1305 使用 8 bytes nonce + 4 bytes zero padding = 12 bytes total)
const NONCE_LEN: usize = 12;

/// ChaCha20-Poly1305 加密密钥 (32 bytes)
#[derive(Clone)]
pub struct AeadKey {
    inner: aead::LessSafeKey,
}

/// Nonce (8 bytes counter + 4 bytes zero)
pub struct Nonce([u8; NONCE_LEN]);

impl AeadKey {
    /// 从 32 字节切片构造密钥
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        let unbound = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key_bytes)
            .expect("invalid key length for ChaCha20-Poly1305");
        Self {
            inner: aead::LessSafeKey::new(unbound),
        }
    }

    /// 使用 counter 构造 nonce (WireGuard 格式: counter || 4 bytes zero)
    pub fn nonce_from_counter(counter: u64) -> Nonce {
        let mut nonce = [0u8; NONCE_LEN];
        nonce[..8].copy_from_slice(&counter.to_le_bytes());
        Nonce(nonce)
    }

    /// AEAD 加密
    ///
    /// 返回: ciphertext || 16-byte tag
    pub fn seal(&self, nonce: &Nonce, plaintext: &[u8]) -> Vec<u8> {
        let mut in_out = plaintext.to_vec();
        let seal_nonce = aead::Nonce::assume_unique_for_key(nonce.0);
        self.inner.seal_in_place_append_tag(seal_nonce, aead::Aad::empty(), &mut in_out)
            .expect("AEAD seal failed");
        in_out
    }

    /// AEAD 解密
    ///
    /// 输入: ciphertext || 16-byte tag
    /// 返回: plaintext (tag 已被 ring 原地剥离)
    pub fn open(&self, nonce: &Nonce, ciphertext_with_tag: &mut [u8]) -> Result<&[u8], AeadError> {
        let open_nonce = aead::Nonce::assume_unique_for_key(nonce.0);
        let plaintext = self.inner.open_in_place(open_nonce, aead::Aad::empty(), ciphertext_with_tag)
            .map_err(|_| AeadError::DecryptionFailed)?;
        Ok(plaintext)
    }

    /// 从原始字节构造 (用于测试)
    #[cfg(test)]
    fn from_bytes(key_bytes: &[u8; 32]) -> Self {
        Self::new(key_bytes)
    }
}

impl Nonce {
    /// 从字节切片构造
    pub fn from_bytes(bytes: &[u8; 12]) -> Self {
        Nonce(*bytes)
    }

    /// 导出为字节数组
    pub fn as_bytes(&self) -> &[u8; 12] {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AeadError {
    #[error("decryption failed: invalid key or ciphertext")]
    DecryptionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key_bytes = [42u8; 32];
        let key = AeadKey::new(&key_bytes);
        let nonce = AeadKey::nonce_from_counter(1);

        let plaintext = b"hello wireguard";
        let mut ciphertext = key.seal(&nonce, plaintext);
        assert!(ciphertext.len() > plaintext.len());

        let decrypted = key.open(&nonce, &mut ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = AeadKey::new(&[1u8; 32]);
        let key2 = AeadKey::new(&[2u8; 32]);
        let nonce = AeadKey::nonce_from_counter(0);

        let mut ciphertext = key1.seal(&nonce, b"secret");
        assert!(key2.open(&nonce, &mut ciphertext).is_err());
    }

    #[test]
    fn wrong_nonce_fails() {
        let key = AeadKey::new(&[42u8; 32]);
        let nonce1 = AeadKey::nonce_from_counter(0);
        let nonce2 = AeadKey::nonce_from_counter(1);

        let mut ciphertext = key.seal(&nonce1, b"secret");
        assert!(key.open(&nonce2, &mut ciphertext).is_err());
    }
}
