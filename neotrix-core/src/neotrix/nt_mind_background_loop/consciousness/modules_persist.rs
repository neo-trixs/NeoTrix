use super::ConsciousnessIntegration;
use crate::core::nt_core_storage::{Record, VsaTag, RT_CONSCIOUSNESS_STATE};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Lightweight serializable snapshot of consciousness state for event-sourced persistence.
/// Written to NTSSEG every SNAPSHOT_INTERVAL cycles as an RT_CONSCIOUSNESS_STATE record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessStateSnapshot {
    pub cycle: u64,
    pub cognitive_load: f64,
    pub last_efe_energy: f64,
    pub da: f64,
    pub ne: f64,
    pub ht: f64,
    pub ach: f64,
    pub arousal: f64,
    pub valence_bias: f64,
    pub coherence: f64,
    pub attractor_hash: u64,
    pub timestamp_nanos: u64,
}

/// How many cycles between consciousness state snapshots
const SNAPSHOT_INTERVAL: u64 = 10;

impl ConsciousnessIntegration {
    /// Persist current consciousness state to NTSSEG as an event-sourced record.
    /// Called every SNAPSHOT_INTERVAL cycles from handle_persist_tick.
    pub fn write_consciousness_snapshot(&mut self) -> Result<(), String> {
        let engine = self
            .storage_engine
            .as_mut()
            .ok_or("storage_engine not initialized")?;
        let nm = self.neuromodulator.stats();
        let snapshot = ConsciousnessStateSnapshot {
            cycle: self.cycle,
            cognitive_load: self.cognitive_load,
            last_efe_energy: self.last_efe_energy,
            da: nm.da,
            ne: nm.ne,
            ht: nm.ht,
            ach: nm.ach,
            arousal: nm.arousal,
            valence_bias: nm.valence_bias,
            coherence: self.valence_axis.coherence(),
            attractor_hash: {
                let h: u64 = self
                    .attractor_state
                    .iter()
                    .fold(0u64, |h, &b| h.wrapping_mul(31).wrapping_add(b as u64));
                h
            },
            timestamp_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        };
        let json = serde_json::to_vec(&snapshot).map_err(|e| format!("serialize: {}", e))?;
        let key = format!("consciousness/cycle_{}", self.cycle);
        let record = Record::new(VsaTag::SelfMemory, RT_CONSCIOUSNESS_STATE, &key, json);
        engine.put(record).map_err(|e| format!("put: {}", e))?;
        log::debug!(
            "PERSIST: wrote cycle {} snapshot ({} bytes)",
            self.cycle,
            snapshot.attractor_hash
        );
        Ok(())
    }

    /// Public shutdown API: wraps write_final_consciousness_snapshot with structured logging.
    /// Graceful if engine is uninitialized — logs a warning instead of panicking.
    pub fn save_on_shutdown(&mut self) {
        let result = self.write_final_consciousness_snapshot();
        let level = if result.starts_with("final_persist:ok") {
            log::Level::Info
        } else {
            log::Level::Warn
        };
        log::log!(
            level,
            "[shutdown] consciousness state saved to NTSSEG: {}",
            result
        );
    }

    /// Write a final snapshot immediately (for shutdown). Falls back to
    /// lazy engine init if not yet initialized so shutdown always captures state.
    pub fn write_final_consciousness_snapshot(&mut self) -> String {
        if self.storage_engine.is_none() {
            let cfg = crate::core::nt_core_storage::StorageConfig {
                data_dir: ".neotrix/storage".into(),
                ..Default::default()
            };
            self.storage_engine = match crate::core::nt_core_storage::StorageEngine::new(cfg) {
                Ok(e) => Some(e),
                Err(e) => return format!("final_persist:init_error_{}", e),
            };
        }
        match self.write_consciousness_snapshot() {
            Ok(_) => format!("final_persist:ok_cycle_{}", self.cycle),
            Err(e) => format!("final_persist:fail_{}", e),
        }
    }

    /// Replay all consciousness state snapshots from NTSSEG, returning them
    /// sorted by cycle. Event-source view: each record is an immutable fact.
    /// This enables "time travel debugging" of consciousness evolution.
    pub fn replay_consciousness_history(&self) -> Vec<ConsciousnessStateSnapshot> {
        let engine = match self.storage_engine.as_ref() {
            Some(e) => e,
            None => return Vec::new(),
        };
        let records = engine.find_by_type(RT_CONSCIOUSNESS_STATE);
        let mut snapshots: Vec<ConsciousnessStateSnapshot> = records
            .iter()
            .filter_map(|r| serde_json::from_slice(&r.data).ok())
            .collect();
        snapshots.sort_by_key(|s| s.cycle);
        snapshots
    }

    /// Build a human-readable timeline report from all historical snapshots.
    /// Each line: cycle → (cognitive_load, arousal, coherence, efe_energy)
    pub fn replay_consciousness_history_report(&self) -> String {
        let history = self.replay_consciousness_history();
        if history.is_empty() {
            return "replay:no_history".into();
        }
        let mut lines: Vec<String> = Vec::with_capacity(history.len() + 2);
        lines.push("=== Consciousness Event Timeline ===".into());
        lines.push(format!(
            "{:>6} | {:>6} | {:>6} | {:>6} | {:>8} | {:>8}",
            "cycle", "load", "arousal", "coher", "efe_ene", "nm_da"
        ));
        lines.push("-".repeat(60));
        for s in &history {
            lines.push(format!(
                "{:>6} | {:>6.2} | {:>6.3} | {:>6.3} | {:>8.3} | {:>8.3}",
                s.cycle, s.cognitive_load, s.arousal, s.coherence, s.last_efe_energy, s.da
            ));
        }
        lines.push(format!("--- {} snapshots total ---", history.len()));
        lines.join("\n")
    }

    /// Load the latest consciousness snapshot from NTSSEG and restore runtime state.
    /// Returns `true` if a snapshot was found and restored, `false` otherwise.
    pub fn load_from_ntsseg(&mut self) -> bool {
        let engine = match self.storage_engine.as_ref() {
            Some(e) => e,
            None => {
                log::info!("No prior consciousness state found, starting fresh");
                return false;
            }
        };
        let records = engine.find_by_type(RT_CONSCIOUSNESS_STATE);
        let latest = records
            .iter()
            .filter_map(|r| serde_json::from_slice::<ConsciousnessStateSnapshot>(&r.data).ok())
            .max_by_key(|s| s.cycle);
        match latest {
            Some(snap) => {
                if snap.cycle > self.cycle {
                    self.cycle = snap.cycle;
                }
                self.cognitive_load = snap.cognitive_load;
                self.last_efe_energy = snap.last_efe_energy;
                self.neuromodulator.da.level = snap.da;
                self.neuromodulator.ne.level = snap.ne;
                self.neuromodulator.ht.level = snap.ht;
                self.neuromodulator.ach.level = snap.ach;
                log::info!(
                    "Restored consciousness state from NTSSEG (cycle {})",
                    snap.cycle
                );
                true
            }
            None => {
                log::info!("No prior consciousness state found, starting fresh");
                false
            }
        }
    }

    /// Load the latest snapshot from NTSSEG and restore key runtime fields.
    /// Call after engine init to continue from previous state.
    /// Returns a description of what was restored (or "no_snapshot").
    pub fn load_latest_consciousness_state(&mut self) -> String {
        let engine = match self.storage_engine.as_ref() {
            Some(e) => e,
            None => return "load:no_engine".into(),
        };
        let records = engine.find_by_type(RT_CONSCIOUSNESS_STATE);
        let latest = records
            .iter()
            .filter_map(|r| serde_json::from_slice::<ConsciousnessStateSnapshot>(&r.data).ok())
            .max_by_key(|s| s.cycle);
        match latest {
            Some(snap) => {
                if snap.cycle > self.cycle {
                    self.cycle = snap.cycle;
                }
                self.cognitive_load = snap.cognitive_load;
                self.last_efe_energy = snap.last_efe_energy;
                // Restore neuromodulator channel levels from snapshot
                self.neuromodulator.da.level = snap.da;
                self.neuromodulator.ne.level = snap.ne;
                self.neuromodulator.ht.level = snap.ht;
                self.neuromodulator.ach.level = snap.ach;
                // arousal & valence_bias are derived from ne/ht levels, not stored directly
                log::info!(
                    "PERSIST: restored state from cycle {} (load={}, efe={})",
                    snap.cycle,
                    snap.cognitive_load,
                    snap.last_efe_energy
                );
                format!("load:ok_cycle_{}", snap.cycle)
            }
            None => "load:no_snapshot".into(),
        }
    }

    /// Replay consciousness state snapshots from NTSSEG over a cycle range,
    /// restoring CI fields on each step for time-travel debugging.
    /// Accepts optional snapshot_count limit. Returns ReplayResult summary.
    pub fn replay_from_ntsseg(
        &mut self,
        from_cycle: u64,
        to_cycle: u64,
        snapshot_count: Option<usize>,
    ) -> ReplayResult {
        let engine = match self.storage_engine.as_ref() {
            Some(e) => e,
            None => {
                return ReplayResult {
                    replayed: 0,
                    skipped: 0,
                    from_cycle,
                    to_cycle,
                };
            }
        };
        let records = engine.find_by_type(RT_CONSCIOUSNESS_STATE);
        let mut snapshots: Vec<ConsciousnessStateSnapshot> = records
            .iter()
            .filter_map(|r| serde_json::from_slice::<ConsciousnessStateSnapshot>(&r.data).ok())
            .filter(|s| s.cycle >= from_cycle && s.cycle <= to_cycle)
            .collect();
        snapshots.sort_by_key(|s| s.cycle);
        if let Some(limit) = snapshot_count {
            snapshots.truncate(limit);
        }
        let mut replayed: u64 = 0;
        let mut skipped: u64 = 0;
        for snap in &snapshots {
            self.cycle = snap.cycle;
            self.cognitive_load = snap.cognitive_load;
            self.last_efe_energy = snap.last_efe_energy;
            self.neuromodulator.da.level = snap.da;
            self.neuromodulator.ne.level = snap.ne;
            self.neuromodulator.ht.level = snap.ht;
            self.neuromodulator.ach.level = snap.ach;
            replayed += 1;
        }
        let total_found = snapshots.len() as u64;
        if records.len() > snapshots.len() {
            skipped = (records.len() - (total_found as usize)) as u64;
        }
        let result = ReplayResult {
            replayed,
            skipped,
            from_cycle,
            to_cycle,
        };
        log::info!(
            "REPLAY: {} snapshots replayed ({} skipped) from cycle {} to {}",
            result.replayed,
            result.skipped,
            result.from_cycle,
            result.to_cycle,
        );
        result
    }

    /// Tick handler: persist consciousness state snapshot every SNAPSHOT_INTERVAL cycles.
    /// Also reads and reports engine stats for monitoring.
    pub fn handle_persist_tick(&mut self) -> String {
        if self.storage_engine.is_none() {
            return "persist:no_engine".into();
        }
        if self.cycle > 0 && self.cycle % SNAPSHOT_INTERVAL == 0 {
            if let Err(e) = self.write_consciousness_snapshot() {
                log::error!("PERSIST: snapshot failed at cycle {}: {}", self.cycle, e);
                return format!("persist:fail_{}", e);
            }
        }
        let stats = self
            .storage_engine
            .as_ref()
            .and_then(|e| {
                let s = e.stats();
                Some(format!(
                    "recs={} segs={} credit={:.0}%",
                    s.record_count,
                    s.segment_count,
                    s.credit_utilization * 100.0
                ))
            })
            .unwrap_or_default();
        format!("persist:ok_{}", stats)
    }
}

/// Result of a `replay_from_ntsseg` operation: how many snapshots were
/// successfully replayed and how many were skipped (failed to deserialize).
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplayResult {
    pub replayed: u64,
    pub skipped: u64,
    pub from_cycle: u64,
    pub to_cycle: u64,
}

impl fmt::Display for ReplayResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "replay:{}_replayed/{}_skipped/from_{}/to_{}",
            self.replayed, self.skipped, self.from_cycle, self.to_cycle
        )
    }
}
