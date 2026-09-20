use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single entry in a Software Bill of Materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomEntry {
    pub package_name: String,
    pub version: String,
    pub license: String,
    pub hashes: HashMap<String, String>,
}

/// Generates SBOM entries by parsing Cargo.lock files
pub struct SbomGenerator;

impl SbomGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Parse a Cargo.lock file and produce SBOM entries
    pub fn generate_from_lock(lock_path: &str) -> Vec<SbomEntry> {
        let content = match std::fs::read_to_string(lock_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        Self::parse_lock_content(&content)
    }

    /// Parse Cargo.lock content string into SBOM entries
    pub fn parse_lock_content(content: &str) -> Vec<SbomEntry> {
        let mut entries = Vec::new();
        let mut in_package = false;
        let mut name = String::new();
        let mut version = String::new();
        let mut license = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "[[package]]" {
                if !name.is_empty() && !version.is_empty() {
                    entries.push(SbomEntry {
                        package_name: std::mem::take(&mut name),
                        version: std::mem::take(&mut version),
                        license: std::mem::take(&mut license),
                        hashes: HashMap::new(),
                    });
                }
                in_package = true;
                continue;
            }

            if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
                if !name.is_empty() && !version.is_empty() {
                    entries.push(SbomEntry {
                        package_name: std::mem::take(&mut name),
                        version: std::mem::take(&mut version),
                        license: std::mem::take(&mut license),
                        hashes: HashMap::new(),
                    });
                }
                in_package = false;
                continue;
            }

            if in_package {
                if let Some((key, value)) = trimmed.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');
                    match key {
                        "name" => name = value.to_string(),
                        "version" => version = value.to_string(),
                        "license" => license = value.to_string(),
                        _ => {}
                    }
                }
            }
        }

        if !name.is_empty() && !version.is_empty() {
            entries.push(SbomEntry {
                package_name: name,
                version,
                license,
                hashes: HashMap::new(),
            });
        }

        entries
    }
}

impl Default for SbomGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        let entries = SbomGenerator::parse_lock_content("");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_single_package() {
        let content = r#"[[package]]
name = "serde"
version = "1.0.193"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e0aa9b8b2ad45dfb0ba14e9e507c90e81b6f271a426f3795e1e4f17f25e015d2"

[metadata]
"checksum serde 1.0.193 (registry+https://github.com/rust-lang/crates.io-index)" = "e0aa9b8b2ad45dfb0ba14e9e507c90e81b6f271a426f3795e1e4f17f25e015d2"
"#;
        let entries = SbomGenerator::parse_lock_content(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].package_name, "serde");
        assert_eq!(entries[0].version, "1.0.193");
    }

    #[test]
    fn test_parse_multiple_packages() {
        let content = r#"[[package]]
name = "serde"
version = "1.0.193"

[[package]]
name = "tokio"
version = "1.34.0"

[[package]]
name = "serde_json"
version = "1.0.108"
"#;
        let entries = SbomGenerator::parse_lock_content(content);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].package_name, "serde");
        assert_eq!(entries[1].package_name, "tokio");
        assert_eq!(entries[2].package_name, "serde_json");
    }

    #[test]
    fn test_sbom_entry_serialization() {
        let entry = SbomEntry {
            package_name: "serde".to_string(),
            version: "1.0.193".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
            hashes: HashMap::from([("sha256".to_string(), "abc123".to_string())]),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let decoded: SbomEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.package_name, "serde");
        assert_eq!(decoded.hashes.len(), 1);
    }
}
