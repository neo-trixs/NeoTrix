use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryOp {
    Evolve,
    CheckAnchor,
    ApplyEdit,
    LoadSnapshot,
    Inspect,
}

impl BoundaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Evolve => "evolve",
            Self::CheckAnchor => "check_anchor",
            Self::ApplyEdit => "apply_edit",
            Self::LoadSnapshot => "load_snapshot",
            Self::Inspect => "inspect",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundaryContext {
    pub operation_id: String,
    pub timestamp: u64,
    pub hook_name: String,
    pub details: HashMap<String, String>,
}

impl BoundaryContext {
    pub fn new(op: BoundaryOp, details: HashMap<String, String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Self {
            operation_id: format!("{}_{:016x}", op.as_str(), now),
            timestamp: now / 1_000_000_000,
            hook_name: String::new(),
            details,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BoundaryError {
    HookRejected {
        hook: String,
        reason: String,
    },
    HookPanic {
        hook: String,
    },
    Timeout {
        hook: String,
    },
    ChainBreaks {
        hook: String,
        suppressed: Vec<String>,
    },
}

impl fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HookRejected { hook, reason } => {
                write!(f, "hook '{hook}' rejected: {reason}")
            }
            Self::HookPanic { hook } => {
                write!(f, "hook '{hook}' panicked")
            }
            Self::Timeout { hook } => {
                write!(f, "hook '{hook}' timed out")
            }
            Self::ChainBreaks { hook, suppressed } => {
                write!(f, "chain break at '{hook}': {}", suppressed.join("; "))
            }
        }
    }
}

impl std::error::Error for BoundaryError {}

pub trait BoundaryHook: fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn before(&self, op: BoundaryOp, ctx: &BoundaryContext) -> Result<(), BoundaryError>;
    fn after(
        &self,
        op: BoundaryOp,
        ctx: &BoundaryContext,
        result: &Result<(), BoundaryError>,
    ) -> Result<(), BoundaryError>;
    fn clone_box(&self) -> Box<dyn BoundaryHook>;
}

#[derive(Debug)]
pub struct BoundaryHookInstance {
    inner: Box<dyn BoundaryHook>,
}

impl BoundaryHookInstance {
    pub fn new(hook: impl BoundaryHook + 'static) -> Self {
        Self {
            inner: Box::new(hook),
        }
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

impl Clone for BoundaryHookInstance {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_box(),
        }
    }
}

impl<T: BoundaryHook + 'static> From<T> for BoundaryHookInstance {
    fn from(hook: T) -> Self {
        Self::new(hook)
    }
}

#[derive(Debug, Clone)]
pub struct BoundaryManager {
    hooks: Vec<BoundaryHookInstance>,
    pub enabled: bool,
    pub chain_on_error: bool,
}

impl Default for BoundaryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundaryManager {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            enabled: true,
            chain_on_error: false,
        }
    }

    pub fn register(&mut self, hook: impl Into<BoundaryHookInstance>) {
        self.hooks.push(hook.into());
    }

    pub fn run_before(&self, op: BoundaryOp) -> Result<BoundaryContext, BoundaryError> {
        self.run_before_with_details(op, HashMap::new())
    }

    pub fn run_before_with_details(
        &self,
        op: BoundaryOp,
        details: HashMap<String, String>,
    ) -> Result<BoundaryContext, BoundaryError> {
        if !self.enabled {
            return Ok(BoundaryContext::new(op, details));
        }

        let mut ctx = BoundaryContext::new(op, details);

        let mut errors: Vec<String> = Vec::new();

        for hook in &self.hooks {
            ctx.hook_name = hook.name().to_string();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                hook.inner.before(op, &ctx)
            })) {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if self.chain_on_error {
                        errors.push(format!("{}: {}", hook.name(), e));
                    } else {
                        return Err(e);
                    }
                }
                Err(_) => {
                    let err = BoundaryError::HookPanic {
                        hook: hook.name().to_string(),
                    };
                    if self.chain_on_error {
                        errors.push(format!("{}: panic", hook.name()));
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(BoundaryError::ChainBreaks {
                hook: "run_before".to_string(),
                suppressed: errors,
            });
        }

        Ok(ctx)
    }

    pub fn run_after(
        &self,
        op: BoundaryOp,
        ctx: &BoundaryContext,
        result: &Result<(), BoundaryError>,
    ) -> Result<(), BoundaryError> {
        if !self.enabled {
            return Ok(());
        }

        let mut errors: Vec<String> = Vec::new();

        for hook in &self.hooks {
            let mut hook_ctx = ctx.clone();
            hook_ctx.hook_name = hook.name().to_string();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                hook.inner.after(op, &hook_ctx, result)
            })) {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if self.chain_on_error {
                        errors.push(format!("{}: {}", hook.name(), e));
                    } else {
                        return Err(e);
                    }
                }
                Err(_) => {
                    let err = BoundaryError::HookPanic {
                        hook: hook.name().to_string(),
                    };
                    if self.chain_on_error {
                        errors.push(format!("{}: panic", hook.name()));
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(BoundaryError::ChainBreaks {
                hook: "run_after".to_string(),
                suppressed: errors,
            });
        }

        Ok(())
    }
}

const DRIFT_THRESHOLD: f64 = 0.35;

#[derive(Debug, Clone)]
pub struct AuditHook;

impl BoundaryHook for AuditHook {
    fn name(&self) -> &'static str {
        "audit"
    }

    fn before(&self, op: BoundaryOp, ctx: &BoundaryContext) -> Result<(), BoundaryError> {
        log::info!(
            "[boundary] BEFORE {} | op={} | ts={}",
            ctx.operation_id,
            op.as_str(),
            ctx.timestamp
        );
        Ok(())
    }

    fn after(
        &self,
        op: BoundaryOp,
        ctx: &BoundaryContext,
        result: &Result<(), BoundaryError>,
    ) -> Result<(), BoundaryError> {
        match result {
            Ok(()) => {
                log::info!(
                    "[boundary] AFTER  {} | op={} | OK",
                    ctx.operation_id,
                    op.as_str()
                )
            }
            Err(e) => {
                log::warn!(
                    "[boundary] AFTER  {} | op={} | ERR={}",
                    ctx.operation_id,
                    op.as_str(),
                    e
                )
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn BoundaryHook> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct DriftCheckHook;

impl BoundaryHook for DriftCheckHook {
    fn name(&self) -> &'static str {
        "drift_check"
    }

    fn before(&self, op: BoundaryOp, ctx: &BoundaryContext) -> Result<(), BoundaryError> {
        if op == BoundaryOp::Evolve {
            if let Some(drift_str) = ctx.details.get("current_drift") {
                if let Ok(drift) = drift_str.parse::<f64>() {
                    if drift > DRIFT_THRESHOLD {
                        return Err(BoundaryError::HookRejected {
                            hook: self.name().to_string(),
                            reason: format!(
                                "drift {:.4} exceeds threshold {:.4}",
                                drift, DRIFT_THRESHOLD
                            ),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn after(
        &self,
        _op: BoundaryOp,
        _ctx: &BoundaryContext,
        _result: &Result<(), BoundaryError>,
    ) -> Result<(), BoundaryError> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn BoundaryHook> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct CoherenceGuardHook {
    pub min_coherence: f64,
}

impl CoherenceGuardHook {
    pub fn new(min_coherence: f64) -> Self {
        Self { min_coherence }
    }
}

impl Default for CoherenceGuardHook {
    fn default() -> Self {
        Self { min_coherence: 0.5 }
    }
}

impl BoundaryHook for CoherenceGuardHook {
    fn name(&self) -> &'static str {
        "coherence_guard"
    }

    fn before(&self, op: BoundaryOp, ctx: &BoundaryContext) -> Result<(), BoundaryError> {
        if matches!(op, BoundaryOp::Evolve | BoundaryOp::LoadSnapshot) {
            if let Some(coherence_str) = ctx.details.get("current_coherence") {
                if let Ok(coherence) = coherence_str.parse::<f64>() {
                    if coherence < self.min_coherence {
                        return Err(BoundaryError::HookRejected {
                            hook: self.name().to_string(),
                            reason: format!(
                                "coherence {:.4} below minimum {:.4}",
                                coherence, self.min_coherence
                            ),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn after(
        &self,
        _op: BoundaryOp,
        _ctx: &BoundaryContext,
        _result: &Result<(), BoundaryError>,
    ) -> Result<(), BoundaryError> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn BoundaryHook> {
        Box::new(self.clone())
    }
}
