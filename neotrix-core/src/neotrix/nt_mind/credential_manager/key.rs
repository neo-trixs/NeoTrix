use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::core::nt_core_util;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};

/// Derive a 32-byte AES key from a password using Argon2id + SHA-256 fallback.
///
/// Uses Argon2id (m=19456, t=2, p=1, 19KiB) when `argon2` crate available;
/// falls back to SHA-256(salt || password) for embedded/no-std targets.
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    match argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(19456, 2, 1, None).unwrap_or_default(),
    )
    .hash_password_into(password.as_bytes(), salt, &mut key)
    {
        Ok(_) => key,
        Err(_) => {
            let mut hasher = Sha256::new();
            hasher.update(salt);
            hasher.update(password.as_bytes());
            key.copy_from_slice(&hasher.finalize());
            key
        }
    }
}

/// Generate or load the master key: env var → Argon2 password → file → generate.
///
/// Priority:
/// 1. `NEOTRIX_MASTER_KEY` env var (64 hex chars → 32 bytes)
/// 2. `NEOTRIX_MASTER_PASSWORD` env var → Argon2id derive (salt from `~/.neotrix/.salt`)
/// 3. `~/.neotrix/.master_key` file (32 raw bytes)
/// 4. Generate new random key, save to `~/.neotrix/.master_key`
pub fn load_or_generate_master_key() -> NeoTrixResult<[u8; 32]> {
    // 1. Explicit hex key
    if let Ok(key_str) = std::env::var("NEOTRIX_MASTER_KEY") {
        let key_str = key_str.trim().to_string();
        if let Ok(decoded) = hex::decode(&key_str) {
            if decoded.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&decoded);
                return Ok(key);
            }
        }
    }

    // 2. Master password → Argon2id
    if let Ok(password) = std::env::var("NEOTRIX_MASTER_PASSWORD") {
        let salt_path = dirs::home_dir()
            .map(|h| h.join(".neotrix").join(".salt"))
            .unwrap_or_else(|| nt_core_util::home_dir().join(".neotrix").join("salt"));
        let salt = if salt_path.exists() {
            std::fs::read(&salt_path).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?
        } else {
            let mut s = vec![0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut s);
            if let Some(parent) = salt_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = crate::core::nt_core_util::atomic_write_bytes(&salt_path, &s);
            s
        };
        let key = derive_key_from_password(&password, &salt);
        return Ok(key);
    }

    // 3. Master key file
    let key_path = dirs::home_dir()
        .map(|h| h.join(".neotrix").join(".master_key"))
        .unwrap_or_else(|| nt_core_util::home_dir().join(".neotrix").join("master_key"));
    if key_path.exists() {
        let data =
            std::fs::read(&key_path).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        if data.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&data);
            return Ok(key);
        }
    }

    // 4. Generate new
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
    }
    let tmp_path = key_path.with_extension("tmp");
    std::fs::write(&tmp_path, &key).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp_path, &key_path).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
    Ok(key)
}

pub(crate) fn default_cred_path() -> std::path::PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".neotrix").join("credentials.json"))
        .unwrap_or_else(|| {
            nt_core_util::home_dir()
                .join(".neotrix")
                .join("credentials.json")
        })
}
