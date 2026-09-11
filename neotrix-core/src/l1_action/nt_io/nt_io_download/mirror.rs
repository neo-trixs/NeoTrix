/// HuggingFace 镜像支持
/// 
/// 自动检测 HuggingFace URL 并尝试镜像加速:
/// - 国内环境: hf-mirror.com (CDN 加速)
/// - 默认: huggingface.co (直连)
///
/// 镜像 URL 转换规则:
///   https://huggingface.co/{repo}/resolve/main/{file}
///   → https://hf-mirror.com/{repo}/resolve/main/{file}

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

/// 镜像速度画像: 记录每个端点的历史吞吐量 (bytes/s)
/// 用于 adaptive URI 选择 (aria2 --uri-selector=adaptive)
static MIRROR_SPEED_MAP: LazyLock<Mutex<HashMap<String, f64>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

/// 记录镜像速度 (下载完成后调用)
pub fn record_mirror_speed(endpoint: &str, bytes_per_sec: f64) {
    if let Ok(mut map) = MIRROR_SPEED_MAP.lock() {
        let entry = map.entry(endpoint.to_string()).or_insert(0.0);
        // 指数移动平均: new = 0.7 * old + 0.3 * current
        *entry = 0.7 * *entry + 0.3 * bytes_per_sec;
    }
}

/// 获取镜像速度排名 (最快优先)
pub fn ranked_mirrors() -> Vec<(String, f64)> {
    let map = MIRROR_SPEED_MAP.lock().unwrap();
    let mut pairs: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    pairs
}

/// 镜像端点列表 (按优先级排列)
const MIRROR_ENDPOINTS: &[&str] = &[
    "https://hf-mirror.com",
    "https://huggingface.co",
];

/// 检查 URL 是否为 HuggingFace 源
pub fn is_huggingface_url(url: &str) -> bool {
    url.contains("huggingface.co") || url.contains("hf-mirror.com")
}

/// 尝试所有镜像端点, 返回第一个可用的 URL
/// 通过 HEAD 请求探测可用性 (2s 超时)
pub async fn resolve_mirror_url(client: &reqwest::Client, original_url: &str) -> String {
    if !is_huggingface_url(original_url) {
        return original_url.to_string();
    }

    // 如果已经是镜像 URL，直接返回
    if original_url.contains("hf-mirror.com") {
        return original_url.to_string();
    }

    // 环境变量覆盖: NT_DOWNLOAD_MIRROR_ENDPOINT=hf-mirror.com
    if let Ok(endpoint) = std::env::var("NT_DOWNLOAD_MIRROR_ENDPOINT") {
        let endpoint = endpoint.trim().to_string();
        if !endpoint.is_empty() {
            let endpoint = if endpoint.starts_with("http") {
                endpoint
            } else {
                format!("https://{}", endpoint)
            };
            let forced = original_url
                .replace("https://huggingface.co", &endpoint)
                .replace("http://huggingface.co", &endpoint);
            if forced != original_url {
                eprintln!("[mirror] env override: {}", endpoint);
                return forced;
            }
        }
    }

    // 按历史速度排序镜像端点 (adaptive selector)
    let speed_ranking = ranked_mirrors();
    let mut sorted_endpoints: Vec<&str> = MIRROR_ENDPOINTS.to_vec();
    if !speed_ranking.is_empty() {
        sorted_endpoints.sort_by(|a, b| {
            let speed_a = speed_ranking.iter().find(|(k, _)| k.contains(a.trim_start_matches("https://"))).map(|(_, v)| *v).unwrap_or(0.0);
            let speed_b = speed_ranking.iter().find(|(k, _)| k.contains(b.trim_start_matches("https://"))).map(|(_, v)| *v).unwrap_or(0.0);
            speed_b.partial_cmp(&speed_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // 生成候选 URL 列表
    let mut candidates = Vec::new();
    for endpoint in &sorted_endpoints {
        let candidate = original_url
            .replace("https://huggingface.co", endpoint)
            .replace("http://huggingface.co", endpoint);
        if candidate != original_url {
            candidates.push(candidate);
        }
    }
    // 原始 URL 作为 fallback
    if !candidates.contains(&original_url.to_string()) {
        candidates.push(original_url.to_string());
    }

    // 并行探测, 选择第一个可用的
    let mut handles = Vec::new();
    for url in &candidates {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                client.head(&url).send(),
            ).await {
                Ok(Ok(resp)) if resp.status().is_success() => Some(url),
                _ => None,
            }
        }));
    }

    for handle in handles {
        if let Ok(Some(url)) = handle.await {
            eprintln!("[mirror] using: {}", url);
            return url;
        }
    }

    // 全部失败, 返回原始 URL
    original_url.to_string()
}

/// 从 URL 提取仓库名和文件名
pub fn parse_hf_url(url: &str) -> Option<(String, String)> {
    // https://huggingface.co/owner/repo/resolve/main/file.gguf
    // 或 https://hf-mirror.com/owner/repo/resolve/main/file.gguf
    let path = url.split("://").nth(1)?;
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 5 {
        return None;
    }
    // skip domain, find "resolve" keyword
    let resolve_idx = parts.iter().position(|p| *p == "resolve")?;
    if resolve_idx + 2 >= parts.len() {
        return None;
    }
    let repo = parts[1..resolve_idx].join("/");
    let file = parts[resolve_idx + 2..].join("/");
    Some((repo, file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_huggingface_url() {
        assert!(is_huggingface_url("https://huggingface.co/repo/resolve/main/f.gguf"));
        assert!(is_huggingface_url("https://hf-mirror.com/repo/resolve/main/f.gguf"));
        assert!(!is_huggingface_url("https://github.com/user/repo"));
    }

    #[test]
    fn test_parse_hf_url() {
        let (repo, file) = parse_hf_url(
            "https://huggingface.co/HauhauCS/Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-MTP-GGUF/resolve/main/Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-IQ4_XS.gguf"
        ).unwrap();
        assert_eq!(repo, "HauhauCS/Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-MTP-GGUF");
        assert_eq!(file, "Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-IQ4_XS.gguf");
    }
}
