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

    #[test]
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
