use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use uuid::Uuid;

use super::types::SubAgentResult;

const CONTEXT_WARN_THRESHOLD: f64 = 0.7;
const CONTEXT_CRITICAL_THRESHOLD: f64 = 0.9;

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionStrategy {
    Summarize,
    TruncateOld,
    OffloadToFile,
    DropToolOutput,
}

#[derive(Debug, Clone)]
pub struct AsyncDelegateConfig {
    pub max_async_agents: usize,
    pub context_window_tokens: usize,
    pub compression_strategy: CompressionStrategy,
    pub summarization_prompt: String,
    pub poll_interval_ms: u64,
}

impl Default for AsyncDelegateConfig {
    fn default() -> Self {
        Self {
            max_async_agents: 5,
            context_window_tokens: 8192,
            compression_strategy: CompressionStrategy::Summarize,
            summarization_prompt: "Compress the following agent output into a concise summary with key findings, open questions, and next steps.".into(),
            poll_interval_ms: 500,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AsyncTaskStatus {
    Pending,
    Running,
    Completed(SubAgentResult),
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct AsyncTaskHandle {
    pub task_id: Uuid,
    pub status: AsyncTaskStatus,
    pub context_usage_pct: f64,
    pub created_at: Instant,
    pub completed_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct AsyncTaskSpec {
    pub prompt: String,
    pub isolated_context: bool,
    pub max_tokens: usize,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AsyncSubAgentDelegate {
    config: AsyncDelegateConfig,
    tasks: Arc<RwLock<HashMap<Uuid, AsyncTaskHandle>>>,
    event_tx: mpsc::Sender<(Uuid, AsyncTaskStatus)>,
    event_rx: Arc<RwLock<Option<mpsc::Receiver<(Uuid, AsyncTaskStatus)>>>>,
}

impl AsyncSubAgentDelegate {
    pub fn new(config: AsyncDelegateConfig) -> Self {
        let (tx, rx) = mpsc::channel(128);
        Self {
            config,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
            event_rx: Arc::new(RwLock::new(Some(rx))),
        }
    }

    pub fn event_receiver(&self) -> Option<mpsc::Receiver<(Uuid, AsyncTaskStatus)>> {
        let mut guard = self.event_rx.try_write().ok()?;
        guard.take()
    }

    pub async fn delegate(&self, spec: AsyncTaskSpec) -> Result<Uuid, String> {
        let task_id = Uuid::new_v4();
        let tasks = self.tasks.clone();
        let event_tx = self.event_tx.clone();
        let max_tokens = spec.max_tokens;
        let prompt = spec.prompt.clone();

        {
            let mut guard = tasks.write().await;
            if guard.len() >= self.config.max_async_agents {
                return Err(format!(
                    "max async agents reached ({})",
                    self.config.max_async_agents
                ));
            }
            guard.insert(
                task_id,
                AsyncTaskHandle {
                    task_id,
                    status: AsyncTaskStatus::Pending,
                    context_usage_pct: 0.0,
                    created_at: Instant::now(),
                    completed_at: None,
                },
            );
        }

        if let Err(e) = event_tx.send((task_id, AsyncTaskStatus::Pending)).await {
            log::warn!("[async_delegate] event_tx send Pending failed: {}", e);
        }

        tokio::spawn(async move {
            if let Err(e) = event_tx.send((task_id, AsyncTaskStatus::Running)).await {
                log::warn!("[async_delegate] event_tx send Running failed: {}", e);
            }
            {
                let mut guard = tasks.write().await;
                if let Some(handle) = guard.get_mut(&task_id) {
                    handle.status = AsyncTaskStatus::Running;
                }
            }

            let simulated_tokens = prompt.len() as u64 * 2;
            sleep(Duration::from_millis(100 + (prompt.len() as u64 % 50) * 10)).await;

            let result = SubAgentResult {
                summary: format!(
                    "Async result for: {}",
                    if prompt.len() > 40 {
                        format!("{}...", &prompt[..40])
                    } else {
                        prompt
                    }
                ),
                evidence: vec!["async evidence 1".into(), "async evidence 2".into()],
                execution_metrics: HashMap::from([("async_latency_ms".into(), 50.0)]),
                total_tokens: simulated_tokens,
                duration_ms: 150,
            };

            let ctx_pct = simulated_tokens as f64 / max_tokens as f64;
            {
                let mut guard = tasks.write().await;
                if let Some(handle) = guard.get_mut(&task_id) {
                    handle.status = AsyncTaskStatus::Completed(result.clone());
                    handle.context_usage_pct = ctx_pct;
                    handle.completed_at = Some(Instant::now());
                }
            }
            if let Err(e) = event_tx
                .send((task_id, AsyncTaskStatus::Completed(result)))
                .await
            {
                log::warn!("[async_delegate] event_tx send Completed failed: {}", e);
            }
        });

        Ok(task_id)
    }

    pub async fn poll(&self, task_id: Uuid) -> Option<AsyncTaskHandle> {
        let guard = self.tasks.read().await;
        guard.get(&task_id).cloned()
    }

    pub async fn cancel(&self, task_id: Uuid) -> bool {
        let mut guard = self.tasks.write().await;
        if let Some(handle) = guard.get_mut(&task_id) {
            match &handle.status {
                AsyncTaskStatus::Pending | AsyncTaskStatus::Running => {
                    handle.status = AsyncTaskStatus::Cancelled;
                    if let Err(e) = self
                        .event_tx
                        .send((task_id, AsyncTaskStatus::Cancelled))
                        .await
                    {
                        ::log::warn!("[async-delegate] event send Cancelled failed: {}", e);
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub async fn collect_result(&self, task_id: Uuid) -> Option<SubAgentResult> {
        let mut guard = self.tasks.write().await;
        if let Some(handle) = guard.remove(&task_id) {
            match handle.status {
                AsyncTaskStatus::Completed(result) => Some(result),
                _ => None,
            }
        } else {
            None
        }
    }

    pub async fn pending_count(&self) -> usize {
        let guard = self.tasks.read().await;
        guard.len()
    }

    pub async fn check_context_threshold(&self, current_usage: f64) -> Option<CompressionStrategy> {
        if current_usage >= CONTEXT_CRITICAL_THRESHOLD {
            Some(self.config.compression_strategy.clone())
        } else if current_usage >= CONTEXT_WARN_THRESHOLD {
            Some(CompressionStrategy::Summarize)
        } else {
            None
        }
    }

    pub fn compress_context(&self, context: &str, strategy: &CompressionStrategy) -> String {
        match strategy {
            CompressionStrategy::Summarize => {
                let lines: Vec<&str> = context.lines().collect();
                if lines.len() > 20 {
                    let head = lines
                        .iter()
                        .take(10)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n");
                    let tail = lines
                        .iter()
                        .rev()
                        .take(10)
                        .cloned()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!(
                        "{}\n\n...[compressed {} lines]...\n\n{}",
                        head,
                        lines.len() - 20,
                        tail
                    )
                } else {
                    context.to_string()
                }
            }
            CompressionStrategy::TruncateOld => {
                let max_chars = (context.len() as f64 * 0.5) as usize;
                if context.len() > max_chars {
                    format!(
                        "...[truncated from {} to {} chars]:\n{}",
                        context.len(),
                        max_chars,
                        &context[context.len().saturating_sub(max_chars)..]
                    )
                } else {
                    context.to_string()
                }
            }
            CompressionStrategy::OffloadToFile => {
                format!(
                    "<context offloaded to file: async_delegate_{}.ctx>",
                    Uuid::new_v4()
                )
            }
            CompressionStrategy::DropToolOutput => {
                let filtered: Vec<&str> = context
                    .lines()
                    .filter(|l| {
                        !l.trim_start().starts_with("Tool:")
                            && !l.trim_start().starts_with("Output:")
                    })
                    .collect();
                filtered.join("\n")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_delegate_returns_id() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let spec = AsyncTaskSpec {
            prompt: "test task".into(),
            isolated_context: true,
            max_tokens: 4096,
            capabilities: vec![],
        };
        let id = delegate.delegate(spec).await.unwrap();
        assert!(delegate.poll(id).await.is_some());
    }

    #[tokio::test]
    async fn test_collect_result() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let spec = AsyncTaskSpec {
            prompt: "collect me".into(),
            isolated_context: false,
            max_tokens: 4096,
            capabilities: vec![],
        };
        let id = delegate.delegate(spec).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let result = delegate.collect_result(id).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_cancel_running() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig {
            max_async_agents: 5,
            context_window_tokens: 8192,
            compression_strategy: CompressionStrategy::Summarize,
            summarization_prompt: "".into(),
            poll_interval_ms: 500,
        });
        let spec = AsyncTaskSpec {
            prompt: "long task".into(),
            isolated_context: true,
            max_tokens: 4096,
            capabilities: vec![],
        };
        let id = delegate.delegate(spec).await.unwrap();
        assert!(delegate.cancel(id).await);
    }

    #[tokio::test]
    async fn test_max_agents() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig {
            max_async_agents: 1,
            context_window_tokens: 8192,
            compression_strategy: CompressionStrategy::Summarize,
            summarization_prompt: "".into(),
            poll_interval_ms: 500,
        });
        let spec = AsyncTaskSpec::default();
        delegate.delegate(spec.clone()).await.unwrap();
        let result = delegate.delegate(spec).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_threshold_warn() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let strategy = delegate.check_context_threshold(0.75).await;
        assert_eq!(strategy, Some(CompressionStrategy::Summarize));
    }

    #[tokio::test]
    async fn test_context_threshold_critical() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let strategy = delegate.check_context_threshold(0.95).await;
        assert_eq!(strategy, Some(CompressionStrategy::Summarize));
    }

    #[tokio::test]
    async fn test_context_threshold_low() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let strategy = delegate.check_context_threshold(0.5).await;
        assert_eq!(strategy, None);
    }

    #[test]
    fn test_compress_summarize() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let long = (0..30)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let compressed = delegate.compress_context(&long, &CompressionStrategy::Summarize);
        assert!(compressed.contains("[compressed"));
    }

    #[test]
    fn test_compress_truncate() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let long = "a".repeat(1000);
        let compressed = delegate.compress_context(&long, &CompressionStrategy::TruncateOld);
        assert!(compressed.contains("[truncated"));
    }

    #[test]
    fn test_compress_drop_tool_output() {
        let delegate = AsyncSubAgentDelegate::new(AsyncDelegateConfig::default());
        let text = "User: hi\nTool: ls\nOutput: files\nUser: ok";
        let compressed = delegate.compress_context(text, &CompressionStrategy::DropToolOutput);
        assert!(!compressed.contains("Tool:"));
        assert!(!compressed.contains("Output:"));
    }
}

impl Default for AsyncTaskSpec {
    fn default() -> Self {
        Self {
            prompt: "default task".into(),
            isolated_context: true,
            max_tokens: 4096,
            capabilities: vec![],
        }
    }
}
