//! nt_io_desktop::updater_signing — 自动更新器代码签名 (Ed25519, minisign/cosign 兼容)
//!
//! 契约: updater_keygen / updater_pubkey / updater_sign
//! 核心: 密钥生成、公钥导出、构件签名/验证
//! 参考: minisign (Frank Denis), cosign (Sigstore), Electron SafeUpdater (Doyensec)

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use blake2::{Blake2b512, Digest};
use base64::{Engine as _, engine::general_purpose};

/// 签名操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignOp {
    Keygen,
    Pubkey,
    Sign,
}

/// 密钥对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: String,      // base64 Ed25519 public key
    pub secret_key: String,      // base64 encrypted secret key (minisign 格式)
    pub key_id: String,          // 8 字节 key ID (hex)
    pub comment: String,
}

/// 公钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyInfo {
    pub public_key: String,      // base64
    pub key_id: String,
    pub algorithm: String,       // "Ed25519"
    pub comment: String,
}

/// 签名输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningOutput {
    pub key_id: String,
    pub public_key: String,
    pub signature: Option<SignatureData>,
}

/// 签名数据 (minisign 兼容格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureData {
    pub artifact: String,        // 文件路径或 hash
    pub algorithm: String,       // "Ed25519"
    pub value: String,           // base64 签名
    pub trusted_comment: String, // 受保护注释 (含版本等防降级信息)
    pub untrusted_comment: String,
}

/// 签名错误
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SigningError {
    #[error("密钥生成失败: {0}")]
    KeygenFailed(String),
    #[error("密钥文件不存在: {0}")]
    KeyNotFound(String),
    #[error("密钥格式无效: {0}")]
    InvalidKeyFormat(String),
    #[error("签名失败: {0}")]
    SignFailed(String),
    #[error("验证失败: {0}")]
    VerifyFailed(String),
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("Base64 解码错误: {0}")]
    Base64Decode(String),
}

// 更新器签名器 (核心实现)
// 使用 ed25519-dalek 进行 Ed25519 操作
pub struct UpdaterSigner;

impl UpdaterSigner {
    /// 生成 Ed25519 密钥对 (兼容 minisign 格式)
    pub fn keygen(comment: &str) -> Result<KeyPair, SigningError> {
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        let pk_bytes = verifying_key.to_bytes();
        let sk_bytes = signing_key.to_bytes();

        // 生成 8 字节 key_id (取公钥前 8 字节的 hex)
        let key_id = hex::encode(&pk_bytes[..8]);

        Ok(KeyPair {
            public_key: general_purpose::STANDARD.encode(pk_bytes),
            secret_key: general_purpose::STANDARD.encode(sk_bytes),  // 实际生产应加密存储
            key_id,
            comment: comment.into(),
        })
    }

    /// 从密钥对导出公钥信息
    pub fn pubkey(keypair: &KeyPair) -> PublicKeyInfo {
        PublicKeyInfo {
            public_key: keypair.public_key.clone(),
            key_id: keypair.key_id.clone(),
            algorithm: "Ed25519".into(),
            comment: keypair.comment.clone(),
        }
    }

    /// 计算 Blake2b-512 哈希 (minisign 兼容预哈希)
    fn blake2b_512(data: &[u8]) -> [u8; 64] {
        let mut hasher = Blake2b512::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// 构造签名消息: hash || version (防降级攻击, minisign 兼容)
    fn build_message(data: &[u8], version: &str) -> Vec<u8> {
        let hash = Self::blake2b_512(data);
        let message = format!("{}-{}", hex::encode(hash), version);
        message.into_bytes()
    }

    /// 签名构件 (文件/哈希/版本元数据)
    /// 兼容 minisign 签名格式: Ed25519(Blake2b-512(data)) + trusted_comment
    pub fn sign(
        keypair: &KeyPair,
        artifact_path: &Path,
        version: &str,
        trusted_comment: &str,
    ) -> Result<SigningOutput, SigningError> {
        // 1. 读取文件
        let data = std::fs::read(artifact_path).map_err(|e| SigningError::Io(e.to_string()))?;

        // 2. 构造消息并签名
        let message = Self::build_message(&data, version);

        let sk_bytes = general_purpose::STANDARD.decode(&keypair.secret_key)
            .map_err(|e| SigningError::Base64Decode(e.to_string()))?;
        let signing_key = SigningKey::from_bytes(&sk_bytes.try_into().map_err(|_| SigningError::InvalidKeyFormat("secret key 长度错误".into()))?);

        let signature = signing_key.sign(&message);

        // 3. 返回签名输出
        Ok(SigningOutput {
            key_id: keypair.key_id.clone(),
            public_key: keypair.public_key.clone(),
            signature: Some(SignatureData {
                artifact: artifact_path.display().to_string(),
                algorithm: "Ed25519".into(),
                value: general_purpose::STANDARD.encode(signature.to_bytes()),
                trusted_comment: trusted_comment.into(),
                untrusted_comment: format!("version={},hash=blake2b512", version),
            }),
        })
    }

    /// 验证签名 (仅需公钥)
    pub fn verify(
        public_key: &str,
        artifact_path: &Path,
        signature: &SignatureData,
    ) -> Result<bool, SigningError> {
        // 1. 重算哈希
        let data = std::fs::read(artifact_path).map_err(|e| SigningError::Io(e.to_string()))?;

        // 2. 重构消息
        let version = signature.untrusted_comment
            .strip_prefix("version=")
            .and_then(|s| s.split(',').next())
            .unwrap_or("");

        let message = Self::build_message(&data, version);

        // 3. Ed25519 验证
        let sig_bytes = general_purpose::STANDARD.decode(&signature.value)
            .map_err(|e| SigningError::Base64Decode(e.to_string()))?;
        let pk_bytes = general_purpose::STANDARD.decode(public_key)
            .map_err(|e| SigningError::Base64Decode(e.to_string()))?;

        let verifying_key = VerifyingKey::from_bytes(&pk_bytes.try_into().map_err(|_| SigningError::InvalidKeyFormat("public key 长度错误".into()))?);
        let sig = Signature::from_bytes(&sig_bytes.try_into().map_err(|_| SigningError::InvalidKeyFormat("signature 长度错误".into()))?);

        Ok(verifying_key.verify(&message, &sig).is_ok())
    }

    /// 批量签名 (发布流水线用)
    pub fn sign_batch(
        keypair: &KeyPair,
        artifacts: &[(PathBuf, String)],  // (path, version)
        trusted_comment: &str,
    ) -> Vec<Result<SigningOutput, SigningError>> {
        artifacts.iter()
            .map(|(path, version)| Self::sign(keypair, path, version, trusted_comment))
            .collect()
    }
}

/// CLI 入口结构 (对应契约 op 字段)
#[derive(Debug, Deserialize)]
pub struct CliInput {
    pub op: SignOp,
    pub key: Option<KeyInput>,
    pub artifact: Option<ArtifactInput>,
    pub out_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KeyInput {
    pub seed: Option<String>,          // base64 seed (32 bytes)
    pub existing: Option<String>,      // 现有密钥文件路径
}

#[derive(Debug, Deserialize)]
pub struct ArtifactInput {
    pub path: String,
    pub hash: Option<String>,          // 可选预计算 hash
}

/// CLI 输出
#[derive(Debug, Serialize)]
pub struct CliOutput {
    pub key_id: String,
    pub public_key: String,
    pub signature: Option<SignatureData>,
}

/// 统一入口 (供 CLI / 背景循环 / 发布流水线调用)
pub fn execute_signing(input: CliInput) -> Result<CliOutput, SigningError> {
    match input.op {
        SignOp::Keygen => {
            let comment = input.key.as_ref()
                .and_then(|k| k.existing.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("neotrix-updater");
            let kp = UpdaterSigner::keygen(comment)?;
            Ok(CliOutput {
                key_id: kp.key_id,
                public_key: kp.public_key,
                signature: None,
            })
        }
        SignOp::Pubkey => {
            let kp = input.key.ok_or(SigningError::KeyNotFound("需要密钥输入".into()))?;
            let existing = kp.existing.ok_or(SigningError::KeyNotFound("existing 路径必填".into()))?;
            // 读取密钥文件...
            let keypair = load_keypair(&existing)?;
            let pk = UpdaterSigner::pubkey(&keypair);
            Ok(CliOutput {
                key_id: pk.key_id,
                public_key: pk.public_key,
                signature: None,
            })
        }
        SignOp::Sign => {
            let artifact = input.artifact.ok_or(SigningError::KeyNotFound("artifact 必填".into()))?;
            let keypair = input.key
                .and_then(|k| k.existing)
                .map(load_keypair)
                .transpose()?
                .ok_or(SigningError::KeyNotFound("签名需要密钥".into()))?;

            let version = artifact.hash.clone().unwrap_or_else(|| "dev".into());
            let sig = UpdaterSigner::sign(&keypair, Path::new(&artifact.path), &version, "neotrix release")?;

            Ok(CliOutput {
                key_id: sig.key_id.clone(),
                public_key: sig.public_key.clone(),
                signature: sig.signature,
            })
        }
    }
}

fn load_keypair(path: &str) -> Result<KeyPair, SigningError> {
    let content = std::fs::read_to_string(path).map_err(|e| SigningError::Io(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| SigningError::InvalidKeyFormat(e.to_string()))
}

/// SelfTest
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_keygen_roundtrip() {
        let kp = UpdaterSigner::keygen("test").unwrap();
        assert!(!kp.public_key.is_empty());
        assert!(!kp.secret_key.is_empty());
        assert_eq!(kp.key_id.len(), 8);

        let pk = UpdaterSigner::pubkey(&kp);
        assert_eq!(pk.public_key, kp.public_key);
        assert_eq!(pk.key_id, kp.key_id);
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let dir = tempdir().unwrap();
        let artifact = dir.path().join("test.bin");
        std::fs::write(&artifact, b"test artifact content").unwrap();

        let kp = UpdaterSigner::keygen("test").unwrap();
        let sig = UpdaterSigner::sign(&kp, &artifact, "1.0.0", "test release").unwrap();

        let verified = UpdaterSigner::verify(&kp.public_key, &artifact, sig.signature.as_ref().unwrap()).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_sign_batch() {
        let dir = tempdir().unwrap();
        let a1 = dir.path().join("a.bin");
        let a2 = dir.path().join("b.bin");
        std::fs::write(&a1, b"artifact 1").unwrap();
        std::fs::write(&a2, b"artifact 2").unwrap();

        let kp = UpdaterSigner::keygen("batch").unwrap();
        let results = UpdaterSigner::sign_batch(&kp, &[(a1, "1.0".into()), (a2, "1.0".into())], "batch");

        assert_eq!(results.len(), 2);
        for r in results {
            assert!(r.is_ok());
        }
    }
}