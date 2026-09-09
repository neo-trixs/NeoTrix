//! HKDF-BLAKE2s 密钥派生函数
//!
//! WireGuard 使用 HKDF-BLAKE2s 进行密钥派生。
//! 标准 HKDF: extract → expand (两阶段)
//!
//! 参考: RFC 5869 + WireGuard 协议规范

use blake2::{Blake2s256, Digest};

/// HKDF-BLAKE2s 输出长度 (32 bytes)
pub const HKDF_OUTPUT_LEN: usize = 32;

/// HKDF-BLAKE2s 密钥派生
///
/// # Arguments
/// * `ikm` - Input Key Material
/// * `salt` - Salt (可选, 传空切片则使用全零)
/// * `info` - Context info
/// * `output_len` - 输出长度 (必须 <= 255 * 32 = 8160)
///
/// # Returns
/// 派生出的密钥字节
pub fn hkdf_blake2s(
    ikm: &[u8],
    salt: &[u8],
    info: &[u8],
    output_len: usize,
) -> Result<Vec<u8>, KdfError> {
    if output_len > 8160 {
        return Err(KdfError::OutputTooLong);
    }

    // Extract: PRK = HMAC-Hash(salt, IKM)
    let prk = blake2s_extract(salt, ikm);

    // Expand: OKM = T(1) || T(2) || ... where T(i) = HMAC-Hash(PRK, T(i-1) || info || i)
    let mut output = Vec::with_capacity(output_len);
    let mut t = Vec::new();
    let mut counter = 1u8;

    while output.len() < output_len {
        let mut hasher = Blake2s256::new();
        hasher.update(&prk);
        hasher.update(&t);
        hasher.update(info);
        hasher.update([counter]);
        let block: [u8; 32] = hasher.finalize().into();

        let needed = output_len - output.len();
        let take = needed.min(32);
        output.extend_from_slice(&block[..take]);

        t = block.to_vec();
        counter += 1;
    }

    Ok(output)
}

/// HKDF-BLAKE2s 三输出变体 (WireGuard 握手专用)
///
/// `ikm = 0 || kem_output` 或 `ikm = chaining_key`
/// 输出: (tag, key1, key2)
pub fn hkdf_blake2s_3(
    ikm: &[u8],
    salt: &[u8],
) -> ([u8; 32], [u8; 32], [u8; 32]) {
    let t1 = hkdf_blake2s(ikm, salt, b"", 32).expect("32 bytes always valid");
    let t2 = hkdf_blake2s(&t1, salt, &[0x01], 32).expect("32 bytes always valid");
    let t3 = hkdf_blake2s(&t1, salt, &[0x02], 32).expect("32 bytes always valid");

    let mut out1 = [0u8; 32];
    let mut out2 = [0u8; 32];
    let mut out3 = [0u8; 32];
    out1.copy_from_slice(&t1);
    out2.copy_from_slice(&t2);
    out3.copy_from_slice(&t3);

    (out1, out2, out3)
}

/// HKDF Extract 阶段 (HMAC-Hash)
fn blake2s_extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2s256::new();
    hasher.update(salt);
    hasher.update(ikm);
    hasher.finalize().into()
}

#[derive(Debug, thiserror::Error)]
pub enum KdfError {
    #[error("output length exceeds HKDF maximum (8160 bytes)")]
    OutputTooLong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hkdf_basic() {
        let ikm = b"hello";
        let salt = b"salt";
        let info = b"info";

        let output = hkdf_blake2s(ikm, salt, info, 32).unwrap();
        assert_eq!(output.len(), 32);
    }

    #[test]
    fn hkdf_deterministic() {
        let out1 = hkdf_blake2s(b"ikm", b"salt", b"info", 64).unwrap();
        let out2 = hkdf_blake2s(b"ikm", b"salt", b"info", 64).unwrap();
        assert_eq!(out1, out2);
    }

    #[test]
    fn hkdf_different_salt_differs() {
        let out1 = hkdf_blake2s(b"ikm", b"salt1", b"info", 32).unwrap();
        let out2 = hkdf_blake2s(b"ikm", b"salt2", b"info", 32).unwrap();
        assert_ne!(out1, out2);
    }

    #[test]
    fn hkdf_3way() {
        let (t1, t2, t3) = hkdf_blake2s_3(b"ikm", b"salt");
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
        assert_ne!(t1, t3);
    }
}
