use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};
use hmac::Hmac;
use hmac::digest::KeyInit;
use serde::{Serialize, Deserialize};

type HmacSha256 = Hmac<Sha256>;

const CHAIN_FILE: &str = "avatar_chain.json";
const IDENTITY_FILE: &str = "identity.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarIdentity {
    pub name: String,
    pub identity_key_hmac: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub edition: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageDirection {
    Outbound,
    Inbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainEntry {
    pub index: u64,
    pub timestamp: i64,
    pub previous_hash: String,
    pub data_hash: String,
    pub signature: String,
    pub encrypted_data: String,
    pub direction: MessageDirection,
    pub from: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarChain {
    pub entries: Vec<ChainEntry>,
    pub genesis_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub from: String,
    pub msg_type: String,
    pub payload: String,
    pub timestamp: i64,
    pub chain_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainCapability {
    pub name: String,
    pub granted: bool,
    pub grant_timestamp: i64,
    pub expiry: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub capability: String,
    pub timestamp: i64,
    pub reasoning: String,
    pub granted: Option<bool>,
    pub response_time: Option<i64>,
}

const CAPABILITY_FILE: &str = "brain_capabilities.json";
const AUTH_FILE: &str = "auth_requests.json";

pub fn load_capabilities() -> Vec<BrainCapability> {
    let path = data_dir().join(CAPABILITY_FILE);
    if path.exists() {
        fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn save_capabilities(caps: &[BrainCapability]) {
    let dir = data_dir();
    let _ = fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(caps) {
        let _ = fs::write(dir.join(CAPABILITY_FILE), &json);
    }
}

pub fn load_auth_requests() -> Vec<AuthRequest> {
    let path = data_dir().join(AUTH_FILE);
    if path.exists() {
        fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn save_auth_requests(reqs: &[AuthRequest]) {
    let dir = data_dir();
    let _ = fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(reqs) {
        let _ = fs::write(dir.join(AUTH_FILE), &json);
    }
}

fn data_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".neotrix")
}

fn derive_key(name: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"neotrix-avatar-v1:");
    hasher.update(name.as_bytes());
    hasher.update(b":secret");
    hasher.finalize().to_vec()
}

fn encrypt_xor(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter().enumerate().map(|(i, b)| b ^ key[i % key.len()]).collect()
}

fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn compute_hmac(key: &[u8], data: &[u8]) -> String {
    use hmac::Mac;
    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(key).expect("HMAC key");
    mac.update(data);
    format!("{:x}", mac.finalize().into_bytes())
}

impl AvatarIdentity {
    pub fn new(name: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let key = derive_key(name);
        let key_hmac = compute_hmac(&key, name.as_bytes());
        Self {
            name: name.to_string(),
            identity_key_hmac: key_hmac,
            created_at: now,
            updated_at: now,
            edition: 1,
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = data_dir();
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(dir.join(IDENTITY_FILE), &json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load() -> Option<Self> {
        let path = data_dir().join(IDENTITY_FILE);
        if path.exists() {
            fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok())
        } else {
            None
        }
    }

    pub fn secret(&self) -> Vec<u8> {
        derive_key(&self.name)
    }
}

impl ChainEntry {
    pub fn new(index: u64, previous_hash: &str, data: &[u8], secret: &[u8],
               direction: MessageDirection, from: &str) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let data_hash = hash_data(data);
        let encrypted = encrypt_xor(data, secret);
        let enc_b64 = base64_encode(&encrypted);
        let sig_input = format!("{}{}{}{}{}{}{}", index, timestamp, previous_hash, data_hash, enc_b64,
            serde_json::to_string(&direction).unwrap_or_default(), from);
        let signature = compute_hmac(secret, sig_input.as_bytes());
        Self {
            index,
            timestamp,
            previous_hash: previous_hash.to_string(),
            data_hash,
            signature,
            encrypted_data: enc_b64,
            direction,
            from: from.to_string(),
        }
    }

    pub fn verify(&self, secret: &[u8]) -> bool {
        let sig_input = format!("{}{}{}{}{}{}{}", self.index, self.timestamp, self.previous_hash, self.data_hash, self.encrypted_data,
            serde_json::to_string(&self.direction).unwrap_or_default(), self.from);
        let expected = compute_hmac(secret, sig_input.as_bytes());
        expected == self.signature
    }

    pub fn decrypt_data(&self, secret: &[u8]) -> Option<Vec<u8>> {
        let encrypted = base64_decode(&self.encrypted_data)?;
        Some(encrypt_xor(&encrypted, secret))
    }
}

impl Default for AvatarChain {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarChain {
    pub fn new() -> Self {
        let genesis = hash_data(b"neotrix-avatar-chain-genesis-v1");
        Self {
            entries: Vec::new(),
            genesis_hash: genesis,
        }
    }

    pub fn push(&mut self, data: &[u8], secret: &[u8],
                direction: MessageDirection, from: &str) -> &ChainEntry {
        let prev = self.entries.last()
            .map(|e| e.data_hash.clone())
            .unwrap_or_else(|| self.genesis_hash.clone());
        let entry = ChainEntry::new(self.entries.len() as u64, &prev, data, secret,
                                    direction, from);
        self.entries.push(entry);
        self.entries.last().expect("result")
    }

    pub fn query_by_direction(&self, dir: &MessageDirection) -> Vec<&ChainEntry> {
        self.entries.iter().filter(|e| e.direction == *dir).collect()
    }

    pub fn query_by_from(&self, from: &str) -> Vec<&ChainEntry> {
        self.entries.iter().filter(|e| e.from == from).collect()
    }

    pub fn query_latest(&self, n: usize, dir: Option<&MessageDirection>) -> Vec<&ChainEntry> {
        let mut filtered: Vec<&ChainEntry> = match dir {
            Some(d) => self.entries.iter().filter(|e| e.direction == *d).collect(),
            None => self.entries.iter().collect(),
        };
        filtered.reverse();
        filtered.truncate(n);
        filtered
    }

    pub fn verify_chain(&self, secret: &[u8]) -> bool {
        let mut prev = Some(self.genesis_hash.clone());
        for entry in &self.entries {
            if !entry.verify(secret) {
                return false;
            }
            if let Some(ref expected_prev) = prev {
                if entry.previous_hash != *expected_prev {
                    return false;
                }
            }
            prev = Some(entry.data_hash.clone());
        }
        true
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = data_dir();
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(dir.join(CHAIN_FILE), &json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load() -> Self {
        let path = data_dir().join(CHAIN_FILE);
        if path.exists() {
            fs::read_to_string(&path).ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::new()
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(data: &str) -> Option<Vec<u8>> {
    let chars: Vec<_> = data.chars().collect();
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for ch in chars {
        if ch == '=' { break; }
        let val = CHARS.iter().position(|&c| c as char == ch)?;
        buffer = (buffer << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(result)
}

pub fn generate_identity(name: &str) -> AvatarIdentity {
    let identity = AvatarIdentity::new(name);
    let _ = identity.save();
    identity
}

pub fn load_or_create_identity(name: Option<&str>) -> Option<AvatarIdentity> {
    if let Some(loaded) = AvatarIdentity::load() {
        return Some(loaded);
    }
    name.map(generate_identity)
}

pub fn avatar_chain_save_path() -> PathBuf {
    data_dir().join(CHAIN_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_identity_new() {
        let id = AvatarIdentity::new("test_avatar");
        assert_eq!(id.name, "test_avatar");
        assert_eq!(id.edition, 1);
        assert!(id.created_at > 0);
        assert_eq!(id.updated_at, id.created_at);
        assert!(!id.identity_key_hmac.is_empty());
    }

    #[test]
    fn test_chain_creation_empty() {
        let chain = AvatarChain::new();
        assert_eq!(chain.len(), 0);
        assert!(chain.entries.is_empty());
        assert!(!chain.genesis_hash.is_empty());
        assert!(chain.verify_chain(&[]));
    }

    #[test]
    fn test_chain_push_and_verify() {
        let identity = AvatarIdentity::new("test_avatar");
        let secret = identity.secret();
        let mut chain = AvatarChain::new();
        let data = b"hello world";

        chain.push(data, &secret, MessageDirection::Outbound, "alice");
        assert_eq!(chain.len(), 1);

        chain.push(data, &secret, MessageDirection::Inbound, "bob");
        assert_eq!(chain.len(), 2);

        assert!(chain.verify_chain(&secret));
    }

    #[test]
    fn test_chain_query_by_direction() {
        let identity = AvatarIdentity::new("test_avatar");
        let secret = identity.secret();
        let mut chain = AvatarChain::new();

        chain.push(b"out1", &secret, MessageDirection::Outbound, "alice");
        chain.push(b"in1", &secret, MessageDirection::Inbound, "bob");
        chain.push(b"out2", &secret, MessageDirection::Outbound, "alice");

        assert_eq!(chain.query_by_direction(&MessageDirection::Outbound).len(), 2);
        assert_eq!(chain.query_by_direction(&MessageDirection::Inbound).len(), 1);
    }

    #[test]
    fn test_chain_entry_decrypt_roundtrip() {
        let identity = AvatarIdentity::new("test_avatar");
        let secret = identity.secret();
        let data = b"secret message";
        let entry = ChainEntry::new(0, "genesis", data, &secret, MessageDirection::Outbound, "alice");

        assert!(entry.verify(&secret));
        let decrypted = entry.decrypt_data(&secret).expect("value should be ok in test");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_chain_entry_verify_fails_on_tampered() {
        let identity = AvatarIdentity::new("test_avatar");
        let secret = identity.secret();
        let mut entry = ChainEntry::new(0, "genesis", b"data", &secret, MessageDirection::Outbound, "alice");
        assert!(entry.verify(&secret));
        entry.signature = "tampered".to_string();
        assert!(!entry.verify(&secret));
    }

    #[test]
    fn test_chain_entry_serialization_roundtrip() {
        let entry = ChainEntry::new(0, "genesis", b"data", b"key12345", MessageDirection::Inbound, "bob");
        let json = serde_json::to_string(&entry).expect("value should be ok in test");
        let deserialized: ChainEntry = serde_json::from_str(&json).expect("value should be ok in test");
        assert_eq!(entry.index, deserialized.index);
        assert_eq!(entry.from, deserialized.from);
        assert_eq!(entry.direction, deserialized.direction);
    }

    #[test]
    fn test_chain_message_direction_partial_eq() {
        assert_eq!(MessageDirection::Outbound, MessageDirection::Outbound);
        assert_ne!(MessageDirection::Outbound, MessageDirection::Inbound);
    }

    #[test]
    fn test_chain_entry_encrypted_data_not_plaintext() {
        let identity = AvatarIdentity::new("test_avatar");
        let secret = identity.secret();
        let data = b"visible";
        let entry = ChainEntry::new(0, "genesis", data, &secret, MessageDirection::Outbound, "alice");
        assert_ne!(entry.encrypted_data, "visible");
    }
}
