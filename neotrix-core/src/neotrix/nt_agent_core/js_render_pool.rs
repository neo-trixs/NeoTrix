#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq)]
pub enum RenderStatus {
    Idle,
    Busy,
    Crashed,
    Starting,
}

#[derive(Debug, Clone)]
pub struct RenderInstance {
    pub id: u64,
    pub session_manager_id: String,
    pub fingerprint: String,
    pub proxy: Option<String>,
    pub status: RenderStatus,
    pub created_at: Instant,
    pub last_used: Instant,
    pub crash_count: u32,
}

#[derive(Debug, Clone)]
pub struct RenderInstanceRef {
    pub instance_id: u64,
    pub session_ws: String,
    pub fingerprint: String,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenderPoolHealth {
    pub active: usize,
    pub total: usize,
    pub crashed: usize,
    pub avg_request_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RenderPoolStats {
    pub total_requests: u64,
    pub active_requests: u64,
    pub instances_created: u64,
    pub instances_crashed: u64,
    pub avg_lifetime_secs: f64,
}

pub struct JsRenderPool {
    instances: RwLock<Vec<RenderInstance>>,
    max_instances: usize,
    min_instances: usize,
    idle_timeout: Duration,
    request_timeout: Duration,
    active_requests: AtomicU64,
    total_requests: AtomicU64,
    total_created: AtomicU64,
    total_crashed: AtomicU64,
}

impl JsRenderPool {
    pub fn new(max: usize, min: usize, idle_timeout_secs: u64) -> Self {
        Self {
            instances: RwLock::new(Vec::new()),
            max_instances: max,
            min_instances: min,
            idle_timeout: Duration::from_secs(idle_timeout_secs),
            request_timeout: Duration::from_secs(30),
            active_requests: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
            total_created: AtomicU64::new(0),
            total_crashed: AtomicU64::new(0),
        }
    }

    pub async fn acquire(
        &self,
        preferred_fingerprint: Option<&str>,
        preferred_proxy: Option<&str>,
    ) -> Result<RenderInstanceRef, String> {
        let mut instances = self.instances.write().await;

        let find_score = |inst: &RenderInstance| -> u8 {
            if inst.status != RenderStatus::Idle {
                return 0;
            }
            let fp_match = preferred_fingerprint
                .map_or(true, |fp| inst.fingerprint == fp);
            let px_match = preferred_proxy
                .map_or(true, |px| inst.proxy.as_deref() == Some(px));
            match (fp_match, px_match) {
                (true, true) => 3,
                (true, false) => 2,
                (false, _) => 1,
            }
        };

        if let Some(inst) = instances
            .iter_mut()
            .filter(|i| i.status == RenderStatus::Idle)
            .max_by_key(|i| find_score(i))
        {
            if find_score(inst) > 0 {
                inst.status = RenderStatus::Busy;
                inst.last_used = Instant::now();
                self.active_requests.fetch_add(1, Ordering::Relaxed);
                self.total_requests.fetch_add(1, Ordering::Relaxed);
                return Ok(RenderInstanceRef {
                    instance_id: inst.id,
                    session_ws: format!(
                        "ws://localhost:9222/devtools/page/{}",
                        inst.session_manager_id
                    ),
                    fingerprint: inst.fingerprint.clone(),
                    proxy: inst.proxy.clone(),
                });
            }
        }

        if instances.len() < self.max_instances {
            let id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);
            let fingerprint = preferred_fingerprint.unwrap_or("default").to_string();
            let proxy = preferred_proxy.map(String::from);
            let session_manager_id = format!("session_{}", id);

            instances.push(RenderInstance {
                id,
                session_manager_id: session_manager_id.clone(),
                fingerprint: fingerprint.clone(),
                proxy: proxy.clone(),
                status: RenderStatus::Busy,
                created_at: Instant::now(),
                last_used: Instant::now(),
                crash_count: 0,
            });

            self.total_created.fetch_add(1, Ordering::Relaxed);
            self.active_requests.fetch_add(1, Ordering::Relaxed);
            self.total_requests.fetch_add(1, Ordering::Relaxed);

            return Ok(RenderInstanceRef {
                instance_id: id,
                session_ws: format!(
                    "ws://localhost:9222/devtools/page/{}",
                    session_manager_id
                ),
                fingerprint,
                proxy,
            });
        }

        Err("All instances are busy and max_instances reached".to_string())
    }

    pub async fn release(&self, instance_id: u64) {
        let mut instances = self.instances.write().await;
        if let Some(inst) = instances.iter_mut().find(|i| i.id == instance_id) {
            inst.status = RenderStatus::Idle;
            inst.last_used = Instant::now();
            self.active_requests.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub async fn start_instance(&self) -> Result<u64, String> {
        let mut instances = self.instances.write().await;
        if instances.len() >= self.max_instances {
            return Err("Max instances reached".to_string());
        }
        let id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);
        let session_manager_id = format!("session_{}", id);
        instances.push(RenderInstance {
            id,
            session_manager_id,
            fingerprint: "default".to_string(),
            proxy: None,
            status: RenderStatus::Idle,
            created_at: Instant::now(),
            last_used: Instant::now(),
            crash_count: 0,
        });
        self.total_created.fetch_add(1, Ordering::Relaxed);
        Ok(id)
    }

    pub async fn stop_instance(&self, id: u64) -> Result<(), String> {
        let mut instances = self.instances.write().await;
        let pos = instances
            .iter()
            .position(|i| i.id == id)
            .ok_or_else(|| format!("Instance {} not found", id))?;
        instances.remove(pos);
        Ok(())
    }

    pub async fn recycle_crashed(&self) -> usize {
        let mut instances = self.instances.write().await;
        let crashed: Vec<usize> = instances
            .iter()
            .enumerate()
            .filter(|(_, inst)| inst.status == RenderStatus::Crashed)
            .map(|(idx, _)| idx)
            .collect();
        let count = crashed.len();
        for idx in crashed.into_iter().rev() {
            instances.remove(idx);
        }
        self.total_crashed.fetch_add(count as u64, Ordering::Relaxed);
        count
    }

    pub async fn scale_down(&self) -> usize {
        let mut instances = self.instances.write().await;
        let current_count = instances.len();
        if current_count <= self.min_instances {
            return 0;
        }
        let excess = current_count - self.min_instances;
        let idle: Vec<usize> = instances
            .iter()
            .enumerate()
            .filter(|(_, inst)| {
                inst.status == RenderStatus::Idle
                    && inst.last_used.elapsed() > self.idle_timeout
            })
            .map(|(idx, _)| idx)
            .collect();
        let to_remove = idle.len().min(excess);
        for idx in idle.into_iter().rev().take(to_remove) {
            instances.remove(idx);
        }
        to_remove
    }

    pub async fn health(&self) -> RenderPoolHealth {
        let instances = self.instances.read().await;
        let total = instances.len();
        let active = instances
            .iter()
            .filter(|i| {
                i.status == RenderStatus::Busy || i.status == RenderStatus::Starting
            })
            .count();
        let crashed = instances
            .iter()
            .filter(|i| i.status == RenderStatus::Crashed)
            .count();
        RenderPoolHealth {
            active,
            total,
            crashed,
            avg_request_time_ms: 0,
        }
    }

    pub fn stats(&self) -> RenderPoolStats {
        RenderPoolStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            active_requests: self.active_requests.load(Ordering::Relaxed),
            instances_created: self.total_created.load(Ordering::Relaxed),
            instances_crashed: self.total_crashed.load(Ordering::Relaxed),
            avg_lifetime_secs: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// CDP helper functions
// ---------------------------------------------------------------------------

fn extract_bool_result(json_text: &str) -> Option<bool> {
    let v: serde_json::Value = serde_json::from_str(json_text).ok()?;
    v.get("result")?.get("value")?.as_bool()
}

pub async fn wait_for_selector(
    session_ws: &str,
    selector: &str,
    timeout_ms: u64,
) -> Result<(), String> {
    let timeout_dur = Duration::from_millis(timeout_ms);
    let connect_timeout = Duration::from_secs(5);
    let start = Instant::now();

    let ws_stream = tokio::time::timeout(connect_timeout, connect_async(session_ws))
        .await
        .map_err(|_| format!("Connection timeout for CDP endpoint: {}", session_ws))?
        .map_err(|e| format!("WebSocket connection failed: {}", e))?
        .0;

    let (mut write, mut read) = ws_stream.split();
    let mut cmd_id = 0u64;

    loop {
        if start.elapsed() > timeout_dur {
            return Err(format!(
                "Timeout waiting for selector '{}' after {}ms",
                selector, timeout_ms
            ));
        }

        cmd_id += 1;
        let cmd = serde_json::json!({
            "id": cmd_id,
            "method": "Runtime.evaluate",
            "params": {
                "expression": format!(
                    "document.querySelector('{}') !== null",
                    selector
                ),
                "returnByValue": true
            }
        });

        write
            .send(Message::Text(cmd.to_string()))
            .await
            .map_err(|e| format!("Failed to send CDP command: {}", e))?;

        match tokio::time::timeout(Duration::from_millis(100), read.next()).await {
            Ok(Some(Ok(msg))) => {
                let text = msg.to_text().unwrap_or("").to_string();
                if let Some(true) = extract_bool_result(&text) {
                    return Ok(());
                }
            }
            Ok(Some(Err(e))) => return Err(format!("WebSocket error: {}", e)),
            Ok(None) => return Err("WebSocket connection closed".to_string()),
            Err(_) => {}
        }
    }
}

pub async fn wait_for_navigation(
    session_ws: &str,
    timeout_ms: u64,
) -> Result<(), String> {
    let timeout_dur = Duration::from_millis(timeout_ms);
    let connect_timeout = Duration::from_secs(5);

    let ws_stream = tokio::time::timeout(connect_timeout, connect_async(session_ws))
        .await
        .map_err(|_| format!("Connection timeout for CDP endpoint: {}", session_ws))?
        .map_err(|e| format!("WebSocket connection failed: {}", e))?
        .0;

    let (mut write, mut read) = ws_stream.split();

    let enable_cmd = serde_json::json!({
        "id": 1,
        "method": "Page.enable",
        "params": {}
    });
    write
        .send(Message::Text(enable_cmd.to_string()))
        .await
        .map_err(|e| format!("Failed to enable Page domain: {}", e))?;

    loop {
        match tokio::time::timeout(timeout_dur, read.next()).await {
            Ok(Some(Ok(msg))) => {
                let text = msg.to_text().unwrap_or("").to_string();
                if text.contains("\"method\":\"Page.frameStoppedLoading\"") {
                    return Ok(());
                }
            }
            Ok(Some(Err(e))) => return Err(format!("WebSocket error: {}", e)),
            Ok(None) => return Err("WebSocket connection closed".to_string()),
            Err(_) => {
                return Err(format!(
                    "Timeout waiting for navigation after {}ms",
                    timeout_ms
                ));
            }
        }
    }
}

pub async fn wait_for_network_idle(
    session_ws: &str,
    idle_ms: u64,
    timeout_ms: u64,
) -> Result<(), String> {
    let timeout_dur = Duration::from_millis(timeout_ms);
    let idle_dur = Duration::from_millis(idle_ms);
    let connect_timeout = Duration::from_secs(5);
    let start = Instant::now();

    let ws_stream = tokio::time::timeout(connect_timeout, connect_async(session_ws))
        .await
        .map_err(|_| format!("Connection timeout for CDP endpoint: {}", session_ws))?
        .map_err(|e| format!("WebSocket connection failed: {}", e))?
        .0;

    let (mut write, mut read) = ws_stream.split();

    let enable_cmd = serde_json::json!({
        "id": 1,
        "method": "Network.enable",
        "params": {}
    });
    write
        .send(Message::Text(enable_cmd.to_string()))
        .await
        .map_err(|e| format!("Failed to enable Network domain: {}", e))?;

    let mut last_activity = Instant::now();

    loop {
        if start.elapsed() > timeout_dur {
            return Err(format!(
                "Timeout waiting for network idle after {}ms",
                timeout_ms
            ));
        }

        if last_activity.elapsed() >= idle_dur {
            return Ok(());
        }

        match tokio::time::timeout(
            idle_dur.saturating_sub(last_activity.elapsed()),
            read.next(),
        )
        .await
        {
            Ok(Some(Ok(msg))) => {
                let text = msg.to_text().unwrap_or("").to_string();
                if text.contains("\"method\":\"Network.requestWillBeSent\"")
                    || text.contains("\"method\":\"Network.responseReceived\"")
                    || text.contains("\"method\":\"Network.loadingFinished\"")
                {
                    last_activity = Instant::now();
                }
            }
            Ok(Some(Err(e))) => return Err(format!("WebSocket error: {}", e)),
            Ok(None) => return Err("WebSocket connection closed".to_string()),
            Err(_) => continue,
        }
    }
}

pub async fn detect_spa(session_ws: &str) -> Result<bool, String> {
    let connect_timeout = Duration::from_secs(5);

    let ws_stream = tokio::time::timeout(connect_timeout, connect_async(session_ws))
        .await
        .map_err(|_| format!("Connection timeout for CDP endpoint: {}", session_ws))?
        .map_err(|e| format!("WebSocket connection failed: {}", e))?
        .0;

    let (mut write, mut read) = ws_stream.split();

    let checks = vec![
        ("history_api", "typeof window.history.pushState === 'function' && window.history.pushState.toString().indexOf('[native code]') !== -1"),
        ("router_globals", "!!(window.__NUXT__ || window.__NEXT_DATA__ || window.__VUE_OPTIONS__ || window.__INITIAL_STATE__ || window.__PRELOADED_STATE__ || window.__remixContext || window.__reactRouterContext)"),
        ("framework_globals", "!!(window.ReactRouter || window.VueRouter || window.Angular || window.__svelte || window.Backbone || window.Ember || window.__FRAMEWORK_READY__)"),
        ("framework_attrs", "document.querySelectorAll('[data-reactroot], [data-reactid], [data-v-], [ng-], [v-cloak], [x-data]').length > 0"),
        ("history_length", "window.history.length > 1"),
    ];

    for (i, (_, expr)) in checks.iter().enumerate() {
        let cmd = serde_json::json!({
            "id": i as u64 + 1,
            "method": "Runtime.evaluate",
            "params": {
                "expression": expr,
                "returnByValue": true
            }
        });
        write
            .send(Message::Text(cmd.to_string()))
            .await
            .map_err(|e| format!("Failed to send CDP command: {}", e))?;
    }

    let mut positive = 0u32;
    let mut received = 0u32;

    loop {
        if received >= checks.len() as u32 {
            break;
        }
        match tokio::time::timeout(Duration::from_secs(10), read.next()).await {
            Ok(Some(Ok(msg))) => {
                received += 1;
                let text = msg.to_text().unwrap_or("").to_string();
                if let Some(true) = extract_bool_result(&text) {
                    positive += 1;
                }
            }
            Ok(Some(Err(e))) => return Err(format!("WebSocket error: {}", e)),
            Ok(None) => return Err("WebSocket connection closed".to_string()),
            Err(_) => break,
        }
    }

    Ok(positive >= 2)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
    }

    #[test]
    fn test_pool_create_empty() {
        let pool = JsRenderPool::new(5, 1, 60);
        assert_eq!(pool.max_instances, 5);
        assert_eq!(pool.min_instances, 1);
        assert_eq!(pool.idle_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_pool_acquire_creates_instance() {
        let rt = runtime();
        rt.block_on(async {
            let pool = Arc::new(JsRenderPool::new(5, 1, 60));
            let inst = pool.acquire(None, None).await.unwrap();
            assert!(inst.instance_id > 0);
            assert_eq!(inst.fingerprint, "default");
            assert!(inst.session_ws.starts_with("ws://localhost:9222"));
            let health = pool.health().await;
            assert_eq!(health.active, 1);
            assert_eq!(health.total, 1);
        });
    }

    #[test]
    fn test_pool_max_instances_respected() {
        let rt = runtime();
        rt.block_on(async {
            let pool = Arc::new(JsRenderPool::new(2, 0, 60));
            let _a = pool.acquire(None, None).await.unwrap();
            let _b = pool.acquire(None, None).await.unwrap();
            let result = pool.acquire(None, None).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("max_instances"));
        });
    }

    #[test]
    fn test_pool_release_reuses_instance() {
        let rt = runtime();
        rt.block_on(async {
            let pool = Arc::new(JsRenderPool::new(3, 0, 60));
            let inst = pool.acquire(None, None).await.unwrap();
            let id = inst.instance_id;
            pool.release(id).await;

            let inst2 = pool.acquire(None, None).await.unwrap();
            assert_eq!(inst2.instance_id, id, "should reuse released instance");
            assert_eq!(pool.health().await.active, 1);
            assert_eq!(pool.health().await.total, 1);
        });
    }

    #[test]
    fn test_recycle_crashed_detects_none() {
        let rt = runtime();
        rt.block_on(async {
            let pool = Arc::new(JsRenderPool::new(5, 1, 60));
            pool.start_instance().await.unwrap();
            let count = pool.recycle_crashed().await;
            assert_eq!(count, 0);
        });
    }

    #[test]
    fn test_recycle_crashed_with_stats() {
        let rt = runtime();
        rt.block_on(async {
            let pool = Arc::new(JsRenderPool::new(5, 1, 60));
            let id = pool.start_instance().await.unwrap();
            {
                let mut instances = pool.instances.write().await;
                if let Some(inst) = instances.iter_mut().find(|i| i.id == id) {
                    inst.status = RenderStatus::Crashed;
                }
            }
            let count = pool.recycle_crashed().await;
            assert_eq!(count, 1);
            assert_eq!(pool.stats().instances_crashed, 1);
            assert_eq!(pool.health().await.total, 0);
        });
    }

    #[test]
    fn test_render_pool_health_format() {
        let rt = runtime();
        rt.block_on(async {
            let pool = Arc::new(JsRenderPool::new(5, 1, 60));
            pool.start_instance().await.unwrap();
            let h = pool.health().await;
            assert_eq!(h.active, 0);
            assert_eq!(h.total, 1);
            assert_eq!(h.crashed, 0);
        });
    }

    #[test]
    fn test_wait_for_selector_timeout() {
        let rt = runtime();
        rt.block_on(async {
            let result =
                wait_for_selector("ws://localhost:1", ".nonexistent", 100).await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_wait_for_navigation_timeout() {
        let rt = runtime();
        rt.block_on(async {
            let result = wait_for_navigation("ws://localhost:1", 100).await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_detect_spa_heuristics() {
        let rt = runtime();
        rt.block_on(async {
            let result = detect_spa("ws://localhost:1").await;
            assert!(result.is_err());
        });
    }
}
