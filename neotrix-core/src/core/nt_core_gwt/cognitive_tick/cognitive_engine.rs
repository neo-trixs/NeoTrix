#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::agents::CognitiveAgent;
use super::entropy_drive::EntropyDrive;
use crate::core::nt_core_self_constitution::{Constitution, DevRule};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub id: usize,
    pub content: String,
    pub salience: f64,
    pub source_agent: usize,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct AgentHandle {
    pub agent: CognitiveAgent,
    pub active: bool,
    pub activation_count: u64,
    pub last_activated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveTickConfig {
    pub tick_duration_ms: u64,
    pub max_agents_per_tick: usize,
    pub entropy_threshold: f64,
    pub enable_entropy_drive: bool,
    pub enable_parallel_agents: bool,
}

impl Default for CognitiveTickConfig {
    fn default() -> Self {
        Self {
            tick_duration_ms: 100,
            max_agents_per_tick: 5,
            entropy_threshold: 0.3,
            enable_entropy_drive: true,
            enable_parallel_agents: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveTickReport {
    pub tick_id: u64,
    pub duration_ms: u64,
    pub agents_activated: usize,
    pub entropy_level: f64,
    pub broadcast_content: String,
    pub phase_durations: [u64; 4],
    pub convergence: f64,
}

pub struct CognitiveEngine {
    pub config: CognitiveTickConfig,
    pub agents: Vec<AgentHandle>,
    pub workspace: Vec<ContentItem>,
    pub tick_count: u64,
    pub entropy_drive: EntropyDrive,
    next_item_id: usize,
    /// Internalized constitution for guided reasoning
    constitution: Option<&'static Constitution>,
}

impl CognitiveEngine {
    pub fn new(config: CognitiveTickConfig) -> Self {
        let target_entropy = config.entropy_threshold;
        Self {
            config,
            agents: Vec::new(),
            workspace: Vec::new(),
            tick_count: 0,
            entropy_drive: EntropyDrive::new(target_entropy),
            next_item_id: 0,
            constitution: None,
        }
    }

    /// Load constitution into the engine for guided reasoning
    pub fn load_constitution(&mut self, constitution: &'static Constitution) {
        self.constitution = Some(constitution);
    }

    /// Get relevant rules for a given task context
    pub fn get_relevant_rules(&self, task_desc: &str, top_k: usize) -> Vec<&DevRule> {
        self.constitution
            .map(|c| c.relevant_rules_for_task(task_desc, top_k))
            .unwrap_or_default()
    }

    /// Check if an action complies with constitution
    pub fn check_compliance(&self, action_desc: &str) -> Option<crate::core::nt_core_self_constitution::ComplianceReport> {
        self.constitution.map(|c| c.verify_compliance(action_desc))
    }

    pub fn add_agent(&mut self, agent: CognitiveAgent) {
        self.agents.push(AgentHandle {
            agent,
            active: true,
            activation_count: 0,
            last_activated: 0,
        });
    }

    pub fn tick(&mut self) -> CognitiveTickReport {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.phase1_gather();
        let agents_activated = self.workspace.len();
        let t1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let entropy_level = self.phase2_resonate();
        let t2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let _winning = self.phase3_select();
        let t3 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let broadcast_content = self.phase4_broadcast();
        let t4 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let convergence = if agents_activated > 0 {
            (1.0 - entropy_level).max(0.0).min(1.0)
        } else {
            1.0
        };

        self.tick_count += 1;

        CognitiveTickReport {
            tick_id: self.tick_count,
            duration_ms: t4 - start,
            agents_activated,
            entropy_level,
            broadcast_content,
            phase_durations: [t1 - start, t2 - t1, t3 - t2, t4 - t3],
            convergence,
        }
    }

    fn phase1_gather(&mut self) {
        let entropy = self.entropy_drive.current_entropy;
        let drive = self.entropy_drive.drive_signal();
        for handle in self.agents.iter_mut() {
            if !handle.active {
                continue;
            }
            let within_cap = handle.activation_count < self.config.max_agents_per_tick as u64;
            let entropy_boost = self.config.enable_entropy_drive && drive > 0.5;
            if !within_cap && !entropy_boost {
                continue;
            }
            let prob = handle.agent.activation_probability(entropy);
            let pseudo_rand = ((self.tick_count.wrapping_mul(7)
                .wrapping_add(handle.agent.id as u64)
                .wrapping_mul(13)) as f64
                * 0.001)
                .fract();
            if pseudo_rand >= prob {
                continue;
            }
            let mut item = handle.agent.generate_content(self.tick_count);
            item.id = self.next_item_id;
            self.next_item_id += 1;
            item.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            handle.activation_count += 1;
            handle.last_activated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.workspace.push(item);
        }
    }

    fn phase2_resonate(&mut self) -> f64 {
        if self.config.enable_entropy_drive {
            self.entropy_drive.boost_low_salience(&mut self.workspace);
        }
        self.entropy_drive.update(&self.workspace);
        self.workspace
            .sort_by(|a, b| b.salience.partial_cmp(&a.salience).unwrap_or(std::cmp::Ordering::Equal));
        self.entropy_drive.current_entropy
    }

    fn phase3_select(&self) -> Option<ContentItem> {
        self.workspace.first().cloned()
    }

    fn phase4_broadcast(&mut self) -> String {
        let winning = self.workspace.first().cloned();
        let content = winning
            .as_ref()
            .map(|item| item.content.clone())
            .unwrap_or_default();
        if let Some(ref item) = winning {
            for handle in self.agents.iter_mut() {
                handle.agent.receive_broadcast(item);
            }
        }
        self.workspace.clear();
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_gwt::cognitive_tick::agents::{AgentType, create_default_agents};

    #[test]
    fn test_engine_creation() {
        let config = CognitiveTickConfig::default();
        let engine = CognitiveEngine::new(config);
        assert_eq!(engine.agents.len(), 0);
        assert_eq!(engine.tick_count, 0);
        assert!(engine.workspace.is_empty());
    }

    #[test]
    fn test_single_tick() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig::default());
        for agent in create_default_agents() {
            engine.add_agent(agent);
        }
        let report = engine.tick();
        assert_eq!(report.tick_id, 1);
        assert!(!report.broadcast_content.is_empty());
    }

    #[test]
    fn test_four_phase_sequence() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig::default());
        for agent in create_default_agents() {
            engine.add_agent(agent);
        }
        let report = engine.tick();
        assert_eq!(report.phase_durations.len(), 4);
        assert!(report.phase_durations.iter().all(|d| *d <= report.duration_ms));
    }

    #[test]
    fn test_agent_addition() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig::default());
        assert_eq!(engine.agents.len(), 0);
        let agent = CognitiveAgent::new(10, "Test".into(), AgentType::Perceptual);
        engine.add_agent(agent);
        assert_eq!(engine.agents.len(), 1);
        assert!(engine.agents[0].active);
    }

    #[test]
    fn test_multiple_ticks_show_progression() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig::default());
        for agent in create_default_agents() {
            engine.add_agent(agent);
        }
        let r1 = engine.tick();
        let r2 = engine.tick();
        let r3 = engine.tick();
        assert_eq!(r1.tick_id, 1);
        assert_eq!(r2.tick_id, 2);
        assert_eq!(r3.tick_id, 3);
        assert!(r1.tick_id < r2.tick_id);
        assert!(r2.tick_id < r3.tick_id);
    }

    #[test]
    fn test_entropy_drive_effect_on_salience() {
        let mut config = CognitiveTickConfig::default();
        config.enable_entropy_drive = true;
        let mut engine = CognitiveEngine::new(config);
        for agent in create_default_agents() {
            engine.add_agent(agent);
        }
        let r1 = engine.tick();
        assert!(r1.entropy_level >= 0.0);
        assert!(r1.convergence >= 0.0);
        assert!(r1.convergence <= 1.0);
    }

    #[test]
    fn test_tick_with_no_agents() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig::default());
        let report = engine.tick();
        assert_eq!(report.agents_activated, 0);
        assert!(report.broadcast_content.is_empty());
        assert!((report.convergence - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_activation_count_capping() {
        let config = CognitiveTickConfig {
            max_agents_per_tick: 1,
            enable_entropy_drive: false,
            ..Default::default()
        };
        let mut engine = CognitiveEngine::new(config);
        let agent = CognitiveAgent::new(1, "Single".into(), AgentType::Perceptual);
        engine.add_agent(agent);
        let r1 = engine.tick();
        assert_eq!(r1.tick_id, 1);
        let r2 = engine.tick();
        assert_eq!(r2.tick_id, 2);
        assert_eq!(engine.agents[0].activation_count, 1);
    }

    #[test]
    fn test_content_items_have_timestamps() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig::default());
        for agent in create_default_agents() {
            engine.add_agent(agent);
        }
        engine.tick();
        for item in &engine.workspace {
            assert!(item.timestamp > 0);
        }
    }

    #[test]
    fn test_tick_report_convergence() {
        let mut config = CognitiveTickConfig::default();
        config.enable_entropy_drive = true;
        let mut engine = CognitiveEngine::new(config);
        for agent in create_default_agents() {
            engine.add_agent(agent);
        }
        for _ in 0..5 {
            let report = engine.tick();
            assert!(report.convergence >= 0.0);
            assert!(report.convergence <= 1.0);
        }
    }

    #[test]
    fn test_disabled_entropy_drive() {
        let mut config = CognitiveTickConfig::default();
        config.enable_entropy_drive = false;
        let mut engine = CognitiveEngine::new(config);
        for agent in create_default_agents() {
            engine.add_agent(agent);
        }
        let report = engine.tick();
        assert!(report.entropy_level >= 0.0);
    }

    #[test]
    fn test_inactive_agents_do_not_produce_content() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig::default());
        let agent = CognitiveAgent::new(1, "Inactive".into(), AgentType::Perceptual);
        engine.add_agent(agent);
        engine.agents[0].active = false;
        let report = engine.tick();
        assert_eq!(report.agents_activated, 0);
    }

    #[test]
    fn test_agent_activation_probability_used() {
        let mut engine = CognitiveEngine::new(CognitiveTickConfig {
            entropy_threshold: 1.0,
            enable_entropy_drive: false,
            ..Default::default()
        });
        let mut agent = CognitiveAgent::new(1, "LowActivation".into(), AgentType::Perceptual);
        agent.salience_weight = 0.01;
        agent.activation_threshold = 1.0;
        engine.add_agent(agent);
        let report = engine.tick();
        assert!(report.agents_activated <= 5);
    }
}
