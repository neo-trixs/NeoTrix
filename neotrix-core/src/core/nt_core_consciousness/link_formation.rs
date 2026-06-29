/// LinkFormation — CTM-AI inspired processor link formation
///
/// Inspired by CTM-AI (arXiv:2605.04097): processors that co-activate form
/// implicit links for unconscious communication, bypassing global workspace
/// broadcast. Links enable direct signal routing between subsystems that
/// frequently co-activate, reducing global broadcast contention.
use std::collections::{HashMap, VecDeque};

/// Strength of a link between two subsystems
#[derive(Debug, Clone, Copy)]
pub struct LinkStrength {
    pub co_activation_count: u64,
    pub ema: f64,
    pub last_cycle: u64,
    pub is_active: bool,
}

impl LinkStrength {
    fn new(cycle: u64) -> Self {
        Self {
            co_activation_count: 1,
            ema: 0.1,
            last_cycle: cycle,
            is_active: false,
        }
    }

    fn update(&mut self, cycle: u64, alpha: f64, threshold: f64) {
        self.co_activation_count += 1;
        self.ema = alpha * 1.0 + (1.0 - alpha) * self.ema;
        self.last_cycle = cycle;
        self.is_active = self.ema >= threshold;
    }

    fn decay(&mut self, cycle: u64, lambda: f64) {
        let gap = cycle.saturating_sub(self.last_cycle) as f64;
        if gap > 0.0 {
            self.ema *= lambda.powf(gap);
            self.is_active = false;
        }
    }
}

/// A link between two subsystems for direct signal routing
#[derive(Debug, Clone)]
pub struct SubsystemLink {
    pub from: String,
    pub to: String,
    pub strength: LinkStrength,
    pub signals_routed: u64,
}

impl SubsystemLink {
    fn new(from: String, to: String, cycle: u64) -> Self {
        Self {
            from,
            to,
            strength: LinkStrength::new(cycle),
            signals_routed: 0,
        }
    }
}

/// Configuration for the link formation mechanism
#[derive(Debug, Clone)]
pub struct LinkFormationConfig {
    pub max_links: usize,
    pub alpha: f64,
    pub activation_threshold: f64,
    pub decay_lambda: f64,
    pub allow_self_links: bool,
    pub max_history: usize,
}

impl Default for LinkFormationConfig {
    fn default() -> Self {
        Self {
            max_links: 200,
            alpha: 0.15,
            activation_threshold: 0.3,
            decay_lambda: 0.95,
            allow_self_links: false,
            max_history: 1000,
        }
    }
}

/// Tracks which subsystems were activated together in a cycle step
#[derive(Debug, Clone)]
pub struct ActivationRecord {
    pub cycle: u64,
    pub step: String,
    pub activated_subsystems: Vec<String>,
}

/// A signal routed through a direct link (bypassing global broadcast)
#[derive(Debug, Clone)]
pub struct RoutedSignal {
    pub from: String,
    pub to: String,
    pub signal_type: String,
    pub payload: f64,
    pub cycle: u64,
}

/// CTM-AI inspired processor link formation engine
///
/// Tracks co-activation patterns between subsystems and forms implicit
/// links for direct signal routing, bypassing the global workspace.
#[derive(Debug, Clone)]
pub struct LinkFormation {
    links: HashMap<(String, String), SubsystemLink>,
    history: VecDeque<ActivationRecord>,
    routed_signals: VecDeque<RoutedSignal>,
    config: LinkFormationConfig,
    cycle: u64,
}

impl LinkFormation {
    pub fn new(config: LinkFormationConfig) -> Self {
        Self {
            links: HashMap::new(),
            history: VecDeque::with_capacity(config.max_history),
            routed_signals: VecDeque::with_capacity(256),
            config,
            cycle: 0,
        }
    }

    /// Record that a set of subsystems were activated together.
    /// Drives co-activation link formation.
    pub fn record_activation(&mut self, step: &str, subsystems: &[String]) {
        self.cycle += 1;
        for link in self.links.values_mut() {
            link.strength.decay(self.cycle, self.config.decay_lambda);
        }
        for i in 0..subsystems.len() {
            for j in (i + 1)..subsystems.len() {
                let a = &subsystems[i];
                let b = &subsystems[j];
                if !self.config.allow_self_links && a == b {
                    continue;
                }
                let key = if a < b {
                    (a.clone(), b.clone())
                } else {
                    (b.clone(), a.clone())
                };
                if let Some(link) = self.links.get_mut(&key) {
                    link.strength.update(
                        self.cycle,
                        self.config.alpha,
                        self.config.activation_threshold,
                    );
                } else if self.links.len() < self.config.max_links {
                    let mut link = SubsystemLink::new(a.clone(), b.clone(), self.cycle);
                    link.strength.update(
                        self.cycle,
                        self.config.alpha,
                        self.config.activation_threshold,
                    );
                    self.links.insert(key, link);
                }
            }
        }
        let record = ActivationRecord {
            cycle: self.cycle,
            step: step.to_string(),
            activated_subsystems: subsystems.to_vec(),
        };
        if self.history.len() >= self.config.max_history {
            self.history.pop_front();
        }
        self.history.push_back(record);
        if self.links.len() > self.config.max_links {
            self.prune_inactive_links();
        }
    }

    /// Route a signal through a direct link if an active link exists.
    /// Returns true if routed through direct link.
    pub fn try_route(&mut self, from: &str, to: &str, signal_type: &str, payload: f64) -> bool {
        let key = if from < to {
            (from.to_string(), to.to_string())
        } else {
            (to.to_string(), from.to_string())
        };
        if let Some(link) = self.links.get_mut(&key) {
            if link.strength.is_active {
                link.signals_routed += 1;
                let routed = RoutedSignal {
                    from: from.to_string(),
                    to: to.to_string(),
                    signal_type: signal_type.to_string(),
                    payload,
                    cycle: self.cycle,
                };
                if self.routed_signals.len() >= 256 {
                    self.routed_signals.pop_front();
                }
                self.routed_signals.push_back(routed);
                return true;
            }
        }
        false
    }

    pub fn active_links(&self) -> Vec<&SubsystemLink> {
        self.links
            .values()
            .filter(|l| l.strength.is_active)
            .collect()
    }

    pub fn link_strength(&self, a: &str, b: &str) -> Option<f64> {
        let key = if a < b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        };
        self.links.get(&key).map(|l| l.strength.ema)
    }

    pub fn connected_subsystems(&self, name: &str) -> Vec<String> {
        let mut connected = Vec::new();
        for ((a, b), link) in &self.links {
            if link.strength.is_active {
                if a == name {
                    connected.push(b.clone());
                } else if b == name {
                    connected.push(a.clone());
                }
            }
        }
        connected
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    pub fn active_link_count(&self) -> usize {
        self.links.values().filter(|l| l.strength.is_active).count()
    }

    pub fn total_signals_routed(&self) -> u64 {
        self.links.values().map(|l| l.signals_routed).sum()
    }

    fn prune_inactive_links(&mut self) {
        self.links.retain(|_, link| link.strength.ema >= 0.01);
    }

    pub fn newly_active_links(
        &self,
        previous_active: &[(String, String)],
    ) -> Vec<(String, String)> {
        let current: std::collections::HashSet<_> = self
            .links
            .iter()
            .filter(|(_, l)| l.strength.is_active)
            .map(|((a, b), _)| (a.clone(), b.clone()))
            .collect();
        let prev: std::collections::HashSet<_> = previous_active.iter().cloned().collect();
        current.difference(&prev).cloned().collect()
    }

    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "total_links": self.links.len(),
            "active_links": self.active_link_count(),
            "signals_routed": self.total_signals_routed(),
            "history_size": self.history.len(),
            "cycle": self.cycle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_co_activation_forms_link() {
        let config = LinkFormationConfig {
            activation_threshold: 0.2,
            ..Default::default()
        };
        let mut lf = LinkFormation::new(config);
        lf.record_activation("GATHER", &vec!["a".into(), "b".into()]);
        assert!(lf.link_strength("a", "b").unwrap() > 0.0);
        assert_eq!(lf.link_count(), 1);
    }

    #[test]
    fn test_repeated_co_activation_activates_link() {
        let config = LinkFormationConfig {
            alpha: 0.5,
            activation_threshold: 0.3,
            ..Default::default()
        };
        let mut lf = LinkFormation::new(config);
        let subsystems = vec!["a".into(), "b".into()];
        for _ in 0..5 {
            lf.record_activation("GATHER", &subsystems);
        }
        assert!(lf.active_link_count() >= 1);
    }

    #[test]
    fn test_routing_through_active_link() {
        let config = LinkFormationConfig {
            alpha: 0.5,
            activation_threshold: 0.1,
            ..Default::default()
        };
        let mut lf = LinkFormation::new(config);
        lf.record_activation("REASON", &vec!["vision".into(), "language".into()]);
        assert!(lf.try_route("vision", "language", "semantic", 0.85));
    }

    #[test]
    fn test_no_routing_without_active_link() {
        let config = LinkFormationConfig {
            activation_threshold: 0.9,
            ..Default::default()
        };
        let mut lf = LinkFormation::new(config);
        lf.record_activation("GATHER", &vec!["a".into(), "b".into()]);
        assert!(!lf.try_route("a", "b", "test", 0.5));
    }

    #[test]
    fn test_decay_over_time() {
        let config = LinkFormationConfig {
            alpha: 0.5,
            activation_threshold: 0.01,
            decay_lambda: 0.5,
            ..Default::default()
        };
        let mut lf = LinkFormation::new(config);
        lf.record_activation("GATHER", &vec!["a".into(), "b".into()]);
        let s1 = lf.link_strength("a", "b").unwrap();
        lf.record_activation("GATHER", &vec!["a".into(), "b".into()]);
        let s2 = lf.link_strength("a", "b").unwrap();
        assert!(s2 > s1);
        for i in 0..5 {
            lf.record_activation("SLEEP", &[format!("x{}", i)]);
        }
        let s3 = lf.link_strength("a", "b").unwrap();
        assert!(s3 < s2);
    }

    #[test]
    fn test_connected_subsystems() {
        let config = LinkFormationConfig {
            alpha: 0.5,
            activation_threshold: 0.1,
            ..Default::default()
        };
        let mut lf = LinkFormation::new(config);
        lf.record_activation(
            "GATHER",
            &vec!["hub".into(), "vision".into(), "language".into()],
        );
        lf.record_activation("GATHER", &vec!["hub".into(), "vision".into()]);
        let c = lf.connected_subsystems("vision");
        assert!(c.contains(&"hub".to_string()));
    }

    #[test]
    fn test_max_links_pruning() {
        let max_links = 5;
        let config = LinkFormationConfig {
            max_links,
            activation_threshold: 0.01,
            ..Default::default()
        };
        let mut lf = LinkFormation::new(config);
        for i in 0..10 {
            for j in (i + 1)..10 {
                lf.record_activation("GATHER", &vec![format!("s{}", i), format!("s{}", j)]);
            }
        }
        assert!(lf.link_count() <= max_links + 5);
    }

    #[test]
    fn test_metrics() {
        let mut lf = LinkFormation::new(LinkFormationConfig::default());
        lf.record_activation("GATHER", &vec!["a".into(), "b".into()]);
        let m = lf.metrics();
        assert_eq!(m["total_links"], 1);
    }
}
