use std::fmt;

/// Failure mode classification — the Cumora taxonomy applied to NeoTrix.
///
/// Every recovery path in the system should be tagged with one of these.
/// This prevents the "spray and pray" anti-pattern where every error is
/// handled identically.
///
/// # Rules
///
/// - **FailOpen**: Coordination signals. Duplicate is better than stall.
///   Network calls, read-through caches, presence updates, typing indicators.
///
/// - **FailClosed**: Data correctness invariants. Block is better than corrupt.
///   File writes, database mutations, auth checks, credential stores.
///
/// - **FailOpenWithRetry**: Network calls that should eventually succeed.
///   API requests, provider calls, sync operations.
///
/// - **FailClosedWithFallback**: Critical operations with a degraded path.
///   Primary DB down → read from backup. Primary provider down → fallback provider.
///
/// - **FailSilent**: Observability-only operations. Logging, metrics, telemetry.
///   Never block the user for observability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FailureMode {
    /// Worst case: duplicate, not lost. Coordination signals, WS events, presence.
    FailOpen,
    /// Worst case: blocked, not corrupted. File writes, DB mutations, auth.
    FailClosed,
    /// Network calls that should eventually succeed. Exponential backoff.
    FailOpenWithRetry,
    /// Critical with degraded fallback path.
    FailClosedWithFallback,
    /// Observability only. Never block user.
    FailSilent,
}

impl FailureMode {
    /// Should we retry on failure?
    pub fn should_retry(&self) -> bool {
        matches!(self, Self::FailOpenWithRetry | Self::FailClosedWithFallback)
    }

    /// Should we log the error?
    pub fn should_log(&self) -> bool {
        !matches!(self, Self::FailSilent)
    }

    /// Should we propagate the error to the caller?
    pub fn should_propagate(&self) -> bool {
        matches!(self, Self::FailClosed | Self::FailClosedWithFallback)
    }
}

impl fmt::Display for FailureMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailOpen => write!(f, "fail-open"),
            Self::FailClosed => write!(f, "fail-closed"),
            Self::FailOpenWithRetry => write!(f, "fail-open-retry"),
            Self::FailClosedWithFallback => write!(f, "fail-closed-fallback"),
            Self::FailSilent => write!(f, "fail-silent"),
        }
    }
}

/// A tagged error that carries its failure mode.
#[derive(Debug)]
pub struct TaggedError<E> {
    pub error: E,
    pub mode: FailureMode,
    pub context: String,
}

impl<E: fmt::Display> fmt::Display for TaggedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.mode, self.context, self.error
        )
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for TaggedError<E> {}

/// Recovery action to take after a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Ignore the error entirely.
    Ignore,
    /// Retry immediately (will be retried with backoff).
    Retry,
    /// Use a fallback path.
    Fallback(String),
    /// Propagate the error to the user.
    Propagate,
    /// Crash the process (nuclear option, last resort).
    Crash,
}

/// Decide the recovery action based on failure mode and error type.
pub fn decide_recovery(mode: FailureMode, is_transient: bool) -> RecoveryAction {
    match mode {
        FailureMode::FailOpen => RecoveryAction::Ignore,
        FailureMode::FailSilent => RecoveryAction::Ignore,
        FailureMode::FailClosed => RecoveryAction::Propagate,
        FailureMode::FailOpenWithRetry => {
            if is_transient {
                RecoveryAction::Retry
            } else {
                RecoveryAction::Ignore
            }
        }
        FailureMode::FailClosedWithFallback => {
            if is_transient {
                RecoveryAction::Fallback("retry with degraded mode".into())
            } else {
                RecoveryAction::Propagate
            }
        }
    }
}

/// Tag an error with its failure mode and context.
pub fn tag<E>(error: E, mode: FailureMode, context: impl Into<String>) -> TaggedError<E> {
    TaggedError {
        error,
        mode,
        context: context.into(),
    }
}

/// Execute a fallible operation with the appropriate failure mode.
///
/// ```ignore
/// let result = with_failure_mode(
///     || async { send_ws_event(event).await },
///     FailureMode::FailOpen,
///     "ws_event_send",
/// ).await;
/// ```
pub async fn with_failure_mode<F, Fut, T, E>(
    op: F,
    mode: FailureMode,
    context: &str,
) -> Result<T, TaggedError<E>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: fmt::Display,
{
    match op().await {
        Ok(val) => Ok(val),
        Err(e) => {
            if mode.should_log() {
                tracing::warn!("[{mode}] {context}: {e}");
            }
            Err(tag(e, mode, context))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_mode_retry_behavior() {
        assert!(!FailureMode::FailOpen.should_retry());
        assert!(!FailureMode::FailClosed.should_retry());
        assert!(FailureMode::FailOpenWithRetry.should_retry());
        assert!(FailureMode::FailClosedWithFallback.should_retry());
        assert!(!FailureMode::FailSilent.should_retry());
    }

    #[test]
    fn failure_mode_propagation() {
        assert!(!FailureMode::FailOpen.should_propagate());
        assert!(FailureMode::FailClosed.should_propagate());
        assert!(!FailureMode::FailOpenWithRetry.should_propagate());
        assert!(FailureMode::FailClosedWithFallback.should_propagate());
        assert!(!FailureMode::FailSilent.should_propagate());
    }

    #[test]
    fn recovery_decisions() {
        assert_eq!(
            decide_recovery(FailureMode::FailOpen, true),
            RecoveryAction::Ignore
        );
        assert_eq!(
            decide_recovery(FailureMode::FailClosed, false),
            RecoveryAction::Propagate
        );
        assert_eq!(
            decide_recovery(FailureMode::FailOpenWithRetry, true),
            RecoveryAction::Retry
        );
        assert_eq!(
            decide_recovery(FailureMode::FailClosedWithFallback, true),
            RecoveryAction::Fallback("retry with degraded mode".into())
        );
    }
}
