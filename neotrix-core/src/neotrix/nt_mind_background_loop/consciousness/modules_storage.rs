#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_hcube::adapt_encoder::AdaptiveVsaEncoder;

// STORAGE handlers extracted from modules.rs
// 2 handlers

impl ConsciousnessIntegration {
    pub fn handle_null_drift_tick(&mut self) -> String {
        if self.attractor_state.is_empty() {
            return "null_drift:no_attractor".into();
        }
        let tag = crate::core::nt_core_storage::VsaTag::SelfThought;
        let slot = self.null_drift.insert(&self.attractor_state, tag);
        let count = self.null_drift.count();
        let mut parts = vec![format!("slot={}", slot), format!("total={}", count)];
        if count > 100 {
            let related = self.null_drift.search(&self.attractor_state, 5);
            let sims: Vec<String> = related.iter().map(|(_, s)| format!("{:.3}", s)).collect();
            parts.push(format!("related=[{}]", sims.join(",")));
        }
        let msg = parts.join(" ");
        log::debug!("MODULES: null_drift_tick {}", msg);
        format!("null_drift:{}", msg)
    }

    // ── Adaptive VSA Encoder ──

    pub fn handle_adaptive_vsa_tick(&mut self) -> String {
        if self.adaptive_vsa.is_none() {
            self.adaptive_vsa = Some(AdaptiveVsaEncoder::new(4096, 42, 4096));
            log::debug!("MODULES: adaptive_vsa_tick initialized");
        }
        if let Some(ref encoder) = self.adaptive_vsa {
            if !self.attractor_state.is_empty() {
                let state_text = format!("attractor_state_{}", self.cycle);
                let _v = encoder.encode_with_tag(&state_text, "cognitive");
            }
            let kw = encoder.kernel_width();
            log::debug!("MODULES: adaptive_vsa_tick kernel_width={}", kw);
            format!("adaptive_vsa:kw={}", kw)
        } else {
            "adaptive_vsa:uninitialized".to_string()
        }
    }

    // ── NTSSEG Storage Engine ──

    pub fn handle_storage_engine_tick(&mut self) -> String {
        if self.storage_engine.is_none() {
            let cfg = crate::core::nt_core_storage::StorageConfig {
                data_dir: ".neotrix/storage".into(),
                ..Default::default()
            };
            match crate::core::nt_core_storage::StorageEngine::new(cfg) {
                Ok(engine) => {
                    self.storage_engine = Some(engine);
                    // 启动时从NTSSEG恢复之前的状态
                    let load_result = self.load_latest_consciousness_state();
                    log::info!("MODULES: storage_engine initialized, {}", load_result);
                    format!("segstore:init_{}", load_result)
                }
                Err(e) => {
                    log::error!("MODULES: storage_engine init failed: {}", e);
                    format!("segstore:init_error_{}", e)
                }
            }
        } else {
            "segstore:ok".into()
        }
    }
}
