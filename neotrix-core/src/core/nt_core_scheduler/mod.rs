pub mod engine;
pub mod event_driven;
pub mod history;
pub mod job_queue;

pub use engine::{ContextGate, ScheduleType, ScheduledJob, SchedulerEngine};
pub use event_driven::{ConsciousnessEvent, EventDrivenScheduler};
pub use history::{JobRunHistory, JobRunRecord, SchedulerStats};
pub use job_queue::{CognitiveJob, CognitiveJobQueue, JobPriority, JobState};

/// Convenience function: create a scheduler with the default cleanup job pre-registered.
///
/// This is the main entry point for integrating the scheduler into BackgroundLoop.
/// Registers:
///   - `build_cleanup`: daily project artifact cleanup (anchor: process start)
///     gated by cognitive load (won't run during heavy thinking)
pub fn default_scheduler(anchor_now: u64) -> SchedulerEngine {
    let mut engine = SchedulerEngine::new();

    // Daily build cleanup: every 86400s, anchor at process start, low-cog gate
    engine.add_job(ScheduledJob {
        id: "build_cleanup".into(),
        name: "Daily Build Cleanup".into(),
        schedule: ScheduleType::Interval { secs: 86400 },
        handler: "handle_build_cleanup".into(),
        enabled: true,
        last_run: None,
        next_run: 0,
        max_retries: 2,
        retry_count: 0,
        cooldown_secs: 3600,
        anchor_ts: Some(anchor_now),
        context_gate: ContextGate::LowCogLoad(0.6),
        description: "Remove stale target/, node_modules/, dist/ build artifacts every 24h".into(),
    });

    // Hourly knowledge aging (was separate ticker)
    engine.add_job(ScheduledJob {
        id: "knowledge_aging".into(),
        name: "Knowledge Aging Scan".into(),
        schedule: ScheduleType::Interval { secs: 3600 },
        handler: "handle_knowledge_aging".into(),
        enabled: true,
        last_run: None,
        next_run: 0,
        max_retries: 3,
        retry_count: 0,
        cooldown_secs: 300,
        anchor_ts: Some(anchor_now),
        context_gate: ContextGate::Any,
        description: "Score decay, stale detection, re-scan scheduling".into(),
    });

    // Evosc self-consolidation (consciousness-aware, every 30min)
    engine.add_job(ScheduledJob {
        id: "evosc_consolidation".into(),
        name: "EvoSC Self-Consolidation".into(),
        schedule: ScheduleType::Interval { secs: 1800 },
        handler: "handle_evosc_tick".into(),
        enabled: true,
        last_run: None,
        next_run: 0,
        max_retries: 1,
        retry_count: 0,
        cooldown_secs: 600,
        anchor_ts: Some(anchor_now),
        context_gate: ContextGate::LowCogLoad(0.7),
        description: "Contrastive reflection + parametric memory compression".into(),
    });

    // Work discovery: triage knowledge gaps + external signals every 30s
    engine.add_job(ScheduledJob {
        id: "work_discovery".into(),
        name: "Work Discovery Loop".into(),
        schedule: ScheduleType::Interval { secs: 30 },
        handler: "handle_work_discovery".into(),
        enabled: true,
        last_run: None,
        next_run: 0,
        max_retries: 1,
        retry_count: 0,
        cooldown_secs: 10,
        anchor_ts: Some(anchor_now),
        context_gate: ContextGate::Any,
        description: "Triage knowledge gaps, curiosity signals, external events into work items".into(),
    });

    // Independent verification: maker-checker evaluation every 2min
    engine.add_job(ScheduledJob {
        id: "independent_verify".into(),
        name: "Independent Verification".into(),
        schedule: ScheduleType::Interval { secs: 120 },
        handler: "handle_independent_verify".into(),
        enabled: true,
        last_run: None,
        next_run: 0,
        max_retries: 3,
        retry_count: 0,
        cooldown_secs: 30,
        anchor_ts: Some(anchor_now),
        context_gate: ContextGate::LowCogLoad(0.5),
        description: "Separated-context evaluation of reasoning outputs against rubrics".into(),
    });

    // Loop audit: readiness + failure detection every 5min
    engine.add_job(ScheduledJob {
        id: "loop_audit".into(),
        name: "Loop Audit".into(),
        schedule: ScheduleType::Interval { secs: 300 },
        handler: "handle_loop_audit".into(),
        enabled: true,
        last_run: None,
        next_run: 0,
        max_retries: 2,
        retry_count: 0,
        cooldown_secs: 60,
        anchor_ts: Some(anchor_now),
        context_gate: ContextGate::LowCogLoad(0.4),
        description: "10-dim readiness scoring + 5 failure pattern detection".into(),
    });

    engine
}
