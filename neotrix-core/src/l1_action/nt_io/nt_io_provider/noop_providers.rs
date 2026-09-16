#![forbid(unsafe_code)]

//! Null Object pattern implementations for LLM providers.
//!
//! - `NoOpLlmProvider` — returns empty stubs; for testing and offline mode.
//! - `MutedLlmProvider` — wraps a real provider, logs requests but does not execute.
//! - `CachedLlmProvider` — wraps a real provider with in-memory response caching.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::mpsc;

use super::common::types::{
    DataTrust, FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage,
};

// ── NoOp ────────────────────────────────────────────────────────────────────

/// NoOp LLM Provider — returns empty stubs. For testing and offline mode.
pub struct NoOpLlmProvider;

#[async_trait]
impl LlmProvider for NoOpLlmProvider {
    fn data_trust(&self) -> DataTrust {
        DataTrust::Untrusted
    }

    fn set_proxy(&mut self, _proxy_url: &str) {}

    async fn complete_raw(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Ok(LlmResponse {
            content: String::new(),
            model: "noop".into(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
            tool_calls: None,
            reasoning: None,
        })
    }

    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let (tx, rx) = mpsc::channel(4);
        let resp = LlmResponse {
            content: String::new(),
            model: request.model.clone(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
            tool_calls: None,
            reasoning: None,
        };
        let _ = tx.try_send(Ok(resp));
        Ok(rx)
    }
}

// ── Muted ───────────────────────────────────────────────────────────────────

/// Muted LLM Provider — wraps a real provider, logs requests but does not execute.
///
/// Every `complete_raw` / `stream_complete_raw` call is recorded in the internal
/// log vector without being forwarded to the inner provider. Useful for debugging
/// request shapes and auditing prompts without incurring token costs.
pub struct MutedLlmProvider {
    inner: Box<dyn LlmProvider + Send + Sync>,
    log: Mutex<Vec<LlmRequest>>,
}

impl MutedLlmProvider {
    pub fn new(inner: Box<dyn LlmProvider + Send + Sync>) -> Self {
        Self {
            inner,
            log: Mutex::new(Vec::new()),
        }
    }

    /// Returns a snapshot of all captured requests.
    pub fn captured_requests(&self) -> Vec<LlmRequest> {
        self.log
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Returns the number of captured requests.
    pub fn request_count(&self) -> usize {
        self.log
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .len()
    }

    /// Clears the request log.
    pub fn clear_log(&self) {
        self.log
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

#[async_trait]
impl LlmProvider for MutedLlmProvider {
    fn data_trust(&self) -> DataTrust {
        self.inner.data_trust()
    }

    fn set_proxy(&mut self, proxy_url: &str) {
        self.inner.set_proxy(proxy_url);
    }

    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.log
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(request.clone());
        Ok(LlmResponse {
            content: String::new(),
            model: request.model.clone(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
            tool_calls: None,
            reasoning: None,
        })
    }

    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        self.log
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(request.clone());
        let (tx, rx) = mpsc::channel(4);
        let resp = LlmResponse {
            content: String::new(),
            model: request.model.clone(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
            tool_calls: None,
            reasoning: None,
        };
        let _ = tx.try_send(Ok(resp));
        Ok(rx)
    }
}

// ── Cached ──────────────────────────────────────────────────────────────────

/// Cache key derived from a request's model + messages + temperature.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    model: String,
    messages_hash: u64,
    temperature: Option<u32>,
    max_tokens: u32,
}

impl CacheKey {
    fn from_request(req: &LlmRequest) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for msg in &req.messages {
            msg.role.hash(&mut hasher);
            msg.content.hash(&mut hasher);
        }
        Self {
            model: req.model.clone(),
            messages_hash: hasher.finish(),
            temperature: req.temperature.map(|t| (t * 100.0) as u32),
            max_tokens: req.max_tokens,
        }
    }
}

/// Cached LLM Provider — wraps a real provider with in-memory response caching.
///
/// Identical requests (same model + messages + temperature + max_tokens) return
/// the cached response without calling the inner provider. The cache is bounded
/// by `capacity`; when full, the oldest entry is evicted (FIFO).
pub struct CachedLlmProvider {
    inner: Box<dyn LlmProvider + Send + Sync>,
    cache: Mutex<HashMap<CacheKey, LlmResponse>>,
    capacity: usize,
    hit_count: Mutex<u64>,
    miss_count: Mutex<u64>,
}

impl CachedLlmProvider {
    pub fn new(inner: Box<dyn LlmProvider + Send + Sync>, capacity: usize) -> Self {
        Self {
            inner,
            cache: Mutex::new(HashMap::with_capacity(capacity)),
            capacity,
            hit_count: Mutex::new(0),
            miss_count: Mutex::new(0),
        }
    }

    /// Returns (cache_hits, cache_misses).
    pub fn stats(&self) -> (u64, u64) {
        (
            *self.hit_count.lock().unwrap_or_else(|e| e.into_inner()),
            *self.miss_count.lock().unwrap_or_else(|e| e.into_inner()),
        )
    }

    /// Clears the cache and resets stats.
    pub fn reset(&self) {
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        *self.hit_count.lock().unwrap_or_else(|e| e.into_inner()) = 0;
        *self.miss_count.lock().unwrap_or_else(|e| e.into_inner()) = 0;
    }

    fn evict_if_needed(cache: &mut HashMap<CacheKey, LlmResponse>, capacity: usize) {
        if cache.len() >= capacity {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }
    }
}

#[async_trait]
impl LlmProvider for CachedLlmProvider {
    fn data_trust(&self) -> DataTrust {
        self.inner.data_trust()
    }

    fn set_proxy(&mut self, proxy_url: &str) {
        self.inner.set_proxy(proxy_url);
    }

    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let key = CacheKey::from_request(request);
        {
            let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = cache.get(&key) {
                *self.hit_count.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                return Ok(cached.clone());
            }
        }
        *self.miss_count.lock().unwrap_or_else(|e| e.into_inner()) += 1;
        let response = self.inner.complete_raw(request).await?;
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        Self::evict_if_needed(&mut cache, self.capacity);
        cache.insert(key, response.clone());
        Ok(response)
    }

    async fn stream_complete_raw(
        &self,
        request: &LlmRequest,
    ) -> Result<mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let key = CacheKey::from_request(request);
        {
            let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = cache.get(&key) {
                *self.hit_count.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                let (tx, rx) = mpsc::channel(4);
                let _ = tx.try_send(Ok(cached.clone()));
                return Ok(rx);
            }
        }
        *self.miss_count.lock().unwrap_or_else(|e| e.into_inner()) += 1;
        // For streaming, cache the full response once the stream completes.
        let inner_rx = self.inner.stream_complete_raw(request).await?;
        let (proxy_tx, proxy_rx) = mpsc::channel(64);
        let cache = self.cache.clone();
        let capacity = self.capacity;
        tokio::spawn(async move {
            let mut full_content = String::new();
            let mut final_usage = Usage::default();
            let mut final_finish = FinishReason::Unknown;
            let mut final_tool_calls = None;
            let mut final_reasoning = None;
            let mut final_model = String::new();
            while let Some(chunk) = inner_rx.recv().await {
                match chunk {
                    Ok(resp) => {
                        full_content.push_str(&resp.content);
                        final_model = resp.model;
                        final_usage = resp.usage;
                        final_finish = resp.finish_reason;
                        if resp.tool_calls.is_some() {
                            final_tool_calls = resp.tool_calls;
                        }
                        if resp.reasoning.is_some() {
                            final_reasoning = resp.reasoning;
                        }
                        let _ = proxy_tx.send(Ok(resp)).await;
                    }
                    Err(e) => {
                        let _ = proxy_tx.send(Err(e)).await;
                        return;
                    }
                }
            }
            // Cache the assembled full response.
            let assembled = LlmResponse {
                content: full_content,
                model: final_model,
                usage: final_usage,
                finish_reason: final_finish,
                tool_calls: final_tool_calls,
                reasoning: final_reasoning,
            };
            let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
            Self::evict_if_needed(&mut c, capacity);
            c.insert(key, assembled);
        });
        Ok(proxy_rx)
    }
}
