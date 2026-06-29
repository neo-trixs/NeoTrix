use super::ConsciousnessIntegration;
use crate::core::nt_core_inference::cascade::CascadeEngine;
use crate::core::nt_core_inference::cascade::CascadeOutcome;

impl ConsciousnessIntegration {
    /// Process text_buffer content through cascade engine.
    /// Enqueues substantive text, then processes one item using E8 internal
    /// reasoning as drafter and the LLM router as optional verifier.
    pub fn handle_cascade_engine_tick(&mut self) -> String {
        let engine = self.cascade_engine.get_or_insert_with(|| {
            crate::core::nt_core_inference::cascade::CascadeEngine::with_defaults()
        });

        // Enqueue substantive text from buffer as cascade queries
        while let Some(text) = self.text_buffer.pop_front() {
            let trimmed = text.trim();
            if trimmed.len() > 30
                && !trimmed.starts_with("cascade:")
                && !trimmed.starts_with("spatial:")
            {
                engine.enqueue_query(trimmed.to_string());
            }
        }

        // Extract VSA decoder output for drafter closure (avoids &mut self capture)
        let drafter_response = if !self.attractor_state.is_empty() {
            let decoded = self.vsa_decoder.decode(
                &self.attractor_state,
                "cascade",
                self.cycle,
                self.specious_present.average_coherence(),
                self.neuromodulator.arousal_contribution(),
            );
            let mut response = decoded.title;
            if response.is_empty() {
                response = decoded
                    .sections
                    .first()
                    .map(|s| s.label.clone())
                    .unwrap_or_default();
            }
            response
        } else {
            String::new()
        };

        // Process one pending query with data-driven drafter closure
        let processed = engine.process_pending_sync(
            &mut |_query: &str| {
                let r = drafter_response.clone();
                (r, 0.001, 5.0)
            },
            None, // verifier disabled for now — see H-3 for async integration
        );

        let stats = engine.stats();
        let outcome_summary = if let Some(ref outcome) = processed {
            format!(
                "_last=esc:{}_conf:{:.2}_tier:{}",
                outcome.escalated, outcome.confidence, outcome.model_tier
            )
        } else {
            String::new()
        };
        format!(
            "cascade:queries={}_esc={}_rate={:.2}_cost={:.4}_pending={}{}",
            stats.total_queries,
            stats.verifier_escalations,
            stats.escalation_rate(),
            stats.total_cost(),
            engine.pending_count(),
            outcome_summary,
        )
    }

    /// Process one pending query through async cascade engine with LLM verifier.
    ///
    /// Uses the E8 drafter for fast draft generation, then asynchronously
    /// invokes the LLM verifier if drafter confidence is below threshold.
    /// Only activates when there are pending queries.
    pub async fn handle_cascade_verifier_tick_async(&mut self) -> String {
        let engine = self
            .cascade_engine
            .get_or_insert_with(|| CascadeEngine::with_defaults());

        if engine.pending_count() == 0 {
            return "cascade_verifier:idle".to_string();
        }

        // Extract VSA decoder output for drafter closure
        let drafter_response = if !self.attractor_state.is_empty() {
            let decoded = self.vsa_decoder.decode(
                &self.attractor_state,
                "cascade",
                self.cycle,
                self.specious_present.average_coherence(),
                self.neuromodulator.arousal_contribution(),
            );
            let mut response = decoded.title;
            if response.is_empty() {
                response = decoded
                    .sections
                    .first()
                    .map(|s| s.label.clone())
                    .unwrap_or_default();
            }
            response
        } else {
            String::new()
        };

        if drafter_response.is_empty() {
            return "cascade_verifier:no_drafter".to_string();
        }

        let verifier_fn = crate::core::nt_core_inference::verifier::create_verifier_fn();

        let processed = engine
            .process_pending_async(
                &mut |_query: &str| (drafter_response.clone(), 0.001, 5.0),
                Some(&verifier_fn),
            )
            .await;

        let outcome_desc = match processed {
            Some(ref o) => format!("_esc:{}_conf:{:.2}", o.escalated, o.confidence),
            None => String::new(),
        };

        format!(
            "cascade_verifier:pending={}{}",
            engine.pending_count(),
            outcome_desc,
        )
    }

    /// Reset cascade engine stats and flush queues.
    pub fn handle_cascade_engine_reset_tick(&mut self) -> String {
        if let Some(ref mut engine) = self.cascade_engine {
            engine.reset_stats();
            engine.pending_queries.clear();
            engine.completed_results.clear();
            "cascade:stats_reset_queues_flushed".to_string()
        } else {
            "cascade:no_engine".to_string()
        }
    }

    /// Return completed cascade outcomes for consumption.
    pub fn take_cascade_results(&mut self) -> Vec<(String, CascadeOutcome)> {
        self.cascade_engine
            .as_mut()
            .map(|e| e.completed_results.drain(..).collect())
            .unwrap_or_default()
    }

    /// Lazy-init causal reasoning engine, reports engine stats.
    /// Prediction generation is deferred — the engine collects causal links
    /// from other subsystems via record_causal_link.
    pub fn handle_causal_reasoning_tick(&mut self) -> String {
        let engine = self.causal_reasoning.get_or_insert_with(|| {
            crate::core::nt_core_inference::causal_chain::CausalReasoningEngine::new()
        });

        format!(
            "causal_reasoning:links={}_counterfactuals={}_predictions={}",
            engine.causal_links.len(),
            engine.counterfactuals.len(),
            engine.predictions.len(),
        )
    }

    /// Lazy-init long-horizon predictor, record an observation snapshot
    /// using composite loss as proxy value, detect regime changes, report stats.
    pub fn handle_long_horizon_tick(&mut self) -> String {
        let engine = self.long_horizon_predictor.get_or_insert_with(|| {
            crate::core::nt_core_inference::long_horizon::LongHorizonPredictor::new()
        });

        // Record current composite loss as proxy observation value
        let value = self.composite_loss.compute().total;
        engine.record_observation(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            value,
            &format!("cycle_{}", self.cycle),
        );

        // Detect regime changes in the time series
        let regime = engine.detect_regime_change();
        let regime_msg = if regime.detected {
            format!("_regime:{}", regime.description)
        } else {
            String::new()
        };

        format!(
            "long_horizon:history={}_forecasts={}_regimes={}_value={:.4}{}",
            engine.history.len(),
            engine.forecasts.len(),
            engine.regime_changes.len(),
            value,
            regime_msg,
        )
    }

    /// Lazy-init SCM (Pearl do-calculus) engine.
    /// Records composite loss as an observation variable, runs a periodic
    /// back-door adjustment if the graph has an edge, and reports engine stats.
    pub fn handle_scm_engine_tick(&mut self) -> String {
        let engine = self
            .scm_engine
            .get_or_insert_with(|| crate::core::nt_core_inference::SCMEngine::new());

        // Periodically inject composite loss as an observed variable
        let loss = self.composite_loss.compute().total;
        engine.observe("composite_loss", loss);
        engine.observe("cognitive_load", self.cognitive_load);
        engine.observe("negentropy", loss);

        // Run a basic back-door adjustment if the graph has at least one edge
        let effects = if engine.graph.edges.values().any(|v| !v.is_empty()) {
            let mut buf = String::new();
            for (from, children) in &engine.graph.edges {
                for to in children {
                    let effect = engine.estimate_do(to, from, 1.0);
                    buf.push_str(&format!(
                        " {}→{}={:.3}({})",
                        effect.intervention_var, effect.target, effect.estimate, effect.method,
                    ));
                }
            }
            buf
        } else {
            String::new()
        };

        format!(
            "scm:vars={}_edges={}_eqs={}_obs={}{}",
            engine.graph.nodes.len(),
            engine.graph.edges.values().map(|v| v.len()).sum::<usize>(),
            engine.graph.equations.len(),
            engine.data.values().map(|v| v.len()).sum::<usize>(),
            effects,
        )
    }
}
