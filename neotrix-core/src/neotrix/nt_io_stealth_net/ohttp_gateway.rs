use std::sync::Arc;
use std::time::Duration;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use k256::{elliptic_curve::sec1::ToEncodedPoint, EncodedPoint, PublicKey, SecretKey};
use rand::RngCore;
use sha2::Sha256;
use tokio::sync::RwLock;

const HPKE_SUITE_ID: &[u8] = b"HPKE-KEM-X25519-HKDF-SHA256-AES-256-GCM";
const OHTTP_VERSION: u8 = 1;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct OhttpKeyConfig {
    pub kem_id: u16,
    pub kdf_id: u16,
    pub aead_id: u16,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OhttpGatewayConfig {
    pub relay_url: String,
    pub gateway_url: String,
    pub key_config_url: Option<String>,
}

impl Default for OhttpGatewayConfig {
    fn default() -> Self {
        Self {
            relay_url: "http://127.0.0.1:8090/relay".into(),
            gateway_url: "http://127.0.0.1:8091/gateway".into(),
            key_config_url: None,
        }
    }
}

pub struct OhttpRelayClient {
    config: OhttpGatewayConfig,
    gateway_key: Arc<RwLock<Option<OhttpKeyConfig>>>,
    client: reqwest::Client,
    local_secret: SecretKey,
    local_public: EncodedPoint,
}

impl OhttpRelayClient {
    pub fn new(config: OhttpGatewayConfig) -> Self {
        let secret = SecretKey::random(&mut OsRng);
        let public = secret.public_key().to_encoded_point(false);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .no_proxy()
            .build()
            .expect("reqwest client");

        Self {
            config,
            gateway_key: Arc::new(RwLock::new(None)),
            client,
            local_secret: secret,
            local_public: public,
        }
    }

    pub fn local_public_key_bytes(&self) -> Vec<u8> {
        self.local_public.as_bytes().to_vec()
    }

    pub async fn set_gateway_key(&self, key: OhttpKeyConfig) {
        *self.gateway_key.write().await = Some(key);
    }

    pub async fn fetch_key_config(&self) -> Result<OhttpKeyConfig, String> {
        let default_url = format!("{}/ohttp-keys", self.config.gateway_url);
        let url = self
            .config
            .key_config_url
            .as_deref()
            .unwrap_or(&default_url);

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("fetch key config: {}", e))?;

        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("read key config: {}", e))?;

        if body.len() < 8 {
            return Err("key config too short".into());
        }

        let kem_id = u16::from_be_bytes([body[0], body[1]]);
        let kdf_id = u16::from_be_bytes([body[2], body[3]]);
        let aead_id = u16::from_be_bytes([body[4], body[5]]);
        let key_len = u16::from_be_bytes([body[6], body[7]]) as usize;

        if body.len() < 8 + key_len {
            return Err("key config truncated".into());
        }

        let public_key = body[8..8 + key_len].to_vec();

        Ok(OhttpKeyConfig {
            kem_id,
            kdf_id,
            aead_id,
            public_key,
        })
    }

    pub async fn relay_request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, String> {
        let key_config =
            self.gateway_key.read().await.clone().ok_or_else(|| {
                "gateway key not loaded — call fetch_key_config first".to_string()
            })?;

        let encrypted = self.seal_request(method, path, body, &key_config)?;

        let relay_url = format!("{}{}", self.config.relay_url.trim_end_matches('/'), path);

        let resp = self
            .client
            .post(&relay_url)
            .header("Content-Type", "application/ohttp-request")
            .body(encrypted)
            .send()
            .await
            .map_err(|e| format!("relay request: {}", e))?;

        let encrypted_response = resp
            .bytes()
            .await
            .map_err(|e| format!("read relay response: {}", e))?;

        self.open_response(&encrypted_response, &key_config)
    }

    fn seal_request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        key_config: &OhttpKeyConfig,
    ) -> Result<Vec<u8>, String> {
        let gateway_pk = PublicKey::from_sec1_bytes(&key_config.public_key)
            .map_err(|e| format!("invalid gateway public key: {}", e))?;

        let ecdh = k256::ecdh::diffie_hellman(
            &self.local_secret.to_nonzero_scalar(),
            gateway_pk.as_affine(),
        );
        let shared_bytes: [u8; 32] = ecdh
            .raw_secret_bytes()
            .as_slice()
            .try_into()
            .map_err(|_| "shared secret length mismatch".to_string())?;

        let info = self.build_hpke_info(key_config);
        let key = hkdf_sha256(&shared_bytes, b"key", &info, 32);
        let nonce_seed = hkdf_sha256(&shared_bytes, b"base_nonce", &info, 12);

        let enc = self.local_public_key_bytes();
        let aad = self.build_aad(method, path);

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        nonce_bytes.copy_from_slice(&nonce_seed[..NONCE_SIZE]);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let pt = self.build_http_message(method, path, body);
        let ct = cipher
            .encrypt(nonce, pt.as_slice())
            .map_err(|e| format!("aes encrypt: {}", e))?;

        let mut output = Vec::with_capacity(1 + 2 + enc.len() + NONCE_SIZE + ct.len());
        output.push(OHTTP_VERSION);
        output.extend_from_slice(&(enc.len() as u16).to_be_bytes());
        output.extend_from_slice(&enc);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ct);
        output.extend_from_slice(&aad);

        Ok(output)
    }

    fn open_response(
        &self,
        encrypted: &[u8],
        _key_config: &OhttpKeyConfig,
    ) -> Result<Vec<u8>, String> {
        if encrypted.is_empty() {
            return Err("empty encrypted response".into());
        }
        let version = encrypted[0];
        if version != OHTTP_VERSION {
            return Err(format!("unexpected ohttp version: {}", version));
        }

        let mut offset = 1;
        if offset + 2 > encrypted.len() {
            return Err("truncated enc length".into());
        }
        let enc_len = u16::from_be_bytes([encrypted[offset], encrypted[offset + 1]]) as usize;
        offset += 2;

        if offset + enc_len > encrypted.len() {
            return Err("truncated enc".into());
        }
        offset += enc_len;

        if offset + NONCE_SIZE > encrypted.len() {
            return Err("truncated nonce".into());
        }
        let nonce_bytes = &encrypted[offset..offset + NONCE_SIZE];
        offset += NONCE_SIZE;

        let ct = &encrypted[offset..];

        let ephem_pk = PublicKey::from_sec1_bytes(&encrypted[1..1 + enc_len])
            .map_err(|_| "response decryption: invalid ephemeral key".to_string())?;
        let ecdh = k256::ecdh::diffie_hellman(
            &self.local_secret.to_nonzero_scalar(),
            ephem_pk.as_affine(),
        );
        let shared_bytes: [u8; 32] = ecdh
            .raw_secret_bytes()
            .as_slice()
            .try_into()
            .map_err(|_| "response decryption: shared secret failed".to_string())?;

        let resp_key = hkdf_sha256(&shared_bytes, b"response_key", b"ohttp response", 32);
        let cipher = Aes256Gcm::new_from_slice(&resp_key)
            .map_err(|e| format!("response cipher init: {}", e))?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let pt = cipher
            .decrypt(nonce, ct)
            .map_err(|e| format!("aes decrypt response: {}", e))?;

        Ok(pt)
    }

    fn build_hpke_info(&self, key_config: &OhttpKeyConfig) -> Vec<u8> {
        let mut info = Vec::new();
        info.extend_from_slice(HPKE_SUITE_ID);
        info.extend_from_slice(&key_config.kem_id.to_be_bytes());
        info.extend_from_slice(&key_config.kdf_id.to_be_bytes());
        info.extend_from_slice(&key_config.aead_id.to_be_bytes());
        info
    }

    fn build_aad(&self, method: &str, path: &str) -> Vec<u8> {
        let mut aad = Vec::new();
        aad.extend_from_slice(method.as_bytes());
        aad.push(0);
        aad.extend_from_slice(path.as_bytes());
        aad
    }

    fn build_http_message(&self, method: &str, path: &str, body: &[u8]) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(method.as_bytes());
        msg.push(b' ');
        msg.extend_from_slice(path.as_bytes());
        msg.extend_from_slice(b" HTTP/1.1\r\n");
        msg.extend_from_slice(b"Host: obfuscated\r\n");
        msg.extend_from_slice(b"Content-Length: ");
        msg.extend_from_slice(body.len().to_string().as_bytes());
        msg.extend_from_slice(b"\r\n\r\n");
        msg.extend_from_slice(body);
        msg
    }
}

fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut prk: [u8; 32] = [0u8; 32];
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(salt).expect("hkdf salt");
    mac.update(ikm);
    prk.copy_from_slice(&mac.finalize().into_bytes());

    let mut okm = Vec::with_capacity(len);
    let mut t: Vec<u8> = Vec::new();
    let mut counter: u8 = 1;
    while okm.len() < len {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&prk).expect("hkdf expand: 32-byte PRK from SHA-256 HMAC should always be valid HMAC key");
        mac.update(&t);
        mac.update(info);
        mac.update(&[counter]);
        t = mac.finalize().into_bytes().to_vec();
        okm.extend_from_slice(&t);
        counter += 1;
    }
    okm.truncate(len);
    okm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_sha256_basic() {
        let derived = hkdf_sha256(b"test input", b"salt", b"info", 32);
        assert_eq!(derived.len(), 32);
        let derived2 = hkdf_sha256(b"test input", b"salt", b"info", 32);
        assert_eq!(derived, derived2, "hkdf must be deterministic");

        let derived3 = hkdf_sha256(b"different", b"salt", b"info", 32);
        assert_ne!(derived, derived3, "different ikm -> different output");
    }

    #[test]
    fn test_keypair_generation() {
        let secret = SecretKey::random(&mut OsRng);
        let public = secret.public_key();
        let encoded = public.to_encoded_point(false);
        assert!(!encoded.as_bytes().is_empty());

        let decoded = PublicKey::from_sec1_bytes(encoded.as_bytes());
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_ohttp_key_config_parse() {
        let mut raw = Vec::new();
        raw.extend_from_slice(&[0x00, 0x20]);
        raw.extend_from_slice(&[0x00, 0x01]);
        raw.extend_from_slice(&[0x00, 0x01]);
        let pk = SecretKey::random(&mut OsRng).public_key();
        let pk_bytes = pk.to_encoded_point(false).as_bytes().to_vec();
        raw.extend_from_slice(&(pk_bytes.len() as u16).to_be_bytes());
        raw.extend_from_slice(&pk_bytes);

        assert!(raw.len() > 8);

        let kem_id = u16::from_be_bytes([raw[0], raw[1]]);
        let kdf_id = u16::from_be_bytes([raw[2], raw[3]]);
        let aead_id = u16::from_be_bytes([raw[4], raw[5]]);
        let key_len = u16::from_be_bytes([raw[6], raw[7]]) as usize;
        let public_key = raw[8..8 + key_len].to_vec();

        assert_eq!(kem_id, 0x0020);
        assert_eq!(kdf_id, 0x0001);
        assert_eq!(aead_id, 0x0001);
        assert_eq!(public_key.len(), pk_bytes.len());
    }

    #[test]
    fn test_seal_request_requires_key() {
        let config = OhttpGatewayConfig::default();
        let client = OhttpRelayClient::new(config);
        let key_config = OhttpKeyConfig {
            kem_id: 0x0020,
            kdf_id: 0x0001,
            aead_id: 0x0001,
            public_key: vec![0u8; 65],
        };

        let result = client.seal_request("GET", "/test", b"hello", &key_config);
        assert!(
            result.is_ok(),
            "seal_request should succeed: {:?}",
            result.err()
        );
        let sealed = result.unwrap();
        assert!(!sealed.is_empty());
        assert_eq!(sealed[0], OHTTP_VERSION);
    }

    #[test]
    fn test_build_http_message() {
        let client = OhttpRelayClient::new(OhttpGatewayConfig::default());
        let msg = client.build_http_message("POST", "/api/data", b"{\"key\":\"value\"}");
        let text = String::from_utf8_lossy(&msg);
        assert!(text.contains("POST /api/data HTTP/1.1"));
        assert!(text.contains("Content-Length: 15"));
        assert!(text.contains("{\"key\":\"value\"}"));
    }

    #[test]
    fn test_build_hpke_info() {
        let client = OhttpRelayClient::new(OhttpGatewayConfig::default());
        let key_config = OhttpKeyConfig {
            kem_id: 0x0020,
            kdf_id: 0x0001,
            aead_id: 0x0001,
            public_key: vec![],
        };
        let info = client.build_hpke_info(&key_config);
        assert!(info.starts_with(HPKE_SUITE_ID));
    }

    #[tokio::test]
    async fn test_set_and_get_gateway_key() {
        let client = OhttpRelayClient::new(OhttpGatewayConfig::default());
        let key = OhttpKeyConfig {
            kem_id: 0x0020,
            kdf_id: 0x0001,
            aead_id: 0x0001,
            public_key: vec![1u8, 2, 3],
        };
        client.set_gateway_key(key.clone()).await;
        let stored = client.gateway_key.read().await;
        assert_eq!(stored.as_ref().unwrap().public_key, vec![1u8, 2, 3]);
    }
}
