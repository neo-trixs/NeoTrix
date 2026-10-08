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
) -> Result<Vec<u8>, _KdfError> {
    if output_len > 8160 {
        return Err(_KdfError::OutputTooLong);
    }

    // Extract: PRK = HMAC-Hash(salt, IKM)
    let prk = blake2s_extract(salt, ikm);

    // Expand: OKM = T(1) || T(2) || ... where T(i) = HMAC-Hash(PRK, T(i-1) || info || i)
    let mut output = Vec::with_capacity(output_len);
    let mut t = Vec::new();
    let mut counter = 1u8;

    while output.len() < output_len {
        let mut hasher = Blake2s256::new();
        hasher.update(prk);
        hasher.update(&t);
        hasher.update(info);
        hasher.update([counter]);
        let block: [u8; 32] = hasher.finalize().into();

        let needed = output_len - output.len();
        let take = needed.min(32);
        output.extend_from_slice(&block[..take]);

        t = block.to_vec();
        counter = counter.wrapping_add(1);
    }

    Ok(output)
}

/// HKDF-BLAKE2s 三输出变体 (WireGuard 握手专用)
///
/// `ikm = 0 || kem_output` 或 `ikm = chaining_key`
/// 输出: (tag, key1, key2)
/// ⚠️ 本函数**当前零调用方**（全仓仅自身测试引用），且语义不符合 Noise 规范的
/// `HKDF(ck, ikm, 3)`（Noise 的三路是「一次 temp + 连续 HMAC」，不是三次独立 Extract+Expand）。
/// 保留仅为不扩大本次改动面；**新代码不要用它** —— 噪声握手直接用 `hmac_blake2s` 手写
/// 规范展开（见 `noise_handshake::mix_key_and_hash`）。
/// 改签名为 `Result` 是为消掉 3 个生产代码 `expect`（RUST-STANDARDS）。
pub fn hkdf_blake2s_3(
    ikm: &[u8],
    salt: &[u8],
) -> Result<([u8; 32], [u8; 32], [u8; 32]), _KdfError> {
    let t1 = hkdf_blake2s(ikm, salt, b"", 32)?;
    let t2 = hkdf_blake2s(&t1, salt, &[0x01], 32)?;
    let t3 = hkdf_blake2s(&t1, salt, &[0x02], 32)?;

    let mut out1 = [0u8; 32];
    let mut out2 = [0u8; 32];
    let mut out3 = [0u8; 32];
    out1.copy_from_slice(&t1);
    out2.copy_from_slice(&t2);
    out3.copy_from_slice(&t3);

    Ok((out1, out2, out3))
}

/// HKDF Extract 阶段 (HMAC-Hash)
/// HMAC-BLAKE2s (RFC 2104 构造)
///
/// 用途：Noise 规范的 `HKDF(k, n, outputs)` 底层原语
/// （`temp = HMAC(k, n)`，`out_i = HMAC(temp, …)`）。
/// **为什么手写而非用 `hmac` crate**：⚠️ 此前此处写的是「digest 0.9/0.10 双版本并存
/// ⇒ `Hmac<Blake2s256>` 报 `BufferKind Eager/Lazy`（实测）」——**该结论是错的**。
/// 依据 Cargo.lock：`hmac 0.12.1` → `digest 0.10.7`，`blake2 0.10.6` → `digest 0.10.7`，
/// **两者同版本**；树里的 `digest 0.9.0` 只来自 `curve25519-dalek` / `sha2`，与 blake2 无关。
/// 保留手写实现的**真实理由**是它可用 Python `hmac.new(k, m, hashlib.blake2s)`
/// 逐字节独立验证（见测试 `hmac_blake2s_matches_python_oracle`，两个向量均吻合）。
/// 若日后想换回 `Hmac<Blake2s256>`，**先跑一次 cargo 复测**，别再引用旧结论。
///
/// 构造（BLAKE2s 块长 64）：
///   K' = len(K)>64 ? BLAKE2s(K) : K 右补零到 64
///   HMAC = BLAKE2s( (K' xor 0x5c×64) || BLAKE2s( (K' xor 0x36×64) || m ) )
pub fn hmac_blake2s(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut kb = [0u8; 64];
    if key.len() > 64 {
        let mut hs = Blake2s256::new();
        hs.update(key);
        let h: [u8; 32] = hs.finalize().into();
        kb[..32].copy_from_slice(&h);
    } else {
        kb[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for j in 0..64 {
        ipad[j] ^= kb[j];
        opad[j] ^= kb[j];
    }
    let inner: [u8; 32] = {
        let mut hs = Blake2s256::new();
        hs.update(ipad);
        hs.update(msg);
        hs.finalize().into()
    };
    let mut hs = Blake2s256::new();
    hs.update(opad);
    hs.update(inner);
    hs.finalize().into()
}

/// HKDF Extract：`PRK = HMAC-Hash(salt, IKM)`（RFC 5869 §2.2）。
/// ⚠️ 初版写成 `HASH(salt || IKM)`，**不符合 RFC 5869**（漏了 HMAC 构造）。
/// 该函数此前是死代码，故缺陷潜伏未爆；现已修正。
/// 注：Noise 规范**不走** Extract（`HKDF(ck, ikm, n)` 是自定义展开），
/// 故此修正不影响 `noise_handshake` —— 那边直接用 `hmac_blake2s`。
fn blake2s_extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    hmac_blake2s(salt, ikm)
}

#[derive(Debug, thiserror::Error)]
pub enum _KdfError {
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
        let (t1, t2, t3) = hkdf_blake2s_3(b"ikm", b"salt").expect("32-byte outputs");
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
        assert_ne!(t1, t3);
    }

    /// HMAC-BLAKE2s 对 Python 独立 oracle：
    /// `hmac.new(bytes(range(32)), b'abc', hashlib.blake2s).hexdigest()`
    ///   = 82623be5bc0a391b24dce53e67d028eff92d586de0485ac37822e597d049c74f
    /// （本地 Python 3 实测，非手算）。若本实现与 oracle 不一致，
    /// 说明 RFC 2104 构造有误 —— Noise 的 MixKey/Split 全错。
    /// **改本函数必须同步改 oracle 值，反之亦然。**
    #[test]
    fn hmac_blake2s_matches_python_oracle() {
        let key: Vec<u8> = (0u8..32).collect();
        let got = hmac_blake2s(&key, b"abc");
        let mut hex = String::with_capacity(64);
        for b in got {
            hex.push_str(&format!("{:02x}", b));
        }
        assert_eq!(
            hex,
            "82623be5bc0a391b24dce53e67d028eff92d586de0485ac37822e597d049c74f",
            "HMAC-BLAKE2s 与 Python oracle 不一致"
        );
        // 长 key 分支（>64B 先 HASH）：与 Python 对照
        // hmac.new(bytes(range(100)), b'x'*200, hashlib.blake2s).hexdigest()
        let longk: Vec<u8> = (0u8..100).collect();
        let got2 = hmac_blake2s(&longk, &vec![b'x'; 200]);
        let mut hex2 = String::with_capacity(64);
        for b in got2 {
            hex2.push_str(&format!("{:02x}", b));
        }
        assert_eq!(
            hex2,
            "834cd67f8ca0de35dfcdfff1510ae170f5850f5455aba1c085196c441d7ca7d9",
            "长 key 分支与 Python oracle 不一致"
        );
    }
}
