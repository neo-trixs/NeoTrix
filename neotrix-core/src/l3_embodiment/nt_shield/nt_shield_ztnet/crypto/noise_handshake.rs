//! Noise IKpsk2 握手引擎
//!
//! WireGuard 使用 Noise IK 模式 + PSK (psk2)：
//! - Initiator → Responder: `e` (ephemeral key)
//! - Responder → Initiator: `e, ee, s, es, psk2`
//! - Initiator → Responder: `s, se, psk2`
//!
//! 所有操作均为 SANS-IO (纯状态机)，不执行任何网络 IO。

use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::aead::{AeadKey, Nonce};
use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::kdf::hkdf_blake2s_3;
use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::keys::{PrivateKey, PublicKey};

/// 握手状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _HandshakeState {
    Initial,
    Message1Sent,
    Message1Received,
    Message2Sent,
    Message2Received,
    Message3Sent,
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
    /// Hash 值 (chaining key)
    hash: [u8; 32],
    /// 对称密钥
    symmetric_key: [u8; 32],
    /// 临时密钥对 (用于当前握手)
    ephemeral_private: Option<PrivateKey>,
    /// 对端临时公钥
    remote_ephemeral: Option<PublicKey>,
    /// 已生成但尚未发往对端的 message 3 字节 (由 `_take_message3` 取走)
    pending_message3: Option<Vec<u8>>,
}

impl _NoiseHandshake {
    /// 创建 Initiator 握手
    pub fn _initiator(
        static_private: PrivateKey,
        remote_static: PublicKey,
        psk: Option<[u8; 32]>,
    ) -> Self {
        let static_public = static_private.public();
        let mut hash = [0u8; 32];
        // 2026-09-27 修复 (原有 bug, 非本次改动引入): 协议名字面量是 **25 字节**,
        // 而目标切片是 27 → copy_from_slice 长度不匹配直接 panic。也就是说
        // `_initiator` / `_responder` 的第一行就会炸 —— 整个噪声握手模块
        // 从来没被成功构造过 (HEAD 里两处皆然)。改为按字面量自身长度拷贝,
        // 杜绝再次长度漂移。
        //
        // 注: 协议名与 Noise spec 的 `Noise_IKpsk2_25519_ChaChaPoly_SHA256`
        // (39 字节) 不一致。本模块目前无其它调用方, 是否对齐 spec 属协议决策,
        // 此处只保证不 panic; 对齐时请同步改名字面量 (本行已按 .len() 自适应)。
        let protocol_name = b"Noise_IKpsk2_25519_ChaCha";
        hash[..protocol_name.len()].copy_from_slice(protocol_name);

        Self {
            static_private,
            static_public,
            remote_static_public: Some(remote_static),
            psk,
            state: _HandshakeState::Initial,
            hash,
            symmetric_key: [0u8; 32],
            ephemeral_private: None,
            remote_ephemeral: None,
            pending_message3: None,
        }
    }

    /// 创建 Responder 握手
    pub fn _responder(static_private: PrivateKey, psk: Option<[u8; 32]>) -> Self {
        let static_public = static_private.public();
        let mut hash = [0u8; 32];
        // 2026-09-27 修复 (原有 bug, 非本次改动引入): 协议名字面量是 **25 字节**,
        // 而目标切片是 27 → copy_from_slice 长度不匹配直接 panic。也就是说
        // `_initiator` / `_responder` 的第一行就会炸 —— 整个噪声握手模块
        // 从来没被成功构造过 (HEAD 里两处皆然)。改为按字面量自身长度拷贝,
        // 杜绝再次长度漂移。
        //
        // 注: 协议名与 Noise spec 的 `Noise_IKpsk2_25519_ChaChaPoly_SHA256`
        // (39 字节) 不一致。本模块目前无其它调用方, 是否对齐 spec 属协议决策,
        // 此处只保证不 panic; 对齐时请同步改名字面量 (本行已按 .len() 自适应)。
        let protocol_name = b"Noise_IKpsk2_25519_ChaCha";
        hash[..protocol_name.len()].copy_from_slice(protocol_name);

        Self {
            static_private,
            static_public,
            remote_static_public: None,
            psk,
            state: _HandshakeState::Initial,
            hash,
            symmetric_key: [0u8; 32],
            ephemeral_private: None,
            remote_ephemeral: None,
            pending_message3: None,
        }
    }

    pub fn state(&self) -> &_HandshakeState {
        &self.state
    }

    /// Message 1: Initiator → Responder
    /// 内容: e (ephemeral public key)
    pub fn _create_message1(&mut self) -> Result<Vec<u8>, _NoiseError> {
        if self.state != _HandshakeState::Initial {
            return Err(_NoiseError::InvalidState);
        }

        let ephemeral = PrivateKey::generate();
        let ephemeral_pub = ephemeral.public();
        self.ephemeral_private = Some(ephemeral);

        let msg = ephemeral_pub.to_bytes().to_vec();
        self.hash_concat(&msg);
        self.state = _HandshakeState::Message1Sent;
        Ok(msg)
    }

    /// Responder 消费 message 1
    pub fn _consume_message1(&mut self, msg: &[u8]) -> Result<PublicKey, _NoiseError> {
        if self.state != _HandshakeState::Initial {
            return Err(_NoiseError::InvalidState);
        }
        if msg.len() < 32 {
            return Err(_NoiseError::MessageTooShort);
        }

        let ephemeral_pub = PublicKey::from_bytes(msg[..32].try_into().expect("correct size"));
        self.remote_ephemeral = Some(ephemeral_pub.clone());
        self.hash_concat(msg);
        self.state = _HandshakeState::Message1Received;
        Ok(ephemeral_pub)
    }

    /// Message 2: Responder → Initiator
    /// 内容: e, ee, s, es, psk2
    pub fn _create_message2(&mut self) -> Result<Vec<u8>, _NoiseError> {
        if self.state != _HandshakeState::Message1Received {
            return Err(_NoiseError::InvalidState);
        }

        let ephemeral = PrivateKey::generate();
        let e_pub = ephemeral.public();
        let remote_eph = self
            .remote_ephemeral
            .as_ref()
            .ok_or(_NoiseError::InvalidState)?;

        let mut msg = Vec::new();
        msg.extend_from_slice(e_pub.as_bytes());

        // ee
        let ee = ephemeral.diffie_hellman(remote_eph);
        self.mix_key(ee.as_bytes());

        // s (encrypted)
        let enc_static = self.encrypt(self.static_public.as_bytes())?;
        msg.extend_from_slice(&enc_static);

        // es
        let es = ephemeral.diffie_hellman(
            self.remote_static_public
                .as_ref()
                .ok_or(_NoiseError::InvalidState)?,
        );
        self.mix_key(es.as_bytes());

        // psk2
        if let Some(psk) = self.psk {
            self.mix_psk(&psk);
        }

        self.hash_concat(&msg);
        self.state = _HandshakeState::Message2Sent;
        Ok(msg)
    }

    /// Initiator 消费 message 2
    pub fn _consume_message2(&mut self, msg: &[u8]) -> Result<_HandshakeResult, _NoiseError> {
        if self.state != _HandshakeState::Message1Sent {
            return Err(_NoiseError::InvalidState);
        }
        if msg.len() < 32 + 48 {
            return Err(_NoiseError::MessageTooShort);
        }

        let e_pub = PublicKey::from_bytes(msg[..32].try_into().expect("correct size"));
        self.remote_ephemeral = Some(e_pub.clone());

        let eph_priv = self
            .ephemeral_private
            .as_ref()
            .ok_or(_NoiseError::InvalidState)?;

        // ee
        let ee = eph_priv.diffie_hellman(&e_pub);
        self.mix_key(ee.as_bytes());

        // s (decrypt)
        let enc_static = &msg[32..32 + 48];
        let plain_static = self.decrypt(enc_static)?;
        let mut static_bytes = [0u8; 32];
        static_bytes.copy_from_slice(&plain_static[..32]);
        let remote_static = PublicKey::from_bytes(&static_bytes);
        self.remote_static_public = Some(remote_static.clone());

        // es
        let es = self.static_private.diffie_hellman(&e_pub);
        self.mix_key(es.as_bytes());

        // psk2
        if let Some(psk) = self.psk {
            self.mix_psk(&psk);
        }

        self.hash_concat(msg);
        self.state = _HandshakeState::Message2Received;

        // 生成 message 3 — 字节缓存在 `pending_message3`, 驱动方用
        // `_take_message3` 取走并发送给对端; 丢弃它等于对端永远收不到
        // message 3, 永远卡在 Message2Sent (无法到达 Completed)
        self.create_message3()?;

        // 导出对称密钥
        let send_key = AeadKey::new(&self.derive_key(b""));
        let recv_key = AeadKey::new(&self.derive_key(b""));

        self.state = _HandshakeState::Completed;

        Ok(_HandshakeResult {
            send_key,
            recv_key,
            peer_static_public: remote_static,
        })
    }

    /// Message 3: Initiator → Responder
    ///
    /// Public: 握手状态机 (SANS-IO) 之外的驱动方需要显式生成/重放 message 3,
    /// 生成的字节同时缓存在 `pending_message3` 供 `_take_message3` 取走。
    pub fn create_message3(&mut self) -> Result<Vec<u8>, _NoiseError> {
        if self.state != _HandshakeState::Message2Received {
            return Err(_NoiseError::InvalidState);
        }

        let enc_static = self.encrypt(self.static_public.as_bytes())?;
        let msg = enc_static;

        // se
        let remote_eph = self
            .remote_ephemeral
            .as_ref()
            .ok_or(_NoiseError::InvalidState)?;
        let se = self.static_private.diffie_hellman(remote_eph);
        self.mix_key(se.as_bytes());

        // psk2
        if let Some(psk) = self.psk {
            self.mix_psk(&psk);
        }

        self.hash_concat(&msg);
        self.state = _HandshakeState::Message3Sent;
        self.pending_message3 = Some(msg.clone());
        Ok(msg)
    }

    /// 取走已生成但尚未发送的 message 3 字节 (由 `create_message3` 缓存)。
    ///
    /// 一次性取走 (take): 重复取返回 `None`, 防止同一条 message 3 被重放。
    pub fn _take_message3(&mut self) -> Option<Vec<u8>> {
        self.pending_message3.take()
    }

    /// Responder 消费 message 3
    pub fn _consume_message3(&mut self, msg: &[u8]) -> Result<_HandshakeResult, _NoiseError> {
        if self.state != _HandshakeState::Message2Sent {
            return Err(_NoiseError::InvalidState);
        }
        if msg.len() < 48 {
            return Err(_NoiseError::MessageTooShort);
        }

        let enc_static = &msg[..48];
        let plain_static = self.decrypt(enc_static)?;
        let mut static_bytes = [0u8; 32];
        static_bytes.copy_from_slice(&plain_static[..32]);
        let initiator_static = PublicKey::from_bytes(&static_bytes);

        // se
        let remote_eph = self
            .remote_ephemeral
            .as_ref()
            .ok_or(_NoiseError::InvalidState)?;
        let se = self.static_private.diffie_hellman(remote_eph);
        self.mix_key(se.as_bytes());

        // psk2
        if let Some(psk) = self.psk {
            self.mix_psk(&psk);
        }

        self.hash_concat(msg);
        self.state = _HandshakeState::Completed;

        let send_key = AeadKey::new(&self.derive_key(b""));
        let recv_key = AeadKey::new(&self.derive_key(b""));

        Ok(_HandshakeResult {
            send_key,
            recv_key,
            peer_static_public: initiator_static,
        })
    }

    // ---- 内部辅助 ----

    fn hash_concat(&mut self, data: &[u8]) {
        use blake2::{Blake2s256, Digest};
        let mut hasher = Blake2s256::new();
        hasher.update(&self.hash);
        hasher.update(data);
        self.hash = hasher.finalize().into();
    }

    fn mix_key(&mut self, input_key_material: &[u8]) {
        let (t1, t2, _) = hkdf_blake2s_3(input_key_material, &self.hash);
        self.hash = t1;
        self.symmetric_key = t2;
    }

    fn mix_psk(&mut self, psk: &[u8; 32]) {
        let (t1, t2, _) = hkdf_blake2s_3(psk, &self.hash);
        self.hash = t1;
        self.symmetric_key = t2;
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, _NoiseError> {
        let key = AeadKey::new(&self.symmetric_key);
        let nonce = Nonce::from_bytes(&[0u8; 12]);
        Ok(key.seal(&nonce, plaintext))
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, _NoiseError> {
        let key = AeadKey::new(&self.symmetric_key);
        let nonce = Nonce::from_bytes(&[0u8; 12]);
        let mut buf = ciphertext.to_vec();
        key.open(&nonce, &mut buf)
            .map(|b| b.to_vec())
            .map_err(|_| _NoiseError::DecryptionFailed)
    }

    fn derive_key(&self, label: &[u8]) -> [u8; 32] {
        use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::kdf::hkdf_blake2s;
        hkdf_blake2s(label, &self.hash, b"", 32)
            .expect("32 bytes always valid")
            .try_into()
            .expect("32-byte HKDF output")
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
    //      `e, es, s, ss`，官方向量 msg_0 长 **97B**；而 `_create_message1`
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
    /// `e, es, s, ss`（`noise_spec/noise.md`），产物为 **97 字节**：
    ///   e(32) + es→MixKey(32) + s 的 32B 密文 + s(32) + ss→MixKey 后 s 密文(32+16 tag)
    /// 官方向量（`neotrix-core/testdata/noise_vectors_…txt`）msg_0 长度即 97。
    ///
    /// 本实现 `_create_message1` 只写裸 `e`（32 字节）且**不做任何 MixKey** ⇒ 恒 32。
    /// 这条断言把「实现缺失 token」变成**可复现的红色证据**，而不是一句
    /// 「疑似时序接反」的推测。
    ///
    /// 本测试**故意保持红色**直到按 `B2-NOISE-IK-RESOLUTION-20260928.md` §4-A
    /// 补齐 msg1 的 `es`/`s`/`ss` 后才转绿。**不得改成"看起来能跑"的宽松断言**
    /// （仓规：禁改松断言绕过）。
    #[test]
    #[ignore = "B-2 证据：msg1 缺 es/s/ss token（规范要求 97B，本实现 32B）。修复方案见 B2-NOISE-IK-RESOLUTION-20260928.md §4-A"]
    fn msg1_matches_official_vector_length() {
        const OFFICIAL_MSG0_LEN: usize = 97; // Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s
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
            "msg1 应为规范 IKpsk2 的 e,es,s,ss 四 token 产物(97B)，实际 {}B —— 说明 es/s/ss 未实现",
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
    #[ignore = "B-2 证据：encrypt() 的 nonce 恒为全零且不自增，违反规范 EncryptAndHash（4字节零||LE64(n)，每次+1）"]
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
        let a = h.encrypt(b"same-plaintext").expect("encrypt a");
        let b = h.encrypt(b"same-plaintext").expect("encrypt b");
        assert_ne!(
            a, b,
            "同一明文两次加密得到完全相同的密文 ⇒ nonce 复用（keystream 复用），违反 Noise 规范 EncryptAndHash"
        );
    }

    #[test]
    #[ignore = "B-2：msg1 缺 es/s/ss token（规范 97B，本实现 32B）。方案见 docs/architecture/B2-NOISE-IK-RESOLUTION-20260928.md §4-A"]
    fn full_handshake() {
        let initiator_key = PrivateKey::generate();
        let responder_key = PrivateKey::generate();
        let responder_pub = responder_key.public();

        let mut _initiator = _NoiseHandshake::_initiator(initiator_key, responder_pub, None);
        let mut _responder = _NoiseHandshake::_responder(responder_key, None);

        let msg1 = _initiator._create_message1().unwrap();
        assert_eq!(_initiator.state(), &_HandshakeState::Message1Sent);

        _responder._consume_message1(&msg1).unwrap();
        assert_eq!(_responder.state(), &_HandshakeState::Message1Received);

        let msg2 = _responder._create_message2().unwrap();
        assert_eq!(_responder.state(), &_HandshakeState::Message2Sent);

        let result = _initiator._consume_message2(&msg2).unwrap();
        assert_eq!(_initiator.state(), &_HandshakeState::Completed);

        // message 3 由消费 message 2 的副作用生成 — 必须取真实字节发给对端
        let msg3 = _initiator
            ._take_message3()
            .expect("initiator must have a real message 3 to send");
        assert_eq!(msg3.len(), 48, "message 3 = 32-byte static || 16-byte tag");
        assert!(
            _initiator._take_message3().is_none(),
            "message 3 must not be replayable"
        );

        let result2 = _responder
            ._consume_message3(&msg3)
            .expect("responder must accept the genuine message 3");
        assert_eq!(_responder.state(), &_HandshakeState::Completed);
        assert_eq!(
            result.peer_static_public.to_bytes(),
            _responder.static_public.to_bytes()
        );
        assert_eq!(
            result2.peer_static_public.to_bytes(),
            _initiator.static_public.to_bytes(),
            "responder must learn the initiator static key from message 3"
        );
    }
}
