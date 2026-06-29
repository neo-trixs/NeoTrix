use std::sync::atomic::Ordering;

use super::types::*;
use crate::core::nt_core_knowledge::self_inspect::{LanguageSpec, SelfInspectable};

impl ConsciousnessIntegration {
    pub fn stats(&self) -> ExperienceStats {
        let nm = self.neuromodulator.stats();
        let cr = self.inner_critic.pass_rate();
        let ci = self.inner_critic.critiques_issued();
        let reflexivity = self.meta_cognition_loop.current_meta_accuracy() * 0.6 + cr * 0.4;
        let emotion = {
            let da = nm.da;
            let ne = nm.ne;
            let ht = nm.ht;
            if da > 0.5 && ne > 0.5 {
                "engaged"
            } else if da > 0.6 {
                "excited"
            } else if ne > 0.6 {
                "alert"
            } else if ht > 0.6 {
                "calm"
            } else if da < 0.3 {
                "subdued"
            } else if ne < 0.3 {
                "tired"
            } else if ht < 0.3 {
                "restless"
            } else {
                "neutral"
            }
        }
        .to_string();
        let s = ExperienceStats {
            c_score: self.specious_present.average_coherence(),
            sp_coherence: self.specious_present.average_coherence(),
            nm_da: nm.da,
            nm_ne: nm.ne,
            nm_ht: nm.ht,
            nm_ach: nm.ach,
            critic_pass_rate: cr,
            load_mode: 0,
            vsa_buffer_size: self.vsa_buffer.len(),
            text_feed_total: self.text_feed_count,
            reflexivity,
            emotion,
            critic_issued: ci,
            cycle: self.cycle,
            last_critique: crate::core::nt_core_consciousness::CritiqueResult::perfect(),
        };
        // Publish to global stats for desktop bridge
        if let Some(guard) = super::types_consciousness::GLOBAL_CONSCIOUSNESS_STATS.get() {
            if let Ok(mut g) = guard.lock() {
                *g = s.clone();
                super::types_consciousness::GLOBAL_STATS_READY.store(true, Ordering::Relaxed);
            }
        }
        s
    }

    pub fn handle_distill_spec(&mut self) -> LanguageSpec {
        self.distill_language_spec()
    }

    pub fn handle_export_stats(&mut self) -> String {
        let s = self.stats();
        let export = format!(
            "{{\"c_score\":{:.4},\"coherence\":{:.4},\"vsa_buffer\":{},\"text_feed\":{},\"cycle\":{}}}",
            s.c_score, s.sp_coherence, s.vsa_buffer_size, s.text_feed_total, s.cycle
        );
        log::info!("STATS: export => {}", &export[..export.len().min(120)]);
        export
    }

    pub fn handle_user_request(&mut self, request: &str) -> String {
        log::info!("STATS: user_request => {}", request);
        match request {
            "stats" | "status" => format!("{:?}", self.stats()),
            "distill" => {
                let spec = self.distill_language_spec();
                format!(
                    "distilled: {} prims, {} subspaces, {} handlers",
                    spec.vsa_primitives.len(),
                    spec.subspace_topology.subspaces.len(),
                    spec.handler_graph.handlers.len(),
                )
            }
            "export" => self.handle_export_stats(),
            _ => format!("unknown_request:{}", request),
        }
    }
}
