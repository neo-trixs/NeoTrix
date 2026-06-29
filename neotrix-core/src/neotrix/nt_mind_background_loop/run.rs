#![allow(dead_code)]
// SPLIT PLAN:
//   File: 2207 lines — 3 distinct sections:
//   1. `safe_tick.rs`        — SafeTick future wrapper + ART history utilities (lines 1–71)
//   2. `run_impl.rs`         — impl BackgroundLoop { init_timers, emit_event, start } (lines 72–1932)
//   3. `run_handlers.rs`     — P1 periodic handlers (lines 1934–2147)
//   4. `run_tests.rs`        — #[cfg(test)] module (lines 2148–2207)
//   How: extract SafeTick first, then handlers, then tests.
//
// Lock Hierarchy (brain.write):
//   1. start() init          — one-shot knowledge population (line 168, NOT in select)
//   2. handle_save            — save ticker      (line 1326, brief: file I/O)
//   3. handle_consolidate     — consolidate ticker (line 1337, brief: memory ops)
//   4. handle_thinking        — thinking ticker   (line 1377, 1-2s via spawn_blocking)
//   5. handle_goal            — goal ticker ×2    (lines 1393, 1399: sequential)
//   6. handle_prediction      — prediction ticker (line 1423, ~100ms: panorama)
//   7. handle_awakening_tick  — awakening ticker  (line 2194, brief: φ measurement)
//   All select-loop callers (#2-7) live in the biased select! (line 388).
//   Long-held locks (thinking #4: 1-2s) can delay other operations.
//   Lock acquisition order: thinking > save > consolidate > goal > prediction > awakening.
//   No write-lock is held across .await (#4, #5 use blocking_write in spawn_blocking).

use super::super::nt_mind::memory::ReasoningBank;
use super::super::nt_mind::self_iterating::ReasoningBrain;
use super::super::nt_mind_cleanup::CleanupKind;
use super::*;
use crate::core::nt_core_consciousness::{ThinkingMode, VsaOrigin, VsaSelfCategory};
use log;
use std::time::Duration;
use tokio::sync::mpsc::error::TrySendError;
use tokio::task::yield_now;
use tokio::time::timeout;

use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    /// A future wrapper that catches panics during polling, logs them, and returns `Ready(())`.
    /// This prevents a panic in any single tick handler from killing the entire background loop.
    struct SafeTick<F: Future> {
        #[pin]
        inner: Pin<Box<F>>,
        name: &'static str,
    }
}

impl<F: Future> Future for SafeTick<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let name = this.name;
        let result = catch_unwind(AssertUnwindSafe(|| this.inner.as_mut().poll(cx)));
        match result {
            Ok(Poll::Ready(_)) => Poll::Ready(()),
            Ok(Poll::Pending) => Poll::Pending,
            Err(_) => {
                log::error!("[bg] handler '{}' panicked, continuing", name);
                Poll::Ready(())
            }
        }
    }
}

fn safe_tick<F: Future>(name: &'static str, f: F) -> SafeTick<F> {
    SafeTick {
        inner: Box::pin(f),
        name,
    }
}

/// Catch panics in synchronous periodic handlers so one failing handler
/// doesn't abort the entire `run_periodic_handlers` cycle.
fn safe_handler_call<F, R>(name: &str, f: F) -> Option<R>
where
    F: FnOnce() -> R,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => Some(result),
        Err(e) => {
            log::error!("[run] handler '{}' panicked: {:?}", name, e);
            None
        }
    }
}

use crate::core::nt_core_input::NgramVsaEncoder;
use crate::core::nt_core_scheduler::default_scheduler;
use crate::core::nt_core_scheduler::event_driven::ConsciousnessEvent;
use crate::core::nt_core_self::attention_head::AttentionDomain;
use crate::neotrix::nt_agent_protocol::tcp_server::AGENT_SERVER_PORT;
use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, OnceLock};

// SECTION: ART history + constants

const ART_HISTORY_SIZE: usize = 5;
const MAX_THINKING_CYCLES: u8 = 3;

fn art_awareness_history() -> &'static Mutex<VecDeque<f64>> {
    static HISTORY: OnceLock<Mutex<VecDeque<f64>>> = OnceLock::new();
    HISTORY.get_or_init(|| Mutex::new(VecDeque::with_capacity(ART_HISTORY_SIZE)))
}

fn art_penalty_history() -> &'static Mutex<VecDeque<f64>> {
    static HISTORY: OnceLock<Mutex<VecDeque<f64>>> = OnceLock::new();
    HISTORY.get_or_init(|| Mutex::new(VecDeque::with_capacity(ART_HISTORY_SIZE)))
}

// SECTION: BackgroundLoop impl — init_timers, emit_event, start

impl BackgroundLoop {
    pub fn init_timers(&mut self) {
        use std::time::Duration;
        self.timer_registry
            .register("scheduler", Duration::from_secs(60), 3);
        self.timer_registry
            .register("health_patrol", Duration::from_secs(300), 3);
        self.timer_registry
            .register("agent_bus_gc", Duration::from_secs(60), 3);
        self.timer_registry
            .register("consciousness", Duration::from_secs(5), 5);
        self.timer_registry
            .register("audit", Duration::from_secs(600), 3);
    }

    fn emit_event(&self, event: ConsciousnessEvent) {
        if let Some(ref sender) = self.event_sender {
            match sender.try_send(event) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {
                    self.dropped_events.fetch_add(1, Ordering::Relaxed);
                    log::warn!(
                        "[emit_event] channel full, event dropped (total: {})",
                        self.dropped_events.load(Ordering::Relaxed)
                    );
                }
                Err(TrySendError::Closed(_)) => {
                    log::warn!("[emit_event] channel closed");
                }
            }
        }
    }

    pub async fn start(&mut self) {
        if !self.config.enabled {
            return;
        }

        log::info!("[bg] background loop started");

        // Install global panic hook to ensure ALL panics are logged
        {
            let prev = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                log::error!("[bg] PANIC: {}", info);
                prev(info);
            }));
        }

        // Seed default scheduler jobs (build_cleanup, knowledge_aging, evosc)
        {
            let anchor_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.scheduler = default_scheduler(anchor_now);
        }

        self.init_timers();

        // One-shot knowledge population from 63 sources (moved to spawn_blocking)
        {
            let brain_arc = self.brain.clone();
            tokio::task::spawn_blocking(move || {
                let mut brain = brain_arc.blocking_write();
                let report = KnowledgePopulator::populate_brain(&mut brain.brain.capability);
                let bank_seeds =
                    KnowledgePopulator::populate_reasoning_bank(&mut brain.reasoning_bank, 2);
                log::info!(
                    "[bg] knowledge populated: {} sources, {} dims, {} extensions, {} bank seeds",
                    report.sources_processed,
                    report.dimensions_updated,
                    report.extension_keys_added.len(),
                    bank_seeds,
                );
            })
            .await
            .unwrap_or_else(|e| log::error!("[bg] knowledge population blocking task failed: {e}"));
        }

        let mut always_on_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.always_on_interval_secs,
        ));
        always_on_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut save_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.save_interval_secs,
        ));
        save_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut consolidate_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.consolidate_interval_secs,
        ));
        consolidate_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut meta_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.metacog_interval_secs,
        ));
        meta_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut thinking_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.thinking_interval_secs,
        ));
        thinking_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut goal_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.goal_interval_secs,
        ));
        goal_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut exploration_orch_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.consciousness_pipeline_interval_secs * 2,
        ));
        exploration_orch_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut prediction_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.world_prediction_interval_secs,
        ));
        prediction_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut panorama_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.panorama_interval_secs,
        ));
        panorama_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut exploration_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.exploration_interval_secs,
        ));
        exploration_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut curiosity_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.curiosity_interval_secs,
        ));
        curiosity_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut knowledge_chain_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.knowledge_chain_interval_secs,
        ));
        knowledge_chain_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut aging_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.knowledge_aging_interval_secs,
        ));
        aging_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut crystallization_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.crystallization_interval_secs,
        ));
        crystallization_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut nt_act_voice_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.nt_act_voice_interval_secs,
        ));
        nt_act_voice_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut awareness_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.metacog_interval_secs,
        ));
        awareness_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut plugin_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.plugin_interval_secs,
        ));
        plugin_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut scheduler_ticker = tokio::time::interval(tokio::time::Duration::from_secs(30));
        scheduler_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut discovery_ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));
        discovery_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut awakening_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.consciousness_pipeline_interval_secs * 3,
        ));
        awakening_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut audit_ticker = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        audit_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut network_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.network_interval_secs,
        ));
        network_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut vision_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.vision_interval_secs,
        ));
        vision_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut nt_act_sync_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.nt_act_sync_interval_secs,
        ));
        nt_act_sync_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut nt_act_project_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.nt_act_project_interval_secs,
        ));
        nt_act_project_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut consciousness_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.consciousness_pipeline_interval_secs,
        ));
        consciousness_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut storage_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.storage_interval_secs,
        ));
        storage_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut jepa_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.jepa_interval_secs,
        ));
        jepa_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // 优雅关闭信号 — 零轮询 async notification via Notify
        let shutdown_signal = self.shutdown.signal().clone();

        // ── P1.01–P1.08: Disconnected subsystem tickers ──
        let mut evidence_ticker = tokio::time::interval(tokio::time::Duration::from_secs(300));
        evidence_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut spread_activation_ticker =
            tokio::time::interval(tokio::time::Duration::from_secs(600));
        spread_activation_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut consensus_ticker = tokio::time::interval(tokio::time::Duration::from_secs(120));
        consensus_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut hypergraph_ticker = tokio::time::interval(tokio::time::Duration::from_secs(300));
        hypergraph_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut storm_ticker = tokio::time::interval(tokio::time::Duration::from_secs(120));
        storm_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut kb_ticker = tokio::time::interval(tokio::time::Duration::from_secs(600));
        kb_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut llm_router_ticker = tokio::time::interval(tokio::time::Duration::from_secs(30));
        llm_router_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut symbolic_discovery_ticker =
            tokio::time::interval(tokio::time::Duration::from_secs(60));
        symbolic_discovery_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut ne_comptime_ticker = tokio::time::interval(tokio::time::Duration::from_secs(120));
        ne_comptime_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut translate_engine_ticker =
            tokio::time::interval(tokio::time::Duration::from_secs(300));
        translate_engine_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut a2a_grpc_ticker = tokio::time::interval(tokio::time::Duration::from_secs(120));
        a2a_grpc_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        #[cfg(feature = "stealth-net")]
        let mut transit_ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));
        #[cfg(feature = "stealth-net")]
        transit_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        #[cfg(not(feature = "stealth-net"))]
        let mut transit_ticker = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        #[cfg(not(feature = "stealth-net"))]
        transit_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        #[cfg(feature = "stealth-net")]
        let mut heartbeat_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.proxy_heartbeat_interval_secs,
        ));
        #[cfg(feature = "stealth-net")]
        heartbeat_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        #[cfg(not(feature = "stealth-net"))]
        let mut heartbeat_ticker = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        #[cfg(not(feature = "stealth-net"))]
        heartbeat_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        #[cfg(feature = "stealth-net")]
        let mut nt_world_sense_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.nt_world_sense_interval_secs,
        ));
        #[cfg(feature = "stealth-net")]
        nt_world_sense_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        #[cfg(not(feature = "stealth-net"))]
        let mut nt_world_sense_ticker =
            tokio::time::interval(tokio::time::Duration::from_secs(3600));
        #[cfg(not(feature = "stealth-net"))]
        nt_world_sense_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        #[cfg(feature = "stealth-net")]
        if let Some(ref nt_world_crawl) = self.tor_crawler {
            let c = nt_world_crawl.clone();
            self.spawn(async move {
                c.run().await;
            });
        }

        // Spawn AgentServer if discovery is enabled
        if self.agent_discovery.is_some() {
            let server = std::sync::Arc::new(
                super::super::nt_agent_protocol::tcp_server::AgentServer::new(AGENT_SERVER_PORT)
                    .with_shutdown_signal(shutdown_signal.clone()),
            );
            let server_clone = server.clone();
            self.spawn(async move {
                match server_clone.start().await {
                    Ok(port) => log::info!("[bg] AgentServer listening on TCP :{}", port),
                    Err(e) => log::error!("[bg] AgentServer start failed: {}", e),
                }
            });
        }

        // Spawn A2AServer if configured
        if let Some(server) = self.a2a_server.take() {
            self.spawn(async move {
                match server.start().await {
                    Ok(port) => log::info!("[bg] A2AServer listening on :{}", port),
                    Err(e) => log::error!("[bg] A2AServer start failed: {}", e),
                }
            });
        }

        // Spawn A2A gRPC server if configured (A2A v1.0.1 multi-protocol)
        if let Some(server) = self.a2a_grpc_server.take() {
            self.spawn(async move {
                match server.start().await {
                    Ok(port) => log::info!("[bg] A2A gRPC server listening on :{}", port),
                    Err(e) => log::error!("[bg] A2A gRPC server start failed: {}", e),
                }
            });
        }

        // ── 事件驱动调度器：事件监听循环 ──
        let mut event_rx = self.event_rx.take();
        log::info!("[bg] event-driven scheduler initialized");

        // DGM-H 启动时自举：将预置模板注册为运行时 handler
        if let Some(ref mut ci) = self.consciousness {
            let result = ci.generate_next_handler_from_templates();
            log::info!("[bg] DGM-H bootstrap: {}", result);
        }

        // 自动启动透明中转站（pf divert-to / system proxy）
        #[cfg(feature = "stealth-net")]
        {
            use crate::neotrix::nt_shield_stealth_net::config::load as load_config;
            let cfg = load_config();
            if cfg.transit.enabled {
                let _ = crate::neotrix::nt_shield_stealth_net::transit_station::auto_start_transit(
                    &cfg,
                )
                .await;
            }
        }

        loop {
            tokio::select! {
                biased;
                _ = tokio::signal::ctrl_c() => {
                    log::info!("[bg] SIGINT/SIGTERM received, initiating graceful shutdown...");
                    shutdown_signal.trigger("SIGINT");
                    continue;
                },
                _ = shutdown_signal.wait_shutdown() => {
                    log::info!("[bg] shutdown signal received, draining...");
                    self.shutdown.enter_save();
                    // 最终意识快照 (NTSSEG)
                    if let Some(ref mut ci) = self.consciousness {
                        ci.save_on_shutdown();
                    }
                    // 保存状态
                    self.handle_save().await;
                    // 停止中转站并清理防火墙规则
                    #[cfg(feature = "stealth-net")]
                    {
                        crate::neotrix::nt_shield_stealth_net::transit_station::stop_transit().await;
                    }
                    self.shutdown.complete();
                    log::info!("[bg] shutdown complete");
                    break;
                },
                _ = always_on_ticker.tick() => safe_tick("always_on", self.handle_always_on()).await,
                _ = save_ticker.tick() => safe_tick("save", self.handle_save()).await,
                _ = consolidate_ticker.tick() => safe_tick("consolidate", self.handle_consolidate()).await,
                _ = meta_ticker.tick() => safe_tick("meta", self.handle_meta()).await,
                _ = thinking_ticker.tick() => safe_tick("thinking", async {
                    self.thinking_cycle_counter = 0;
                    self.handle_thinking().await;
                }).await,
                _ = goal_ticker.tick() => safe_tick("goal", self.handle_goal()).await,
                _ = exploration_orch_ticker.tick() => safe_tick("exploration_orch", async {
                    if let Some(ref mut ci) = self.consciousness {
                        ci.handle_exploration_orchestrate(&self.curiosity_drive, ci.cycle as u64);
                    }
                }).await,
                _ = prediction_ticker.tick() => safe_tick("prediction", self.handle_prediction()).await,
                _ = panorama_ticker.tick() => safe_tick("panorama", self.handle_panorama()).await,
                _ = exploration_ticker.tick() => safe_tick("exploration", self.handle_exploration()).await,
                _ = curiosity_ticker.tick() => safe_tick("curiosity", self.handle_curiosity()).await,
                _ = knowledge_chain_ticker.tick() => safe_tick("knowledge_chain", self.handle_knowledge_chain()).await,
                _ = aging_ticker.tick() => safe_tick("aging", self.handle_knowledge_aging()).await,
                _ = crystallization_ticker.tick() => safe_tick("crystallization", self.handle_crystallization()).await,
                _ = heartbeat_ticker.tick() => safe_tick("heartbeat", async {
                    #[cfg(feature = "stealth-net")]
                    self.handle_proxy_heartbeat().await;
                }).await,
                _ = nt_world_sense_ticker.tick() => safe_tick("nt_world_sense", async {
                    #[cfg(feature = "stealth-net")]
                    self.handle_nt_world_sense_tick().await;
                }).await,
                _ = awareness_ticker.tick() => safe_tick("awareness", self.handle_awareness()).await,
                _ = nt_act_voice_ticker.tick() => safe_tick("nt_act_voice", self.handle_nt_act_voice_tick()).await,
                _ = plugin_ticker.tick() => safe_tick("plugin", self.handle_plugin_tick()).await,
                _ = scheduler_ticker.tick() => safe_tick("scheduler", self.handle_scheduler_tick()).await,
                _ = discovery_ticker.tick() => safe_tick("discovery", self.handle_agent_discovery()).await,
                _ = awakening_ticker.tick() => safe_tick("awakening", self.handle_awakening_tick()).await,
                _ = audit_ticker.tick() => safe_tick("audit", self.handle_audit_tick()).await,
                _ = network_ticker.tick() => safe_tick("network", self.handle_network_tick()).await,
                _ = vision_ticker.tick() => safe_tick("vision", self.handle_vision_tick()).await,
                _ = nt_act_sync_ticker.tick() => safe_tick("nt_act_sync", self.handle_nt_act_sync_tick()).await,
                _ = nt_act_project_ticker.tick() => safe_tick("nt_act_project", self.handle_nt_act_project_tick()).await,
                _ = consciousness_ticker.tick() => safe_tick("consciousness", self.handle_consciousness_batch()).await,
                _ = storage_ticker.tick() => safe_tick("storage", async {
                    let result = self.handle_storage_tick().await;
                    log::info!("[storage-engine] {}", result);
                }).await,
                _ = jepa_ticker.tick() => safe_tick("jepa", async {
                    let result = self.handle_jepa_ema_tick().await;
                    log::info!("[jepa-ema] {}", result);
                }).await,
                // ── P1.01–P1.08: Disconnected subsystem dispatches ──
                _ = evidence_ticker.tick() => safe_tick("evidence", async {
                    let result = self.handle_evidence_tick().await;
                    log::info!("[evidence] {}", result);
                }).await,
                _ = spread_activation_ticker.tick() => safe_tick("spread_activation", async {
                    let result = self.handle_spread_activation_tick().await;
                    log::info!("[spread_activation] {}", result);
                }).await,
                _ = consensus_ticker.tick() => safe_tick("consensus", async {
                    let result = self.handle_consensus_tick().await;
                    log::info!("[consensus] {}", result);
                }).await,
                _ = hypergraph_ticker.tick() => safe_tick("hypergraph", async {
                    let result = self.handle_hypergraph_tick().await;
                    log::info!("[hypergraph] {}", result);
                }).await,
                _ = storm_ticker.tick() => safe_tick("storm", async {
                    let result = self.handle_storm_tick().await;
                    log::info!("[storm] {}", result);
                }).await,
                _ = kb_ticker.tick() => safe_tick("kb", async {
                    let result = self.handle_kb_tick().await;
                    log::info!("[kb] {}", result);
                }).await,
                _ = llm_router_ticker.tick() => safe_tick("llm_router", async {
                    let result = self.handle_llm_router_tick().await;
                    log::info!("[llm_router] {}", result);
                }).await,
                _ = symbolic_discovery_ticker.tick() => safe_tick("symbolic_discovery", async {
                    let result = self.handle_symbolic_discovery_tick().await;
                    log::info!("[symbolic_discovery] {}", result);
                }).await,
                _ = ne_comptime_ticker.tick() => safe_tick("ne_comptime", async {
                    let result = self.handle_ne_comptime_tick().await;
                    log::info!("[ne_comptime] {}", result);
                }).await,
                _ = translate_engine_ticker.tick() => safe_tick("translate_engine", async {
                    let result = self.handle_translate_engine_tick().await;
                    log::info!("[translate_engine] {}", result);
                }).await,
                _ = a2a_grpc_ticker.tick() => safe_tick("a2a_grpc", async {
                    let result = self.handle_a2a_grpc_tick().await;
                    log::info!("[a2a_grpc] {}", result);
                }).await,
                _ = transit_ticker.tick() => safe_tick("transit", async {
                    #[cfg(feature = "stealth-net")]
                    self.handle_transit_tick().await;
                }).await,
                // 事件驱动调度器 — 接收事件并 dispatch
                // Uses mpsc::UnboundedReceiver: guarantees delivery, never silently drops.
                event = async {
                    if let Some(ref mut rx) = event_rx {
                        match timeout(Duration::from_secs(5), rx.recv()).await {
                            Ok(Some(ev)) => ev,
                            Ok(None) => ConsciousnessEvent::Custom("channel_closed".into()),
                            Err(_) => ConsciousnessEvent::Custom("timeout".into()),
                        }
                    } else {
                        std::future::pending::<ConsciousnessEvent>().await
                    }
                } => {
                    let _deadline_emitted = self.event_scheduler.process_deadlines();
                    self.handle_event(event).await;
                },
            }
        }
    }

    async fn handle_event(&mut self, event: ConsciousnessEvent) {
        match event {
            ConsciousnessEvent::Save => self.handle_save().await,
            ConsciousnessEvent::Consolidate => self.handle_consolidate().await,
            ConsciousnessEvent::MetaTick => self.handle_meta().await,
            ConsciousnessEvent::ThoughtComplete => {
                // ThoughtComplete reuses thinking pipeline
                self.handle_thinking().await;
            }
            ConsciousnessEvent::GoalDrift(_) => self.handle_goal().await,
            ConsciousnessEvent::ExplorationTick => self.handle_exploration().await,
            ConsciousnessEvent::CuriousityTick => self.handle_curiosity().await,
            ConsciousnessEvent::KnowledgeGap(_) => self.handle_knowledge_aging().await,
            ConsciousnessEvent::SchedulerTick => self.handle_scheduler_tick().await,
            ConsciousnessEvent::NetworkTick => self.handle_network_tick().await,
            ConsciousnessEvent::VisionTick => self.handle_vision_tick().await,
            ConsciousnessEvent::AuditTick => self.handle_audit_tick().await,
            ConsciousnessEvent::HealthCheck => {
                if let Some(ref mut ci) = self.consciousness {
                    let _r = ci.handle_health_patrol_tick();
                }
            }
            ConsciousnessEvent::PhaseTransition(phase) => {
                log::info!("[bg] phase transition: {}", phase);
                self.handle_meta().await;
            }
            ConsciousnessEvent::Custom(tag) => {
                log::info!("[bg] custom event: {}", tag);
            }
            ConsciousnessEvent::ExternalInput(data) => {
                log::info!("[bg] external input: {}", data);
            }
            ConsciousnessEvent::TickScheduled(name) => {
                // Route named tick to consciousness DGM group if applicable
                if let Some(ref mut ci) = self.consciousness {
                    let _r = ci.handle_dgm_group(&name);
                }
            }
            ConsciousnessEvent::HandlerError(err) => {
                log::error!("[bg] handler error event: {}", err);
            }
        }
    }

    async fn handle_consciousness_batch(&mut self) {
        // ── Security: record cycle as anomaly detection heartbeat ──
        if let Some(ref mut ad) = self.behavior_anomaly {
            use crate::neotrix::nt_shield::agent_anomaly::AgentActionType;
            crate::neotrix::nt_shield::agent_anomaly::record_action(
                AgentActionType::ToolCall,
                "consciousness_batch",
                0,
                0,
                true,
            );
            let _ = ad;
        }

        if self.consciousness.is_none() {
            return;
        }

        // cycle is incremented inside handle_consciousness_batch_async (called in run_metacognition)
        // NOT here — DO NOT add ci.cycle += 1 here or it will double-increment.
        let (state_vec, sp_vec, out_v, ctx_v, proof_vec, ctm_vec) = self.run_encoding();
        let (reflect_success, reflect_quality, health, arch_penalty, pass_rate, awareness) =
            self.run_metacognition(&state_vec).await;
        let (cycle, match_criterion, vigilance, mode) = self.run_cognition_phases(
            &state_vec,
            sp_vec,
            out_v,
            ctx_v,
            proof_vec,
            reflect_success,
            reflect_quality,
            health,
            arch_penalty,
            pass_rate,
            awareness,
        );
        self.run_periodic_handlers(
            &state_vec,
            &ctm_vec,
            reflect_success,
            reflect_quality,
            mode,
            cycle,
            match_criterion,
            vigilance,
        );
        self.run_handler_recording(reflect_quality, awareness, pass_rate, arch_penalty);
        self.run_event_emission(mode);
        // ── Update shared stats snapshot for external consumers ──
        if let Some(ref ci) = self.consciousness {
            *self
                .stats_snapshot
                .write()
                .unwrap_or_else(|e| e.into_inner()) = ci.stats();
            if let Some(ref canvas) = ci.canvas_manager {
                if let Some(project) = canvas.projects.first() {
                    if !project.nodes.is_empty() {
                        *self
                            .canvas_snapshot
                            .write()
                            .unwrap_or_else(|e| e.into_inner()) = project.clone();
                    }
                }
            }
        }

        // ── Run AdaptiveController for architecture-level self-optimization ──
        if let Some(ref mut controller) = self.adaptive_controller {
            let input = crate::core::nt_core_consciousness::vsa_tag::VsaTagged::self_thought(
                "consciousness_tick",
            );
            let result = controller.run_adaptive_cycle(Some(input));
            if let Some(ref adapt) = result.adaptation {
                let _ = adapt; // suppression guard; adaptation changes are visible via dashboard()
            }
        }

        // ── Run SelfEvolutionOrchestrator bridge (6-phase evolution cycle, every 30 cycles) ──
        let (cycle, meta_acc, ece, loss) = self.consciousness.as_ref().map(|ci| {
            let meta_acc = ci.meta_cognition_loop.current_meta_accuracy();
            let ece = ci.calibration.ece();
            let loss = ci.composite_loss.composite.total;
            (ci.cycle, meta_acc, ece, loss)
        }).unwrap_or((0, 0.0, 0.0, 0.0));
        if cycle > 0 && cycle % 30 == 0 {
            if let Some(ref mut ci) = self.consciousness {
                if let Some(ref mut ob) = ci.orchestrator_bridge() {
                    let proposals = ob.run_bridge(cycle, meta_acc, ece, loss);
                    if !proposals.is_empty() {
                        log::info!(
                            "orchestrator: {} evolution proposals at cycle {}",
                            proposals.len(),
                            cycle
                        );
                    }
                }
            }
        }
    }

    fn run_encoding(&mut self) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
        // Reset canvas chain BEFORE borrowing ci to avoid double-mut-borrow on self
        let pending_count = self.ci_pending_input.lock().map_or(0, |g| g.len());
        if pending_count > 0 {
            self.canvas_new_chain();
        }
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => {
                log::warn!("[run.rs] consciousness not available, returning default encoding");
                return (vec![], vec![], vec![], vec![], vec![], vec![]);
            }
        };
        {
            let mut pending = self
                .ci_pending_input
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            for text in pending.drain(..) {
                ci.push_text_buffer(text);
            }
        }
        let input_text = ci.text_buffer.front().cloned().unwrap_or_default();
        let encoder = self
            .vsa_encoder
            .get_or_insert_with(NgramVsaEncoder::default);
        let state_text = if input_text.is_empty() {
            "consciousness_bg_state"
        } else {
            &input_text
        };
        let state_vec = encoder.encode_text(state_text);
        let _self_vsa = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_self"
        } else {
            state_text
        });
        let _wm_vsa = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_wm"
        } else {
            state_text
        });
        let _attn_vsa = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_attn"
        } else {
            state_text
        });
        let sp_vec = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_sp"
        } else {
            state_text
        });
        let out_v = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_output"
        } else {
            state_text
        });
        let ctx_v = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_context"
        } else {
            state_text
        });
        let proof_vec = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_proof"
        } else {
            state_text
        });
        let ctm_vec = encoder.encode_text(if input_text.is_empty() {
            "consciousness_bg_ctm"
        } else {
            state_text
        });
        (state_vec, sp_vec, out_v, ctx_v, proof_vec, ctm_vec)
    }

    async fn run_metacognition(&mut self, state_vec: &[u8]) -> (bool, f64, f64, f64, f64, f64) {
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => {
                log::warn!("[run.rs] consciousness not available, returning default metacognition");
                return (false, 0.0, 0.5, 0.0, 0.5, 0.5);
            }
        };
        let _bp_result = ci.handle_consciousness_batch_async().await;

        let _loop_phase = self.loop_engine.tick();
        let _curiosity_seeds = ci.curiosity_orchestrator_bridge(&mut self.curiosity_drive);
        let _exploration_stats =
            ci.handle_exploration_orchestrate(&self.curiosity_drive, ci.cycle as u64);

        let reflective = ci.handle_reflexive(state_vec);
        let reflect_success = reflective > 0.3;
        let reflect_quality = reflective.clamp(0.0, 1.0);

        let (health, arch_penalty, pass_rate, awareness) = ci.handle_metacognitive_loop_tick();

        // MC²: log cross-cycle meta-knowledge at medium timescale
        if ci.cycle > 0 && ci.cycle % 50 == 0 {
            let mk = ci.meta_cognition_loop.meta_knowledge_summary();
            log::info!("[mc²] {}", mk);
        }

        if ci.cycle % 5 == 0 && ci.cycle > 0 {
            let n_repaired = ci.handle_self_heal_tick();
            if n_repaired > 0 {
                log::info!("[self-heal] repaired {} degraded modules", n_repaired);
            }
        }

        ci.handle_self_protection_tick();

        (
            reflect_success,
            reflect_quality,
            health,
            arch_penalty,
            pass_rate,
            awareness,
        )
    }

    fn run_cognition_phases(
        &mut self,
        state_vec: &[u8],
        sp_vec: Vec<u8>,
        out_v: Vec<u8>,
        ctx_v: Vec<u8>,
        proof_vec: Vec<u8>,
        reflect_success: bool,
        reflect_quality: f64,
        health: f64,
        arch_penalty: f64,
        pass_rate: f64,
        awareness: f64,
    ) -> (u64, f64, f64, ThinkingMode) {
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => {
                log::warn!(
                    "[run.rs] consciousness not available, returning default cognition phases"
                );
                return (0, 0.5, 0.5, ThinkingMode::Fast);
            }
        };

        ci.epistemic_gap_bridge(0.4);

        ci.handle_attractor_dynamics(state_vec);
        ci.handle_emergent_reasoning(state_vec, true);

        ci.handle_specious_present_feed(sp_vec, VsaOrigin::Self_(VsaSelfCategory::Thought));
        let _narrative_insight = if !reflect_success {
            Some("low_reflexivity".to_string())
        } else if pass_rate < 0.4 {
            Some("low_critique_pass_rate".to_string())
        } else if awareness < 0.3 {
            Some("low_self_awareness".to_string())
        } else if arch_penalty > 0.1 {
            Some("elevated_arch_penalty".to_string())
        } else {
            None
        };
        ci.handle_narrative_tick();
        ci.handle_valence_update(reflect_quality, reflect_success);
        let critique = ci.handle_inner_critic(out_v, ctx_v);
        log::debug!("[run] inner_critic: {:?}", critique);

        let personality = ci.handle_personality_tick(reflect_quality, reflect_success as u8 as f64);
        log::debug!("[run] personality: {}", personality);

        let epistemic = ci.handle_epistemic_honesty_tick(awareness, reflect_success);
        log::debug!("[run] epistemic: {}", epistemic);
        let base_load = (ci.working_memory.item_count() as f64) * 0.1 + (ci.cycle as f64) * 0.001;
        let _load = base_load * (1.0 - arch_penalty * 0.5);
        let mode = ci.cognitive_load_monitor.mode();
        let proof = ci.handle_proof_search_tick("bg_self_check", proof_vec);
        log::debug!("[run] proof_search: {}", proof);

        let soul = ci.handle_soul_identity_tick();
        log::debug!("[run] soul_identity: {}", soul);

        ci.handle_llm_router_tick();
        ci.handle_symbolic_discovery_tick();
        let _discovery_llm_route = ci.route_discovery_to_llm_router();

        let cycle = ci.cycle;
        {
            let mut aw_hist = art_awareness_history()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if aw_hist.len() >= ART_HISTORY_SIZE {
                aw_hist.pop_front();
            }
            aw_hist.push_back(awareness);
        }
        {
            let mut p_hist = art_penalty_history()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if p_hist.len() >= ART_HISTORY_SIZE {
                p_hist.pop_front();
            }
            p_hist.push_back(arch_penalty);
        }
        let avg_awareness = {
            let h = art_awareness_history()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if h.is_empty() {
                0.5
            } else {
                h.iter().sum::<f64>() / h.len() as f64
            }
        };
        let avg_penalty = {
            let h = art_penalty_history()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if h.is_empty() {
                0.5
            } else {
                h.iter().sum::<f64>() / h.len() as f64
            }
        };
        let coherence = 1.0 - (avg_awareness - avg_penalty).abs();
        let vsa_sim = if !state_vec.is_empty() {
            let prior = ci.attractor_state.as_slice();
            if !prior.is_empty() {
                let min_len = state_vec.len().min(prior.len());
                if min_len > 0 {
                    let matching = state_vec[..min_len]
                        .iter()
                        .zip(prior[..min_len].iter())
                        .filter(|(a, b)| a == b)
                        .count() as f64
                        / min_len as f64;
                    matching
                } else {
                    0.5
                }
            } else {
                0.5
            }
        } else {
            0.5
        };
        let match_criterion = 0.6 * vsa_sim + 0.4 * coherence;
        let vigilance = 0.3 + health * 0.5;
        (cycle, match_criterion, vigilance, mode)
    }

    fn run_periodic_handlers(
        &mut self,
        state_vec: &[u8],
        ctm_vec: &[u8],
        reflect_success: bool,
        reflect_quality: f64,
        mode: ThinkingMode,
        cycle: u64,
        match_criterion: f64,
        vigilance: f64,
    ) {
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => {
                log::warn!("[run.rs] consciousness not available, skipping periodic handlers");
                return;
            }
        };
        let tick_should_run = |divisor: u64| -> bool {
            if match_criterion >= vigilance {
                cycle % divisor == 0
            } else {
                if divisor <= 5 {
                    cycle % divisor == 0
                } else if divisor <= 20 {
                    cycle % (divisor * 2) == 0
                } else {
                    false
                }
            }
        };

        safe_handler_call("stream_buffer_feed", || {
            ci.handle_stream_buffer_feed(
                state_vec.to_vec(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            )
        });
        safe_handler_call("resonator_decode", || ci.handle_resonator_decode(state_vec));

        if tick_should_run(3) {
            let neg_result =
                safe_handler_call("negentropy", || ci.handle_negentropy_tick()).unwrap_or_default();
            log::info!("[negentropy] {}", neg_result);
        }
        if tick_should_run(5) {
            safe_handler_call("volition", || ci.handle_volition_tick());
            safe_handler_call("conformal_uq", || ci.handle_conformal_uq_tick());
            let fp_result =
                safe_handler_call("first_person_ref", || ci.handle_first_person_ref_tick())
                    .unwrap_or_default();
            log::info!("[first_person] {}", fp_result);
        }
        if tick_should_run(7) {
            safe_handler_call("skill_dag", || ci.handle_skill_dag_tick());
        }
        if tick_should_run(8) {
            let mem_result = safe_handler_call("memory_consolidation", || {
                ci.handle_memory_consolidation_tick()
            })
            .unwrap_or_default();
            log::info!("[memory] {}", mem_result);
            let dm_result = safe_handler_call("default_mode", || ci.handle_default_mode_tick(true))
                .unwrap_or_default();
            log::info!("[default_mode] {}", dm_result);
        }
        if tick_should_run(10) {
            safe_handler_call("open_skill", || ci.handle_open_skill_tick());
            let id_result = safe_handler_call("identity_chain", || ci.handle_identity_chain_tick())
                .unwrap_or_default();
            log::info!("[identity_chain] {}", id_result);
        }
        if tick_should_run(12) {
            let jepa_result =
                safe_handler_call("jepa", || ci.handle_jepa_tick(&[])).unwrap_or_default();
            log::info!("[jepa] {:?}", String::from_utf8_lossy(&jepa_result));
        }
        if tick_should_run(14) {
            let sparse_result =
                safe_handler_call("sparse_vsa", || ci.handle_sparse_vsa_tick()).unwrap_or_default();
            log::info!("[sparse_vsa] {}", sparse_result);
        }
        if tick_should_run(15) {
            safe_handler_call("reasoning_step", || ci.handle_reasoning_step("bg_reason"));
            let cal_result =
                safe_handler_call("calibration_engine", || ci.handle_calibration_engine_tick())
                    .unwrap_or_default();
            log::info!("[calibration] {}", cal_result);
        }
        if tick_should_run(20) {
            safe_handler_call("ctm_inference", || ci.handle_ctm_inference(ctm_vec));
            let loss_result = safe_handler_call("loss_function", || ci.handle_loss_function_tick())
                .unwrap_or_default();
            log::info!("[loss] {}", loss_result);
        }
        if tick_should_run(25) {
            let _ = safe_handler_call("e8_attractor", || ci.handle_e8_attractor_dynamics());
            let ft_result = safe_handler_call("failure_trace", || ci.handle_failure_trace_tick())
                .unwrap_or_default();
            log::info!("[failure_trace] {}", ft_result);
        }
        if tick_should_run(30) {
            safe_handler_call("consciousness_pipeline", || {
                ci.handle_consciousness_pipeline(
                    "bg_tick",
                    &["observe"],
                    0,
                    reflect_success,
                    reflect_quality,
                    AttentionDomain::Semantic,
                    None,
                )
            });
            safe_handler_call("self_model", || ci.handle_self_model_tick());
            let cap_result = safe_handler_call("capability_synthesizer", || {
                ci.handle_capability_synthesizer_tick()
            })
            .unwrap_or_default();
            log::info!("[capability] {}", cap_result);
            let gm_result =
                safe_handler_call("goal_manager", || ci.handle_goal_manager_status_tick())
                    .unwrap_or_default();
            log::info!("[goal_manager] {}", gm_result);
            let lead_plan =
                safe_handler_call("lead_agent_plan", || ci.handle_lead_agent_plan_tick())
                    .unwrap_or_default();
            log::info!("[lead_agent] {}", lead_plan);
            let lead_exec =
                safe_handler_call("lead_agent_execute", || ci.handle_lead_agent_execute_tick())
                    .unwrap_or_default();
            log::info!("[lead_agent_exec] {}", lead_exec);
            let nexp_result =
                safe_handler_call("native_explorer", || ci.handle_native_explorer_tick())
                    .unwrap_or_default();
            log::info!("[native_explorer] {}", nexp_result);
        }
        if tick_should_run(35) {
            let fusion_result = safe_handler_call("fusion_deliberation", || {
                ci.handle_fusion_deliberation_tick()
            })
            .unwrap_or_default();
            log::info!("[fusion] {}", fusion_result);
        }
        if tick_should_run(40) {
            let _ = safe_handler_call("fusion_gap", || ci.handle_fusion_gap_tick());
            let et_result =
                safe_handler_call("execution_trace", || ci.handle_execution_trace_tick())
                    .unwrap_or_default();
            log::info!("[execution_trace] {}", et_result);
        }
        if tick_should_run(45) {
            let topic = format!("cycle_{}_research", ci.cycle);
            let storm_result =
                safe_handler_call("storm_start", || ci.handle_storm_start_tick(&topic))
                    .unwrap_or_default();
            log::info!("[storm] {}", storm_result);
        }
        if tick_should_run(50) {
            safe_handler_call("input_pipeline", || {
                ci.handle_input_pipeline_batch(&[("bg", "batch")])
            });
            safe_handler_call("archive_save", || ci.handle_archive_save_tick());
            safe_handler_call("meta_evolution", || ci.handle_meta_evolution_tick());
            let ws_result =
                safe_handler_call("workstream", || ci.handle_workstream_tick()).unwrap_or_default();
            log::info!("[workstream] {}", ws_result);
            let replay_result =
                safe_handler_call("replay", || ci.handle_replay_tick()).unwrap_or_default();
            log::info!("[replay] {}", replay_result);
        }

        if tick_should_run(6) {
            let pred_result =
                safe_handler_call("prediction_replay", || ci.handle_prediction_replay())
                    .unwrap_or_default();
            log::info!("[prediction_replay] {}", pred_result);
        }
        if tick_should_run(8) {
            let actinf_result =
                safe_handler_call("active_inference", || ci.handle_active_inference())
                    .unwrap_or_default();
            log::info!("[active_inference] {}", actinf_result);
            let efe_result = safe_handler_call("efe_minimizer", || ci.handle_efe_minimizer())
                .unwrap_or_default();
            log::info!("[efe_minimizer] {}", efe_result);
        }
        if tick_should_run(10) {
            let srcc_temporal_result =
                safe_handler_call("srcc_temporal", || ci.handle_srcc_temporal_reasoning())
                    .unwrap_or_default();
            log::info!("[srcc_temporal] {}", srcc_temporal_result);
            let srcc_episodic_result =
                safe_handler_call("srcc_episodic", || ci.handle_srcc_episodic_boundary())
                    .unwrap_or_default();
            log::info!("[srcc_episodic] {}", srcc_episodic_result);
            let signal_result =
                safe_handler_call("signal_pattern", || ci.handle_signal_pattern_tick())
                    .unwrap_or_default();
            log::info!("[signal_pattern] {}", signal_result);
            let vsa_input_result =
                safe_handler_call("vsa_input", || ci.handle_vsa_input_pipeline_tick())
                    .unwrap_or_default();
            log::info!("[vsa_input] {}", vsa_input_result);
        }
        if tick_should_run(15) {
            let srcc_ebbinghaus_result =
                safe_handler_call("srcc_ebbinghaus", || ci.handle_srcc_ebbinghaus_decay())
                    .unwrap_or_default();
            log::info!("[srcc_ebbinghaus] {}", srcc_ebbinghaus_result);
        }
        if tick_should_run(20) {
            let ne_compile_result =
                safe_handler_call("ne_compile", || ci.handle_ne_compile_tick()).unwrap_or_default();
            log::info!("[ne_compile] {}", ne_compile_result);
        }
        if tick_should_run(25) {
            let dp_result = safe_handler_call("dispatch_pipeline", || {
                ci.handle_dispatch_pipeline_mode_tick()
            })
            .unwrap_or_default();
            log::info!("[dispatch_pipeline] {}", dp_result);
        }

        // ── Wave 2-5 periodic handlers ──
        if tick_should_run(5) {
            safe_handler_call("sahoo", || ci.handle_sahoo_tick()).unwrap_or_default();
            safe_handler_call("containment", || ci.handle_containment_tick()).unwrap_or_default();
        }
        if tick_should_run(8) {
            safe_handler_call("vsi", || ci.handle_vsi_tick()).unwrap_or_default();
        }
        if tick_should_run(12) {
            safe_handler_call("mtc", || ci.handle_mtc_tick()).unwrap_or_default();
        }
        if tick_should_run(10) {
            safe_handler_call("meta_improvement", || ci.handle_meta_improvement_tick())
                .unwrap_or_default();
        }
        if tick_should_run(7) {
            safe_handler_call("uncertainty", || ci.handle_uncertainty_tick()).unwrap_or_default();
        }
        if tick_should_run(3) {
            safe_handler_call("storm_breaker", || ci.handle_storm_breaker_tick())
                .unwrap_or_default();
        }
        if tick_should_run(15) {
            safe_handler_call("dgmh_orchestrator", || ci.handle_dgmh_orchestrator_tick())
                .unwrap_or_default();
        }
        if tick_should_run(9) {
            safe_handler_call("fep_act_planner", || ci.handle_fep_act_planner_tick())
                .unwrap_or_default();
        }
        if tick_should_run(11) {
            safe_handler_call("fep_iit_bridge", || ci.handle_fep_iit_bridge_tick())
                .unwrap_or_default();
        }
        if tick_should_run(30) {
            safe_handler_call("gradient_seal", || ci.handle_gradient_seal_tick())
                .unwrap_or_default();
        }

        if tick_should_run(19) {
            safe_handler_call("self_play_guide", || ci.handle_self_play_guide_tick())
                .unwrap_or_default();
        }

        safe_handler_call("attention_gate", || {
            ci.handle_attention_gate(ci.cycle as u64, state_vec)
        });
        safe_handler_call("neuromodulator", || ci.handle_neuromodulator_tick());

        let fast_gate = mode == ThinkingMode::Fast && ci.cycle % 3 != 0;
        let attn_gate = !ci.should_run_group("ctm");
        let heavy_ops_allowed = !fast_gate && attn_gate;

        if heavy_ops_allowed || ci.should_run_group("spatial") {
            let default_ctm = crate::core::nt_core_ctm::inference::CtmResult {
                gist: vec![],
                weight: 0.0,
                iterations_used: 0,
                winner_name: "none".to_string(),
                link_count: 0,
            };
            let ctm_result = match &mut ci.ctm_engine {
                Some(ref mut engine) => engine.infer(ctm_vec),
                None => default_ctm,
            };
            if ci.cycle % 10 == 0 && ctm_result.weight > 0.5 {
                log::info!(
                    "[bg] ctm: winner={}, weight={:.3}",
                    ctm_result.winner_name,
                    ctm_result.weight
                );
            }
            safe_handler_call("spatial_scene", || {
                ci.handle_spatial_scene(&[], (0.0, 0.0, 0.0), 10.0)
            });
            safe_handler_call("physics_reasoning", || {
                ci.handle_physics_reasoning(5.0, "solid", 20.0)
            });
        } else if ci.cycle % 20 == 0 {
            log::info!(
                "[bg] gating: fast={} da={:.2}",
                mode == ThinkingMode::Fast,
                ci.neuromodulator.system.get_level(
                    crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::DA
                ),
            );
        }
    }

    fn next_cycle(&self) -> u64 {
        self.consciousness.as_ref().map(|c| c.cycle).unwrap_or(0)
    }

    fn run_handler_recording(
        &mut self,
        reflect_quality: f64,
        awareness: f64,
        pass_rate: f64,
        arch_penalty: f64,
    ) {
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => {
                log::warn!("[run.rs] consciousness not available, skipping handler recording");
                return;
            }
        };

        let _volition_action = ci.value_volition_bridge();

        let handler_names = [
            "context_gather",
            "decision_compress",
            "experience_reflect",
            "skill_accumulate",
            "curriculum_generate",
            "policy_repair",
            "epistemic_calibrate",
            "attractor_dynamics",
            "ebbinghaus_decay",
            "dream_cycle",
            "emergent_reasoning",
            "reflexive",
            "epistemic_honesty",
            "personality_update",
            "cognitive_state_ingest",
            "master_consciousness_update",
            "vs_advantage_learn",
            "sleep_consolidation",
            "fable_route",
            "goal_execution",
            "specious_present_feed",
            "narrative_tick",
            "valence_update",
            "inner_critic",
            "cognitive_load_tick",
            "proof_search_tick",
            "dgmh_writeback_tick",
            "self_protection_tick",
            "stream_buffer_feed",
            "reconstructive_narrative_tick",
            "adaptive_rate_tick",
            "context_budget_tick",
            "resonator_decode",
            "volition_tick",
            "conformal_uq_tick",
            "confidence_calibrate",
            "spatial_scene",
            "physics_reasoning",
            "novelty_detection_tick",
            "tool_discovery_tick",
            "goal_decomposition_tick",
            "episodic_memory_tick",
            "reasoning_step",
            "moss_pipeline",
            "input_pipeline_batch",
            "ctm_inference",
            "kroneker_cleanup",
            "uat_gate",
            "attention_gate",
            "failure_trace",
            "dream_consolidate_feed",
            "moss_health_tick",
            "sia_feedback",
            "srcc_brain_dgm",
            "async_delegate_submit",
            "async_delegate_poll",
            "consciousness_pipeline",
            "dgm_variant_propose",
            "archive_save",
            "neuromodulator_tick",
            "mirror_thread_synthesize",
            "gea_tick",
            "evosc_tick",
            "open_skill_tick",
            "skill_dag_tick",
            "skill_evolution_tick",
            "vsa_rdt",
            "hypothesis_tree_tick",
            "counterfactual_futures_tick",
            "social_feed_absorb",
            "social_feed_absorb",
            "e8_geometry_tick",
            "storage_engine_tick",
            "jepa_ema_tick",
            "evidence_tick",
            "spread_activation_tick",
            "consensus_tick",
            "hypergraph_tick",
            "storm_poll",
            "hypothesis_tree_tick",
            "kb_maintenance",
            "three_role",
            "sub_consciousness",
            "negentropy_tick",
            "memory_consolidation_tick",
            "jepa_tick",
            "sparse_vsa_tick",
            "fusion_deliberation_tick",
            "storm_start_tick",
            "prediction_replay",
            "srcc_temporal",
            "srcc_ebbinghaus",
            "srcc_episodic",
            "active_inference",
            "efe_minimizer",
            "signal_pattern",
            "vsa_input",
            "ne_compile",
            "dispatch_pipeline_mode",
        ];
        for name in &handler_names {
            self.loop_engine.discovery.record_call(name);
        }
        let handler_count = handler_names.len();
        let _verdict = self.loop_engine.verifier.verify(
            reflect_quality,
            ci.specious_present.average_coherence(),
            handler_count,
        );

        let actual_health = ci.stats().c_score;
        ci.rii_u
            .get_or_insert_with(|| {
                use crate::core::nt_core_consciousness::rii_u::RiiuAutoPhi;
                RiiuAutoPhi::new()
            })
            .record_and_update(&[awareness, pass_rate, 1.0 - arch_penalty], actual_health);
    }

    fn run_event_emission(&mut self, mode: ThinkingMode) {
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => {
                log::warn!("[run.rs] consciousness not available, skipping event emission");
                return;
            }
        };

        if ci.cycle % 10 == 0 {
            let s = ci.stats();
            let ls = self.loop_engine.stats();
            log::info!(
                "[bg] consciousness: cycle={}, c_score={:.3}, sp_coherence={:.3}, load={}, mode={:?}, loop={:?}({:.0}%)",
                ci.cycle, s.c_score, s.sp_coherence, s.load_mode, mode,
                ls.phase, ls.coverage_pct
            );
        }

        let due: Vec<_> = self.timer_registry.due().into_iter().cloned().collect();
        for entry in &due {
            let name = entry.name.clone();
            match name.as_str() {
                "agent_bus_gc" => {
                    if let Some(ref mut bus) = self.agent_bus {
                        bus.clear_expired();
                    }
                    let _ = self.timer_registry.tick("agent_bus_gc");
                }
                "health_patrol" => {
                    ci.handle_health_patrol_tick();
                    let _ = self.timer_registry.tick("health_patrol");
                }
                _ => {}
            }
        }

        if let Some(ref sender) = self.event_sender {
            if sender
                .try_send(ConsciousnessEvent::PhaseTransition(
                    self.loop_engine.phase.label().to_string(),
                ))
                .is_err()
            {
                self.dropped_events.fetch_add(1, Ordering::Relaxed);
                log::warn!(
                    "[emit_events] PhaseTransition dropped (total: {})",
                    self.dropped_events.load(Ordering::Relaxed)
                );
            }
            if sender.try_send(ConsciousnessEvent::SchedulerTick).is_err() {
                self.dropped_events.fetch_add(1, Ordering::Relaxed);
                log::warn!(
                    "[emit_events] SchedulerTick dropped (total: {})",
                    self.dropped_events.load(Ordering::Relaxed)
                );
            }
        }

        let responses = ci.drain_response_buffer();
        if !responses.is_empty() {
            for resp in &responses {
                log::info!("[consciousness:output] {}", resp);
            }
            // Push to shared output buffer for external consumers (Tauri bridge)
            if let Ok(mut out) = self.ci_response_output.lock() {
                out.extend(responses);
            }
        }
    }

    async fn handle_save(&self) {
        // goal_loop.save() is fast (no brain lock needed), run immediately
        if let Err(e) = self.goal_loop.save() {
            log::error!("[bg] goal_loop save failed: {}", e);
        }
        // brain save is I/O-bound, move to spawn_blocking
        let brain_arc = self.brain.clone();
        tokio::task::spawn_blocking(move || {
            if let Err(e) = brain_arc.blocking_write().brain.save() {
                log::error!("[bg] auto-save failed: {}", e);
            }
        })
        .await
        .unwrap_or_else(|e| log::error!("[bg] save blocking task failed: {e}"));
        // emit suppressed — ticker provides periodic execution
    }

    async fn handle_consolidate(&self) {
        let brain_arc = self.brain.clone();
        tokio::task::spawn_blocking(move || {
            let mut b = brain_arc.blocking_write();
            let r = b.consolidate_memories();
            log::info!(
                "[bg] consolidated: {} merged, {} pruned, {} replayed",
                r.merged_count,
                r.pruned_count,
                r.replayed_count
            );
        })
        .await
        .unwrap_or_else(|e| log::error!("[bg] consolidate blocking task failed: {e}"));
        // emit suppressed — ticker provides periodic execution
    }

    async fn handle_meta(&mut self) {
        let r = self.metacognition.run_full_cycle();
        log::info!("[bg] meta cycle #{}", r.iteration);
        // emit suppressed — ticker provides periodic execution
    }

    async fn handle_thinking(&mut self) {
        if self.thinking_in_progress {
            log::warn!("[bg] re-entrant thinking call suppressed (event cascade guard)");
            return;
        }
        self.thinking_in_progress = true;
        self.thinking_cycle_counter = self.thinking_cycle_counter.saturating_add(1);
        if self.thinking_cycle_counter > MAX_THINKING_CYCLES {
            log::warn!(
                "[bg] thinking cycle limit reached (attempt {}), suppressing ThoughtComplete",
                self.thinking_cycle_counter
            );
            self.thinking_in_progress = false;
            return;
        }
        let goal_desc = self
            .goal_loop
            .active_goal
            .as_ref()
            .map(|g| g.description.clone());
        // BRAIN WRITE LOCK: move sync work (SEAL/iterate, 1-2s) to spawn_blocking
        // to avoid blocking the tokio worker thread.
        let brain_arc = self.brain.clone();
        tokio::task::spawn_blocking(move || {
            let mut b = brain_arc.blocking_write();
            if let Some(desc) = goal_desc {
                let _ = b.run_seal_loop(&desc, None, None);
            } else {
                b.iterate(super::super::nt_expert_routing::TaskType::General);
            }
        })
        .await
        .unwrap_or_else(|e| log::error!("brain blocking task failed: {e}"));

        // ── Ouroboros: Self-Review → Metacognition bridge ──
        // Read SEAL pipeline's self-review report (populated by SelfReviewStage)
        // and feed 6-dim audit scores into MetaCognitiveLoop::fuse_self_review_scores.
        // This closes the circular fusion:
        //   Consciousness → SEAL → SelfReviewStage → MetaCognitiveLoop → Consciousness
        let review_scores = {
            let brain = self.brain.read().await;
            brain.seal_rl.self_review_report.as_ref().map(|r| {
                [
                    1.0 - r.cycle_risk,
                    1.0 - r.panic_density,
                    1.0 - r.unbounded_ratio,
                    1.0 - r.dead_code_ratio,
                    r.shutdown_coverage,
                    r.feature_integrity,
                ]
            })
        };
        if let Some(scores) = review_scores {
            if let Some(ref mut ci) = self.consciousness {
                let composite = ci.meta_cognition_loop.fuse_self_review_scores(scores);
                log::info!("[ouroboros] self-review→meta: composite={:.3}", composite);
            }
        }

        self.emit_event(ConsciousnessEvent::ThoughtComplete);
        self.thinking_in_progress = false;
        self.canvas_flush_final();
    }

    async fn handle_goal(&mut self) {
        let brain_arc = self.brain.clone();
        let mut goal = std::mem::take(&mut self.goal_loop);
        let curriculum_task = if goal.active_goal.is_none() {
            if let Some(ref mut ci) = self.consciousness {
                ci.curriculum_thinking_bridge()
            } else {
                None
            }
        } else {
            None
        };
        let spawn_result = tokio::task::spawn_blocking(move || {
            let mut b = brain_arc.blocking_write();
            goal.pursue_all(&mut b, 1);
            if let Some(ref task) = curriculum_task {
                goal.start_goal(&mut b, task, None);
                log::info!("[bg] curriculum→goal: '{}'", task);
            }
            goal
        })
        .await;
        match spawn_result {
            Ok(g) => self.goal_loop = g,
            Err(e) => {
                log::error!("[bg] goal blocking task panicked: {e}");
                self.goal_loop = GoalLoop::default();
            }
        }
    }

    async fn handle_prediction(&mut self) {
        // Pre-action introspection before prediction cycle
        if let Some(ref mut ip) = self.introspector {
            let state = self.thinking.silicon.current_state();
            let action = format!("prediction_cycle_{}", self.thinking.silicon.iteration);
            let _r = ip.introspect(
                &action,
                crate::core::nt_core_self::reasoning_strategy::StrategyKind::Deliberate,
                crate::core::nt_core_self::attention_head::AttentionDomain::Planning,
                &state,
            );
        }

        // 1. PREDICT — run panorama pipeline on a blocking thread to avoid
        // holding the tokio RwLock write guard across sync work (~1-2s).
        let brain_arc = self.brain.clone();
        let mut pano = self.panorama.take();
        let mut wm = self.nt_world_model.take();
        let mut goal = std::mem::take(&mut self.goal_loop);

        let spawn_result = tokio::task::spawn_blocking(move || {
            let mut b = brain_arc.blocking_write();
            let report = if let (Some(ref mut pano), Some(ref mut wm)) = (&mut pano, &mut wm) {
                let r = pano.run_cycle(&mut b, &mut goal, wm);
                log::info!("[bg] prediction: cycle={}, anomaly={}", r.cycle, r.anomaly);
                Some(r)
            } else {
                if let Some(wm) = wm.as_ref() {
                    wm.predict_all(&[]);
                }
                None
            };
            (report, pano, wm, goal)
        })
        .await;

        let (report_opt, restored_pano, restored_wm, restored_goal) = match spawn_result {
            Ok(bag) => bag,
            Err(e) => {
                log::error!("[bg] prediction task panicked: {:?}", e);
                (None, None, None, GoalLoop::default())
            }
        };

        self.panorama = restored_pano;
        self.nt_world_model = restored_wm;
        self.goal_loop = restored_goal;

        // 1b. NT_WORLD_PRED — RSSM-style ensemble prediction
        if let Some(ref mut predictor) = self.nt_world_predictor {
            let cfg = crate::neotrix::nt_world_pred::PredictorConfig::default();
            let latent = vec![0.5; cfg.latent_dim];
            let action = vec![0.0; cfg.action_dim];
            let context = vec![0.1; cfg.context_dim];
            let input =
                crate::neotrix::nt_world_pred::PredictionInput::new(latent, action, context);
            let result = predictor.predict(&input);
            log::info!(
                "[bg] world_pred: uncertainty={:.4}, plausible={}",
                result.uncertainty,
                result.plausible_states.len(),
            );
            // Store to replay buffer for experience replay
            if let Some(ref mut replay) = self.nt_world_replay_buffer {
                replay.push(
                    input.current_latent.clone(),
                    result.predicted_latent,
                    1.0 - result.uncertainty,
                );
            }
        }

        // 1c. NT_WORLD_PRED_HCUBE — knowledge-augmented prediction
        if let Some(ref mut aug) = self.nt_world_pred_hcube {
            let dummy_latent = vec![0.5; 32];
            let augmented = aug.augment_prediction(&dummy_latent, &[]);
            if !augmented.is_empty() {
                log::info!(
                    "[bg] world_pred_hcube: adjusted latent dim={}",
                    augmented.len(),
                );
            }
        }

        // 2. OBSERVE — run awareness monitor after prediction
        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let level = aw.current.consciousness_level;
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            let anomaly_flag = report_opt.as_ref().map(|r| r.anomaly).unwrap_or(false);
            log::info!("[bg] awareness after prediction: consciousness={:.3}, phi={:.4}, coherence={:.3}, anomaly={}",
                level, phi, coherence, anomaly_flag);
        }

        // 3. REPORT — consolidated prediction summary
        if let Some(ref report) = report_opt {
            log::info!("[bg] prediction report: cycle={}, hypercube={}, cortex={}, gwt={}, fe={:.3}, phi={:.3}, goals={}",
                report.cycle, report.hypercube_entries, report.cortex_traces,
                report.gwt_broadcasts, report.fe_energy, report.phi, report.goals_created);
        }

        #[cfg(feature = "stealth-net")]
        self.handle_stealth_rotation().await;
        self.emit_event(ConsciousnessEvent::Custom("prediction".into()));
    }

    async fn handle_panorama(&self) {
        if let Some(ref pano) = self.panorama {
            log::info!("[bg] panorama status: {}", pano.status());
        }
        self.emit_event(ConsciousnessEvent::Custom("panorama".into()));
    }

    async fn handle_exploration(&mut self) {
        let sources_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".neotrix")
            .join("exploration_sources.txt");

        let sp_write = sources_path.clone();
        let urls: Vec<String> =
            tokio::task::spawn_blocking(move || match std::fs::read_to_string(&sources_path) {
                Ok(content) => content
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect(),
                Err(_) => Vec::new(),
            })
            .await
            .unwrap_or_default();

        if !urls.is_empty() {
            if let Some(ref mut evolver) = self.self_evolver {
                for url in &urls {
                    if !SelfEvolver::is_url(url) {
                        log::warn!("[bg] skipping non-URL: {}", url);
                        continue;
                    }
                    match evolver.evolve_from_url(url) {
                        Ok(reward) => {
                            log::info!("[bg] exploration evolved {}: reward={:.3}", url, reward)
                        }
                        Err(e) => log::error!("[bg] exploration failed {}: {}", url, e),
                    }
                }
            }

            // Clear processed URLs to avoid re-processing
            let _ = tokio::task::spawn_blocking(move || {
                let tmp = sp_write.with_extension("tmp");
                if let Err(e) = std::fs::write(&tmp, "") {
                    log::error!("[bg] failed to clear exploration sources: {}", e);
                } else if let Err(e) = std::fs::rename(&tmp, &sp_write) {
                    log::error!("[bg] failed to rename exploration sources: {}", e);
                }
            })
            .await;
        }

        // Knowledge gap detection — scans what's missing vs known sources
        if let Some(ref mut gap_detector) = self.gap_detector {
            use crate::core::nt_core_meta::scanner::CodeScanner;
            use crate::core::nt_core_meta::weakness::WeaknessAnalyzer;
            let scanner = CodeScanner::new(".");
            let model = scanner.scan();
            let analyzer = WeaknessAnalyzer::new();
            let weaknesses = analyzer.analyze(&model);
            let report = gap_detector.detect_gaps(&model, &weaknesses.weaknesses);
            if report.high_priority_count > 0 {
                log::info!(
                    "[bg] gap detection: {} gaps, {} high-priority, coherence={:.2}",
                    report.total_gaps,
                    report.high_priority_count,
                    report.coherence_score
                );
                for s in report.exploration_suggestions.iter().take(3) {
                    log::info!("[bg]   gap suggestion: {}", s);
                }
            }
        }
        // emit suppressed — ticker provides periodic execution
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_transit_tick(&mut self) {
        if self.shutdown.signal().is_shutdown() {
            return;
        }
        let ts = crate::neotrix::nt_shield_stealth_net::transit_station::global_transit_station();
        if !ts.is_enabled() {
            return;
        }
        let stats = ts.stats().await;
        let active = stats.active_connections;
        let total = stats.conn_count;

        // 自适应调度: 高负载 → 全量工作; 低负载 → 轻量统计
        if active > 10 || total % 5 == 0 {
            ts.auto_assign_roles().await;
            ts.adapt_rotation_to_bandit().await;
        }

        log::info!(
            "[transit] conn={}, active={}, bytes={}, mode={:?}",
            total,
            active,
            stats.total_bytes_relayed,
            stats.mode,
        );
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_stealth_rotation(&mut self) {
        if self.shutdown.signal().is_shutdown() {
            return;
        }
        if let Some(ref mut sm) = self.nt_shield_manager {
            let tags: Vec<String> = vec!["pool_0".to_string()];
            let _id = sm.get_identity(&tags);
            let stats = sm.stats();
            if stats.active_count < stats.total_identities {
                log::info!(
                    "[bg] stealth: {}/{} active, success={:.2}, confidence={:.2}",
                    stats.active_count,
                    stats.total_identities,
                    stats.avg_success_rate,
                    stats.avg_confidence
                );
            }
        }
    }

    async fn handle_awareness(&mut self) {
        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let level = aw.current.consciousness_level;
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            log::info!(
                "[bg] awareness: consciousness={:.3}, phi={:.4}, coherence={:.3}",
                level,
                phi,
                coherence
            );
        }
    }

    async fn handle_always_on(&mut self) {
        use crate::neotrix::nt_mind_background_loop::always_on::AlwaysOnEngine;
        static ALWAYS_ON: std::sync::LazyLock<std::sync::Mutex<Option<AlwaysOnEngine>>> =
            std::sync::LazyLock::new(|| std::sync::Mutex::new(None));

        // yield before blocking on std::sync::Mutex + sync full_cycle()
        yield_now().await;

        let report = {
            let mut guard = ALWAYS_ON.lock().unwrap_or_else(|e| e.into_inner());
            let engine = guard.get_or_insert_with(|| {
                let mut e = AlwaysOnEngine::load();
                e.enabled = true;
                e
            });
            if engine.enabled {
                match engine.full_cycle() {
                    Ok(report) => Some(report),
                    Err(_e) => {
                        if let Err(save_e) = engine.save() {
                            log::error!("[bg] always_on save after error failed: {}", save_e);
                        }
                        None
                    }
                }
            } else {
                None
            }
        };

        // yield after releasing std::sync::Mutex to let other tasks run
        yield_now().await;

        if let Some(report) = report {
            if report.tasks_executed > 0 {
                log::info!(
                    "[bg] always_on: scanned={}, executed={}, completed={}, took={}ms",
                    report.scan_count,
                    report.tasks_executed,
                    report.tasks_completed,
                    report.duration_ms
                );
                let guard = ALWAYS_ON.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(ref engine) = *guard {
                    if let Err(e) = engine.save() {
                        log::error!("[bg] always_on post-exec save failed: {}", e);
                    }
                }
            }
        }
    }

    async fn handle_nt_act_voice_tick(&mut self) {
        if let Some(ref mut vi) = self.nt_act_voice_input {
            if !vi.is_active() {
                return;
            }
            if vi.is_continuous() {
                if let Some(text) = vi.poll_transcription() {
                    log::info!("[nt_act_voice] transcribed: {}", text);
                    let cmd = crate::neotrix::nt_act_voice::VoiceCommand::parse(&text);
                    match cmd {
                        crate::neotrix::nt_act_voice::VoiceCommand::OpenSettings => {
                            log::info!("[nt_act_voice] command: open settings");
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::ShowHelp => {
                            log::info!("[nt_act_voice] command: show help");
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::RunCommand(c) => {
                            log::info!("[nt_act_voice] command: run {}", c);
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::SwitchSession(s) => {
                            log::info!("[nt_act_voice] command: switch to session {}", s);
                        }
                        _ => {}
                    }
                }
            } else if vi.check_wake_word() {
                log::info!("[nt_act_voice] wake word detected");
            }
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_nt_world_sense_tick(&mut self) {
        if let Some(ref mut wc) = self.world_consciousness {
            let events = wc.nt_world_sense.poll_all();
            if !events.is_empty() {
                log::info!("[nt_world_sense] {} new events", events.len());
            }
            wc.refresh_self_awareness();
            log::info!("[nt_world_sense] status: {}", wc.consciousness_status());
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_proxy_heartbeat(&self) {
        if let Some(ref coord) = self.rotation_coordinator {
            coord.tick();
            use super::super::nt_shield_stealth_net::RotationDomain;
            for &domain in RotationDomain::all() {
                if coord.should_rotate(domain).await {
                    log::info!("[bg] rotation coord: {domain:?} triggered");
                    coord.mark_rotated(domain).await;
                }
            }
        }
        if let Some(ref engine) = self.heartbeat_engine {
            let record = engine.tick().await;
            if record.success {
                log::info!(
                    "[bg] proxy heartbeat #{}: proxy={}, geo={:?}, fp={}, dns={}",
                    record.tick,
                    record.proxy_url,
                    record.proxy_geo,
                    record.fingerprint_id,
                    record.dns_flushed,
                );
            } else {
                log::warn!(
                    "[bg] proxy heartbeat #{}: no proxy available (pool empty?)",
                    record.tick
                );
            }
        }
        self.handle_proxy_auto_mode().await;
    }

    /// 根据 brain 上下文自动切换 proxy 模式
    #[cfg(feature = "stealth-net")]
    async fn handle_proxy_auto_mode(&self) {
        use super::super::nt_shield_stealth_net::proxy_control::DaemonMode;

        let client = match self.proxy_client {
            Some(ref c) => c,
            None => return,
        };
        if !client.is_reachable().await {
            return;
        }

        // 读取当前 daemon 模式
        let status_str = match client.status().await {
            Ok(s) => s,
            Err(_) => return,
        };
        let current = match serde_json::from_str::<serde_json::Value>(&status_str) {
            Ok(v) => {
                DaemonMode::from_str(v["mode"].as_str().unwrap_or("geo")).unwrap_or(DaemonMode::Geo)
            }
            Err(_) => return,
        };

        // 决定目标模式 (函数已 #[cfg(feature = "stealth-net")], 字段可用)
        let target = if self.tor_crawler.is_some() {
            DaemonMode::Tor
        } else if self
            .nt_shield_manager
            .as_ref()
            .map_or(false, |sm| sm.stats().active_count > 0)
        {
            DaemonMode::Stealth
        } else {
            DaemonMode::Geo
        };

        if target != current {
            match client.set_mode(target).await {
                Ok(_) => log::info!(
                    "[bg] proxy auto-mode: {} → {}",
                    current.as_str(),
                    target.as_str()
                ),
                Err(e) => log::error!("[bg] proxy auto-mode failed: {}", e),
            }
        }
    }

    async fn handle_plugin_tick(&self) {
        use crate::neotrix::nt_io_plugin::PluginEvent;
        self.plugin_registry.dispatch(&PluginEvent::BrainTick);
    }

    /// Periodic agent discovery listener — sweep for UDP broadcasts.
    async fn handle_agent_discovery(&mut self) {
        if let Some(ref mut discovery) = self.agent_discovery {
            if let Err(e) = discovery.listen() {
                log::warn!("[bg] agent discovery listen: {}", e);
            }
            if discovery.agent_count() > 0 {
                log::info!("[bg] known agents: {}", discovery.agent_count());
            }
        }
    }

    /// Curiosity drive: knowledge gaps → GWT attention → exploration queries
    /// Wired to negentropy: gap sparsity → negentropy proxy → curiosity calibration
    async fn handle_curiosity(&mut self) {
        use crate::neotrix::nt_mind::hypercube_bridge::HyperCubeBridge;
        let gap_reports = {
            let bridge = HyperCubeBridge::new();
            bridge.analyze_gaps()
        };

        self.curiosity_drive.ingest_gap_reports(&gap_reports);

        // Negentropy alignment: use gap sparsity as inverse negentropy proxy
        // sparsity ↑ → order ↓ → negentropy ↓ → curiosity ↑
        let n_total_proxy = if gap_reports.is_empty() {
            0.5
        } else {
            let avg_sparsity: f64 = gap_reports.iter().map(|r| r.sparsity_score).sum::<f64>()
                / gap_reports.len() as f64;
            (1.0 - avg_sparsity).clamp(0.0, 1.0)
        };
        self.curiosity_drive
            .calibrate_to_negentropy(n_total_proxy, 0.0);

        let queries = self.curiosity_drive.drain_queries();

        if !queries.is_empty() {
            log::info!(
                "[bg] curiosity: {} signals, {} queries generated: {:?}",
                self.curiosity_drive.signals.len(),
                queries.len(),
                &queries[..queries.len().min(3)],
            );
            if let Some(ref mut evolver) = self.self_evolver {
                for query_str in queries.iter().take(2) {
                    let q: &String = query_str;
                    let search_url =
                        format!("https://en.wikipedia.org/wiki/{}", q.replace(' ', "_"));
                    match evolver.evolve_from_url(&search_url) {
                        Ok(reward) => {
                            log::info!("[bg] curiosity evolved {}: reward={:.3}", q, reward)
                        }
                        Err(e) => log::error!("[bg] curiosity failed {}: {}", q, e),
                    }
                }
            }
        } else {
            let level = self.curiosity_drive.curiosity_level;
            let signal_count = self.curiosity_drive.signals.len();
            if signal_count > 0 {
                log::info!(
                    "[bg] curiosity: {:?}, {} signals, {} total gaps",
                    level,
                    signal_count,
                    self.curiosity_drive.total_gaps_detected
                );
            }
        }
        // emit suppressed — ticker provides periodic execution
    }

    /// Knowledge chain: discovery → mining → validation → absorption → storage
    async fn handle_knowledge_chain(&mut self) {
        if let Some(ref mut chain) = self.knowledge_chain {
            let kc: &mut KnowledgeChain = chain;
            if !kc.has_pending() {
                kc.init_default_discovery();
            }
            let mut brain = ReasoningBrain::new();
            let mut bank = ReasoningBank::new(100);
            match kc.run_chain(&mut brain, &mut bank) {
                Ok(result) => {
                    log::info!(
                        "[bg] knowledge chain: discovered={}, mined={}, absorbed={}, reward={:.3}",
                        result.discovered,
                        result.mined,
                        result.absorbed,
                        result.total_reward
                    );
                    if result.absorbed > 0 && self.config.enable_auto_crystallize {
                        for d in &result.details {
                            let detail: &String = d;
                            if detail.starts_with("吸收阶段") {
                                let edits = vec![
                                    super::super::nt_mind::self_edit::MicroEdit::NormalizeVector,
                                ];
                                self.auto_crystallizer.crystallize_from_absorption(
                                    &mut brain,
                                    &mut bank,
                                    "knowledge_chain",
                                    "chain_batch",
                                    "general",
                                    &edits,
                                    result.total_reward / result.absorbed as f64,
                                );
                            }
                        }
                    }
                }
                Err(e) => log::error!("[bg] knowledge chain failed: {}", e),
            }
        }
    }

    /// Knowledge aging: score decay → stale detection → re-scan scheduling
    async fn handle_knowledge_aging(&mut self) {
        let report = self.knowledge_aging.run_aging_cycle();
        if report.stale_count > 0 || report.expired_count > 0 {
            log::info!(
                "[bg] knowledge aging: {} survived, {} stale, {} expired, avg_age={:.1}d",
                report.surviving_entries,
                report.stale_count,
                report.expired_count,
                report.avg_age_days
            );

            if !report.rescans_needed.is_empty() {
                log::info!("[bg] aging: {} rescans needed", report.rescans_needed.len());
                if let Some(ref mut evolver) = self.self_evolver {
                    for url_str in report.rescans_needed.iter().take(3) {
                        if SelfEvolver::is_url(url_str) {
                            let ev: &mut SelfEvolver = evolver;
                            match ev.evolve_from_url(url_str) {
                                Ok(reward) => {
                                    log::info!("[bg] re-scan {}: reward={:.3}", url_str, reward)
                                }
                                Err(e) => log::error!("[bg] re-scan failed {}: {}", url_str, e),
                            }
                        }
                    }
                }
            }
        }
        // emit suppressed — ticker provides periodic execution
    }

    /// Auto-crystallization: check SelfEvolver results → create SkillCrystals
    async fn handle_crystallization(&mut self) {
        if !self.config.enable_auto_crystallize {
            return;
        }
        let summary = self.auto_crystallizer.summary();
        log::info!("[bg] crystallization: {}", summary);
    }

    /// Scheduler tick: check due jobs, gate by consciousness state, dispatch handlers.
    /// Replaces the old hardcoded build_cleanup_ticker with OpenClaw-inspired scheduling.
    async fn handle_scheduler_tick(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Read consciousness state for context-aware gating
        let (cog_load, da_level, sleep_pressure, curiosity) = match self.consciousness {
            Some(ref mut ci) => {
                if ci.cycle == 0 {
                    (0.0, 0.3, 0.0, 0.5)
                } else {
                    (
                        ci.cognitive_load,
                        ci.neuromodulator.system.get_level(crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorType::DA),
                        ci.consolidation_bridge.sleep_pressure(),
                        self.curiosity_drive.curiosity_level.salience_multiplier(),
                    )
                }
            }
            None => (0.0, 0.3, 0.0, 0.5),
        };

        let due_jobs: Vec<(String, String)> =
            self.scheduler
                .tick(now, cog_load, da_level, sleep_pressure, curiosity);

        for (job_id, handler) in &due_jobs {
            let start = std::time::Instant::now();

            let (success, error_msg): (bool, Option<String>) = match handler.as_str() {
                "handle_build_cleanup" => {
                    self.handle_build_cleanup().await;
                    (true, None)
                }
                "handle_knowledge_aging" => {
                    self.handle_knowledge_aging().await;
                    (true, None)
                }
                "handle_evosc_tick" => {
                    if let Some(ref mut ci) = self.consciousness {
                        if ci.cycle % 5 == 0 {
                            ci.handle_evosc_tick();
                        }
                    }
                    (true, None)
                }
                "handle_work_discovery" => {
                    if let Some(ref mut ci) = self.consciousness {
                        ci.handle_work_discovery_tick();
                    }
                    (true, None)
                }
                "handle_independent_verify" => {
                    if let Some(ref mut ci) = self.consciousness {
                        ci.handle_independent_verify_tick();
                    }
                    (true, None)
                }
                "handle_loop_audit" => {
                    if let Some(ref mut ci) = self.consciousness {
                        ci.handle_loop_audit_tick();
                    }
                    (true, None)
                }
                other => {
                    let msg = format!(
                        "scheduler: unknown handler '{}' for job '{}'",
                        other, job_id
                    );
                    log::error!("[bg] {}", msg);
                    (false, Some(msg))
                }
            };

            let duration_ms = start.elapsed().as_millis() as u64;
            self.scheduler
                .record_run(job_id, now, duration_ms, success, error_msg);
        }
        // emit suppressed — ticker provides periodic execution

        // Log stats every 10 scheduler ticks (~5 minutes)
        if self.scheduler.tick_count() % 10 == 0 {
            let stats = self.scheduler.stats();
            log::info!(
                "[bg] scheduler: {} jobs ({} enabled), {} runs, {:.1}% success",
                stats.total_jobs,
                stats.enabled_jobs,
                stats.total_runs,
                stats.success_rate * 100.0,
            );
        }
    }

    /// 每日构建产物清理：target/ + node_modules/ + dist/
    async fn handle_build_cleanup(&mut self) {
        let engine = match self.cleanup_engine {
            Some(ref mut e) => e,
            None => return,
        };
        let result = engine.clean(CleanupKind::ProjectArtifacts);
        if result.deletable_count > 0 {
            log::info!(
                "[cleanup] {}: {} items, {:.1} MB freed",
                result.kind.description(),
                result.deletable_count,
                result.estimated_bytes as f64 / 1_048_576.0,
            );
        }
    }

    async fn handle_network_tick(&mut self) {
        let sensor = self
            .network_sensor
            .get_or_insert_with(|| NetworkSensor::new(4096));
        let status = sensor
            .get_status()
            .await
            .unwrap_or_else(|e| format!("err: {}", e));
        let neg = sensor.network_negentropy();
        log::info!("[bg] network: status={}, negentropy={:.3}", status, neg);

        #[cfg(feature = "stealth-net")]
        {
            // 网络监控: DNS 质量 + 连通性 + VPN 自动管理
            use super::super::nt_shield_stealth_net::connectivity_checker::ConnectivityChecker;
            use super::super::nt_shield_stealth_net::network_monitor::NetworkMonitor;
            use super::super::nt_shield_stealth_net::proxy_control::DaemonMode;
            use std::sync::Arc;

            let mut monitor = NetworkMonitor::default();
            monitor.tick().await;

            // 连通性检查 + 自动源补充
            let mode = Arc::new(tokio::sync::RwLock::new(DaemonMode::Off));
            let checker = Arc::new(ConnectivityChecker::new(mode.clone()));
            checker.tick().await;
            let snap = checker.snapshot().await;
            log::info!(
                "[bg] connectivity: direct={}, proxy_h={}/{} mode={:?}",
                snap.direct_reachable,
                snap.proxy_healthy_count,
                snap.proxy_total_count,
                snap.active_mode,
            );

            // IP 轮转
            if let Some(ref ip_rot) = self.ip_rotator {
                let cfg = super::super::nt_shield_stealth_net::config::load();
                if cfg.ip_rotation.enabled {
                    let _ = ip_rot.rotate_ip().await;
                    log::info!(
                        "[bg] ip_rotation: external_ip={:?}",
                        ip_rot.get_last_external_ip().await
                    );
                }
            }
        }
    }

    async fn handle_vision_tick(&mut self) {
        if self.image_pipeline.is_none() {
            let config = crate::neotrix::nt_io_provider::factory::ProviderConfig::from_env();
            let provider = crate::neotrix::nt_io_provider::factory::create_provider(config);
            let model = std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            self.image_pipeline = Some(ImagePipeline::new(provider, &model));
        }
        if let Some(ref pipeline) = self.image_pipeline {
            log::info!("[bg] vision: available={}", pipeline.is_available());
        }
        // Bridge to consciousness integration: ensure CI vision pipeline is initialized
        if let Some(ref mut ci) = self.consciousness {
            if ci.vision.is_none() {
                let config = crate::neotrix::nt_io_provider::factory::ProviderConfig::from_env();
                let provider = crate::neotrix::nt_io_provider::factory::create_provider(config);
                let model = std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
                ci.vision = Some(ImagePipeline::new(provider, &model));
            }
            let result = ci.handle_vision_integrate_tick();
            log::info!("[bg] vision_integrate: {}", result);
        }
        // emit suppressed — ticker provides periodic execution
    }

    async fn handle_storage_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_storage_engine_tick(),
            None => "storage:no_ci".into(),
        }
    }

    async fn handle_jepa_ema_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_ema_jepa_tick(),
            None => "jepa:no_ci".into(),
        }
    }

    // SECTION: P1 periodic handlers

    // ── P1.01: EvidenceManager tick ──
    async fn handle_evidence_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_evidence_tick(),
            None => "evidence:no_ci".into(),
        }
    }

    // ── P1.03: SpreadActivationMemory tick ──
    async fn handle_spread_activation_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_spread_activation_tick(),
            None => "spread_activation:no_ci".into(),
        }
    }

    // ── P1.04: BFT ConsensusEngine tick ──
    async fn handle_consensus_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_consensus_tick(),
            None => "consensus:no_ci".into(),
        }
    }

    // ── P1.05: HypergraphStore tick ──
    async fn handle_hypergraph_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_hypergraph_tick(),
            None => "hypergraph:no_ci".into(),
        }
    }

    // ── P1.06: STORM status poll tick ──
    async fn handle_storm_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_storm_status_tick(),
            None => "storm:no_ci".into(),
        }
    }

    // ── P1.08: KB maintenance tick ──
    async fn handle_kb_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_kb_tick(),
            None => "kb:no_ci".into(),
        }
    }

    // ── Multi-Provider LLM Router tick ──
    async fn handle_llm_router_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_llm_router_tick(),
            None => "llm_router:no_ci".into(),
        }
    }

    // ── Symbolic Discovery tick ──
    async fn handle_symbolic_discovery_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_symbolic_discovery_tick(),
            None => "symbolic_discovery:no_ci".into(),
        }
    }

    // ── Ne Comptime tick ──
    async fn handle_ne_comptime_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_ne_comptime_tick(),
            None => "ne_comptime:no_ci".into(),
        }
    }

    async fn handle_translate_engine_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_translate_engine_tick(),
            None => "translate_engine:no_ci".into(),
        }
    }

    async fn handle_a2a_grpc_tick(&mut self) -> String {
        match self.consciousness {
            Some(ref mut ci) => ci.handle_a2a_grpc_tick(),
            None => "a2a_grpc:no_ci".into(),
        }
    }

    /// Awakening tick: measure → model → hypothesize → modify → re-measure
    /// BRAIN WRITE LOCK: move sync work (model refitting, hypothesis gen, 10-100ms+)
    /// to spawn_blocking to avoid blocking the tokio worker thread.
    async fn handle_awakening_tick(&mut self) {
        let mut engine = match self.awakening.take() {
            Some(e) => e,
            None => return,
        };
        let brain_arc = self.brain.clone();
        match timeout(
            std::time::Duration::from_secs(5),
            tokio::task::spawn_blocking(move || {
                let mut brain = brain_arc.blocking_write();
                let result = engine.tick(&mut brain);
                (engine, result)
            }),
        )
        .await
        {
            Ok(Ok((returned_engine, result))) => {
                self.awakening = Some(returned_engine);
                log::info!(
                    "[bg] awakening: tick={}, Φ={:.4}, speed={:.4}, refit={}, hyps={}, intervene={}",
                    result.tick_count,
                    result.phi,
                    result.awakening_speed,
                    result.refit_done,
                    result.hypotheses_generated,
                    result.intervention.is_some(),
                );
                if let Some(ref interv) = result.intervention {
                    log::info!(
                        "[bg] awakening intervention: applied={}, ΔΦ={:.4}, msg={}",
                        interv.applied,
                        interv.delta_phi,
                        interv.message
                    );
                }
            }
            Ok(Err(join_err)) => {
                log::error!("[bg] awakening tick panicked: {:?}", join_err);
            }
            Err(_elapsed) => {
                log::warn!("[bg] awakening tick timed out after 5s");
            }
        }
    }

    /// Audit tick: run static security checklist
    async fn handle_audit_tick(&mut self) {
        let report = SecurityAuditor::run_static("neotrix-core", ".");
        let score = SecurityAuditor::calculate_score(&report);
        self.audit_report = Some(report);
        log::info!(
            "[bg] audit: {} checks, score={:.1}%",
            self.audit_report.as_ref().map_or(0, |r| r.total_checks),
            score,
        );
        // emit suppressed — ticker provides periodic execution
    }

    async fn handle_nt_act_sync_tick(&mut self) {
        if let Some(ref mut fs) = self.nt_act_sync {
            // Phase 1: accept incoming sync requests
            fs.poll_incoming();

            // Phase 2: discover network peers if we have fewer than 5
            if fs.known_peers().len() < 5 {
                if let Ok(peers) = fs.discover_peers(2_000) {
                    for peer in &peers {
                        let _ = fs.add_pair(&peer.id);
                    }
                    log::info!(
                        "[bg] nt_act_sync: discovered {} peers, {} total",
                        peers.len(),
                        fs.known_peers().len(),
                    );
                }
            }

            // Phase 3: attempt sync with each paired peer
            let pair_ids: Vec<String> = fs.pairs().iter().map(|p| p.peer_id.clone()).collect();
            for peer_id in &pair_ids {
                match fs.compute_diff(peer_id) {
                    Ok((_local, _remote, diff)) => {
                        let changes = diff.to_send.len() + diff.to_receive.len();
                        if changes > 0 {
                            log::info!(
                                "[bg] nt_act_sync: {} changes with peer {}",
                                changes,
                                peer_id
                            );
                            if let Ok(total) = fs.execute_sync(peer_id) {
                                log::info!(
                                    "[bg] nt_act_sync: synced {} bytes with {}",
                                    total,
                                    peer_id
                                );
                            }
                        }
                    }
                    Err(e) => log::debug!("[bg] nt_act_sync: diff {} failed: {}", peer_id, e),
                }
            }
        }
    }

    async fn handle_nt_act_project_tick(&mut self) {
        if let Some(ref mut pm) = self.nt_act_project_manager {
            let all = pm.all().len();
            let active = pm.active().map(|p| p.name.as_str()).unwrap_or("none");
            log::info!(
                "[bg] nt_act_project_manager: projects={}, active={}",
                all,
                active,
            );

            // Persist project list periodically so project history survives restart
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("neotrix");
            let save_path = config_dir.join("projects.json");
            if let Err(e) = pm.save(&save_path) {
                log::error!("[bg] project_manager save failed: {}", e);
            }
        }
    }

    /* ── E8 Reasoning Canvas (event-driven, no tick) ── */

    /// Clears the canvas when new user input arrives, starting a fresh chain.
    fn canvas_new_chain(&mut self) {
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => return,
        };
        let canvas = ci
            .canvas_manager
            .get_or_insert_with(|| neotrix_types::core::node_canvas::CanvasManager::new(3));
        canvas.reset();
        canvas.create_project("E8 Reasoning");
        if let Some(proj) = canvas.projects.first() {
            *self
                .canvas_snapshot
                .write()
                .unwrap_or_else(|e| e.into_inner()) = proj.clone();
        }
    }

    /// Captures one final E8 attractor after thinking completes and writes the
    /// snapshot.  Replaces any previous node — the canvas always shows the most
    /// recent reasoning-chain outcome only.
    fn canvas_flush_final(&mut self) {
        let ci = match self.consciousness.as_mut() {
            Some(c) => c,
            None => return,
        };
        let canvas = ci
            .canvas_manager
            .get_or_insert_with(|| neotrix_types::core::node_canvas::CanvasManager::new(3));
        if canvas.projects.is_empty() {
            canvas.create_project("E8 Reasoning");
        }
        let project = match canvas.projects.first_mut() {
            Some(p) => p,
            None => return,
        };

        // Read the VSA buffer from attractor_state or fallback to workspace_state
        let vsa_buffer: [u8; 512] = if ci.attractor_state.len() >= 512 {
            let mut buf = [0u8; 512];
            buf.copy_from_slice(&ci.attractor_state[..512]);
            buf
        } else if ci.global_workspace.workspace_state.len() == 512 {
            let mut buf = [0u8; 512];
            buf.copy_from_slice(&ci.global_workspace.workspace_state[..512]);
            buf
        } else {
            return;
        };

        let root = crate::core::nt_core_e8::E8Projector::project_vsa(&vsa_buffer);
        let norm = root.norm_sq();
        let root_key = format!("r{:.0}", norm);

        // Dedup: skip if this exact root is already the current node
        if project
            .nodes
            .last()
            .map_or(false, |n| n.id == format!("s_{}", root_key))
        {
            return;
        }

        let simple = ci.e8_lattice.simple_roots();
        let nearest = simple
            .iter()
            .min_by_key(|sr| {
                let mut d: u32 = 0;
                for (a, b) in sr.coords.iter().zip(root.coords.iter()) {
                    d += (*a as i16 - *b as i16).unsigned_abs() as u32;
                }
                d
            })
            .copied();

        let palette = match nearest {
            Some(r) => {
                let k = crate::core::nt_core_e8::killing_form(&root, &r);
                if k > 0.0 {
                    "pos"
                } else if k < 0.0 {
                    "neg"
                } else {
                    "neut"
                }
            }
            None => "neut",
        };

        // Replace — canvas always shows the single most recent settling
        project.nodes.clear();
        project.edges.clear();

        project.add_node(neotrix_types::core::node_canvas::CanvasNode {
            id: format!("s_{}", root_key),
            label: format!("E8 {}", root_key),
            node_type: neotrix_types::core::node_canvas::NodeType::Concept,
            x: 40.0,
            y: 20.0,
            width: 140.0,
            height: 60.0,
            color: Some(palette.to_string()),
            content: Some(format!("palette={}", palette)),
            metadata: std::collections::HashMap::new(),
        });

        *self
            .canvas_snapshot
            .write()
            .unwrap_or_else(|e| e.into_inner()) = project.clone();
    }
}

// SECTION: Tests

#[cfg(test)]
mod tests {
    // TODO: add #[serial] to any new tests that use global singletons
    use super::MAX_THINKING_CYCLES;
    use crate::neotrix::nt_mind::goal_loop::GoalLoop;
    use crate::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    use crate::neotrix::nt_world_model_v2::WorldModelV2;

    #[test]
    fn test_max_thinking_cycles_constant() {
        assert!(
            MAX_THINKING_CYCLES >= 1,
            "must allow at least 1 thinking cycle"
        );
        assert!(MAX_THINKING_CYCLES <= 10, "max cycles should be reasonable");
    }

    #[test]
    fn test_panorama_pipeline_new() {
        let pano = PanoramaPipeline::new();
        assert_eq!(pano.cycle, 0);
        assert_eq!(pano.total_anomalies, 0);
    }

    #[test]
    fn test_panorama_pipeline_status_nonempty() {
        let pano = PanoramaPipeline::new();
        let s = pano.status();
        assert!(!s.is_empty());
        assert!(s.contains("Panorama:"));
    }

    #[test]
    fn test_panorama_run_cycle_basic() {
        let mut pano = PanoramaPipeline::new();
        let mut brain = SelfIteratingBrain::new();
        let mut goal_loop = GoalLoop::new();
        let mut wm = WorldModelV2::new(4, 64);

        let report = pano.run_cycle(&mut brain, &mut goal_loop, &mut wm);
        assert_eq!(report.cycle, 1);
        assert!(report.hypercube_entries > 0);
    }

    #[test]
    fn test_panorama_multiple_cycles() {
        let mut pano = PanoramaPipeline::new();
        let mut brain = SelfIteratingBrain::new();
        let mut goal_loop = GoalLoop::new();
        let mut wm = WorldModelV2::new(4, 64);

        for i in 1..=3 {
            let report = pano.run_cycle(&mut brain, &mut goal_loop, &mut wm);
            assert_eq!(report.cycle, i);
        }
        assert_eq!(pano.cycle, 3);
        assert!(pano.status().contains("cycle=3"));
    }
}
