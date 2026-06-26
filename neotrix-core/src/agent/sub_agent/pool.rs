use std::collections::HashMap;
use std::mem::drop;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::timeout;
use uuid::Uuid;

use crate::core::nt_core_cap::CapabilityVector;

use super::execution::{simulate_agent_execution, AgentKind};
use super::types::{SubAgentConfig, SubAgentEvent, SubAgentHandle, SubAgentResult, SubAgentStatus};

/// Manages a pool of concurrent sub-agents.
///
/// # Example
///
/// ```ignore
/// use neotrix::agent::sub_agent::{SubAgentPool, SubAgentConfig};
///
/// let pool = SubAgentPool::new(SubAgentConfig::default());
/// let id = pool.launch("analyze the problem", None).await.unwrap();
/// let result = pool.wait_for(id).await;
/// ```
pub struct SubAgentPool {
    config: SubAgentConfig,
    handles: Arc<RwLock<HashMap<Uuid, SubAgentHandle>>>,
    active_count: Arc<AtomicUsize>,
    semaphore: Arc<Semaphore>,
    shutdown_signal: Arc<RwLock<bool>>,
    event_tx: mpsc::Sender<SubAgentEvent>,
    event_rx: Arc<RwLock<Option<mpsc::Receiver<SubAgentEvent>>>>,
}

impl SubAgentPool {
    /// Create a new pool with the given configuration.
    ///
    /// Configuration is validated: `max_concurrency` is clamped to `[1, 20]`,
    /// and `idle_timeout_secs` is clamped to `>= 1`.
    pub fn new(mut config: SubAgentConfig) -> Self {
        config.validate();
        let (tx, rx) = mpsc::channel(64);
        Self {
            active_count: Arc::new(AtomicUsize::new(0)),
            semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            handles: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
            event_rx: Arc::new(RwLock::new(Some(rx))),
            shutdown_signal: Arc::new(RwLock::new(false)),
            config,
        }
    }

    /// Return the event receiver for consuming sub-agent completion events.
    pub fn event_receiver(&self) -> Option<mpsc::Receiver<SubAgentEvent>> {
        let mut guard = match self.event_rx.try_write() {
            Ok(g) => g,
            Err(e) => {
                log::warn!("[sub-agent] event_rx try_write: {}", e);
                return None;
            }
        };
        guard.take()
    }

    /// Attach for enhanced sub-agent reasoning.
    ///
    /// Currently a placeholder for future integration.
    pub fn with_engine(_engine: ()) -> Self {
        Self::new(SubAgentConfig::default())
    }

    /// Launch a new sub-agent with the given prompt.
    ///
    /// Returns the assigned `Uuid` on success. Blocks (via semaphore) if the
    /// pool is at maximum concurrency.
    ///
    /// The sub-agent runs in a `tokio::spawn` task. When it completes, the
    /// result is written to the handle's `result_slot` and a
    /// [`SubAgentEvent::Done`] is emitted.
    pub async fn launch(
        &self,
        prompt: &str,
        capabilities: Option<CapabilityVector>,
    ) -> Result<Uuid, String> {
        if *self.shutdown_signal.read().await {
            return Err("pool is shut down".into());
        }

        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("semaphore error: {}", e))?;

        let id = Uuid::new_v4();
        let started_at = chrono::Utc::now().timestamp() as u64;
        let result_slot: Arc<RwLock<Option<SubAgentResult>>> = Arc::new(RwLock::new(None));

        let handle = SubAgentHandle {
            id,
            status: SubAgentStatus::Running,
            started_at,
            result_slot: result_slot.clone(),
        };

        self.handles.write().await.insert(id, handle);
        self.active_count.fetch_add(1, Ordering::SeqCst);

        let active_count = self.active_count.clone();
        let handles = self.handles.clone();
        let event_tx = self.event_tx.clone();
        let prompt_owned = prompt.to_string();
        let capabilities2 = capabilities.clone();
        let config = self.config.clone();

        drop(tokio::spawn(async move {
            let deadline = Duration::from_secs(config.idle_timeout_secs);
            let start = Instant::now();

            let sim_fut =
                simulate_agent_execution(&prompt_owned, AgentKind::Worker, capabilities2.as_ref());

            match timeout(deadline, sim_fut).await {
                Ok(Ok(mut result)) => {
                    let elapsed = start.elapsed();
                    result.duration_ms = elapsed.as_millis() as u64;
                    *result_slot.write().await = Some(result.clone());
                    if let Err(e) = event_tx
                        .send(SubAgentEvent::Done {
                            id,
                            summary: result.summary.clone(),
                            duration_ms: result.duration_ms,
                            total_tokens: result.total_tokens,
                        })
                        .await
                    {
                        log::warn!("[pool] event_tx send Done failed: {}", e);
                    }
                    if let Some(old_handle) = handles.write().await.get_mut(&id) {
                        old_handle.status = SubAgentStatus::Completed;
                        old_handle.result_slot = Arc::new(RwLock::new(Some(result)));
                    }
                }
                Ok(Err(e)) => {
                    if let Some(old_handle) = handles.write().await.get_mut(&id) {
                        old_handle.status = SubAgentStatus::Failed(e);
                    }
                    if let Err(e) = event_tx
                        .send(SubAgentEvent::Failed {
                            id,
                            error: "execution error".into(),
                        })
                        .await
                    {
                        log::warn!("[pool] event_tx send Failed (exec) failed: {}", e);
                    }
                }
                Err(_elapsed) => {
                    let elapsed_secs = start.elapsed().as_secs();
                    if let Some(old_handle) = handles.write().await.get_mut(&id) {
                        old_handle.status = SubAgentStatus::TimedOut;
                    }
                    if let Err(e) = event_tx
                        .send(SubAgentEvent::TimedOut { id, elapsed_secs })
                        .await
                    {
                        log::warn!("[pool] event_tx send TimedOut failed: {}", e);
                    }
                }
            }

            active_count.fetch_sub(1, Ordering::SeqCst);
            drop(_permit);
        }));

        Ok(id)
    }

    /// Launch a sub-agent with a custom timeout override.
    pub async fn launch_with_timeout(
        &self,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<Uuid, String> {
        if *self.shutdown_signal.read().await {
            return Err("pool is shut down".into());
        }

        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("semaphore error: {}", e))?;

        let id = Uuid::new_v4();
        let started_at = chrono::Utc::now().timestamp() as u64;
        let result_slot: Arc<RwLock<Option<SubAgentResult>>> = Arc::new(RwLock::new(None));

        let handle = SubAgentHandle {
            id,
            status: SubAgentStatus::Running,
            started_at,
            result_slot: result_slot.clone(),
        };

        self.handles.write().await.insert(id, handle);
        self.active_count.fetch_add(1, Ordering::SeqCst);

        let active_count = self.active_count.clone();
        let handles = self.handles.clone();
        let event_tx = self.event_tx.clone();
        let prompt_owned = prompt.to_string();
        let capabilities = None::<CapabilityVector>;

        drop(tokio::spawn(async move {
            let deadline = Duration::from_secs(timeout_secs);
            let start = Instant::now();

            let sim_fut =
                simulate_agent_execution(&prompt_owned, AgentKind::Worker, capabilities.as_ref());

            match timeout(deadline, sim_fut).await {
                Ok(Ok(mut result)) => {
                    let elapsed = start.elapsed();
                    result.duration_ms = elapsed.as_millis() as u64;
                    *result_slot.write().await = Some(result.clone());
                    if let Err(e) = event_tx
                        .send(SubAgentEvent::Done {
                            id,
                            summary: result.summary.clone(),
                            duration_ms: result.duration_ms,
                            total_tokens: result.total_tokens,
                        })
                        .await
                    {
                        log::warn!("[pool] event_tx send Done failed: {}", e);
                    }
                    if let Some(old_handle) = handles.write().await.get_mut(&id) {
                        old_handle.status = SubAgentStatus::Completed;
                        old_handle.result_slot = Arc::new(RwLock::new(Some(result)));
                    }
                }
                Ok(Err(e)) => {
                    if let Some(old_handle) = handles.write().await.get_mut(&id) {
                        old_handle.status = SubAgentStatus::Failed(e);
                    }
                    if let Err(e) = event_tx
                        .send(SubAgentEvent::Failed {
                            id,
                            error: "execution error".into(),
                        })
                        .await
                    {
                        log::warn!("[pool] event_tx send Failed (exec) failed: {}", e);
                    }
                }
                Err(_elapsed) => {
                    let elapsed_secs = start.elapsed().as_secs();
                    if let Some(old_handle) = handles.write().await.get_mut(&id) {
                        old_handle.status = SubAgentStatus::TimedOut;
                    }
                    if let Err(e) = event_tx
                        .send(SubAgentEvent::TimedOut { id, elapsed_secs })
                        .await
                    {
                        log::warn!("[pool] event_tx send TimedOut failed: {}", e);
                    }
                }
            }

            active_count.fetch_sub(1, Ordering::SeqCst);
            drop(_permit);
        }));

        Ok(id)
    }

    /// Wait for a sub-agent to complete and return its result.
    ///
    /// Polls the result slot until the sub-agent finishes or the pool is shut
    /// down. Returns `None` if the handle does not exist.
    pub async fn wait_for(&self, id: Uuid) -> Option<SubAgentResult> {
        let start = Instant::now();
        let timeout_dur = Duration::from_secs(self.config.idle_timeout_secs + 5);

        loop {
            if start.elapsed() > timeout_dur {
                return None;
            }

            let handle = self.handles.read().await.get(&id).cloned();
            match handle {
                Some(h) => {
                    if matches!(
                        h.status,
                        SubAgentStatus::Completed
                            | SubAgentStatus::Failed(_)
                            | SubAgentStatus::TimedOut
                    ) {
                        return h.result_slot.read().await.clone();
                    }
                }
                None => return None,
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Read a slice of evidence from a completed sub-agent's result.
    ///
    /// Returns `None` if the sub-agent does not exist or has not completed.
    pub fn handle_read_slice(&self, id: Uuid, start: usize, end: usize) -> Option<Vec<String>> {
        let handles = match self.handles.try_read() {
            Ok(h) => h,
            Err(e) => {
                log::warn!("[sub-agent] try_read handles: {}", e);
                return None;
            }
        };
        let handle = handles.get(&id)?;
        let guard = match handle.result_slot.try_read() {
            Ok(g) => g,
            Err(e) => {
                log::warn!("[sub-agent] try_read result_slot: {}", e);
                return None;
            }
        };
        let result = guard.as_ref()?;

        if start >= result.evidence.len() {
            return Some(Vec::new());
        }

        let end = end.min(result.evidence.len());
        Some(result.evidence[start..end].to_vec())
    }

    /// Get the current status of a sub-agent.
    pub fn status(&self, id: Uuid) -> Option<SubAgentStatus> {
        let handles = match self.handles.try_read() {
            Ok(h) => h,
            Err(e) => {
                log::warn!("[sub-agent] try_read status: {}", e);
                return None;
            }
        };
        handles.get(&id).map(|h| h.status.clone())
    }

    /// Number of currently active sub-agents.
    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    /// Cancel a single sub-agent by ID.
    ///
    /// Marks the sub-agent as `Failed` and removes it from tracking. Returns
    /// `true` if the sub-agent existed.
    pub fn cancel(&self, id: Uuid) -> bool {
        let mut handles = match self.handles.try_write() {
            Ok(h) => h,
            Err(_) => return false,
        };

        if let Some(handle) = handles.get_mut(&id) {
            handle.status = SubAgentStatus::Failed("cancelled".into());
            true
        } else {
            false
        }
    }

    /// Cancel all running sub-agents.
    pub fn cancel_all(&self) {
        let mut handles = match self.handles.try_write() {
            Ok(h) => h,
            Err(_) => return,
        };
        for (_, handle) in handles.iter_mut() {
            handle.status = SubAgentStatus::Failed("cancelled by pool".into());
        }
        handles.clear();
        self.active_count.store(0, Ordering::SeqCst);
    }

    /// Shut down the pool gracefully.
    ///
    /// Sets the shutdown flag, cancels all sub-agents, and waits up to 5
    /// seconds for active tasks to drain.
    pub async fn shutdown(&self) {
        *self.shutdown_signal.write().await = true;
        self.cancel_all();

        let deadline = Instant::now() + Duration::from_secs(5);
        while self.active_count() > 0 && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
