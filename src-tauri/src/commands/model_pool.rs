//! Model Pool — 对接 neotrix-core ProviderPool/GatewayV2
//!
//! 提供模型池的增删查状态查询，前端通过 domain_call 或直接调用。

use serde::{Deserialize, Serialize};

/// 模型池条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPoolEntry {
    pub label: String,
    pub provider: String,
    pub api_key_masked: String,
    pub model: String,
    pub tags: Vec<String>,
    pub base_url: Option<String>,
    pub created_ts: u64,
}

/// 模型池状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPoolStatus {
    pub total: usize,
    pub active: usize,
    pub providers: Vec<ModelPoolEntry>,
    pub config_path: String,
}

/// 从 provider_pool.toml 读取条目（脱敏 api_key）
fn load_pool_entries() -> Result<Vec<ModelPoolEntry>, String> {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("neotrix")
        .join("provider_pool.toml");

    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&path).map_err(|e| format!("Read provider pool: {e}"))?;

    #[derive(Deserialize)]
    struct RawPool {
        entries: Vec<RawEntry>,
    }

    #[derive(Deserialize)]
    struct RawEntry {
        label: String,
        provider: String,
        api_key: String,
        model: String,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        created_ts: Option<u64>,
    }

    let pool: RawPool =
        toml::from_str(&content).map_err(|e| format!("Parse provider pool: {e}"))?;

    let entries = pool
        .entries
        .into_iter()
        .map(|e| {
            // 脱敏: 只显示前 8 位
            let masked = if e.api_key.len() > 8 {
                format!(
                    "{}...{}",
                    &e.api_key[..4],
                    &e.api_key[e.api_key.len() - 4..]
                )
            } else if e.api_key.starts_with("env:") {
                e.api_key.clone()
            } else {
                "****".into()
            };
            ModelPoolEntry {
                label: e.label,
                provider: e.provider,
                api_key_masked: masked,
                model: e.model,
                tags: e.tags,
                base_url: e.base_url,
                created_ts: e.created_ts.unwrap_or(0),
            }
        })
        .collect();

    Ok(entries)
}

/// 读取 pool 配置文件原始内容（用于写入）
fn read_pool_raw() -> Result<String, String> {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("neotrix")
        .join("provider_pool.toml");

    if !path.exists() {
        return Ok("[[entries]]\n".into());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("Read provider pool: {e}"))
}

/// 写入 pool 配置文件
fn write_pool_raw(content: &str) -> Result<(), String> {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("neotrix")
        .join("provider_pool.toml");

    // 确保目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create config dir: {e}"))?;
    }

    std::fs::write(&path, content).map_err(|e| format!("Write provider pool: {e}"))
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取模型池状态
#[tauri::command]
pub async fn model_pool_status() -> Result<ModelPoolStatus, String> {
    let entries = load_pool_entries()?;
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("neotrix")
        .join("provider_pool.toml")
        .to_string_lossy()
        .to_string();

    Ok(ModelPoolStatus {
        total: entries.len(),
        active: entries.len(), // 所有注册的条目都是 active
        providers: entries,
        config_path,
    })
}

/// 添加模型提供者
#[tauri::command]
pub async fn model_pool_add(
    label: String,
    provider: String,
    api_key: String,
    model: String,
    tags: Vec<String>,
    base_url: Option<String>,
) -> Result<ModelPoolEntry, String> {
    let mut raw = read_pool_raw()?;

    // 追加新条目
    let new_entry = format!(
        r#"

[[entries]]
label = "{label}"
provider = "{provider}"
api_key = "{api_key}"
model = "{model}"
tags = [{tags}]
base_url = {base_url}
created_ts = {created_ts}"#,
        created_ts = chrono::Utc::now().timestamp() as u64,
        label = label,
        provider = provider,
        api_key = api_key,
        model = model,
        tags = tags
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", "),
        base_url = match &base_url {
            Some(url) => format!("\"{}\"", url),
            None => "null".into(),
        },
    );

    raw.push_str(&new_entry);
    write_pool_raw(&raw)?;

    // 返回脱敏版本
    let masked = if api_key.len() > 8 {
        format!("{}...{}", &api_key[..4], &api_key[api_key.len() - 4..])
    } else {
        "****".into()
    };

    Ok(ModelPoolEntry {
        label,
        provider,
        api_key_masked: masked,
        model,
        tags,
        base_url,
        created_ts: chrono::Utc::now().timestamp() as u64,
    })
}

/// 删除模型提供者
#[tauri::command]
pub async fn model_pool_remove(label: String) -> Result<bool, String> {
    let raw = read_pool_raw()?;
    let lines: Vec<&str> = raw.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_until_next = false;
    let mut found = false;

    for line in &lines {
        if line.trim().starts_with("[[entries]]") {
            skip_until_next = false;
            // 检查下一个 entry 的 label
            continue;
        }
        if skip_until_next {
            continue;
        }
        new_lines.push(*line);
    }

    // 更精确的解析: 找到 label 匹配的 entry 并删除
    let mut result = String::new();
    let mut in_entry = false;
    let mut current_entry = String::new();
    let mut skip_entry = false;

    for line in raw.lines() {
        if line.trim().starts_with("[[entries]]") {
            // 处理前一个 entry
            if in_entry && !skip_entry {
                result.push_str(&current_entry);
                result.push('\n');
            }
            in_entry = true;
            current_entry = line.to_string();
            current_entry.push('\n');
            skip_entry = false;
            continue;
        }

        if in_entry {
            if line.contains(&format!("label = \"{}\"", label)) {
                skip_entry = true;
                found = true;
            }
            current_entry.push_str(line);
            current_entry.push('\n');
        }
    }

    // 处理最后一个 entry
    if in_entry && !skip_entry {
        result.push_str(&current_entry);
    }

    if !found {
        return Ok(false);
    }

    write_pool_raw(&result)?;
    Ok(true)
}

/// 更新模型提供者的 api_key
#[tauri::command]
pub async fn model_pool_update_key(label: String, new_api_key: String) -> Result<bool, String> {
    let raw = read_pool_raw()?;
    let mut result = String::new();
    let mut in_entry = false;
    let mut found = false;
    let mut skip_old_key = false;

    for line in raw.lines() {
        if line.trim().starts_with("[[entries]]") {
            in_entry = true;
            skip_old_key = false;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_entry && line.contains(&format!("label = \"{}\"", label)) {
            found = true;
        }

        if found && line.trim().starts_with("api_key") && !skip_old_key {
            result.push_str(&format!("api_key = \"{}\"", new_api_key));
            result.push('\n');
            skip_old_key = true;
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !found {
        return Ok(false);
    }

    write_pool_raw(&result)?;
    Ok(true)
}

/// 检查模型提供者 API 连通性
#[tauri::command]
pub async fn model_pool_check(label: String) -> Result<String, String> {
    let entries = load_pool_entries()?;
    let entry = entries
        .iter()
        .find(|e| e.label == label)
        .ok_or_else(|| format!("Provider '{}' not found", label))?;

    // 简单连通性检查
    let base_url = entry
        .base_url
        .as_deref()
        .unwrap_or(match entry.provider.as_str() {
            "openai" => "https://api.openai.com",
            "anthropic" => "https://api.anthropic.com",
            "ollama" => "http://localhost:11434",
            "siliconflow" => "https://api.siliconflow.cn",
            _ => "https://api.openai.com",
        });

    match reqwest::Client::new()
        .get(&format!("{}/v1/models", base_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok("connected".into())
            } else {
                Ok(format!("http_{}", resp.status().as_u16()))
            }
        }
        Err(e) => Ok(format!("error: {}", e)),
    }
}
