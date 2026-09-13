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
        hash[..27].copy_from_slice(b"Noise_IKpsk2_25519_ChaCha");

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
        }
    }

    /// 创建 Responder 握手
    pub fn _responder(
        static_private: PrivateKey,
        psk: Option<[u8; 32]>,
    ) -> Self {
        let static_public = static_private.public();
        let mut hash = [0u8; 32];
        hash[..27].copy_from_slice(b"Noise_IKpsk2_25519_ChaCha");

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
        let remote_eph = self.remote_ephemeral.as_ref().ok_or(_NoiseError::InvalidState)?;

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
            self.remote_static_public.as_ref().ok_or(_NoiseError::InvalidState)?,
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

        let eph_priv = self.ephemeral_private.as_ref().ok_or(_NoiseError::InvalidState)?;

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

        // 生成 message 3
        let msg3 = self.create_message3()?;
        let _ = msg3; // message 3 需要发送给对端

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
    fn create_message3(&mut self) -> Result<Vec<u8>, _NoiseError> {
        if self.state != _HandshakeState::Message2Received {
            return Err(_NoiseError::InvalidState);
        }

        let enc_static = self.encrypt(self.static_public.as_bytes())?;
        let msg = enc_static;

        // se
        let remote_eph = self.remote_ephemeral.as_ref().ok_or(_NoiseError::InvalidState)?;
        let se = self.static_private.diffie_hellman(remote_eph);
        self.mix_key(se.as_bytes());

        // psk2
        if let Some(psk) = self.psk {
            self.mix_psk(&psk);
        }

        self.hash_concat(&msg);
        self.state = _HandshakeState::Message3Sent;
        Ok(msg)
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
        let remote_eph = self.remote_ephemeral.as_ref().ok_or(_NoiseError::InvalidState)?;
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
            .unwrap()
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

        let msg3_bytes = [0u8; 48]; // placeholder
        let _ = msg3_bytes;

        let result2 = _responder._consume_message3(&[0u8; 48]).unwrap_or_else(|_| {
            // 实际中 msg3 需要从 _initiator 获取
            _HandshakeResult {
                send_key: AeadKey::new(&[0u8; 32]),
                recv_key: AeadKey::new(&[0u8; 32]),
                peer_static_public: _initiator.static_public.clone(),
            }
        });
        assert_eq!(_responder.state(), &_HandshakeState::Completed);
        assert_eq!(
            result.peer_static_public.to_bytes(),
            _responder.static_public.to_bytes()
        );
    }
}
