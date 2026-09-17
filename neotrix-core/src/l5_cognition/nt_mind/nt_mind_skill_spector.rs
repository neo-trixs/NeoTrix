#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// SkillSpector — SKILL.md 合约验证器
pub struct SkillSpector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSpecReport {
    pub valid: bool,
    pub errors: Vec<String>,
    pub content_hash: String,
}

impl SkillSpector {
    /// 验证 SKILL-SPEC.md 合约
    pub fn validate_spec(content: &str) -> Result<SkillSpecReport, String> {
        let mut errors = Vec::new();
        let hash = Self::content_hash(content);

        // 检查必要字段
        if !content.contains("name:") && !content.contains("# ") {
            errors.push("Missing required field: name".into());
        }
        if !content.contains("description:") {
            errors.push("Missing required field: description".into());
        }
        if !content.contains("version:") {
            errors.push("Missing required field: version".into());
        }

        // 检查内容长度
        if content.trim().is_empty() {
            errors.push("Content is empty".into());
        }

        // 检查是否有超过200行（SKILL-SPEC.md 合约限制）
        let line_count = content.lines().count();
        if line_count > 200 {
            errors.push(format!("Content exceeds 200 lines (found {})", line_count));
        }

        Ok(SkillSpecReport {
            valid: errors.is_empty(),
            errors,
            content_hash: hash,
        })
    }

    /// 计算内容哈希 (SHA-256)
    pub fn content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_spec_valid() {
        let spec = "name: test-skill\n\
                     description: A test skill\n\
                     version: 1.0.0\n\n\
                     # Test Skill\n\
                     This is a test skill.";
        let report = SkillSpector::validate_spec(spec).unwrap();
        assert!(report.valid);
        assert!(report.errors.is_empty());
        assert!(!report.content_hash.is_empty());
    }

    #[test]
    fn test_validate_spec_missing_fields() {
        let spec = "# Just a title\n\
                     Some content without required fields.";
        let report = SkillSpector::validate_spec(spec).unwrap();
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("name")));
        assert!(report.errors.iter().any(|e| e.contains("description")));
        assert!(report.errors.iter().any(|e| e.contains("version")));
    }

    #[test]
    fn test_validate_spec_empty() {
        let report = SkillSpector::validate_spec("").unwrap();
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_content_hash_deterministic() {
        let content = "test content";
        let hash1 = SkillSpector::content_hash(content);
        let hash2 = SkillSpector::content_hash(content);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex output length
    }

    #[test]
    fn test_content_hash_different_content() {
        let hash1 = SkillSpector::content_hash("content A");
        let hash2 = SkillSpector::content_hash("content B");
        assert_ne!(hash1, hash2);
    }
}
