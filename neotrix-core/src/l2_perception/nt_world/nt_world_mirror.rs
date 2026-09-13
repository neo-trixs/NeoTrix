//! HuggingFace mirror speed profiling and adaptive resolution.
//!
//! Moved from `nt_media::streaming` (L1 Action) to NT-WORLD (L2 Perception)
//! because endpoint performance knowledge is world/perception domain, not action.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static MIRROR_SPEED_MAP: LazyLock<Mutex<HashMap<String, f64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MIRROR_ENDPOINTS: &[&str] = &["https://hf-mirror.com", "https://huggingface.co"];

/// Record mirror speed using exponential moving average (α=0.3).
pub fn record_mirror_speed(endpoint: &str, bytes_per_sec: f64) {
    if let Ok(mut map) = MIRROR_SPEED_MAP.lock() {
        let entry = map.entry(endpoint.to_string()).or_insert(0.0);
        *entry = 0.7 * *entry + 0.3 * bytes_per_sec;
    }
}

/// Return mirrors sorted by speed (fastest first).
pub fn ranked_mirrors() -> Vec<(String, f64)> {
    let map = MIRROR_SPEED_MAP.lock().unwrap();
    let mut pairs: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    pairs
}

/// Inline HuggingFace URL check (moved from nt_media::router to avoid circular dep).
fn is_huggingface_url(url: &str) -> bool {
    url.contains("huggingface.co") || url.contains("hf-mirror.com")
}

/// Resolve HuggingFace URL to fastest mirror. Returns original if not HF.
pub async fn resolve_mirror(client: &reqwest::Client, original_url: &str) -> String {
    if !is_huggingface_url(original_url) || original_url.contains("hf-mirror.com") {
        return original_url.to_string();
    }

    // Environment variable override
    if let Ok(endpoint) = std::env::var("NT_DOWNLOAD_MIRROR_ENDPOINT") {
        let ep = endpoint.trim();
        if !ep.is_empty() {
            let ep = if ep.starts_with("http") {
                ep.to_string()
            } else {
                format!("https://{}", ep)
            };
            let forced = original_url
                .replace("https://huggingface.co", &ep)
                .replace("http://huggingface.co", &ep);
            if forced != original_url {
                return forced;
            }
        }
    }

    // Sort endpoints by known speed
    let speed_ranking = ranked_mirrors();
    let mut endpoints: Vec<&str> = MIRROR_ENDPOINTS.to_vec();
    if !speed_ranking.is_empty() {
        endpoints.sort_by(|a, b| {
            let sa = speed_ranking
                .iter()
                .find(|(k, _)| k.contains(a.trim_start_matches("https://")))
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            let sb = speed_ranking
                .iter()
                .find(|(k, _)| k.contains(b.trim_start_matches("https://")))
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // Build candidate list
    let mut candidates: Vec<String> = endpoints
        .iter()
        .filter_map(|ep| {
            let c = original_url
                .replace("https://huggingface.co", ep)
                .replace("http://huggingface.co", ep);
            if c != original_url {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    candidates.push(original_url.to_string());

    // Parallel HEAD probe — first success wins
    let mut handles = Vec::new();
    for url in &candidates {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            match tokio::time::timeout(Duration::from_secs(2), client.head(&url).send()).await {
                Ok(Ok(resp)) if resp.status().is_success() => Some(url),
                _ => None,
            }
        }));
    }
    for h in handles {
        if let Ok(Some(url)) = h.await {
            return url;
        }
    }
    original_url.to_string()
}
