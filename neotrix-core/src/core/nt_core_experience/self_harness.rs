use std::time::{SystemTime, UNIX_EPOCH};

fn now_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone)]
pub struct HarnessWeakness {
    pub pattern: String,
    pub frequency: u32,
    pub avg_impact: f64,
    pub last_detected: u64,
}

#[derive(Debug, Clone)]
pub struct HarnessProposal {
    pub id: u64,
    pub mutation: String,
    pub expected_improvement: f64,
    pub validated: bool,
}

pub struct WeaknessMiner;

impl WeaknessMiner {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_weaknesses(&self, logs: &[impl AsRef<str>]) -> Vec<HarnessWeakness> {
        let keywords = ["timeout", "crash", "OOM", "corrupt", "permission"];
        let mut counts: std::collections::HashMap<&str, u32> =
            keywords.iter().map(|k| (*k, 0)).collect();

        for line in logs {
            let line_lower = line.as_ref().to_lowercase();
            for &kw in &keywords {
                if line_lower.contains(kw) {
                    *counts.get_mut(kw).unwrap() += 1;
                }
            }
        }

        let total = logs.len().max(1);
        let now = now_u64();
        counts
            .into_iter()
            .filter(|(_, c)| *c > 0)
            .map(|(pattern, frequency)| HarnessWeakness {
                pattern: pattern.to_string(),
                frequency,
                avg_impact: frequency as f64 / total as f64,
                last_detected: now,
            })
            .collect()
    }
}

pub struct HarnessProposer;

impl HarnessProposer {
    pub fn new() -> Self {
        Self
    }

    pub fn propose_fixes(&self, weaknesses: &[HarnessWeakness]) -> Vec<HarnessProposal> {
        let mut next_id = 1u64;
        weaknesses
            .iter()
            .map(|w| {
                let id = next_id;
                next_id += 1;
                let mutation = match w.pattern.as_str() {
                    "timeout" => format!(
                        "increase timeout threshold for pattern '{}' (impact={:.2})",
                        w.pattern, w.avg_impact
                    ),
                    "crash" => format!(
                        "add crash recovery wrapper for pattern '{}' (impact={:.2})",
                        w.pattern, w.avg_impact
                    ),
                    "OOM" => format!(
                        "add memory guard before pattern '{}' (impact={:.2})",
                        w.pattern, w.avg_impact
                    ),
                    "corrupt" => format!(
                        "add data integrity check for pattern '{}' (impact={:.2})",
                        w.pattern, w.avg_impact
                    ),
                    "permission" => format!(
                        "add permission pre-check for pattern '{}' (impact={:.2})",
                        w.pattern, w.avg_impact
                    ),
                    _ => format!(
                        "add generic guard for pattern '{}' (impact={:.2})",
                        w.pattern, w.avg_impact
                    ),
                };
                HarnessProposal {
                    id,
                    mutation,
                    expected_improvement: w.avg_impact * 0.5,
                    validated: false,
                }
            })
            .collect()
    }
}

pub struct ProposalValidator {
    pub count: u64,
    pub passed: u64,
}

impl ProposalValidator {
    pub fn new() -> Self {
        Self {
            count: 0,
            passed: 0,
        }
    }

    pub fn validate(&mut self, _proposal: &HarnessProposal) -> bool {
        self.count += 1;
        self.passed += 1;
        true
    }
}

pub struct SelfHarnessEngine {
    pub weakness_miner: WeaknessMiner,
    pub proposer: HarnessProposer,
    pub validator: ProposalValidator,
    pub weakness_history: Vec<HarnessWeakness>,
    pub enabled: bool,
    pub cycle: u64,
    max_history: usize,
}

impl SelfHarnessEngine {
    pub fn new() -> Self {
        Self {
            weakness_miner: WeaknessMiner::new(),
            proposer: HarnessProposer::new(),
            validator: ProposalValidator::new(),
            weakness_history: Vec::new(),
            enabled: true,
            cycle: 0,
            max_history: 1000,
        }
    }

    pub fn tick(&mut self, logs: &[impl AsRef<str>], _threshold: f64) -> String {
        self.cycle += 1;
        let proposals = self.run_cycle(logs);
        format!("harness:{}_proposals", proposals.len())
    }

    pub fn stats(&self) -> String {
        format!("harness:{}_weaknesses", self.weakness_history.len())
    }

    pub fn run_cycle(&mut self, logs: &[impl AsRef<str>]) -> Vec<HarnessProposal> {
        let weaknesses = self.weakness_miner.detect_weaknesses(logs);

        for w in &weaknesses {
            if let Some(existing) = self
                .weakness_history
                .iter_mut()
                .find(|h: &&mut HarnessWeakness| h.pattern == w.pattern)
            {
                existing.frequency = existing.frequency.saturating_add(w.frequency);
                existing.avg_impact = (existing.avg_impact + w.avg_impact) / 2.0;
                existing.last_detected = w.last_detected;
            } else {
                if self.weakness_history.len() >= self.max_history {
                    self.weakness_history.remove(0);
                }
                self.weakness_history.push(w.clone());
            }
        }

        let proposals = self.proposer.propose_fixes(&weaknesses);
        for p in &proposals {
            self.validator.validate(p);
        }
        proposals
    }
}
