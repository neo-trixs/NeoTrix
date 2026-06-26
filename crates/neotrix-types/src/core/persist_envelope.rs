//! Versioned persistence envelope — wraps all persisted data with version + schema info.
//! Enables safe migration when data structures change between releases.

use serde::{Serialize, Deserialize};
use std::path::Path;

/// Schema version — bump when breaking changes are made to persisted structures.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Latest NeoTrix store format version.
pub const STORE_FORMAT_VERSION: &str = "neotrix-store-v1";

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Wrapper envelope for all persisted data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistEnvelope<T: Serialize> {
    /// Format identifier (e.g., "neotrix-store-v1")
    pub format: String,
    /// Schema version for migration checks
    pub schema_version: u32,
    /// Unix timestamp of when this was written
    pub written_at: i64,
    /// Number of times this store has been written (monotonic counter)
    pub write_count: u64,
    /// The actual payload
    pub data: T,
}

impl<T: Serialize> PersistEnvelope<T> {
    /// Create a new envelope wrapping data.
    pub fn new(data: T) -> Self {
        Self {
            format: STORE_FORMAT_VERSION.to_string(),
            schema_version: CURRENT_SCHEMA_VERSION,
            written_at: now_unix(),
            write_count: 0,
            data,
        }
    }

    /// Create with explicit write count (for monotonic tracking).
    pub fn with_count(data: T, count: u64) -> Self {
        let mut env = Self::new(data);
        env.write_count = count;
        env
    }
}

/// Read a versioned envelope from a JSON file with compatibility checking.
pub fn read_envelope<T: serde::de::DeserializeOwned>(
    _path: &Path,
    data: &[u8],
) -> Result<T, String> {
    let value: serde_json::Value = serde_json::from_slice(data)
        .map_err(|e| format!("parse envelope: {}", e))?;

    // Check format
    if let Some(fmt) = value.get("format").and_then(|v| v.as_str()) {
        if fmt != STORE_FORMAT_VERSION {
            return Err(format!(
                "format mismatch: expected '{}', got '{}'",
                STORE_FORMAT_VERSION, fmt
            ));
        }
    }

    // Check schema version (allow reading older versions)
    if let Some(ver) = value.get("schema_version").and_then(|v| v.as_u64()) {
        if ver > CURRENT_SCHEMA_VERSION as u64 {
            return Err(format!(
                "schema version {} > current {}, upgrade required",
                ver, CURRENT_SCHEMA_VERSION
            ));
        }
    }

    // Extract inner data
    if value.get("data").is_some() {
        serde_json::from_value(value["data"].clone())
            .map_err(|e| format!("extract data: {}", e))
    } else {
        // Plain JSON without envelope — try direct deserialization
        serde_json::from_value(value)
            .map_err(|e| format!("parse plain: {}", e))
    }
}

/// Create a JSON string with the envelope.
pub fn write_envelope<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
    let envelope = PersistEnvelope::new(data);
    serde_json::to_vec_pretty(&envelope)
        .map_err(|e| format!("serialize envelope: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_roundtrip() {
        let data = vec![1, 2, 3];
        let bytes = write_envelope(&data).expect("write_envelope should succeed");
        let recovered: Vec<i32> = read_envelope(Path::new(""), &bytes).expect("read_envelope should succeed");
        assert_eq!(recovered, vec![1, 2, 3]);
    }

    #[test]
    fn test_compatible_format() {
        // Old format without envelope (just raw JSON)
        let bytes = b"[1,2,3]";
        let recovered: Vec<i32> = read_envelope(Path::new(""), bytes).expect("read_envelope should handle old format");
        assert_eq!(recovered, vec![1, 2, 3]);
    }

    #[test]
    fn test_format_mismatch() {
        let bad = r#"{"format":"neotrix-store-v0","schema_version":1,"data":[1]}"#;
        let result: Result<Vec<i32>, String> = read_envelope(Path::new(""), bad.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_future_schema_rejected() {
        let future = r#"{"format":"neotrix-store-v1","schema_version":99,"data":[1]}"#;
        let result: Result<Vec<i32>, String> = read_envelope(Path::new(""), future.as_bytes());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("upgrade required"));
    }

    #[test]
    fn test_write_count_monotonic() {
        let data = "test";
        let env = PersistEnvelope::with_count(data, 42);
        assert_eq!(env.write_count, 42);
        assert_eq!(env.data, "test");
    }
}
