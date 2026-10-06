//! Noise IKpsk2 握手引擎
//!
//! WireGuard 使用 Noise IK 模式 + PSK (psk2)：
//! - Initiator → Responder: `e` (ephemeral key)
//! - Responder → Initiator: `e, ee, s, es, psk2`
//! - Initiator → Responder: `s, se, psk2`
//!
//! 所有操作均为 SANS-IO (纯状态机)，不执行任何网络 IO。

use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::aead::{AeadKey, Nonce};
use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::keys::{_SharedSecret, PrivateKey, PublicKey};
use blake2::{Blake2s256, Digest};
use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::kdf::hmac_blake2s;

/// 规范协议名（39B）。`Noise_IKpsk2_25519_ChaCha`（25B）不是合法 Noise 名 ——
/// 规范要求 4 段 `pattern_DH_cipher_hash`，且本仓 HASH 是 BLAKE2s（不是 SHA256）。
/// 39 > BLAKE2s HASHLEN(32) ⇒ `Initialize` 时 `h = HASH(protocol_name)`。
const PROTOCOL_NAME: &[u8] = b"Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s";



/// 握手状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _HandshakeState {
    Initial,
    Message1Sent,
    Message1Received,
    Message2Sent,
    Message2Received,
    Completed,
    Failed,
}

/// 握手完成后的输出
#[derive(Debug)]
pub struct _HandshakeResult {
    /// 发送方向密钥
    pub send_key: AeadKey,
    /// 接收方向密钥
    pub recv_key: AeadKey,
    /// 对端静态公钥
    pub peer_static_public: PublicKey,
}

/// Noise IKpsk2 握手引擎
pub struct _NoiseHandshake {
    /// 本地静态私钥
    static_private: PrivateKey,
    /// 本地静态公钥
    static_public: PublicKey,
    /// 对端静态公钥
    remote_static_public: Option<PublicKey>,
    /// PSK (可选)
    psk: Option<[u8; 32]>,
    /// 当前状态
    state: _HandshakeState,
    /// 握手哈希 h。`EncryptAndHash` 的 AEAD associated data。
    /// 规范 §5.2：`MixHash` 只推进本字段。
    hash: [u8; 32],
    /// 链式密钥 ck。HKDF 的基键。规范 §5.2：`MixKey` 只推进本字段。
    /// ⚠️ 必须与 `hash` 严格分离 —— 初版复用同一字段，导致每次 `MixHash`
    /// 污染 ck、每次 `MixKey` 污染 h，msg_0 自第 32B 起分叉（官方向量测试抓到）。
    chaining_key: [u8; 32],
    /// 对称密钥 k（`InitializeKey` 的产物）
    symmetric_key: [u8; 32],
    /// AEAD nonce 计数器。规范 `EncryptAndHash`：nonce = 4 字节零 || LE64(n)，
    /// 每条消息 n += 1。恒零 nonce 在 ChaCha20-Poly1305 下是严重误用。
    nonce: u64,
    /// 临时密钥对 (用于当前握手)
    ephemeral_private: Option<PrivateKey>,
    /// 对端临时公钥
    remote_ephemeral: Option<PublicKey>,
}

impl _NoiseHandshake {
    /// 创建 Initiator 握手
    ///
    /// 规范 `Initialize` + IK pre-message（`<- s`）：
    /// `h = HASH(protocol_name)`（39B > 32B，故哈希而非填充），
    /// `ck = h`，随后 `MixHash(s_r)`（initiator 预知 responder 静态公钥）。
    pub fn _initiator(
        static_private: PrivateKey,
        remote_static: PublicKey,
        psk: Option<[u8; 32]>,
    ) -> Self {
        let static_public = static_private.public();
        // 协议名 39B > BLAKE2s HASHLEN(32B) ⇒ h = HASH(name)，不是零填充。
        let mut hasher = Blake2s256::new();
        hasher.update(PROTOCOL_NAME);
        let hash: [u8; 32] = hasher.finalize().into();

        let mut hs = Self {
            static_private,
            static_public,
            remote_static_public: Some(remote_static.clone()),
            psk,
            state: _HandshakeState::Initial,
            hash,
            // 规范 Initialize：h = ck = HASH(protocol_name)
            chaining_key: hash,
            symmetric_key: [0u8; 32],
            nonce: 0,
            ephemeral_private: None,
            remote_ephemeral: None,
        };
        // 规范 Initialize 恒调 MixHash(prologue)，即使空 prologue 也要 H(h||"")。
        // 漏掉它 h 与 ck 从第一步就分叉错误（实测抓到，见 B-2 文档）。
        hs.mix_hash(&[]);
        // IK pre-message `<- s`：initiator MixHash 对端静态公钥。
        hs.mix_hash(remote_static.as_bytes());
        hs
    }

    /// 创建 Responder 握手
    ///
    /// 同上。Responder 对 pre-message `<- s` MixHash 自身的静态公钥
    /// （与 initiator 侧对称，保证两侧 h 一致）。
    pub fn _responder(static_private: PrivateKey, psk: Option<[u8; 32]>) -> Self {
        let static_public = static_private.public();
        let mut hasher = Blake2s256::new();
        hasher.update(PROTOCOL_NAME);
        let hash: [u8; 32] = hasher.finalize().into();

        let mut hs = Self {
            static_private,
            static_public: static_public.clone(),
            remote_static_public: None,
            psk,
            state: _HandshakeState::Initial,
            hash,
            // 规范 Initialize：h = ck = HASH(protocol_name)
            chaining_key: hash,
            symmetric_key: [0u8; 32],
            nonce: 0,
            ephemeral_private: None,
            remote_ephemeral: None,
        };
        hs.mix_hash(&[]);
        hs.mix_hash(static_public.as_bytes());
        hs
    }

    pub fn state(&self) -> &_HandshakeState {
        &self.state
    }

    /// Message 1: Initiator → Responder
    ///
    /// 规范 IKpsk2 `-> e, es, s, ss`（+ 空 payload 的 EncryptAndHash tag）：
    ///   e(32) + enc_s(32+16) + enc_empty(16) = **96B**（官方向量 msg_0 实测）。
    /// 临时密钥若已预置（向量测试固定 ephemeral 用）则直接取用，否则现场生成。
    pub fn _create_message1(&mut self) -> Result<Vec<u8>, _NoiseError> {
        if self.state != _HandshakeState::Initial {
            return Err(_NoiseError::InvalidState);
        }
        let remote_sr = self
            .remote_static_public
            .clone()
            .ok_or(_NoiseError::InvalidState)?;

        // e
        let ephemeral = self
            .ephemeral_private
            .take()
            .unwrap_or_else(PrivateKey::generate);
        let ephemeral_pub = ephemeral.public();
        self.mix_hash(ephemeral_pub.as_bytes());
        // 规范 §9 PSK 绑定规则：PSK 握手的每个 `e` token 后追加 MixKey(e.pub)。
        // 无它则与官方向量差结构性偏差（第三方参考实现同行为，且复现官方逐字节一致）。
        // 仅 psk.is_some() 时执行（非 PSK 的纯 IK 不走）。
        if self.psk.is_some() {
            self.mix_key(ephemeral_pub.as_bytes());
        }
        let mut msg = ephemeral_pub.to_bytes().to_vec();

        // es = DH(e_i, s_r)
        let es = ephemeral.diffie_hellman(&remote_sr);
        Self::_check_shared(&es)?;
        self.mix_key(es.as_bytes());

        // s (EncryptAndHash)。先拷贝再 &mut（否则 E0502：不可变借用与可变借用同时存活）。
        let local_spk = *self.static_public.as_bytes();
        let enc_static = self.encrypt_and_hash(&local_spk)?;
        msg.extend_from_slice(&enc_static);

        // ss = DH(s_i, s_r)
        let ss = self.static_private.diffie_hellman(&remote_sr);
        self.mix_key(ss.as_bytes());

        // 空 payload 的 EncryptAndHash（规范 WriteMessage 恒加密 payload，
        // 空 payload 产 16B tag —— 这正是 msg_0 96B 的最后 16B）。
        let enc_empty = self.encrypt_and_hash(&[])?;
        msg.extend_from_slice(&enc_empty);

        // e_i 留给读 msg2 时的 `ee` 用。
        self.ephemeral_private = Some(ephemeral);
        self.state = _HandshakeState::Message1Sent;
        Ok(msg)
    }

    /// Responder 消费 message 1
    ///
    /// 规范镜像：`e` → `es = DH(s_r, e_i)` → 解密 `s` → `ss = DH(s_r, s_i)` →
    /// 解密空 payload。msg1 必须恰 96B（32 + 48 + 16）。
    pub fn _consume_message1(&mut self, msg: &[u8]) -> Result<PublicKey, _NoiseError> {
        if self.state != _HandshakeState::Initial {
            return Err(_NoiseError::InvalidState);
        }
        if msg.len() != 96 {
            return Err(_NoiseError::MessageTooShort);
        }

        // e
        let ephemeral_pub = PublicKey::from_bytes(
            msg[..32]
                .try_into()
                .map_err(|_| _NoiseError::MessageTooShort)?,
        );
        self.mix_hash(&msg[..32]);
        if self.psk.is_some() {
            self.mix_key(&msg[..32]);
        }
        self.remote_ephemeral = Some(ephemeral_pub.clone());

        // es = DH(s_r, e_i)
        let es = self.static_private.diffie_hellman(&ephemeral_pub);
        self.mix_key(es.as_bytes());

        // s (DecryptAndHash)
        let enc_static = &msg[32..32 + 48];
        let plain_static = self.decrypt_and_hash(enc_static)?;
        let mut static_bytes = [0u8; 32];
        static_bytes.copy_from_slice(&plain_static[..32]);
        let initiator_static = PublicKey::from_bytes(&static_bytes);
        self.remote_static_public = Some(initiator_static.clone());

        // ss = DH(s_r, s_i)
        let ss = self
            .static_private
            .diffie_hellman(&initiator_static);
        self.mix_key(ss.as_bytes());

        // 空 payload（16B tag）
        let _ = self.decrypt_and_hash(&msg[80..96])?;

        self.state = _HandshakeState::Message1Received;
        Ok(ephemeral_pub)
    }

    /// Message 2: Responder → Initiator
    ///
    /// 规范 IKpsk2 `<- e, ee, se, psk`（+ 空 payload tag）：
    ///   e(32) + enc_empty(16) = **48B**（官方向量 msg_1 实测）。
    /// 写完即 Split，返回 (msg, result)。Responder 发 c2、收 c1。
    pub fn _create_message2(&mut self) -> Result<(Vec<u8>, _HandshakeResult), _NoiseError> {
        if self.state != _HandshakeState::Message1Received {
            return Err(_NoiseError::InvalidState);
        }
        let remote_eph = self
            .remote_ephemeral
            .clone()
            .ok_or(_NoiseError::InvalidState)?;
        let remote_si = self
            .remote_static_public
            .clone()
            .ok_or(_NoiseError::InvalidState)?;

        // e（预置则取用，否则现场生成；e_r 留给传输轮换，可丢弃）
        let ephemeral = self
            .ephemeral_private
            .take()
            .unwrap_or_else(PrivateKey::generate);
        let e_pub = ephemeral.public();
        self.mix_hash(e_pub.as_bytes());
        if self.psk.is_some() {
            self.mix_key(e_pub.as_bytes());
        }
        let mut msg = e_pub.to_bytes().to_vec();

        // ee = DH(e_r, e_i)
        let ee = ephemeral.diffie_hellman(&remote_eph);
        Self::_check_shared(&ee)?;
        self.mix_key(ee.as_bytes());

        // se = DH(e_r, s_i) —— 规范 §10.3：responder 侧是 DH(e, rs)。
        // ⚠️ 初版写成 DH(s_r, e_i)（与 msg1 的 es 同一个 DH 值），而 initiator 侧混
        // DH(s_i, e_r) ⇒ 两侧 ck 立刻分叉，Split 永不可能一致。
        let se = ephemeral.diffie_hellman(&remote_si);
        self.mix_key(se.as_bytes());

        // psk（psk2：在 msg2 末 MixKeyAndHash）
        if let Some(psk) = self.psk {
            self.mix_key_and_hash(&psk);
        }

        // 空 payload tag（16B）
        let enc_empty = self.encrypt_and_hash(&[])?;
        msg.extend_from_slice(&enc_empty);

        // Split。Responder：发 c2、收 c1。
        let (c1, c2) = self.split();
        self.state = _HandshakeState::Message2Sent;
        // responder 写完 msg2 即完成（规范无 msg3）。
        self.state = _HandshakeState::Completed;

        Ok((
            msg,
            _HandshakeResult {
                send_key: c2,
                recv_key: c1,
                peer_static_public: remote_si,
            },
        ))
    }

    /// Initiator 消费 message 2
    ///
    /// 规范镜像：`e` → `ee = DH(e_i, e_r)` → `se = DH(s_i, e_r)` →
    /// `psk` → 解密空 payload → Split。msg2 必须恰 48B（32 + 16）。
    /// 完成后 initiator 发 c1、收 c2。**规范 IKpsk2 无 message 3。**
    pub fn _consume_message2(&mut self, msg: &[u8]) -> Result<_HandshakeResult, _NoiseError> {
        if self.state != _HandshakeState::Message1Sent {
            return Err(_NoiseError::InvalidState);
        }
        if msg.len() != 48 {
            return Err(_NoiseError::MessageTooShort);
        }

        // e
        let e_pub = PublicKey::from_bytes(
            msg[..32]
                .try_into()
                .map_err(|_| _NoiseError::MessageTooShort)?,
        );
        self.mix_hash(&msg[..32]);
        if self.psk.is_some() {
            self.mix_key(&msg[..32]);
        }
        self.remote_ephemeral = Some(e_pub.clone());

        let eph_priv = self
            .ephemeral_private
            .clone()
            .ok_or(_NoiseError::InvalidState)?;

        // ee = DH(e_i, e_r)
        let ee = eph_priv.diffie_hellman(&e_pub);
        Self::_check_shared(&ee)?;
        self.mix_key(ee.as_bytes());

        // se = DH(s_i, e_r)
        let se = self.static_private.diffie_hellman(&e_pub);
        self.mix_key(se.as_bytes());

        // psk
        if let Some(psk) = self.psk {
            self.mix_key_and_hash(&psk);
        }

        // 空 payload（16B tag）
        let _ = self.decrypt_and_hash(&msg[32..48])?;

        let remote_static = self
            .remote_static_public
            .clone()
            .ok_or(_NoiseError::InvalidState)?;

        // Split。Initiator：发 c1、收 c2。
        let (c1, c2) = self.split();
        self.state = _HandshakeState::Completed;

        Ok(_HandshakeResult {
            send_key: c1,
            recv_key: c2,
            peer_static_public: remote_static,
        })
    }

    // ---- 内部辅助 ----

    /// 小阶点防护。X25519 对低阶公钥返回**全零**共享秘密，而 `e` 完全由对端控制
    /// ⇒ 攻击者可强制 `es`/`ee` 退化为已知值。所有用到对端 `e` 的 DH 都必须过这道。
    fn _check_shared(shared: &_SharedSecret) -> Result<(), _NoiseError> {
        if shared.as_bytes().iter().all(|&b| b == 0) {
            return Err(_NoiseError::InvalidState);
        }
        Ok(())
    }

    /// 规范 MixHash(data)：h = HASH(h || data)。原名 `hash_concat`，改名以对齐规范术语。
    fn mix_hash(&mut self, data: &[u8]) {
        use blake2::{Blake2s256, Digest};
        let mut hasher = Blake2s256::new();
        hasher.update(&self.hash);
        hasher.update(data);
        self.hash = hasher.finalize().into();
    }

    fn mix_key(&mut self, input_key_material: &[u8]) {
        // 规范 MixKey：(ck, temp_k) = HKDF(ck, ikm, 2)，其中
        //   HKDF(k, n, 2) = ( HMAC(k,n),  HMAC( HMAC(k,n), 0x01 ) )
        // 故：temp = HMAC(ck, ikm)；ck' = HMAC(temp, 0x01)；k' = HMAC(temp, ck'||0x02)；n 置零。
        // ⚠️ 初版此处曾把 ck' 取成 temp 本身（少算一轮输出），官方向量测试会抓到。
        let temp = hmac_blake2s(&self.chaining_key, input_key_material);
        let ck_new = hmac_blake2s(&temp, &[0x01]);
        let mut k_in = ck_new.to_vec();
        k_in.push(0x02);
        let k_new = hmac_blake2s(&temp, &k_in);
        self.chaining_key = ck_new;
        self.symmetric_key = k_new;
        self.nonce = 0;
    }

    /// 规范 §5.2 `MixKeyAndHash(ikm)` = **三路** HKDF，不是 MixKey+MixHash：
    ///   `ck, temp_h, temp_k = HKDF(ck, ikm, 3)`；`MixHash(temp_h)`；`InitializeKey(temp_k)`
    /// ⚠️ 初版实现成 `MixKey(psk)` + `MixHash(psk)`（两路且混的是 psk 本身），
    /// 使 msg_1 的 tag 变成 `311c6ddf…`；三路版得官方向量 `8c46d966…`。
    fn mix_key_and_hash(&mut self, ikm: &[u8; 32]) {
        let temp = hmac_blake2s(&self.chaining_key, ikm);
        let ck_new = hmac_blake2s(&temp, &[0x01]);
        let mut h_in = ck_new.to_vec();
        h_in.push(0x02);
        let temp_h = hmac_blake2s(&temp, &h_in);
        let mut k_in = temp_h.to_vec();
        k_in.push(0x03);
        let k_new = hmac_blake2s(&temp, &k_in);
        self.chaining_key = ck_new;
        self.symmetric_key = k_new;
        self.nonce = 0;
        // 注意：混进 h 的是 temp_h，不是 psk 本身。
        self.mix_hash(&temp_h);
    }

    /// 规范 nonce：4 字节零 || LE64(n)。WireGuard 格式（LE64 || 4 零）**相反**，
    /// 故不用 `AeadKey::_nonce_from_counter`，此处显式构造。
    fn current_nonce(&self) -> Nonce {
        let mut b = [0u8; 12];
        b[4..12].copy_from_slice(&self.nonce.to_le_bytes());
        Nonce::from_bytes(&b)
    }

    /// 规范 EncryptAndHash(plaintext)：AEAD 加密并 MixHash(密文)，nonce 自增。
    /// 本 IKpsk2 流中每次加密前 MixKey 已执行，故 k 恒已就绪，直接走 AEAD。
    fn encrypt_and_hash(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, _NoiseError> {
        let key = AeadKey::new(&self.symmetric_key);
        // 规范 §5.2：EncryptAndHash = EncryptWithAd(h, plaintext)，AD 必须传 h
        let ct = key
            .seal_with_ad(&self.current_nonce(), &self.hash, plaintext)
            .map_err(|_| _NoiseError::EncryptionFailed)?;
        self.nonce = self.nonce.saturating_add(1);
        self.mix_hash(&ct);
        Ok(ct)
    }

    /// 规范 DecryptAndHash：解密成功后 MixHash(**密文**，不是明文)。
    fn decrypt_and_hash(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, _NoiseError> {
        let key = AeadKey::new(&self.symmetric_key);
        let nonce = self.current_nonce();
        let mut buf = ciphertext.to_vec();
        let pt = key
            .open_with_ad(&nonce, &self.hash, &mut buf)
            .map(|b| b.to_vec())
            .map_err(|_| _NoiseError::DecryptionFailed)?;
        self.nonce = self.nonce.saturating_add(1);
        self.mix_hash(ciphertext);
        Ok(pt)
    }

    /// 规范 Split()：(k1, k2) = HKDF(ck, zerolen, 2)。
    /// initiator 用 (k1=发, k2=收)，responder 反之。
    fn split(&self) -> (AeadKey, AeadKey) {
        // ⚠️ `zerolen` 是**空切片**，不是 32 个零字节。规范 §5.2：`HKDF(ck, zerolen, 2)`。
        // 用 `&[0u8; 32]` 会让 k1/k2 全错（实测 c1×"yellowsubmarine" 不等于官方 msg_2）。
        let temp = hmac_blake2s(&self.chaining_key, &[]);
        let k1 = hmac_blake2s(&temp, &[0x01]);
        let mut k2_in = k1.to_vec();
        k2_in.push(0x02);
        let k2 = hmac_blake2s(&temp, &k2_in);
        (AeadKey::new(&k1), AeadKey::new(&k2))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum _NoiseError {
    #[error("invalid handshake state")]
    InvalidState,
    #[error("message too short")]
    MessageTooShort,
    #[error("decryption failed")]
    DecryptionFailed,
    #[error("encryption failed")]
    EncryptionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ⛔ 本模块**从未成功完成过一次握手**，测试断言的是一个不存在的实现，
    //    按仓规「实现或 `#[ignore]`，禁改松断言」挂起。
    //
    // 根因（2026-09-28 读现场确认，非转述）：`_create_message2`（responder 侧）
    // 计算 `es = DH(自身 ephemeral, remote_static)`（`:190-194`），而按
    // **Noise IK** 模式，msg2 里加密送达的是**responder 自己的静态公钥** `re`，
    // 而 `es` 应为 `DH(e_i, rs_r)` = **initiator 的临时密钥 × responder 的静态**。
    // 现实现把两个角色接反了：
    //   1. 角色反了 —— `es` 用 responder 的 `e` 配 initiator 的 `rs`；
    //   2. 时序反了 —— `remote_static_public` 在此刻尚未被 msg2 的密文填充
    //      （msg2 密文装的是 `self.static_public`，即 responder 自己的）。
    // ⇒ `remote_static_public.as_ref()` 必为 `None` ⇒ **恒** `InvalidState`，
    //   握手在第 2 条消息就死，`Completed` 状态永不可达。
    //
    // 为何不就地"顺手修"（crypto，不宜）：
    // ⚠️ **2026-09-28 订正：本段原诊断「es 角色/时序接反」是错的**（R-SCAN-1：
    //    先读现场再下结论）。用官方向量逐字节核对后的实测根因是：
    //      规范 `Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s` 的 msg1 token 序为
    //      `e, es, s, ss`，官方向量 msg_0 长 **96B（实测）**；而 `_create_message1`
    //      （:133）**只写裸 `e`、一次 `MixKey` 都没有** ⇒ 恒 32B。
    //      即**协议实现缺 token**，不是参数或时序错误。
    //    第二处独立缺陷：`encrypt()`（:367）nonce 恒零且不自增，违反规范
    //      `EncryptAndHash`（4 字节零 || LE64(n)，每条 +1）。
    //    官方向量已取到并落盘 `neotrix-core/testdata/noise_vectors_…txt`
    //    ⇒ 「需外部向量」这个阻塞**已解除**，不再是「无解」。
    // - 手写一个"看起来能跑通"但密钥派生次序不对的实现，比红测试**更坏** ——
    //   它会把一个坏掉的握手伪装成可用的加密通道；
    // - 本模块**零生产调用方**（全仓仅 `crypto/mod.rs:14` 的 `pub mod`），
    //   挂起不损失任何在用能力，也没有安全敞口。
    // - 协议名：`Noise_IKpsk2_25519_ChaCha`（25B）**不是合法 Noise 名** —— 规范要求
    //   4 段 `pattern_DH_cipher_hash`，且本仓 HASH 是 BLAKE2s（不是 SHA256），
    //   故正确名为 `Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s`（与向量 suite 一致）。
    //
    // 台账：`docs/architecture/OPEN-TASKS-2026-09-28.md` §5 D-2。
    // 正确形态参考同仓已验证的版本链实现：
    // `l4_emotion/nt_memory/nt_memory_historian/nt_temporal_facts.rs`（每版本独立 id）。
    /// B-2 证据测试：对照 **Noise 官方测试向量** 断言 msg_0 的**长度**。
    ///
    /// 规范 `Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s` 的 msg_1 token 序是
    /// `e, es, s, ss`（`noise_spec/noise.md`）。
    /// 官方向量（`neotrix-core/testdata/noise_vectors_…txt`）msg_0 长度
    /// **实测 96 字节**（192 hex chars）。
    ///
    /// ⚠️ 96 这个数是**量出来的**，不是推出来的 —— 初版注释手推为 97，
    /// 实测 96。手算 token 长度极易错（AEAD tag、token 边界），L10:
    /// **手推 ≠ 实证**。最终的 token 逐段边界应由 §4-A 重写后
    /// **逐字节比对官方向量**确认，而不是靠这里的长度断言。
    ///
    /// 本实现 `_create_message1` 只写裸 `e`（32 字节）且**不做任何 MixKey** ⇒ 恒 32。
    /// 这条断言把「实现缺失 token」变成**可复现的红色证据**，而不是一句
    /// 「疑似时序接反」的推测。
    ///
    /// 本测试**故意保持红色**直到按 `B2-NOISE-IK-RESOLUTION-20260928.md` §4-A
    /// 补齐 msg1 的 `es`/`s`/`ss` 后才转绿。**不得改成"看起来能跑"的宽松断言**
    /// （仓规：禁改松断言绕过）。
    #[test]
    // ⚠️ 2026-10-05 解除 `#[ignore]`：原理由「msg1 缺 es/s/ss token（规范 96B，
    // 本实现 32B）」**前提已失效** —— `_create_message1`（:183-193）现在依次写
    // `e`(32) + `enc_static`(48) + `enc_empty`(16) = 96B，与 :150 的注释一致。
    // TODO.md:961/1004 亦记载「B-2 已按官方向量重写为 2-message IKpsk2」并转绿。
    //
    // ✅ 解除前**实测**：`cargo test --lib noise_handshake -- --ignored`
    //    ⇒ 2 passed。故这不是「改松断言」，而是让一个已修好的 crypto 门
    //    重新参与守门。
    fn msg1_matches_official_vector_length() {
        const OFFICIAL_MSG0_LEN: usize = 96; // 实测自官方向量：192 hex chars（手推 97 是错的）
        let initiator_key = PrivateKey::generate();
        let responder_key = PrivateKey::generate();
        let mut initiator = _NoiseHandshake::_initiator(
            initiator_key,
            responder_key.public(),
            None,
        );
        let msg1 = initiator._create_message1().expect("msg1");
        assert_eq!(
            msg1.len(),
            OFFICIAL_MSG0_LEN,
            "msg1 应为规范 IKpsk2 的 e,es,s,ss 四 token 产物(96B, 实测官方向量)，实际 {}B —— 说明 es/s/ss 未实现",
            msg1.len()
        );
    }

    /// B-2 证据测试之二：`encrypt()` 的 nonce 必须按规范自增
    /// （`EncryptAndHash`：nonce = 4 字节零 || LE64(n)，每条消息 n += 1）。
    ///
    /// 实现用 `Nonce::from_bytes(&[0u8; 12])` **恒零且不自增** ⇒ 同一 key 下
    /// 两条不同明文会复用同一 (key, nonce) 对 —— 在 ChaCha20-Poly1305 上
    /// 这是**严重误用**（nonce 复用可恢复明文关系）。
    ///
    /// 用同状态下两次加密不同明文来暴露：若 nonce 自增，密文前缀（ChaCha20
    /// keystream）必不同；若恒零，密文前缀（同一 keystream）**必然相同**。
    #[test]
    // ⚠️ 2026-10-05 解除 `#[ignore]`：原理由称「`encrypt()` 的 nonce 恒为全零
    // 且不自增」，但该模块**根本不存在 `fn encrypt`**（旧 :367 现已是
    // `ok_or(_NoiseError::InvalidState)?`）；实际调用的是 `encrypt_and_hash`
    // （:450），其中 `self.nonce = self.nonce.saturating_add(1)`，且
    // `current_nonce()`（:436-440）编码的是计数器而非常量零。
    //
    // 这条比上一条更要紧：活动测试 `full_handshake_matches_official_vectors`
    // 在传输轮用 `seal(&zero12, …)`（:676）**绕过** `encrypt_and_hash`
    // ⇒ 「nonce 必须自增」此前**没有任何活动测试覆盖**，
    //    只靠这个被挂起的门 ⇒ nonce 复用回归会静默通过。
    // ✅ 解除前实测：`--ignored` ⇒ passed。
    fn encrypt_nonce_must_not_be_reused() {
        // 直接构造一个 symmetric_key 已就绪的状态（绕开未完成的握手），
        // 以便单独检验 encrypt 的 nonce 行为。
        let initiator_key = PrivateKey::generate();
        let responder_key = PrivateKey::generate();
        let mut h = _NoiseHandshake::_initiator(initiator_key, responder_key.public(), None);
        h._create_message1().expect("msg1");
        // 手动置一个非零 symmetric_key，模拟 MixKey 已执行
        h.symmetric_key = [7u8; 32];

        // 同一明文加密两次：nonce 若复用 ⇒ ChaCha20 keystream 相同 ⇒ **密文逐字节相同**
        // （这正是 nonce 复用的可观测后果）；nonce 若自增 ⇒ 密文必不同。
        // ⚠️ 不能用『不同明文比首字节』来判：不同明文即使 keystream 相同，
        //    p[0] XOR k 也不同 —— 那种写法恒通过，等于没测（本轮已踩过一次）。
        let a = h.encrypt_and_hash(b"same-plaintext").expect("encrypt a");
        let b = h.encrypt_and_hash(b"same-plaintext").expect("encrypt b");
        assert_ne!(
            a, b,
            "同一明文两次加密得到完全相同的密文 ⇒ nonce 复用（keystream 复用），违反 Noise 规范 EncryptAndHash"
        );
    }

    /// 官方向量逐字节回归（`Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s`）。
    /// 覆盖：msg0(96B) → msg1(48B) → Split → 传输轮 msg2/msg3(各 31B)。
    /// 这是本模块**唯一**的验收闸门，任何协议改动都必须让它转绿。
    #[test]
    fn full_handshake_matches_official_vectors() {
        fn hx(s: &str) -> Vec<u8> {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
                .collect()
        }
        fn bx32(v: &[u8]) -> [u8; 32] {
            v.try_into().expect("32 bytes")
        }

        // ---- 官方向量输入 ----
        let init_static = PrivateKey::from_bytes(&bx32(&hx(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        )));
        let resp_static = PrivateKey::from_bytes(&bx32(&hx(
            "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        )));
        let psk = bx32(&hx(
            "2176657279736563726574766572797365637265747665727973656372657421",
        ));
        let eph_i = PrivateKey::from_bytes(&bx32(&hx(
            "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
        )));
        let eph_r = PrivateKey::from_bytes(&bx32(&hx(
            "4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60",
        )));
        let exp_msg0 = hx("358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254d06f15f78ad0914d9715147bb5a5004b27345a838bab4aa8bc5f144afc2cf4cca972105ba526e8c92b759e028200e766f827aa12a04ecbc0bdcd9e574e007945");
        let exp_msg1 = hx("64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484668c46d966ca4fe339f9e47fd25f68de8a");
        let exp_msg2 = hx("6013ea114b4c4884afb82bf029f72f924bd8a32c487a15a1cef4855ba234be");
        let exp_msg3 = hx("8a2e7119635e41a35b7e64e0adac5483b66b1a9827895124ea07d58440b654");

        // ---- 握手 ----
        let mut initiator =
            _NoiseHandshake::_initiator(init_static, resp_static.public(), Some(psk));
        initiator.ephemeral_private = Some(eph_i);
        let mut responder = _NoiseHandshake::_responder(resp_static, Some(psk));

        // msg1（initiator →）：必须逐字节等于官方向量 msg_0
        let msg1 = initiator._create_message1().expect("msg1");
        assert_eq!(
            msg1, exp_msg0,
            "msg1 与官方向量 msg_0 不一致（initiator 侧 e,es,s,ss 仍有偏差）"
        );
        assert_eq!(initiator.state(), &_HandshakeState::Message1Sent);

        // responder 读 msg1
        responder._consume_message1(&msg1).expect("consume msg1");
        assert_eq!(responder.state(), &_HandshakeState::Message1Received);

        // msg2（responder →）：必须逐字节等于官方向量 msg_1
        responder.ephemeral_private = Some(eph_r);
        let (msg2, res_r) = responder._create_message2().expect("msg2");
        assert_eq!(
            msg2, exp_msg1,
            "msg2 与官方向量 msg_1 不一致（responder e,ee,se,psk 仍有偏差）"
        );

        // initiator 读 msg2 → Split
        let res_i = initiator._consume_message2(&msg2).expect("consume msg2");
        assert_eq!(initiator.state(), &_HandshakeState::Completed);

        // ---- 传输轮（Split 正确性 + 对称性）----
        let zero12 = Nonce::from_bytes(&[0u8; 12]);
        // initiator.send(== c1) 加密 "hellosubmarine"（向量 msg_2 payload）
        let ct2 = res_i.send_key.seal(&zero12, b"yellowsubmarine");
        assert_eq!(
            ct2, exp_msg2,
            "传输轮 initiator→responder 与官方向量 msg_2 不一致（Split c1 错误）"
        );
        // responder.send(== c2) 加密 "submarineyellow"（向量 msg_3 payload）
        let ct3 = res_r.send_key.seal(&zero12, b"submarineyellow");
        assert_eq!(
            ct3, exp_msg3,
            "传输轮 responder→initiator 与官方向量 msg_3 不一致（Split c2 错误）"
        );
        // 交叉解密往返：证明 send/recv 配对正确（对称性）
        let mut b2 = ct2.clone();
        let pt2 = res_r
            .recv_key
            .open(&zero12, &mut b2)
            .expect("responder recv 解密");
        assert_eq!(pt2, b"yellowsubmarine");
        let mut b3 = ct3.clone();
        let pt3 = res_i
            .recv_key
            .open(&zero12, &mut b3)
            .expect("initiator recv 解密");
        assert_eq!(pt3, b"submarineyellow");
    }
}
