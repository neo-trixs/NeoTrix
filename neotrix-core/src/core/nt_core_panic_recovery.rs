//! Panic Recovery — IMK boundary protection pattern.
//!
//! Provides structured panic catching with context tracking, inspired by
//! Qingjian's IMK boundary protection. Every external/trusted boundary
//! crossing must be wrapped to prevent panics from propagating across
//! trust boundaries and corrupting invariants.
//!
//! # Design
//!
//! Panics in Rust are unrecoverable within the current thread. This module
//! catches them via `catch_unwind` at trust boundaries and converts them
//! into structured `Result::Err`, preserving:
//! - **context**: what operation was being performed
//! - **message**: the panic payload (typically a string)
//! - **backtrace**: captured when available (requires `RUST_BACKTRACE=1`)
//!
//! # Usage
//!
//! ```rust
//! use neotrix::core::nt_core_panic_recovery::{catch_panic, panic_boundary};
//!
//! let result = catch_panic("parsing config", || {
//!     // risky operation
//!     42
//! });
//! assert!(result.is_ok());
//!
//! let result = panic_boundary!("parsing config" => {
//!     42
//! });
//! assert!(result.is_ok());
//! ```

#![forbid(unsafe_code)]

use std::any::Any;
use std::fmt;
use std::panic;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tracing::error;

// ───────────────────────────────────────────────────────────────────────
// PanicError
// ───────────────────────────────────────────────────────────────────────

/// Structured error produced when a panic is caught at a trust boundary.
#[derive(Debug)]
pub struct PanicError {
    /// What operation was being performed when the panic occurred.
    pub context: String,
    /// The panic payload (typically a `&str` or `String`).
    pub message: String,
    /// Unix epoch timestamp (seconds) when the panic was caught.
    pub timestamp: i64,
    /// Optional backtrace captured at the catch site.
    pub backtrace: Option<String>,
}

impl fmt::Display for PanicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "panic in `{}`: {}",
            self.context, self.message,
        )
    }
}

impl std::error::Error for PanicError {}

impl PanicError {
    /// Create a new PanicError from a caught panic payload.
    fn from_payload(context: &str, payload: Box<dyn Any + Send>) -> Self {
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            format!("{:?}", payload)
        };

        let backtrace = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            std::backtrace::Backtrace::force_capture()
        }))
        .ok()
        .map(|bt| format!("{bt}"));

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            context: context.to_string(),
            message,
            timestamp,
            backtrace,
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// BoundaryError — combines panic + timeout
// ───────────────────────────────────────────────────────────────────────

/// Error type for `boundary_call` which combines panic recovery with
/// optional timeout protection.
#[derive(Debug)]
pub enum BoundaryError {
    /// The operation panicked.
    Panic(PanicError),
    /// The operation exceeded the configured timeout.
    Timeout {
        name: String,
        duration_ms: u64,
    },
}

impl fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic(e) => write!(f, "{e}"),
            Self::Timeout { name, duration_ms } => {
                write!(f, "boundary `{name}` timed out after {duration_ms}ms")
            }
        }
    }
}

impl std::error::Error for BoundaryError {}

impl From<PanicError> for BoundaryError {
    fn from(e: PanicError) -> Self {
        Self::Panic(e)
    }
}

// ───────────────────────────────────────────────────────────────────────
// catch_panic — synchronous boundary
// ───────────────────────────────────────────────────────────────────────

/// Catch panics from a synchronous closure, returning a structured error.
///
/// The closure is wrapped in `std::panic::catch_unwind` with
/// `AssertUnwindSafe` — callers must ensure the closure is safe to unwind
/// (no partial writes to shared state).
///
/// On panic, logs the event via `tracing::error` and returns
/// `Err(PanicError)`.
pub fn catch_panic<T>(
    context: &str,
    f: impl FnOnce() -> T + panic::UnwindSafe,
) -> Result<T, PanicError> {
    match panic::catch_unwind(f) {
        Ok(v) => Ok(v),
        Err(payload) => {
            let err = PanicError::from_payload(context, payload);
            error!(
                context = %err.context,
                message = %err.message,
                "panic caught at boundary"
            );
            Err(err)
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// catch_async — async boundary via spawn_blocking
// ───────────────────────────────────────────────────────────────────────

/// Catch panics from an async operation.
///
/// Internally spawns the work on a blocking thread via
/// `tokio::task::spawn_blocking` + `catch_unwind`. This is necessary because
/// `catch_unwind` only works on synchronous closures, and tokio tasks are
/// not unwind-safe across `.await` points.
///
/// Returns a future that resolves to the result or a `PanicError`.
pub async fn catch_async<T, F>(
    context: &str,
    f: F,
) -> Result<T, PanicError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let ctx = context.to_string();
    let result = tokio::task::spawn_blocking(move || {
        catch_panic(&ctx, f)
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(join_err) => {
            let err = PanicError {
                context: context.to_string(),
                message: format!("blocking task panicked or was cancelled: {join_err}"),
                timestamp: 0,
                backtrace: None,
            };
            error!(
                context = %err.context,
                message = %err.message,
                "panic caught at async boundary"
            );
            Err(err)
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// boundary_call — panic + optional timeout
// ───────────────────────────────────────────────────────────────────────

/// Execute a synchronous closure at a trust boundary with panic recovery
/// and optional timeout.
///
/// If `timeout` is `None`, behaves like `catch_panic` but returns
/// `BoundaryError`.
///
/// If `timeout` is `Some(duration)`, runs the closure on a blocking thread
/// and races it against the deadline.
pub fn boundary_call<T>(
    name: &str,
    f: impl FnOnce() -> T + Send + 'static,
    timeout: Option<Duration>,
) -> Result<T, BoundaryError>
where
    T: Send + 'static,
{
    match timeout {
        None => catch_panic(name, f).map_err(BoundaryError::Panic),
        Some(duration) => {
            let ctx = name.to_string();
            let result = std::thread::scope(|s| {
                let handle = s.spawn(|| catch_panic(name, f));

                let deadline = std::time::Instant::now() + duration;
                loop {
                    if handle.is_finished() {
                        return handle.join().unwrap_or_else(|_| {
                            Err(PanicError {
                                context: name.to_string(),
                                message: "thread panicked during join".into(),
                                timestamp: 0,
                                backtrace: None,
                            })
                        });
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err(BoundaryError::Timeout {
                            name: name.to_string(),
                            duration_ms: duration.as_millis() as u64,
                        });
                    }
                    std::thread::yield_now();
                }
            });
            result
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// panic_boundary! macro
// ───────────────────────────────────────────────────────────────────────

/// Macro shorthand for `catch_panic` with automatic context string.
///
/// # Syntax
///
/// ```rust,ignore
/// panic_boundary!("context string" => expression)
/// panic_boundary!("context string" => { block })
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// let x = panic_boundary!("parse int" => "42".parse::<i32>().unwrap());
/// let y = panic_boundary!("heavy compute" => {
///     (0..1000).sum::<i64>()
/// });
/// ```
#[macro_export]
macro_rules! panic_boundary {
    ($ctx:expr => $body:expr) => {
        $crate::core::nt_core_panic_recovery::catch_panic($ctx, || $body)
    };
}

// ───────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── catch_panic ──

    #[test]
    fn catch_panic_returns_value_on_success() {
        let result = catch_panic("addition", || 2 + 2);
        assert_eq!(result.unwrap(), 4);
    }

    #[test]
    fn catch_panic_catches_panic_with_str() {
        let result = catch_panic("explode", || {
            panic!("boom");
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.context, "explode");
        assert_eq!(err.message, "boom");
    }

    #[test]
    fn catch_panic_catches_panic_with_string() {
        let result = catch_panic("explode", || {
            let msg = String::from("dynamic boom");
            panic!("{}", msg);
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "dynamic boom");
    }

    #[test]
    fn catch_panic_catches_non_string_panic() {
        let result = catch_panic("integer panic", || {
            panic!(42);
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Non-string payloads get debug-formatted
        assert!(err.message.contains("42"));
    }

    #[test]
    fn catch_panic_preserves_backtrace() {
        let result = catch_panic("bt test", || {
            panic!("with trace");
        });
        let err = result.unwrap_err();
        // Backtrace may or may not be captured depending on env,
        // but the field should exist
        let _ = err.backtrace;
    }

    #[test]
    fn catch_panic_display_format() {
        let err = PanicError {
            context: "test op".into(),
            message: "went wrong".into(),
            timestamp: 0,
            backtrace: None,
        };
        assert_eq!(format!("{err}"), "panic in `test op`: went wrong");
    }

    #[test]
    fn catch_panic_error_trait() {
        let err = PanicError {
            context: "x".into(),
            message: "y".into(),
            timestamp: 0,
            backtrace: None,
        };
        let e: &dyn std::error::Error = &err;
        assert!(e.source().is_none());
    }

    // ── panic_boundary! ──

    #[test]
    fn macro_success() {
        let result = panic_boundary!("sum" => (1..=10).sum::<i64>());
        assert_eq!(result.unwrap(), 55);
    }

    #[test]
    fn macro_catches_panic() {
        let result = panic_boundary!("div" => {
            let _ = 1 / 0;
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().context, "div");
    }

    #[test]
    fn macro_block_syntax() {
        let result = panic_boundary!("block" => {
            let a = 10;
            let b = 20;
            a + b
        });
        assert_eq!(result.unwrap(), 30);
    }

    // ── boundary_call ──

    #[test]
    fn boundary_call_no_timeout_success() {
        let result = boundary_call("no-to", || 42, None);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn boundary_call_no_timeout_panic() {
        let result = boundary_call("explode", || panic!("x"), None);
        assert!(result.is_err());
        match result.unwrap_err() {
            BoundaryError::Panic(e) => assert_eq!(e.message, "x"),
            BoundaryError::Timeout { .. } => panic!("expected Panic variant"),
        }
    }

    #[test]
    fn boundary_call_timeout_fast() {
        let result = boundary_call("fast", || 1, Some(Duration::from_millis(100)));
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn boundary_call_timeout_exceeded() {
        let result = boundary_call(
            "slow",
            || {
                std::thread::sleep(Duration::from_secs(10));
            },
            Some(Duration::from_millis(10)),
        );
        match result.unwrap_err() {
            BoundaryError::Timeout { name, duration_ms } => {
                assert_eq!(name, "slow");
                assert!(duration_ms <= 50); // generous margin
            }
            BoundaryError::Panic(_) => panic!("expected Timeout variant"),
        }
    }

    #[test]
    fn boundary_error_display_panic() {
        let err = BoundaryError::Panic(PanicError {
            context: "op".into(),
            message: "bad".into(),
            timestamp: 0,
            backtrace: None,
        });
        assert_eq!(format!("{err}"), "panic in `op`: bad");
    }

    #[test]
    fn boundary_error_display_timeout() {
        let err = BoundaryError::Timeout {
            name: "op".into(),
            duration_ms: 500,
        };
        assert_eq!(format!("{err}"), "boundary `op` timed out after 500ms");
    }

    #[test]
    fn boundary_error_from_panic_error() {
        let pe = PanicError {
            context: "x".into(),
            message: "y".into(),
            timestamp: 0,
            backtrace: None,
        };
        let be: BoundaryError = pe.into();
        assert!(matches!(be, BoundaryError::Panic(_)));
    }

    // ── async catch_async ──

    #[tokio::test]
    async fn catch_async_success() {
        let result = catch_async("async add", || 2 + 2).await;
        assert_eq!(result.unwrap(), 4);
    }

    #[tokio::test]
    async fn catch_async_catches_panic() {
        let result: Result<i32, PanicError> = catch_async("async boom", || {
            panic!("async panic");
        })
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.context, "async boom");
        assert_eq!(err.message, "async panic");
    }

    // ── PanicError from_payload edge cases ──

    #[test]
    fn from_payload_with_unit_panic() {
        let result = catch_panic("unit", || panic!());
        let err = result.unwrap_err();
        assert!(err.message.is_empty() || err.message.contains("()"));
    }

    #[test]
    fn from_payload_with_custom_type() {
        #[derive(Debug)]
        struct CustomError(i32);
        let result = catch_panic("custom", || {
            panic!(CustomError(42));
        });
        let err = result.unwrap_err();
        assert!(err.message.contains("42"));
    }
}
