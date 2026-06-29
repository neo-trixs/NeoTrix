#![allow(unused_imports)]
use std::collections::VecDeque;

use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_e8::{killing_form, E8BlockDiagonal, E8Projector};
use crate::core::nt_core_gwt::resonance::{
    boost_low_weight_axes, compute_axis_weights, E8ModulationReport, MODULE_COUNT,
};
use crate::core::nt_core_hex::ReasoningHexagram;

// E8 handlers extracted from modules.rs
// 3 handlers

impl ConsciousnessIntegration {
    pub fn handle_e8_geometry_tick(&mut self) -> String {
        if self.attractor_state.len() < 64 {
            return "E8: waiting for attractor state".into();
        }
        let distance = self
            .geometric_ssm
            .update_from_vsa(&self.attractor_state, "attractor");
        let (beta_0, beta_1) = self.geometric_ssm.estimate_betti();
        let energy = self
            .geometric_ssm
            .current_state
            .as_ref()
            .map(|s| s.energy)
            .unwrap_or(0.0);
        format!(
            "E8: geodesic={:.4} β₀={} β₁={} energy={:.2}",
            distance, beta_0, beta_1, energy
        )
    }

    // ── E8 Cortical Mapping Tick ──

    pub fn handle_e8_cortical_tick(&mut self) -> String {
        if self.e8_cortical.is_initialized() {
            format!("e8_cortical:{}_roots", self.e8_cortical.root_count())
        } else {
            "e8_cortical:uninitialized".to_string()
        }
    }

    // ── E8 Attractor Dynamics (VSA → E8 root projection) ──

    pub fn handle_e8_attractor_dynamics(&mut self) -> String {
        let vsa_buffer: [u8; 512] = if self.global_workspace.workspace_state.len() == 512 {
            let mut buf = [0u8; 512];
            buf.copy_from_slice(&self.global_workspace.workspace_state[..512]);
            buf
        } else if self.attractor_state.len() >= 512 {
            let mut buf = [0u8; 512];
            buf.copy_from_slice(&self.attractor_state[..512]);
            buf
        } else {
            log::warn!("E8: no VSA buffer available, using cycle-seeded default");
            let mut buf = [0u8; 512];
            buf[0] = self.cycle as u8;
            buf
        };

        let root = E8Projector::project_vsa(&vsa_buffer);
        let simple = self.e8_lattice.simple_roots();
        let nearest_root = simple
            .iter()
            .min_by_key(|sr| {
                sr.coords
                    .iter()
                    .zip(root.coords.iter())
                    .map(|(a, b): (&i8, &i8)| (*a as i16 - *b as i16).unsigned_abs() as u32)
                    .sum::<u32>()
            })
            .copied()
            .unwrap_or(&root);
        let k = killing_form(&root, nearest_root);
        let norm = root.norm_sq();
        log::debug!("E8: projected root norm={:.1} Killing={}", norm, k);
        format!("e8:root_norm_{:.1}_weight_{}", norm, k)
    }

    // ── E8 Training Tick ──

    pub fn handle_e8_training_tick(&mut self) -> String {
        let energy = self.e8_block_diagonal.compute_energy();
        let block_stats: Vec<String> = (0..8)
            .map(|i| {
                let block = &self.e8_block_diagonal.blocks[i];
                let frob: f64 = block.iter().flat_map(|r: &[f64; 30]| r.iter()).map(|x| x * x).sum();
                format!("b{}={:.1}", i, frob)
            })
            .collect();
        format!("e8_training:energy={:.2}_{}", energy, block_stats.join("_"))
    }

    // ── Contrastive Reflection (P0.14) ──

    // ── Phase 8.3 — E8 Adaptive Modulation Tick ──

    pub fn handle_e8_modulation_tick(&mut self) -> E8ModulationReport {
        // Read axis weights from specialist states
        let states = &self.specialist_states;
        let mut axis_weights = compute_axis_weights(states);

        // Update from broadcast history if available
        if !self.broadcast_history.is_empty() {
            let winner = self.broadcast_history[self.broadcast_history.len() - 1];
            for axis in 0..6 {
                let winner_val = states[winner].axis(axis);
                let same_count = states.iter().filter(|s| s.axis(axis) == winner_val).count();
                let discriminative = 1.0 - (same_count as f64 - 1.0) / (MODULE_COUNT as f64 - 1.0);
                axis_weights[axis] = 0.5 * axis_weights[axis] + 0.5 * discriminative;
            }
        }

        let sum_w: f64 = axis_weights.iter().sum();
        if sum_w > 0.0 {
            for w in &mut axis_weights {
                *w /= sum_w;
            }
        }

        // Compute modulation entropy from weight distribution
        let modulation_entropy = {
            let total: f64 = axis_weights.iter().sum();
            if total > 0.0 {
                -axis_weights
                    .iter()
                    .filter(|&&w| w > 0.0)
                    .map(|&w| {
                        let p = w / total;
                        p * p.log2()
                    })
                    .sum::<f64>()
            } else {
                0.0
            }
        };

        let dominant_axis = axis_weights
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Encourage exploration if one axis dominates (low entropy)
        const MODULATION_ENTROPY_THRESHOLD: f64 = 1.2;
        let exploration_boosted = boost_low_weight_axes(
            &mut axis_weights,
            modulation_entropy,
            MODULATION_ENTROPY_THRESHOLD,
        );

        // Store in e8_modulation state
        self.e8_modulation = Some(E8ModulationState {
            axis_weights,
            modulation_entropy,
        });

        E8ModulationReport {
            axis_weights,
            dominant_axis,
            modulation_entropy,
            exploration_boosted,
        }
    }

    // ── Phase 10.1 — Async Deep Processing Tick ──

    pub fn handle_async_deep_processing(&mut self) -> String {
        if self.async_tasks.is_empty() {
            return "async_deep:idle".to_string();
        }
        let task_id = match self.async_tasks.front() {
            Some(t) if t.result.is_none() => t.id,
            _ => return "async_deep:completed".to_string(),
        };
        let task_type = {
            let t = self.async_tasks.iter().find(|t| t.id == task_id).unwrap();
            match t.task_type {
                AsyncTaskType::E8DeepReason => {
                    let cycles = self.cycle;
                    format!("e8_deep_reason:cycle={}", cycles)
                }
                AsyncTaskType::IdentityReflection => {
                    let coherence = self.specious_present.average_coherence();
                    format!("identity_reflection:coherence={:.3}", coherence)
                }
                AsyncTaskType::ExperienceConsolidation => {
                    let entry_count = self.self_experience_buffer.len();
                    format!("experience_consolidation:entries={}", entry_count)
                }
                AsyncTaskType::GWTReplay => {
                    let hist_len = self.broadcast_history.len();
                    format!("gwt_replay:history={}", hist_len)
                }
            }
        };
        if let Some(task) = self.async_tasks.iter_mut().find(|t| t.id == task_id) {
            task.result = Some(task_type.clone());
        }
        format!("async_deep:completed_{}", task_id)
    }

    // ── Schedule an async task ──

    pub fn schedule_async_task(&mut self, task_type: AsyncTaskType) -> u64 {
        let id = self.async_task_counter;
        self.async_task_counter += 1;
        self.async_tasks.push_back(AsyncTask {
            id,
            task_type,
            created_at: self.cycle as f64,
            result: None,
        });
        id
    }
}
