#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_consciousness::first_person_ref::ExperienceRecord;
use crate::core::nt_core_consciousness::value_alignment::ValueAlignmentEngine;
use crate::core::nt_core_consciousness::ConsciousnessAwakening;
use crate::core::nt_core_consciousness::ThinkingMode;

// CORE consciousness handlers (extracted from modules_core.rs)

impl ConsciousnessIntegration {
    pub fn handle_bridge_tick(&mut self) -> String {
        if let Some(ref se) = self.storage_engine {
            let s = se.stats();
            log::debug!(
                "MODULES: bridge_tick active rec={} seg={} credit={:.2}",
                s.record_count,
                s.segment_count,
                s.credit_utilization
            );
            format!(
                "bridge:ok:active rec={} seg={} credit={:.2}",
                s.record_count, s.segment_count, s.credit_utilization
            )
        } else {
            log::debug!("MODULES: bridge_tick no storage engine");
            "bridge:no_storage".to_string()
        }
    }

    // ── CTM engine ──

    pub fn handle_ctm_tick(&mut self) -> String {
        if self.ctm_engine.is_none() {
            self.ctm_engine = Some(crate::core::nt_core_ctm::inference::CtmEngine::new(vec![
                Box::new(crate::core::nt_core_ctm::processor::SpatialProcessor::new()),
                Box::new(crate::core::nt_core_ctm::processor::PhysicsProcessor::new()),
                Box::new(crate::core::nt_core_ctm::processor::GoalProcessor::new()),
                Box::new(crate::core::nt_core_ctm::processor::EpisodicProcessor::new()),
            ]));
            return "ctm_tick:init".to_string();
        }
        let engine = match self.ctm_engine.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        let stats = engine.stats();
        format!(
            "ctm_tick:inferences={}_processors={}",
            stats.total_inferences, stats.processor_count
        )
    }

    // ── Source cognition ──

    pub fn handle_source_cognition_tick(&mut self) -> String {
        let s = self.source_cognition.stats();
        log::debug!("MODULES: source_cognition_tick total={}", s.total_items);
        format!(
            "source_cognition:total={}_visual={}_auditory={}",
            s.total_items, s.visual_items, s.auditory_items
        )
    }

    // ── Input processing ──

    pub fn handle_vsa_input_pipeline_tick(&mut self) -> String {
        let count = self.vsa_buffer.len();
        let text_feed = self.text_feed_count;
        self.text_feed_count = text_feed.wrapping_add(1);
        format!("vsa_input_pipeline:buf={}_feed={}", count, text_feed)
    }

    // ── Temporal attention ──

    pub fn handle_temporal_attention_tick(&mut self) -> String {
        let s = self.temporal_attention.stats();
        log::debug!("MODULES: temporal_attention_tick seen={}", s.total_seen);
        format!(
            "temporal_attention_tick:seen={}_novelty={:.4}_decay={:.4}",
            s.total_seen, s.novelty_ratio, s.mean_decay
        )
    }

    // ── Cross-modal alignment ──

    pub fn handle_cross_modal_alignment_tick(&mut self) -> String {
        let stats = self.temporal_attention.stats();
        format!(
            "cross_modal_alignment:seen={}_novelty={:.4}",
            stats.total_seen, stats.novelty_ratio
        )
    }

    // ── Value alignment ──

    pub fn handle_value_alignment_tick(&mut self) -> String {
        let va = self
            .value_alignment_engine
            .get_or_insert_with(ValueAlignmentEngine::new);
        let stats = format!("profiles={}", va.profiles.len());
        log::debug!("MODULES: value_alignment_tick {}", stats);
        format!("value_alignment:{}", stats)
    }

    // ── Value system ──

    pub fn handle_value_system_tick(&mut self, _threshold: f64) -> String {
        let unsat = self.value_system.unsatisfied_values(_threshold);
        let diag = self.value_system.value_diagnostic();
        log::debug!(
            "MODULES: value_system_tick threshold={} unsat={} diag={}",
            _threshold,
            unsat.len(),
            diag
        );
        format!("value_system:unsat={}_diag={}", unsat.len(), diag)
    }

    // ── Volition ──

    pub fn handle_volition_tick(&mut self) -> String {
        match self.volition.select_best() {
            Some(candidate) => {
                let desc = &candidate.description;
                log::debug!("MODULES: volition_tick action={}", desc);
                format!(
                    "volition_tick:action={}_conf={:.4}_val={:.4}",
                    desc, candidate.confidence, candidate.expected_value
                )
            }
            None => {
                log::debug!("MODULES: volition_tick no_action");
                "volition_tick:no_action".to_string()
            }
        }
    }

    // ── Inner critic ──

    pub fn handle_inner_critic_tick(&mut self) -> String {
        let pr = self.inner_critic.pass_rate();
        let ci = self.inner_critic.critiques_issued();
        log::debug!(
            "MODULES: inner_critic_tick pass_rate={:.4} issued={}",
            pr,
            ci
        );
        format!("inner_critic_tick:pass={:.4}_issued={}", pr, ci)
    }

    // ── Specious present ──

    pub fn handle_specious_present_tick(&mut self) -> String {
        let coh = self.specious_present.average_coherence();
        log::debug!("MODULES: specious_present_tick coherence={:.4}", coh);
        format!("specious_present_tick:coh={:.4}", coh)
    }

    // ── Narrative self ──

    pub fn handle_narrative_self_tick(&mut self) -> String {
        self.narrative_self
            .record_iteration("consciousness_cycle", 0.0, None);
        let insight_count = self.narrative_self.current_session_insights.len();
        let total_reward = self.narrative_self.total_reward;
        log::debug!(
            "MODULES: narrative_self_tick insights={} reward={:.3}",
            insight_count,
            total_reward
        );
        format!(
            "narrative_self_tick:insights={},reward={:.3}",
            insight_count, total_reward
        )
    }

    // ── Valence axis ──

    pub fn handle_valence_axis_tick(&mut self) -> String {
        let emotion = self.valence_axis.current_emotion();
        let intensity = self.valence_axis.emotional_intensity();
        log::debug!(
            "MODULES: valence_axis_tick emotion={:?} intensity={:.4}",
            emotion,
            intensity
        );
        format!("valence:emotion={:?}_intensity={:.4}", emotion, intensity)
    }

    // ── Drive selector (PAD emotional drive → strategy selection) ──

    pub fn handle_drive_selector_tick(&mut self) -> String {
        let va = &self.valence_axis;
        let context_hash = self.compute_context_hash();
        let drive = self
            .drive_selector
            .select_drive(va.valence, va.arousal, context_hash);
        if self.cycle % 30 == 0 {
            let diag = self.drive_selector.diagnostic();
            log::debug!("MODULES: drive_selector_tick {} -> {:?}", diag, drive);
        }
        format!("drive:{:?}|{}", drive, self.drive_selector.diagnostic())
    }

    pub fn handle_drive_feedback(&mut self, success: bool, negentropy_delta: f64) -> String {
        self.drive_selector
            .update_from_experience(success, negentropy_delta);
        let context_hash = self.compute_context_hash();
        let drive_name = self.drive_selector.current_drive();
        self.drive_selector.record_feedback(
            &drive_name,
            success,
            context_hash,
            (1.0 - negentropy_delta.abs()).max(0.0),
        );
        format!(
            "drive_feedback:success={}_delta_n={:.4}",
            success, negentropy_delta
        )
    }

    /// Derive a context hash from the current VSA attractor state,
    /// falling back to a hash of the cycle count when no VSA context is available.

    fn compute_context_hash(&self) -> u64 {
        // Note: DefaultHasher output is not stable across Rust versions — FxHasher preferred for deterministic hashing
        use std::hash::{Hash, Hasher};
        if self.attractor_state.len() >= 4 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            self.attractor_state.hash(&mut hasher);
            self.cycle.hash(&mut hasher);
            hasher.finish()
        } else {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            self.cycle.hash(&mut hasher);
            hasher.finish()
        }
    }

    // ── VSA vocabulary (content-addressed semantic patterns) ──

    pub fn handle_vsa_vocabulary_tick(&mut self) -> String {
        let diag = self.vsa_vocabulary.diagnostic();
        log::debug!("MODULES: vsa_vocabulary_tick {}", diag);
        format!("vocab:{}", diag)
    }

    pub fn handle_vsa_vocabulary_query(&self, query: &[u8], top_k: usize) -> String {
        let nearest = self.vsa_vocabulary.nearest(query, top_k);
        if nearest.is_empty() {
            return "vocab_query:empty".to_string();
        }
        let terms: Vec<String> = nearest
            .iter()
            .map(|(p, s)| format!("{}:{:.3}", p.name(), s))
            .collect();
        format!("vocab_query:{}", terms.join(","))
    }

    // ── Cognitive load ──

    pub fn handle_cognitive_load_tick(&mut self, _load: f64) -> ThinkingMode {
        let avg = self.cognitive_load_monitor.average_load();
        let mode = self.cognitive_load_monitor.mode();
        log::debug!(
            "MODULES: cognitive_load_tick load={} avg_load={:.4} mode={:?}",
            _load,
            avg,
            mode
        );
        mode
    }

    // ── Default mode ──

    pub fn handle_default_mode_tick(&mut self, _active: bool) -> String {
        let coh = self.default_mode.average_reverberation_coherence();
        let idle = self.default_mode.is_idle();
        let novelty = self.default_mode.average_novelty();
        log::debug!(
            "MODULES: default_mode_tick active={} coh={:.4} idle={} novelty={:.4}",
            _active,
            coh,
            idle,
            novelty
        );
        format!(
            "default_mode:coh={:.4}_idle={}_novelty={:.4}",
            coh, idle, novelty
        )
    }

    // ── Stream buffer ──

    pub fn handle_stream_buffer_tick(&mut self) -> String {
        let len = self.stream_buffer.len();
        let total = self.stream_buffer.total_pushed();
        let coherence = self.stream_buffer.self_world_coherence();
        log::debug!(
            "MODULES: stream_buffer_tick len={} total={} coherence={:.4}",
            len,
            total,
            coherence
        );
        format!(
            "stream_buffer_tick:len={},total={},coherence={:.4}",
            len, total, coherence
        )
    }

    // ── First person ref ──

    pub fn handle_first_person_ref_tick(&mut self) -> String {
        let evolve_interval = 10u64;
        let mut evolved = 0u64;

        if self.cycle % evolve_interval == 0 && self.cycle > 0 {
            let experiences: Vec<ExperienceRecord> =
                self.self_experience_buffer.drain(..).collect();
            if !experiences.is_empty() {
                evolved = self
                    .first_person_ref
                    .evolve_from_experiences(&experiences, 0.4) as u64;
            }

            if !self.global_workspace.workspace_state.is_empty() {
                let coherence = self.first_person_ref.average_coherence();
                self.self_experience_buffer.push(ExperienceRecord {
                    vector: self.global_workspace.workspace_state.clone(),
                    coherence: coherence.max(0.5),
                    cycle: self.cycle,
                    source: "workspace".to_string(),
                    summary: "workspace state snapshot".to_string(),
                });
            }
        }

        if self.attractor_state.len() == crate::core::nt_core_hcube::VSA_DIM {
            let sim = crate::core::nt_core_hcube::vsa_quantized::similarity_packed(
                &crate::core::nt_core_hcube::vsa_quantized::pack_binary(&self.attractor_state),
                &crate::core::nt_core_hcube::vsa_quantized::pack_binary(
                    self.first_person_ref.self_vector(),
                ),
            );
            self.first_person_ref.record_coherence(sim);

            let threshold = self.first_person_ref.self_similarity_threshold();
            if sim < threshold * 0.8 {
                self.evolution_coordinator.report_degradation(
                    "first_person_ref",
                    &format!("coherence {:.4} below 0.8×threshold {:.4}", sim, threshold),
                );
            }
        }

        let avg_coh = self.first_person_ref.average_coherence();
        let threshold = self.first_person_ref.self_similarity_threshold();
        log::debug!(
            "MODULES: first_person_ref_tick birth={} evolved={} avg_coh={:.3} threshold={:.4}",
            self.first_person_ref.birth_step(),
            evolved,
            avg_coh,
            threshold,
        );
        format!(
            "first_person:birth={},coh={:.3},evolved={},avg_coh={:.3},thresh={:.4}",
            self.first_person_ref.birth_step(),
            avg_coh,
            evolved,
            avg_coh,
            threshold,
        )
    }

    // ── Awakening ──

    pub fn handle_awakening_tick(&mut self) -> String {
        if !self.awakening.is_awakened() && self.stream_buffer.total_pushed() >= 7 {
            let report = self.awakening.awaken(
                &mut self.stream_buffer,
                &mut self.specious_present,
                self.cycle,
            );
            self.first_person_ref = report.self_reference.clone();

            // Populate workspace with awakening content
            ConsciousnessAwakening::populate_workspace(
                &report,
                &mut self.global_workspace,
                self.cycle,
            );

            let birth = report.self_reference.birth_step();
            let coh = report.initial_coherence;
            log::info!(
                "AWAKENED at cycle {} birth_step={} coherence={:.3}",
                self.cycle,
                birth,
                coh
            );
            return format!("awakening_tick:awakened,birth={},coh={:.3}", birth, coh);
        }
        let buf_total = self.stream_buffer.total_pushed();
        log::debug!(
            "MODULES: awakening_tick awake={} total_pushed={}",
            self.awakening.is_awakened(),
            buf_total
        );
        format!(
            "awakening_tick:awake={},pushed={}",
            self.awakening.is_awakened(),
            buf_total
        )
    }

    // ── Global Workspace (selection-broadcast cycle) ──

    pub fn handle_workspace_tick(&mut self) -> String {
        if self.cycle % 3 != 0 {
            return "workspace: waiting".into();
        }
        if let Some(bc) = self.global_workspace.broadcast(self.cycle) {
            // Broadcast winner content to attractor_state
            self.attractor_state = bc.vector.clone();
            format!(
                "workspace: {} broadcast (salience {:.3})",
                bc.winner, bc.salience
            )
        } else {
            self.global_workspace.decay();
            "workspace: idle".into()
        }
    }

    // ── Constitution integrity ──

    pub fn handle_constitution_tick(&mut self) -> String {
        let report = self.constitution.check_integrity();
        let violations: Vec<String> = report
            .checks
            .iter()
            .filter(|c| !c.passed)
            .map(|c| format!("{}:{}", c.id, c.detail))
            .collect();
        if !violations.is_empty() {
            log::warn!(
                "CONSTITUTION: integrity FAIL — {} violations: {}",
                violations.len(),
                violations.join("; ")
            );
        }
        let status = if report.all_passed { "pass" } else { "FAIL" };
        log::debug!(
            "MODULES: constitution_tick status={} checks={}",
            status,
            report.checks.len()
        );
        format!(
            "constitution:status={}_checks={}_violations={}",
            status,
            report.checks.len(),
            violations.len(),
        )
    }
}
