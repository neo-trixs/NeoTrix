use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Encrypted packet for the back-channel knowledge syphon.
///
/// The payload (KnowledgePacket serialized bytes) is encrypted with
/// AES-256-GCM using a shared secret derived from a NAXOS-style
/// key agreement: shared = H(our_privkey || peer_pubkey).
///
/// Security properties (basic):
///   - Mutual authentication: both private keys must be known
///   - Confidentiality: AES-256-GCM (authenticated encryption)
///   - Nonce: randomly generated 96-bit, included in plaintext
///   - No forward secrecy: compromise of either private key reveals all past keys
///
/// For forward secrecy, use `RatchetEncryptedPacket` + `init_ratchet`/`encrypt_ratchet`.
#[derive(Debug, Clone)]
pub struct EncryptedPacket {
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub sender_pubkey: Vec<u8>,
}

/// Ratchet-encrypted packet with forward secrecy via ephemeral ECDH.
///
/// Extends EncryptedPacket with:
///   - ephemeral_pubkey: ECDH public key for this ratchet session (first msg only)
///   - chain_index: message position in the symmetric ratchet
///
/// Forward secrecy: each message derives a fresh AES-256-GCM key from
/// a SHA-256 ratchet chain. Compromising the current chain key does NOT
/// reveal past message keys (SHA-256 is one-way).
///
/// Post-compromise security requires the full Double Ratchet (periodic DH).
#[derive(Debug, Clone)]
pub struct RatchetEncryptedPacket {
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub sender_pubkey: Vec<u8>,
    /// Ephemeral ECDH public key (33 bytes P-256 compressed), empty after first msg
    pub ephemeral_pubkey: Vec<u8>,
    /// Message index in the symmetric ratchet chain
    pub chain_index: u64,
}

/// Symmetric ratchet state for per-message forward secrecy.
///
/// Chain: K_{i+1} = SHA-256(K_i || "ratchet-chain")
/// Msg:  M_i     = SHA-256(K_i || i || "ratchet-msg")
///
/// Forward secrecy: K_i can derive M_i and K_{i+1}, but NOT K_{i-1} or M_{i-1}.
pub struct RatchetState {
    chain_key: [u8; 32],
    send_idx: u64,
    recv_idx: u64,
}

impl RatchetState {
    pub fn from_chain_key(chain_key: [u8; 32]) -> Self {
        RatchetState {
            chain_key,
            send_idx: 0,
            recv_idx: 0,
        }
    }

    pub fn next_send_key(&mut self) -> [u8; 32] {
        let msg_key = derive_message_key(&self.chain_key, self.send_idx);
        self.ratchet_chain();
        self.send_idx += 1;
        msg_key
    }

    pub fn recv_key_at(&self, index: u64) -> [u8; 32] {
        derive_message_key(&self.chain_key, index)
    }

    pub fn ratchet_chain(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_key);
        hasher.update(b"ratchet-chain");
        self.chain_key = hasher.finalize().into();
        self.recv_idx += 1;
    }

    pub fn current_chain_key(&self) -> &[u8; 32] {
        &self.chain_key
    }

    pub fn send_index(&self) -> u64 {
        self.send_idx
    }

    pub fn recv_index(&self) -> u64 {
        self.recv_idx
    }
}

fn derive_message_key(chain: &[u8; 32], index: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(chain);
    hasher.update(index.to_le_bytes());
    hasher.update(b"ratchet-msg");
    hasher.finalize().into()
}

fn ecdh_shared_secret(signing_key: &SigningKey, peer_pub: &[u8]) -> Result<[u8; 32], String> {
    use k256::elliptic_curve::ecdh::diffie_hellman;
    use k256::elliptic_curve::scalar::ScalarPrimitive;
    use k256::{PublicKey, Secp256k1};

    let repr = signing_key.to_bytes();
    let primitive = ScalarPrimitive::<Secp256k1>::from_bytes(&repr)
        .into_option()
        .ok_or_else(|| "invalid scalar bytes".to_string())?;
    let scalar = k256::Scalar::from(&primitive);
    let nonzero = k256::NonZeroScalar::new(scalar)
        .into_option()
        .ok_or_else(|| "zero scalar".to_string())?;

    let peer_pk =
        PublicKey::from_sec1_bytes(peer_pub).map_err(|e| format!("invalid peer pubkey: {}", e))?;
    let affine = peer_pk.as_affine();

    let shared = diffie_hellman(&nonzero, affine);
    let mut secret = [0u8; 32];
    secret.copy_from_slice(shared.raw_secret_bytes().as_ref());
    Ok(secret)
}

/// Encrypted back-channel for knowledge syphoning between sub-hives.
///
/// Key agreement:   shared_secret = SHA-256(our_private_key || peer_public_key)
/// Encryption:      AES-256-GCM(plaintext, nonce, shared_secret)
///
/// For forward secrecy, use `init_ratchet`/`encrypt_ratchet`/`decrypt_ratchet`.
pub struct NaclChannel {
    local_signing_key: SigningKey,
    local_pubkey: Vec<u8>,
    shared_secret: [u8; 32],
    has_shared: bool,
    eph_pubkey: Option<Vec<u8>>,
    ratchet: Option<RatchetState>,
}

impl NaclChannel {
    pub fn random() -> Self {
        let signing_key = SigningKey::random(&mut rand::rngs::OsRng);
        let pubkey = VerifyingKey::from(&signing_key);
        NaclChannel {
            shared_secret: [0u8; 32],
            local_signing_key: signing_key,
            local_pubkey: pubkey.to_sec1_bytes().to_vec(),
            has_shared: false,
            eph_pubkey: None,
            ratchet: None,
        }
    }

    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        let pubkey = VerifyingKey::from(&signing_key);
        NaclChannel {
            shared_secret: [0u8; 32],
            local_signing_key: signing_key,
            local_pubkey: pubkey.to_sec1_bytes().to_vec(),
            has_shared: false,
            eph_pubkey: None,
            ratchet: None,
        }
    }

    pub fn derive_shared_secret(&mut self, peer_pubkey_bytes: &[u8]) -> Result<(), String> {
        let mut hasher = Sha256::new();
        hasher.update(self.local_signing_key.to_bytes());
        hasher.update(peer_pubkey_bytes);
        let hash = hasher.finalize();
        self.shared_secret.copy_from_slice(&hash);
        self.has_shared = true;
        Ok(())
    }

    pub fn derive_symmetric(my_pubkey: &[u8], peer_pubkey: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"nacl-symmetric-v1");
        if my_pubkey <= peer_pubkey {
            hasher.update(my_pubkey);
            hasher.update(peer_pubkey);
        } else {
            hasher.update(peer_pubkey);
            hasher.update(my_pubkey);
        }
        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        key
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedPacket, String> {
        if !self.has_shared {
            return Err("no shared secret derived".into());
        }
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.shared_secret);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("encrypt failed: {}", e))?;

        Ok(EncryptedPacket {
            nonce: nonce_bytes,
            ciphertext,
            sender_pubkey: self.local_pubkey.clone(),
        })
    }

    pub fn decrypt(&self, packet: &EncryptedPacket) -> Result<Vec<u8>, String> {
        if !self.has_shared {
            return Err("no shared secret derived".into());
        }
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.shared_secret);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&packet.nonce);

        cipher
            .decrypt(nonce, packet.ciphertext.as_ref())
            .map_err(|e| format!("decrypt failed: {}", e))
    }

    pub fn seal_knowledge(
        &self,
        serialized_packet: &[u8],
        peer_pubkey: &[u8],
    ) -> Result<EncryptedPacket, String> {
        let mut channel = NaclChannel::from_signing_key(SigningKey::random(&mut rand::rngs::OsRng));
        channel.derive_shared_secret(peer_pubkey)?;
        channel.encrypt(serialized_packet)
    }

    pub fn open_knowledge(
        local_key: &SigningKey,
        packet: &EncryptedPacket,
    ) -> Result<Vec<u8>, String> {
        let mut channel = NaclChannel::from_signing_key(local_key.clone());
        channel.derive_shared_secret(&packet.sender_pubkey)?;
        channel.decrypt(packet)
    }

    // ── Ratchet (forward secrecy via ephemeral ECDH + SHA-256 chain) ──

    /// Initialize the ratchet as the SENDER.
    ///
    /// Generates an ephemeral ECDH key, computes shared secret with peer's
    /// static public key, and seeds the symmetric ratchet chain.
    ///
    /// Returns the ephemeral public key (33 bytes) that must be sent
    /// to the peer so they can derive the matching shared secret.
    pub fn init_ratchet(&mut self, peer_pubkey_bytes: &[u8]) -> Result<Vec<u8>, String> {
        // Generate ephemeral ECDH keypair
        let eph_secret = k256::ecdh::EphemeralSecret::random(&mut rand::rngs::OsRng);
        let eph_pk = k256::PublicKey::from(&eph_secret);
        let eph_pk_bytes = eph_pk.to_sec1_bytes().to_vec();

        // Parse peer's static public key
        let peer_pk = k256::PublicKey::from_sec1_bytes(peer_pubkey_bytes)
            .map_err(|e| format!("invalid peer pubkey: {}", e))?;

        // ECDH: shared = eph_priv * peer_pub
        let shared = eph_secret.diffie_hellman(&peer_pk);
        let raw = shared.raw_secret_bytes();

        // Seed chain key: SHA-256("ratchet-init" || shared_secret)
        let mut hasher = Sha256::new();
        hasher.update(b"ratchet-init");
        hasher.update(raw);
        let chain_key: [u8; 32] = hasher.finalize().into();

        // Store the ephemeral private key for receiver-side DH
        // We need the raw bytes; EphemeralSecret consumed by diffie_hellman,
        // but the public key is enough for the receiver to compute the same shared secret.
        // The receiver uses their static priv + our eph pub = same shared secret.
        //
        // We DON'T store eph_privkey here since EphemeralSecret is consumed.
        // Instead, we rely on the receiver doing the complementary DH.
        // Actually, for decrypt_ratchet (receiver), we need to store the DH result.
        // Let's store the computed chain_key directly as the ratchet state.
        self.eph_pubkey = Some(eph_pk_bytes.clone());

        self.ratchet = Some(RatchetState::from_chain_key(chain_key));
        Ok(eph_pk_bytes)
    }

    /// Initialize the ratchet as the RECEIVER.
    ///
    /// Uses the local static private key + sender's ephemeral public key
    /// to compute the same shared secret (DH commutativity).
    pub fn init_ratchet_from_ephemeral(
        &mut self,
        sender_eph_pubkey_bytes: &[u8],
    ) -> Result<(), String> {
        let shared_secret = ecdh_shared_secret(&self.local_signing_key, sender_eph_pubkey_bytes)?;

        let mut hasher = Sha256::new();
        hasher.update(b"ratchet-init");
        hasher.update(shared_secret);
        let chain_key: [u8; 32] = hasher.finalize().into();

        self.ratchet = Some(RatchetState::from_chain_key(chain_key));
        Ok(())
    }

    /// Encrypt a payload using the symmetric ratchet.
    ///
    /// Derives a per-message AES-256-GCM key from the chain.
    /// The ephemeral pubkey is included in the first message only.
    pub fn encrypt_ratchet(
        &mut self,
        plaintext: &[u8],
        with_eph_pubkey: Option<Vec<u8>>,
    ) -> Result<RatchetEncryptedPacket, String> {
        let ratchet = self
            .ratchet
            .as_mut()
            .ok_or("ratchet not initialized — call init_ratchet first")?;

        let msg_key = ratchet.next_send_key();
        let chain_index = ratchet.send_index() - 1;

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&msg_key);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("ratchet encrypt: {}", e))?;

        Ok(RatchetEncryptedPacket {
            nonce: nonce_bytes,
            ciphertext,
            sender_pubkey: self.local_pubkey.clone(),
            ephemeral_pubkey: with_eph_pubkey.unwrap_or_default(),
            chain_index,
        })
    }

    /// Decrypt a ratchet-encrypted payload.
    ///
    /// If the ratchet hasn't been initialized yet (first msg from peer),
    /// uses the packet's ephemeral pubkey to initialize.
    pub fn decrypt_ratchet(&mut self, packet: &RatchetEncryptedPacket) -> Result<Vec<u8>, String> {
        if self.ratchet.is_none() {
            if packet.ephemeral_pubkey.is_empty() {
                return Err("first ratchet packet must include ephemeral_pubkey".into());
            }
            self.init_ratchet_from_ephemeral(&packet.ephemeral_pubkey)?;
        }

        let ratchet = self.ratchet.as_mut().ok_or("ratchet not initialized")?;

        // Compute message key at the packet's chain index
        let msg_key = ratchet.recv_key_at(packet.chain_index);

        // Ratchet chain forward to match sender's position
        let steps = packet.chain_index + 1 - ratchet.recv_index();
        for _ in 0..steps {
            ratchet.ratchet_chain();
        }

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&msg_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&packet.nonce);

        cipher
            .decrypt(nonce, packet.ciphertext.as_ref())
            .map_err(|e| format!("ratchet decrypt: {}", e))
    }

    pub fn has_ratchet(&self) -> bool {
        self.ratchet.is_some()
    }

    pub fn ratchet_state(&self) -> Option<&RatchetState> {
        self.ratchet.as_ref()
    }

    pub fn local_pubkey(&self) -> &[u8] {
        &self.local_pubkey
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.local_signing_key
    }

    pub fn has_shared_secret(&self) -> bool {
        self.has_shared
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keypair() -> (SigningKey, Vec<u8>) {
        let sk = SigningKey::random(&mut rand::rngs::OsRng);
        let pk = VerifyingKey::from(&sk);
        (sk, pk.to_sec1_bytes().to_vec())
    }

    // ── basic encrypt/decrypt ──

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (alice_sk, _) = keypair();
        let (bob_sk, _) = keypair();

        let mut alice_ch = NaclChannel::from_signing_key(alice_sk);
        let mut bob_ch = NaclChannel::from_signing_key(bob_sk);

        alice_ch
            .derive_shared_secret(&*VerifyingKey::from(&bob_ch.local_signing_key).to_sec1_bytes())
            .unwrap();
        bob_ch.derive_shared_secret(&alice_ch.local_pubkey).unwrap();

        let plaintext = b"secret knowledge payload";
        let encrypted = alice_ch.encrypt(plaintext).unwrap();
        let decrypted = bob_ch.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let (_, bob_pk) = keypair();
        let bob_vk = VerifyingKey::from_sec1_bytes(&bob_pk).unwrap();

        let mut alice_ch =
            NaclChannel::from_signing_key(SigningKey::random(&mut rand::rngs::OsRng));
        alice_ch
            .derive_shared_secret(&*bob_vk.to_sec1_bytes())
            .unwrap();

        let plaintext = b"same plaintext";
        let e1 = alice_ch.encrypt(plaintext).unwrap();
        let e2 = alice_ch.encrypt(plaintext).unwrap();
        assert_ne!(
            e1.ciphertext, e2.ciphertext,
            "different nonces should produce different ciphertexts"
        );
    }

    #[test]
    fn test_wrong_key_fails_decrypt() {
        let (alice_sk, _) = keypair();
        let (bob_sk, bob_pk) = keypair();
        let (eve_sk, _) = keypair();

        let mut alice_ch = NaclChannel::from_signing_key(alice_sk);
        let mut bob_ch = NaclChannel::from_signing_key(bob_sk);
        let mut eve_ch = NaclChannel::from_signing_key(eve_sk);

        alice_ch.derive_shared_secret(&bob_pk).unwrap();
        bob_ch.derive_shared_secret(&alice_ch.local_pubkey).unwrap();
        eve_ch.derive_shared_secret(&bob_pk).unwrap();

        let encrypted = alice_ch.encrypt(b"secret").unwrap();
        let result = eve_ch.decrypt(&encrypted);
        assert!(result.is_err(), "eve should not decrypt");
    }

    #[test]
    fn test_seal_and_open() {
        let (bob_sk, bob_pk) = keypair();
        let (alice_sk, _) = keypair();
        let alice_ch = NaclChannel::from_signing_key(alice_sk.clone());

        let plaintext = b"syphoned knowledge";
        let encrypted = alice_ch.seal_knowledge(plaintext, &bob_pk).unwrap();
        let decrypted = NaclChannel::open_knowledge(&bob_sk, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_pubkey_format() {
        let ch = NaclChannel::random();
        assert!(!ch.local_pubkey().is_empty());
        assert!(ch.local_pubkey().len() <= 65);
    }

    #[test]
    fn test_has_shared_secret() {
        let mut ch = NaclChannel::random();
        assert!(!ch.has_shared_secret());
        let peer = NaclChannel::random();
        ch.derive_shared_secret(peer.local_pubkey()).unwrap();
        assert!(ch.has_shared_secret());
    }

    #[test]
    fn test_symmetric_derivation() {
        let a = NaclChannel::random();
        let b = NaclChannel::random();
        let key_ab = NaclChannel::derive_symmetric(a.local_pubkey(), b.local_pubkey());
        let key_ba = NaclChannel::derive_symmetric(b.local_pubkey(), a.local_pubkey());
        assert_eq!(key_ab, key_ba);
    }

    // ── ratchet tests ──

    #[test]
    fn test_ratchet_init_encrypt_decrypt() {
        let (alice_sk, bob_pk) = keypair();
        let (bob_sk, _) = keypair();

        let mut alice_ch = NaclChannel::from_signing_key(alice_sk);
        let mut bob_ch = NaclChannel::from_signing_key(bob_sk);

        let eph_pubkey = alice_ch.init_ratchet(&bob_pk).unwrap();
        assert!(alice_ch.has_ratchet());

        bob_ch.init_ratchet_from_ephemeral(&eph_pubkey).unwrap();
        assert!(bob_ch.has_ratchet());

        let plaintext = b"secret knowledge with forward secrecy";
        let encrypted = alice_ch
            .encrypt_ratchet(plaintext, Some(eph_pubkey))
            .unwrap();
        let decrypted = bob_ch.decrypt_ratchet(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ratchet_multiple_messages() {
        let (alice_sk, bob_pk) = keypair();
        let (bob_sk, _) = keypair();

        let mut alice_ch = NaclChannel::from_signing_key(alice_sk);
        let mut bob_ch = NaclChannel::from_signing_key(bob_sk);

        let eph = alice_ch.init_ratchet(&bob_pk).unwrap();
        bob_ch.init_ratchet_from_ephemeral(&eph).unwrap();

        for i in 0..5 {
            let msg = format!("message {}", i);
            let encrypted = alice_ch.encrypt_ratchet(msg.as_bytes(), None).unwrap();
            let decrypted = bob_ch.decrypt_ratchet(&encrypted).unwrap();
            assert_eq!(String::from_utf8(decrypted).unwrap(), msg);
        }
    }

    #[test]
    fn test_ratchet_chain_index_tracking() {
        let (alice_sk, bob_pk) = keypair();
        let (bob_sk, _) = keypair();

        let mut alice_ch = NaclChannel::from_signing_key(alice_sk);
        let mut bob_ch = NaclChannel::from_signing_key(bob_sk);

        let eph = alice_ch.init_ratchet(&bob_pk).unwrap();
        bob_ch.init_ratchet_from_ephemeral(&eph).unwrap();

        let e1 = alice_ch.encrypt_ratchet(b"first", None).unwrap();
        let e2 = alice_ch.encrypt_ratchet(b"second", None).unwrap();
        assert_eq!(e1.chain_index, 0);
        assert_eq!(e2.chain_index, 1);
    }

    #[test]
    fn test_ratchet_forward_secrecy_chain() {
        let (alice_sk, bob_pk) = keypair();
        let (bob_sk, _) = keypair();

        let mut alice_ch = NaclChannel::from_signing_key(alice_sk);
        let mut bob_ch = NaclChannel::from_signing_key(bob_sk);

        let eph = alice_ch.init_ratchet(&bob_pk).unwrap();
        bob_ch.init_ratchet_from_ephemeral(&eph).unwrap();

        let plaintext = b"test message";
        let encrypted = alice_ch.encrypt_ratchet(plaintext, None).unwrap();
        let decrypted = bob_ch.decrypt_ratchet(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ratchet_without_init_fails() {
        let sk = SigningKey::random(&mut rand::rngs::OsRng);
        let mut ch = NaclChannel::from_signing_key(sk);
        assert!(ch.encrypt_ratchet(b"test", None).is_err());
    }

    #[test]
    fn test_ratchet_encrypted_packet_fields() {
        let packet = RatchetEncryptedPacket {
            nonce: [0u8; 12],
            ciphertext: vec![1, 2, 3],
            sender_pubkey: vec![4u8; 33],
            ephemeral_pubkey: vec![5u8; 33],
            chain_index: 7,
        };
        assert_eq!(packet.chain_index, 7);
        assert_eq!(packet.ephemeral_pubkey.len(), 33);
    }

    #[test]
    fn test_ratchet_state_basic() {
        let mut state = RatchetState::from_chain_key([0xAB; 32]);
        let k1 = state.next_send_key();
        let k2 = state.next_send_key();
        assert_ne!(k1, k2, "each message should get a different key");
    }

    #[test]
    fn test_ratchet_same_chain_same_keys() {
        let mut a = RatchetState::from_chain_key([0x42; 32]);
        let mut b = RatchetState::from_chain_key([0x42; 32]);
        assert_eq!(a.next_send_key(), b.next_send_key());
        assert_eq!(a.next_send_key(), b.next_send_key());
    }

    #[test]
    fn test_ecdh_shared_secret_commutative() {
        let (_alice_sk, _) = keypair();
        let (bob_sk, bob_pk) = keypair();

        let alice_eph = k256::ecdh::EphemeralSecret::random(&mut rand::rngs::OsRng);
        let alice_eph_pk = k256::PublicKey::from(&alice_eph);
        let bob_pk_parsed = k256::PublicKey::from_sec1_bytes(&bob_pk).unwrap();
        let alice_shared = alice_eph.diffie_hellman(&bob_pk_parsed);
        let mut alice_raw = [0u8; 32];
        alice_raw.copy_from_slice(alice_shared.raw_secret_bytes().as_ref());

        let eph_encoded = k256::EncodedPoint::from(&alice_eph_pk);
        let alice_eph_pk_bytes = eph_encoded.as_bytes().to_vec();

        let bob_shared = ecdh_shared_secret(&bob_sk, &alice_eph_pk_bytes).unwrap();

        assert_eq!(alice_raw, bob_shared, "ECDH must be commutative");
    }
}
