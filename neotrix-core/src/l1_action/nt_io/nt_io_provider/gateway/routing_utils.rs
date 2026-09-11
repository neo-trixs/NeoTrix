use std::collections::HashMap;
use std::sync::RwLock;

use std::time::Duration;

use super::GatewayV2;
use crate::core::nt_core_llm::{LlmError, LlmRequest, LlmResponse};

// ═══════════════════════════════════════════════════════════════════
// Consistent Hash — 一致性哈希环 (基于虚拟节点的负载均衡)
// ═══════════════════════════════════════════════════════════════════

/// 一致性哈希环 — 基于虚拟节点的负载均衡
pub struct ConsistentHash {
    ring: RwLock<Vec<(u64, String)>>,
    replicas: u32,
}

impl ConsistentHash {
    pub fn new(replicas: u32) -> Self {
        Self {
            ring: RwLock::new(Vec::new()),
            replicas,
        }
    }

    fn hash(key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    pub fn add_node(&self, node: &str) {
        let mut ring = self.ring.write().unwrap();
        for i in 0..self.replicas {
            let key = format!("{}#{}", node, i);
            let hash = Self::hash(&key);
            ring.push((hash, node.to_string()));
        }
        ring.sort_by_key(|(h, _)| *h);
    }

    pub fn remove_node(&self, node: &str) {
        let mut ring = self.ring.write().unwrap();
        ring.retain(|(_, n)| n != node);
    }

    pub fn get_node(&self, key: &str) -> Option<String> {
        let ring = self.ring.read().unwrap();
        if ring.is_empty() {
            return None;
        }
        let hash = Self::hash(key);
        let pos = ring.partition_point(|(h, _)| *h < hash);
        Some(ring[pos % ring.len()].1.clone())
    }

    pub fn get_distribution(&self, keys: &[String]) -> HashMap<String, usize> {
        let mut dist: HashMap<String, usize> = HashMap::new();
        for key in keys {
            if let Some(node) = self.get_node(key) {
                *dist.entry(node).or_insert(0) += 1;
            }
        }
        dist
    }
}

// ═══════════════════════════════════════════════════════════════════
// Keyless Routing — 匿名免费端点路由 (P0, Cycle 160)
// ═══════════════════════════════════════════════════════════════════

impl GatewayV2 {
    /// 有序 keyless 候选 (匿名, 无需 API key)。动态从已注册 provider 中收集
    /// `opencode-zen/*` (E: 实时发现全部 zen 免费模型) 与 `llm7/*` 端点;
    /// 注册表为空 (如单测) 时回退静态已知匿名端点。
    pub fn keyless_candidates(&self) -> Vec<String> {
        if let Ok(guard) = self.providers.read() {
            let mut v: Vec<String> = guard
                .keys()
                .filter(|k| k.starts_with("opencode-zen/") || k.starts_with("llm7/"))
                .cloned()
                .collect();
            if !v.is_empty() {
                v.sort();
                return v;
            }
        }
        vec![
            "opencode-zen/big-pickle".into(),
            "opencode-zen/mimo-v2.5-free".into(),
            "llm7/codestral-latest".into(),
        ]
    }

    /// 单 provider RateLimit 退避重试: 遇 `RateLimit` 按 `retry_after` (默认 1s)
    /// 退避, 最多重试 `MAX_ATTEMPTS` 次; 其它错误立即透传。
    pub(super) async fn call_provider_backoff(
        &self,
        name: &str,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempt = 0u32;
        loop {
            match self.call_provider(name, request).await {
                Ok(resp) => return Ok(resp),
                Err(LlmError::RateLimit(msg)) => {
                    if attempt >= MAX_ATTEMPTS {
                        return Err(LlmError::RateLimit(msg));
                    }
                    let backoff = parse_retry_after(&msg).unwrap_or(1.0);
                    tokio::time::sleep(Duration::from_secs_f32(backoff)).await;
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 跨 keyless 候选路由: 依次尝试 `keyless_candidates`, 每个候选经
    /// `call_provider_backoff` 退避重试; 全部失败返回最后一个错误。
    pub async fn route_keyless(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let candidates = self.keyless_candidates();
        let mut last_err = LlmError::Unknown("no keyless candidates configured".into());
        for cand in candidates {
            match self.call_provider_backoff(&cand, request).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    last_err = e;
                    continue;
                }
            }
        }
        Err(last_err)
    }
}

/// 从 RateLimit 错误 JSON 提取 `retry_after` (秒)。解析失败回退 1.0s。
fn parse_retry_after(msg: &str) -> Option<f32> {
    let v: serde_json::Value = serde_json::from_str(msg).ok()?;
    v.get("error")?
        .get("retry_after")?
        .as_f64()
        .map(|x| x as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_distribution() {
        let ch = ConsistentHash::new(100);
        ch.add_node("node_a");
        ch.add_node("node_b");
        ch.add_node("node_c");

        let keys: Vec<String> = (0..1000).map(|i| format!("key_{}", i)).collect();
        let dist = ch.get_distribution(&keys);
        assert_eq!(dist.len(), 3);
        for count in dist.values() {
            assert!(*count > 100, "distribution too uneven: {:?}", dist);
        }
    }

    #[test]
    fn test_node_removal() {
        let ch = ConsistentHash::new(100);
        ch.add_node("node_a");
        ch.add_node("node_b");

        let before = ch.get_node("test_key").unwrap();
        ch.remove_node(&before);
        let after = ch.get_node("test_key").unwrap();
        assert_ne!(before, after);
    }
}
