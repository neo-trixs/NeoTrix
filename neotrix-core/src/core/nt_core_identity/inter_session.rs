use std::collections::HashMap;

use super::identity_core::IdentityCore;

#[derive(Debug, Clone)]
pub struct ReflectionReport {
    pub vsa_drift: f64,
    pub personality_deltas: HashMap<String, f64>,
    pub values_changed: Vec<String>,
    pub drift_speed: f64,
    pub is_stable: bool,
    pub recommendation: &'static str,
    pub evolution_applied: bool,
    pub new_version: Option<u64>,
    pub evolution_summary: Option<String>,
}

impl ReflectionReport {
    pub fn summary(&self) -> String {
        let evo = if self.evolution_applied {
            format!("_evolved_v{}", self.new_version.unwrap_or(0))
        } else {
            String::new()
        };
        format!(
            "reflection:vsa_drift_{:.4}_values_changed_{}_stable_{}_recommend_{}{}",
            self.vsa_drift,
            self.values_changed.len(),
            self.is_stable,
            self.recommendation,
            evo,
        )
    }
}

#[derive(Debug, Clone)]
pub struct InterSessionReflector {
    pub last_session_id: Option<String>,
    pub last_self_vsa: Option<Vec<u8>>,
    pub last_personality_count: Option<usize>,
    pub last_core_values: Vec<String>,
    pub session_count: u64,
    pub drift_history: Vec<f64>,
}

impl InterSessionReflector {
    pub fn new() -> Self {
        Self {
            last_session_id: None,
            last_self_vsa: None,
            last_personality_count: None,
            last_core_values: Vec::new(),
            session_count: 0,
            drift_history: Vec::with_capacity(64),
        }
    }

    pub fn init_session(&mut self, identity: &IdentityCore) {
        self.last_self_vsa = Some(identity.self_vsa.clone());
        self.last_personality_count = Some(identity.personality_traits.len());
        self.last_core_values = identity.core_values.clone();
        self.session_count += 1;
        self.last_session_id = Some(format!(
            "session_{}_{}",
            self.session_count,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        ));
    }

    pub fn end_session(
        &mut self,
        identity: &mut IdentityCore,
        session_success_rate: Option<f64>,
    ) -> ReflectionReport {
        let vsa_drift = match &self.last_self_vsa {
            Some(last) if last.len() == identity.self_vsa.len() && !last.is_empty() => {
                let same = last
                    .iter()
                    .zip(identity.self_vsa.iter())
                    .filter(|(a, b)| a == b)
                    .count();
                1.0 - (same as f64 / last.len() as f64)
            }
            _ => 0.0,
        };

        let mut personality_deltas = HashMap::new();
        personality_deltas.insert(
            "pre_count".to_string(),
            self.last_personality_count.unwrap_or(0) as f64,
        );
        personality_deltas.insert(
            "post_count".to_string(),
            identity.personality_traits.len() as f64,
        );
        let count_delta = (identity.personality_traits.len() as f64)
            - (self.last_personality_count.unwrap_or(0) as f64);
        personality_deltas.insert("count_delta".to_string(), count_delta);

        let values_changed: Vec<String> = identity
            .core_values
            .iter()
            .filter(|v| !self.last_core_values.contains(v))
            .cloned()
            .chain(
                self.last_core_values
                    .iter()
                    .filter(|v| !identity.core_values.contains(v))
                    .cloned(),
            )
            .collect();

        self.drift_history.push(vsa_drift);
        if self.drift_history.len() > 64 {
            self.drift_history.remove(0);
        }

        let recent_drifts: Vec<f64> = self.drift_history.iter().rev().take(10).copied().collect();
        let drift_speed = if recent_drifts.len() >= 2 {
            let sum_deltas: f64 = recent_drifts.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
            sum_deltas / (recent_drifts.len() - 1) as f64
        } else {
            vsa_drift
        };

        let is_stable = vsa_drift < 0.05 && drift_speed < 0.01;
        let recommendation = if !is_stable {
            "identity drift detected — consider anchor fusion or coherence boost"
        } else if !values_changed.is_empty() {
            "values evolved — log in experience tree"
        } else {
            "identity stable — no action needed"
        };

        let mut report = ReflectionReport {
            vsa_drift,
            personality_deltas,
            values_changed,
            drift_speed,
            is_stable,
            recommendation,
            evolution_applied: false,
            new_version: None,
            evolution_summary: None,
        };

        if let Some(success_rate) = session_success_rate {
            if identity.evolution_enabled {
                identity.evolve(success_rate);
                report.evolution_applied = true;
                report.new_version = identity
                    .evolution
                    .as_ref()
                    .and_then(|e| e.latest_version().map(|v| v.version));
                report.evolution_summary = Some(identity.evolution_report());
            }
        }

        report
    }
}

impl Default for InterSessionReflector {
    fn default() -> Self {
        Self::new()
    }
}
