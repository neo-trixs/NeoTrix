//! 结构化数据读写 (D5/D6): JSON/YAML 读写 + ContentSnapshot 快照存读。

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::encoding::decode_bytes;
use super::types::{ContentSnapshot, FileAbilityError, Result};

/// 结构化文件读取结果 — 统一 JSON/YAML 为 serde_json::Value
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredData {
    pub format: String,
    pub value: serde_json::Value,
}

/// 读取 JSON/YAML 结构化文件 (D5)
pub fn read_structured(path: impl AsRef<Path>) -> Result<StructuredData> {
    let raw = std::fs::read(path.as_ref()).map_err(FileAbilityError::Io)?;
    let text = decode_bytes(&raw);
    let ext = path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "json" | "jsonc" => Ok(StructuredData {
            format: "json".to_string(),
            value: serde_json::from_str(&text)
                .map_err(|e| FileAbilityError::Parse(e.to_string()))?,
        }),
        "yaml" | "yml" => {
            let v: noyalib::compat::serde_yaml::Value = noyalib::compat::serde_yaml::from_str(&text)
                .map_err(|e| FileAbilityError::Parse(e.to_string()))?;
            let value = serde_json::to_value(v)
                .map_err(|e| FileAbilityError::Parse(e.to_string()))?;
            Ok(StructuredData {
                format: "yaml".to_string(),
                value,
            })
        }
        other => Err(FileAbilityError::UnsupportedFormat {
            ext: other.to_string(),
        }),
    }
}

/// 写入 JSON 文件 (D5)
pub fn write_json(path: impl AsRef<Path>, value: &serde_json::Value, pretty: bool) -> Result<()> {
    let text = if pretty {
        serde_json::to_string_pretty(value)
            .map_err(|e| FileAbilityError::Parse(e.to_string()))?
    } else {
        serde_json::to_string(value)
            .map_err(|e| FileAbilityError::Parse(e.to_string()))?
    };
    std::fs::write(path.as_ref(), text).map_err(FileAbilityError::Io)
}

/// 将 ContentSnapshot 持久化为 JSON 文件 (D6)
pub fn store_snapshot(snapshot: &ContentSnapshot, target: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| FileAbilityError::Parse(e.to_string()))?;
    std::fs::write(target.as_ref(), json).map_err(FileAbilityError::Io)
}

/// 从 JSON 文件加载 ContentSnapshot (D6)
pub fn load_snapshot(path: impl AsRef<Path>) -> Result<ContentSnapshot> {
    let raw = std::fs::read(path.as_ref()).map_err(FileAbilityError::Io)?;
    serde_json::from_slice(&raw).map_err(|e| FileAbilityError::Parse(e.to_string()))
}