// G396 + G412: Loop engineering meta-skill — run ledger, cross-session persistence
use crate::core::nt_core_hcube::VsaVector;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Reverted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopIteration {
    pub step: u64,
    pub proposal_id: u64,
    pub proposal_desc: String,
    pub predicted_gain: f64,
    pub realized_gain: Option<f64>,
    pub status: LoopStatus,
    pub vsa_snapshot: VsaVector,
    pub timestamp: u64,
    pub kept: bool,
    pub cycle: u64,
    pub steps_passed: u32,
    pub steps_failed: u32,
    pub c_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunLedger {
    pub loop_id: String,
    pub name: String,
    pub iterations: VecDeque<LoopIteration>,
    pub started_at: u64,
    pub last_updated: u64,
    pub total_kept: u64,
    pub total_reverted: u64,
    pub best_score: f64,
}

impl RunLedger {
    pub fn new(loop_id: &str, name: &str) -> Self {
        Self {
            loop_id: loop_id.to_string(),
            name: name.to_string(),
            iterations: VecDeque::with_capacity(10000),
            started_at: 0,
            last_updated: 0,
            total_kept: 0,
            total_reverted: 0,
            best_score: 0.0,
        }
    }

    pub fn record_iteration(&mut self, iteration: LoopIteration) {
        if iteration.kept {
            self.total_kept += 1;
            if iteration.realized_gain.unwrap_or(0.0) > self.best_score {
                self.best_score = iteration.realized_gain.unwrap_or(0.0);
            }
        } else {
            self.total_reverted += 1;
        }
        self.last_updated = iteration.timestamp;
        self.iterations.push_back(iteration);
    }

    pub fn summary(&self) -> String {
        format!(
            "Loop[{}]: {} steps, kept={}, reverted={}, best={:.4}",
            self.loop_id,
            self.iterations.len(),
            self.total_kept,
            self.total_reverted,
            self.best_score
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStateStore {
    pub ledgers: Vec<RunLedger>,
    pub max_ledgers: usize,
}

impl LoopStateStore {
    pub fn new() -> Self {
        Self {
            ledgers: Vec::new(),
            max_ledgers: 100,
        }
    }

    pub fn get(&self, loop_id: &str) -> Option<&RunLedger> {
        self.ledgers.iter().find(|l| l.loop_id == loop_id)
    }

    pub fn get_mut(&mut self, loop_id: &str) -> Option<&mut RunLedger> {
        self.ledgers.iter_mut().find(|l| l.loop_id == loop_id)
    }

    pub fn add_or_update(&mut self, ledger: RunLedger) {
        if let Some(existing) = self
            .ledgers
            .iter_mut()
            .find(|l| l.loop_id == ledger.loop_id)
        {
            *existing = ledger;
        } else {
            if self.ledgers.len() >= self.max_ledgers {
                self.ledgers.remove(0);
            }
            self.ledgers.push(ledger);
        }
    }

    pub fn serialize(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut out = String::new();
        for ledger in &self.ledgers {
            writeln!(out, "{}", ledger.summary())?;
        }
        Ok(out)
    }
}
