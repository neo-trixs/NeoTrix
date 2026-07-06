use std::path::Path;

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Atomically write data to a file: write to a temporary file first, then rename.
/// Prevents partial/corrupt writes if the process crashes mid-write.
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {}", e))?;
    }
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, data).map_err(|e| format!("write tmp: {}", e))?;
    std::fs::rename(&tmp_path, path).map_err(|e| format!("rename: {}", e))?;
    Ok(())
}

/// Atomically write data with an HMAC-SHA256 signature file.
fn derive_hmac_key() -> Vec<u8> {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "localhost".to_string());
    let salt = b"neotrix-store-integrity-v1";
    let mut key = Vec::with_capacity(hostname.len() + salt.len());
    key.extend_from_slice(hostname.as_bytes());
    key.extend_from_slice(salt);
    key
}

pub fn compute_hmac(data: &[u8]) -> Result<String, String> {
    let key = derive_hmac_key();
    let mut mac = HmacSha256::new_from_slice(&key)
        .map_err(|e| format!("HMAC key init: {}", e))?;
    mac.update(data);
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// Atomically write data and a `.sig` HMAC file for integrity verification.
pub fn atomic_write_signed(path: &Path, data: &[u8]) -> Result<(), String> {
    atomic_write(path, data)?;
    let sig = compute_hmac(data)?;
    let sig_path = format!("{}.sig", path.display());
    atomic_write(Path::new(&sig_path), sig.as_bytes())?;
    Ok(())
}

/// Verify an HMAC signature file against data. Returns Ok(()) if valid.
pub fn verify_sig(path: &Path, data: &[u8]) -> Result<(), String> {
    let sig_path = format!("{}.sig", path.display());
    let sig_path = Path::new(&sig_path);
    if !sig_path.exists() {
        return Err("signature file not found".to_string());
    }
    let expected_hex = std::fs::read_to_string(sig_path)
        .map_err(|e| format!("read sig: {}", e))?;
    let expected = hex::decode(expected_hex.trim())
        .map_err(|e| format!("decode sig hex: {}", e))?;
    let key = derive_hmac_key();
    let mut mac = HmacSha256::new_from_slice(&key)
        .map_err(|e| format!("HMAC key init: {}", e))?;
    mac.update(data);
    mac.verify_slice(&expected).map_err(|_| {
        "integrity check failed: HMAC mismatch — file may be corrupted or tampered".to_string()
    })
}
