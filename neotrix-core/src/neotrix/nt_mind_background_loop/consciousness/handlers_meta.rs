#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_experience::meta_cog_mera::{MetaObservation, ObsType, ReasoningStep};
use crate::core::nt_core_util;

// METACOGNITION handlers extracted from modules_core.rs
// 7 handlers

impl ConsciousnessIntegration {
    // ── Meta-cognition loop ──

    pub fn handle_meta_cognition_loop_tick(&mut self) -> String {
        // ── Lazy init KPI ring buffer on first tick ──
        if self.kpi_buffer.is_none() {
            let soul_dir = std::env::var("NEOTRIX_SOUL_DIR").unwrap_or_else(|_| {
                nt_core_util::home_dir()
                    .join(".neotrix")
                    .to_string_lossy()
                    .to_string()
            });
            let path = std::path::PathBuf::from(&soul_dir).join("kpi_ring_buffer.json");
            self.kpi_buffer = Some(crate::core::nt_core_meta::KpiRingBuffer::load(&path, 1000));
        }

        let result = self.meta_cognition_loop.run_cycle();

        // ── KPI persistence: push result to ring buffer ──
        let record = crate::core::nt_core_meta::KpiRecord {
            cycle: self.cycle as u64,
            iteration: result.iteration,
            meta_accuracy: result.meta_accuracy,
            meta_accuracy_trend: result.meta_accuracy_trend,
            alert_count: result.alerts.len(),
            plan_count: result.plans.len(),
            weakness_count: result.report.summary.total_count,
            compilation_ok: result.health_check.compilation_ok,
            timestamp: chrono::Utc::now(),
        };
        if let Some(ref mut buf) = self.kpi_buffer {
            buf.push(record);
        }

        // Periodically persist KPI buffer to disk every 100 cycles
        if self.cycle > 0 && self.cycle % 100 == 0 {
            if let Some(ref buf) = self.kpi_buffer {
                if let Err(e) = buf.persist() {
                    log::error!("[kpi] persist error: {}", e);
                }
            }
        }

        log::debug!(
            "MODULES: meta_cognition_loop_tick iter={} alerts={} plans={}",
            result.iteration,
            result.alerts.len(),
            result.plans.len()
        );
        format!(
            "meta_cognition:iter={}_alerts={}_plans={}",
            result.iteration,
            result.alerts.len(),
            result.plans.len()
        )
    }

    // ── MERA meta-cognitive loop ──

    pub fn handle_meta_cog_plan_tick(&mut self) -> String {
        let load = self.cognitive_load_monitor.average_load();
        let handlers = self.handler_registry.handler_names();
        let plan = self
            .meta_cog_monitor
            .proactive_plan(&handlers, load, self.cycle);
        log::debug!(
            "MODULES: meta_cog_plan_tick strategy={} budget={} diff={:.3}",
            plan.strategy,
            plan.allocated_budget,
            plan.difficulty_estimate
        );
        format!(
            "MERAPLAN: strategy={} budget={} diff={:.3}",
            plan.strategy, plan.allocated_budget, plan.difficulty_estimate
        )
    }

    pub fn handle_meta_cog_regulate_tick(&mut self) -> String {
        let obs = self.meta_cog_monitor.online_regulate();
        if obs.is_empty() {
            return "MERAREG: nominal".into();
        }
        let critical: Vec<&MetaObservation> = obs.iter().filter(|o| o.severity > 0.5).collect();
        for o in &critical {
            if matches!(o.observation_type, ObsType::ErrorSpike) {
                self.meta_cog_monitor.meta_state.intervention_active = true;
            }
        }
        let msg = format!(
            "MERAREG: {} observations ({} critical)",
            obs.len(),
            critical.len()
        );
        log::debug!("MODULES: {}", msg);
        msg
    }

    /// Record a trace step around handler execution. Call this after each handler.

    pub fn record_handler_trace(&mut self, handler: &str, result: &str, duration_ms: u64) {
        self.meta_cog_monitor.record_step(ReasoningStep {
            step_id: self.meta_cog_monitor.step_counter,
            handler_name: handler.to_string(),
            input_summary: String::new(),
            output_summary: result.to_string(),
            duration_ms,
            confidence: if result.contains("error") { 0.2 } else { 0.6 },
            error_flag: result.contains("error") || result.contains("FAIL"),
            cycle: self.cycle,
        });
        self.meta_cog_monitor.step_counter += 1;
        self.meta_cog_monitor.prune();
    }

    // ── Calibration engine ──

    pub fn handle_calibration_engine_tick(&mut self) -> String {
        let s = self.calibration.stats();
        log::debug!(
            "MODULES: calibration_engine_tick ece={:.4} m_ratio={:.4}",
            s.ece,
            s.m_ratio
        );
        format!(
            "calibration:ece={:.4}_m_ratio={:.4}_pairs={}",
            s.ece, s.m_ratio, s.pair_count
        )
    }

    // ── Phase 59 — Runtime Self-Introspection ──

    /// Collect diagnostic snapshot, detect defect patterns, execute auto-corrections,
    /// and push self-diagnosis into the response pipeline for external expression.
    ///
    /// Now with three nested loops:
    ///   Loop 1 (analyze): defect detection + handler_registry GC
    ///   Loop 2 (meta_audit): engine introspection + auto-GC via HealthCheckable
    ///   Loop 3 (auto_distill): novel patterns → DistilledExperience nodes

    pub fn handle_introspection_tick(&mut self) -> String {
        use crate::core::nt_core_experience::health_checkable::HealthCheckable;
        use crate::core::nt_core_experience::operational_mirror::MirrorState;
        use crate::core::nt_core_experience::self_introspection::DiagnosticSnapshot;

        // ── Build snapshot ──
        let handler_freq: Vec<(String, usize)> = self
            .handler_registry
            .handler_names()
            .into_iter()
            .map(|name| (name.to_string(), 1usize))
            .take(10)
            .collect();
        let component_sizes = vec![
            (
                "handler_registry".to_string(),
                self.handler_registry.count(),
            ),
            ("hooks".to_string(), self.hooks.len()),
        ];

        let stats = self.handler_registry.stats();
        let hot_ratio = if stats.total > 0 {
            stats.hot as f64 / stats.total as f64
        } else {
            0.0
        };
        let warm_ratio = if stats.total > 0 {
            stats.warm as f64 / stats.total as f64
        } else {
            0.0
        };
        let cold_ratio = if stats.total > 0 {
            stats.cold as f64 / stats.total as f64
        } else {
            0.0
        };

        let snapshot = DiagnosticSnapshot {
            cycle: self.cycle,
            active_handler_count: self.handler_registry.count(),
            pending_actions: self.introspect_engine.actions().len(),
            component_sizes,
            handler_frequencies: handler_freq,
        };

        // ── Operational Mirror: capture pre-state ──
        let pre_state = MirrorState::build_state(
            &snapshot,
            self.handler_registry.count(),
            hot_ratio,
            warm_ratio,
            cold_ratio,
            self.handler_registry.total_calls(),
            self.composite_loss.stats().total_loss,
            self.cognitive_load,
            self.neuromodulator.arousal_contribution(),
            self.neuromodulator.plasticity(),
        );

        // ── Loop 1: analyze → detect defects ──
        let new_actions = self.introspect_engine.tick(snapshot);
        let action_count = new_actions.len();
        let mut executed_descriptions: Vec<String> = Vec::new();

        // Execute auto-correction for high-priority actions
        for action in &new_actions {
            if action.priority >= 120 {
                match action.pattern {
                    crate::core::nt_core_experience::self_introspection::DefectPattern::AccumulationWithoutPruning { ref component, .. } if component == "handler_registry" => {
                        let stale = self.handler_registry.stale_handlers(std::time::Duration::from_secs(300));
                        for name in &stale {
                            self.handler_registry.mark_unloaded(name);
                        }
                        executed_descriptions.push(format!("GC'd {} stale handlers ({})", stale.len(), component));
                    }
                    crate::core::nt_core_experience::self_introspection::DefectPattern::ExcessiveProbing { .. } => {
                        let over_count = self.handler_registry.count();
                        let stale = self.handler_registry.stale_handlers(std::time::Duration::from_secs(120));
                        for name in &stale {
                            self.handler_registry.mark_unloaded(name);
                        }
                        executed_descriptions.push(format!("Reduced handler load from {} to {} (quarantined {} stale)", over_count, self.handler_registry.count(), stale.len()));
                    }
                    // HealthCheckable-driven GC for subsystems
                    crate::core::nt_core_experience::self_introspection::DefectPattern::AccumulationWithoutPruning { ref component, .. } if component == "translation_lexicon" => {
                        if let Some(ref mut engine) = self.translate_engine {
                            let reclaimed = engine.lexicon.prune();
                            executed_descriptions.push(format!("Pruned {} translation entries", reclaimed));
                        }
                    }
                    crate::core::nt_core_experience::self_introspection::DefectPattern::ActionAccumulation { .. } => {
                        let drained = self.introspect_engine.drain_actions().len();
                        executed_descriptions.push(format!("Drained {} introspection actions", drained));
                    }
                    _ => {}
                }
            }

            log::debug!(
                "INTROSPECT: [{}/{}] {}",
                action.priority,
                action.executed,
                action.suggestion
            );
        }

        // Mark executed actions for tracking
        self.introspect_engine.mark_executed();

        // ── Loop 2: meta-audit — check engine health ──
        let meta_result = self.handler_registry.check_health();
        if let Some((name, size)) = meta_result {
            let reclaimed = self.handler_registry.health_gc();
            executed_descriptions.push(format!(
                "HealthCheck GC'd {} (size {}, reclaimed {})",
                name, size, reclaimed
            ));
        }

        // ── Loop 3: auto-distill → experience nodes ──
        let new_experiences = self.introspect_engine.auto_distill();
        for exp in &new_experiences {
            log::debug!(
                "INTROSPECT: distilled experience '{}' (conf={})",
                exp.title,
                exp.confidence
            );
            executed_descriptions.push(format!("Distilled '{}': {}", exp.title, exp.rule));
        }

        // ── Loop 3b: three-pass self-dialectic (audit→reconcile→write) ──
        let dialectic_experiences = self.introspect_engine.run_self_dialectic();
        for exp in &dialectic_experiences {
            log::debug!(
                "INTROSPECT: dialectic '{}' (conf={})",
                exp.title,
                exp.confidence
            );
            executed_descriptions.push(format!("Dialectic '{}': {}", exp.title, exp.rule));
        }

        // ── Operational Mirror: record transitions ──
        let post_stats = self.handler_registry.stats();
        let post_hot = if post_stats.total > 0 {
            post_stats.hot as f64 / post_stats.total as f64
        } else {
            0.0
        };
        let post_warm = if post_stats.total > 0 {
            post_stats.warm as f64 / post_stats.total as f64
        } else {
            0.0
        };
        let post_cold = if post_stats.total > 0 {
            post_stats.cold as f64 / post_stats.total as f64
        } else {
            0.0
        };

        let post_state = MirrorState::build_state(
            &DiagnosticSnapshot {
                cycle: self.cycle,
                active_handler_count: self.handler_registry.count(),
                pending_actions: self.introspect_engine.actions().len(),
                component_sizes: Vec::new(),
                handler_frequencies: Vec::new(),
            },
            self.handler_registry.count(),
            post_hot,
            post_warm,
            post_cold,
            self.handler_registry.total_calls(),
            self.composite_loss.stats().total_loss,
            self.cognitive_load,
            self.neuromodulator.arousal_contribution(),
            self.neuromodulator.plasticity(),
        );

        let loss_before = self.composite_loss.stats().total_loss;
        let loss_after = self.composite_loss.compute().total;
        // Reward = improvement: inverted params so lower loss → positive reward
        let reward = crate::core::nt_core_experience::operational_mirror::compute_reward(
            loss_after,
            loss_before,
        );

        for action in &new_actions {
            self.operational_mirror.record_transition(
                pre_state.clone(),
                action,
                reward,
                post_state.clone(),
            );
        }

        if !new_actions.is_empty() {
            executed_descriptions.push(self.operational_mirror.report());
        }

        // ── External expression: push report into response pipeline ──
        if !executed_descriptions.is_empty() {
            let report = format!(
                "Self-diagnosis (cycle {}): {}",
                self.cycle,
                executed_descriptions.join("; ")
            );
            self.response_buffer.push_back(report.clone());
            log::debug!("INTROSPECT: pushed to response pipeline — {}", report);
        }

        // Observability: collect profiler report into response pipeline every 50 cycles
        if self.cycle > 0 && self.cycle % 50 == 0 {
            let profiler_report = self.profiler.sorted_report();
            if !profiler_report.is_empty() {
                let report = format!(
                    "Profiler (cycle {}): {} handlers, {} samples",
                    self.cycle,
                    profiler_report.len(),
                    self.profiler.total_samples()
                );
                self.response_buffer.push_back(report);
            }
        }

        format!(
            "introspect:{}_actions_mirror:{}",
            action_count,
            self.operational_mirror.transition_count()
        )
    }

    /// Handle operational mirror tick — report mirror stats on demand.

    pub fn handle_mirror_tick(&mut self) -> String {
        let stats = self.operational_mirror.stats();
        format!(
            "mirror:{}_transitions:cum_r={:.4}_actions:{}_best:{}_worst:{}",
            stats.total_transitions,
            stats.cumulative_reward,
            stats.unique_actions_seen,
            stats.best_avg_action,
            stats.worst_avg_action,
        )
    }
}
