//! Efficiency mechanisms for NeoTrix harness layer.
//! Inspired by SoL-Pi (NVIDIA): Action Fusion, ObservationPack,
//! Evidence-Preserving Reducer, Online Context Compact.

/// Trait for all efficiency mechanisms
pub trait EfficiencyMechanism: Send + Sync {
    fn name(&self) -> &str;
    fn should_activate(&self, ctx: &HarnessContext) -> bool;
    fn apply(&self, ctx: &mut HarnessContext) -> Result<(), String>;
}

/// Context passed to efficiency mechanisms
#[derive(Debug, Clone, Default)]
pub struct HarnessContext {
    pub turn_count: usize,
    pub total_tokens: u64,
    pub pending_actions: Vec<String>,
    pub output_buffer: String,
    pub evidence_log: Vec<String>,
}

/// Action Fusion — merge follow-up validation with write operations
pub struct ActionFusion;

impl EfficiencyMechanism for ActionFusion {
    fn name(&self) -> &str { "action_fusion" }
    fn should_activate(&self, ctx: &HarnessContext) -> bool {
        ctx.pending_actions.len() >= 2
    }
    fn apply(&self, ctx: &mut HarnessContext) -> Result<(), String> {
        // Merge consecutive write+validate pairs
        let merged: Vec<String> = ctx.pending_actions
            .windows(2)
            .map(|w| format!("{}+{}", w[0], w[1]))
            .collect();
        if !merged.is_empty() {
            ctx.pending_actions = merged;
        }
        Ok(())
    }
}

/// ObservationPack — compress repeated large outputs into stable handles
pub struct ObservationPack;

impl EfficiencyMechanism for ObservationPack {
    fn name(&self) -> &str { "observation_pack" }
    fn should_activate(&self, ctx: &HarnessContext) -> bool {
        ctx.output_buffer.len() > 1000
    }
    fn apply(&self, ctx: &mut HarnessContext) -> Result<(), String> {
        if ctx.output_buffer.len() > 1000 {
            let handle = format!("[packed:{}bytes]", ctx.output_buffer.len());
            ctx.output_buffer = handle;
        }
        Ok(())
    }
}

/// Evidence-Preserving Reducer — compress logs while preserving source references
pub struct EvidencePreservingReducer;

impl EfficiencyMechanism for EvidencePreservingReducer {
    fn name(&self) -> &str { "evidence_reducer" }
    fn should_activate(&self, ctx: &HarnessContext) -> bool {
        ctx.evidence_log.len() > 10
    }
    fn apply(&self, ctx: &mut HarnessContext) -> Result<(), String> {
        if ctx.evidence_log.len() > 10 {
            // Keep first 3, last 3, and a summary of the rest
            let total = ctx.evidence_log.len();
            let reduced: Vec<String> = ctx.evidence_log.iter().take(3).cloned()
                .chain(std::iter::once(format!("[...{} entries compressed...]", total - 6)))
                .chain(ctx.evidence_log.iter().skip(total - 3).cloned())
                .collect();
            ctx.evidence_log = reduced;
        }
        Ok(())
    }
}

/// Online Context Compact — trigger compression at subtask boundaries
pub struct OnlineContextCompact;

impl EfficiencyMechanism for OnlineContextCompact {
    fn name(&self) -> &str { "context_compact" }
    fn should_activate(&self, ctx: &HarnessContext) -> bool {
        ctx.turn_count > 0 && ctx.turn_count.is_multiple_of(5)
    }
    fn apply(&self, ctx: &mut HarnessContext) -> Result<(), String> {
        // At subtask boundaries, compact the output buffer
        if ctx.output_buffer.len() > 500 {
            ctx.output_buffer = format!("[compact:{}]", ctx.output_buffer.len());
        }
        Ok(())
    }
}

pub mod compaction;

/// Registry of all efficiency mechanisms
pub struct HarnessRegistry {
    mechanisms: Vec<Box<dyn EfficiencyMechanism>>,
}

impl HarnessRegistry {
    pub fn new() -> Self {
        let mechanisms: Vec<Box<dyn EfficiencyMechanism>> = vec![
            Box::new(ActionFusion),
            Box::new(ObservationPack),
            Box::new(EvidencePreservingReducer),
            Box::new(OnlineContextCompact),
        ];
        Self { mechanisms }
    }

    pub fn apply_all(&self, ctx: &mut HarnessContext) -> Vec<String> {
        let mut applied = Vec::new();
        for m in &self.mechanisms {
            if m.should_activate(ctx)
                && m.apply(ctx).is_ok() {
                    applied.push(m.name().to_string());
                }
        }
        applied
    }

    pub fn list(&self) -> Vec<&str> {
        self.mechanisms.iter().map(|m| m.name()).collect()
    }
}

impl Default for HarnessRegistry {
    fn default() -> Self { Self::new() }
}
