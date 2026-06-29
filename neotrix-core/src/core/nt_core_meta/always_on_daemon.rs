use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DaemonInput {
    pub epistemic_confidence: f64,
    pub cognitive_load: f64,
    pub curiosity_signal: f64,
    pub conflict_level: f64,
    pub prediction_error: f64,
    pub recent_consolidation_quality: f64,
    pub time_since_last_input_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DaemonActivity {
    Idle,
    Consolidating,
    Exploring,
    Reflecting,
    Sleeping,
}

pub enum DaemonAction {
    Consolidate,
    Explore { topic_hint: String },
    Reflect { focus: String },
    Sleep { duration_ticks: u64 },
    Log { message: String },
    None,
}

pub struct DaemonConfig {
    pub consolidation_interval: u64,
    pub exploration_curiosity_threshold: f64,
    pub reflection_interval: u64,
    pub max_activity_log: usize,
    pub idle_before_sleep: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            consolidation_interval: 10,
            exploration_curiosity_threshold: 0.5,
            reflection_interval: 20,
            max_activity_log: 100,
            idle_before_sleep: 100,
        }
    }
}

pub struct AlwaysOnDaemon {
    pub activity: DaemonActivity,
    pub tick_count: u64,
    pub idle_ticks: u64,
    pub consolidation_interval: u64,
    pub exploration_curiosity_threshold: f64,
    pub reflection_interval: u64,
    pub last_consolidation_tick: u64,
    pub last_exploration_tick: u64,
    pub last_reflection_tick: u64,
    pub activity_log: VecDeque<(u64, DaemonActivity, String)>,
    pub max_log: usize,
}

impl AlwaysOnDaemon {
    pub fn new() -> Self {
        Self::with_config(&DaemonConfig::default())
    }

    pub fn with_config(config: &DaemonConfig) -> Self {
        Self {
            activity: DaemonActivity::Idle,
            tick_count: 0,
            idle_ticks: 0,
            consolidation_interval: config.consolidation_interval,
            exploration_curiosity_threshold: config.exploration_curiosity_threshold,
            reflection_interval: config.reflection_interval,
            last_consolidation_tick: 0,
            last_exploration_tick: 0,
            last_reflection_tick: 0,
            activity_log: VecDeque::with_capacity(config.max_activity_log),
            max_log: config.max_activity_log,
        }
    }

    pub fn tick(&mut self, input: &DaemonInput) -> Vec<DaemonAction> {
        self.tick_count += 1;
        let mut actions = Vec::new();

        if self.should_consolidate(input) {
            self.activity = DaemonActivity::Consolidating;
            self.last_consolidation_tick = self.tick_count;
            self.idle_ticks = 0;
            actions.push(DaemonAction::Consolidate);
            self.log_activity(
                DaemonActivity::Consolidating,
                "consolidation triggered".into(),
            );
        } else if self.should_explore(input) {
            self.activity = DaemonActivity::Exploring;
            self.last_exploration_tick = self.tick_count;
            self.idle_ticks = 0;
            let topic = if input.prediction_error > 0.6 {
                "high prediction error sources"
            } else if input.epistemic_confidence < 0.3 {
                "low confidence domains"
            } else {
                "novel knowledge areas"
            };
            actions.push(DaemonAction::Explore {
                topic_hint: topic.into(),
            });
            self.log_activity(DaemonActivity::Exploring, format!("exploring: {}", topic));
        } else if self.should_reflect(input) {
            self.activity = DaemonActivity::Reflecting;
            self.last_reflection_tick = self.tick_count;
            self.idle_ticks = 0;
            let focus = if input.conflict_level > 0.7 {
                "high conflict resolution"
            } else {
                "recent conflict patterns"
            };
            actions.push(DaemonAction::Reflect {
                focus: focus.into(),
            });
            self.log_activity(
                DaemonActivity::Reflecting,
                format!("reflecting on: {}", focus),
            );
        } else {
            self.idle_ticks += 1;
            if self.idle_ticks > 0 && self.idle_ticks % 10 == 0 {
                actions.push(DaemonAction::Log {
                    message: format!("idle for {} ticks", self.idle_ticks),
                });
            }
            if input.time_since_last_input_ms > 60_000 && self.activity != DaemonActivity::Sleeping
            {
                self.activity = DaemonActivity::Sleeping;
                actions.push(DaemonAction::Sleep { duration_ticks: 5 });
                self.log_activity(DaemonActivity::Sleeping, "extended idle: sleeping".into());
            } else {
                self.activity = DaemonActivity::Idle;
            }
        }

        actions.push(DaemonAction::None);
        actions
    }

    pub fn should_consolidate(&self, input: &DaemonInput) -> bool {
        let ticks_since_consolidation = self.tick_count - self.last_consolidation_tick;
        let threshold_met = input.recent_consolidation_quality < 0.3 || input.cognitive_load > 0.8;
        ticks_since_consolidation >= self.consolidation_interval || threshold_met
    }

    pub fn should_explore(&self, input: &DaemonInput) -> bool {
        let ticks_since_exploration = self.tick_count - self.last_exploration_tick;
        let curiosity_high = input.curiosity_signal >= self.exploration_curiosity_threshold;
        let safe_to_explore = input.cognitive_load < 0.8;
        curiosity_high && safe_to_explore && ticks_since_exploration >= 3
    }

    pub fn should_reflect(&self, input: &DaemonInput) -> bool {
        let ticks_since_reflection = self.tick_count - self.last_reflection_tick;
        let conflict_high = input.conflict_level > 0.3;
        let long_idle = input.time_since_last_input_ms > 30_000;
        conflict_high && long_idle && ticks_since_reflection >= self.reflection_interval
    }

    pub fn daemon_status(&self) -> String {
        format!(
            "AlwaysOnDaemon[tick={}] activity={:?} idle_ticks={} log={} last_consolidation={} last_exploration={} last_reflection={}",
            self.tick_count,
            self.activity,
            self.idle_ticks,
            self.activity_log.len(),
            self.last_consolidation_tick,
            self.last_exploration_tick,
            self.last_reflection_tick,
        )
    }

    pub fn activity_summary(&self, n: usize) -> Vec<String> {
        self.activity_log
            .iter()
            .rev()
            .take(n)
            .map(|(tick, activity, desc)| format!("[tick={}] {:?}: {}", tick, activity, desc))
            .collect()
    }

    pub fn log_activity(&mut self, activity: DaemonActivity, description: String) {
        if self.activity_log.len() >= self.max_log {
            self.activity_log.pop_front();
        }
        self.activity_log
            .push_back((self.tick_count, activity, description));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_input() -> DaemonInput {
        DaemonInput {
            epistemic_confidence: 0.5,
            cognitive_load: 0.3,
            curiosity_signal: 0.1,
            conflict_level: 0.1,
            prediction_error: 0.2,
            recent_consolidation_quality: 0.8,
            time_since_last_input_ms: 5_000,
        }
    }

    #[test]
    fn test_new_daemon_starts_idle() {
        let daemon = AlwaysOnDaemon::new();
        assert_eq!(daemon.activity, DaemonActivity::Idle);
        assert_eq!(daemon.tick_count, 0);
        assert_eq!(daemon.idle_ticks, 0);
    }

    #[test]
    fn test_tick_idle_no_action() {
        let mut daemon = AlwaysOnDaemon::new();
        let actions = daemon.tick(&default_input());
        assert!(actions.iter().any(|a| matches!(a, DaemonAction::None)));
        assert_eq!(daemon.tick_count, 1);
    }

    #[test]
    fn test_tick_triggers_consolidation() {
        let mut daemon = AlwaysOnDaemon::new();
        daemon.consolidation_interval = 1;
        let input = DaemonInput {
            recent_consolidation_quality: 0.9,
            cognitive_load: 0.2,
            ..default_input()
        };
        let actions = daemon.tick(&input);
        assert!(actions
            .iter()
            .any(|a| matches!(a, DaemonAction::Consolidate)));
        assert_eq!(daemon.activity, DaemonActivity::Consolidating);
    }

    #[test]
    fn test_high_curiosity_triggers_exploration() {
        let mut daemon = AlwaysOnDaemon::new();
        daemon.last_consolidation_tick = daemon.tick_count;
        let input = DaemonInput {
            curiosity_signal: 0.9,
            cognitive_load: 0.2,
            recent_consolidation_quality: 0.8,
            ..default_input()
        };
        let actions = daemon.tick(&input);
        assert!(actions
            .iter()
            .any(|a| matches!(a, DaemonAction::Explore { .. })));
        assert_eq!(daemon.activity, DaemonActivity::Exploring);
    }

    #[test]
    fn test_reflection_triggers_on_conflict() {
        let mut daemon = AlwaysOnDaemon::new();
        daemon.last_consolidation_tick = daemon.tick_count;
        daemon.reflection_interval = 0;
        let input = DaemonInput {
            conflict_level: 0.6,
            curiosity_signal: 0.1,
            time_since_last_input_ms: 60_000,
            recent_consolidation_quality: 0.8,
            ..default_input()
        };
        let actions = daemon.tick(&input);
        assert!(actions
            .iter()
            .any(|a| matches!(a, DaemonAction::Reflect { .. })));
        assert_eq!(daemon.activity, DaemonActivity::Reflecting);
    }

    #[test]
    fn test_activity_log_capped() {
        let mut daemon = AlwaysOnDaemon::new();
        daemon.max_log = 2;
        daemon.log_activity(DaemonActivity::Idle, "first".into());
        daemon.log_activity(DaemonActivity::Idle, "second".into());
        daemon.log_activity(DaemonActivity::Idle, "third".into());
        assert_eq!(daemon.activity_log.len(), 2);
        assert_eq!(daemon.activity_log[0].2, "second");
        assert_eq!(daemon.activity_log[1].2, "third");
    }

    #[test]
    fn test_daemon_status_format() {
        let daemon = AlwaysOnDaemon::new();
        let status = daemon.daemon_status();
        assert!(status.starts_with("AlwaysOnDaemon"));
        assert!(status.contains("tick=0"));
        assert!(status.contains("Idle"));
    }

    #[test]
    fn test_config_custom_values() {
        let config = DaemonConfig {
            consolidation_interval: 20,
            exploration_curiosity_threshold: 0.7,
            reflection_interval: 40,
            max_activity_log: 50,
            idle_before_sleep: 200,
        };
        let daemon = AlwaysOnDaemon::with_config(&config);
        assert_eq!(daemon.consolidation_interval, 20);
        assert_eq!(daemon.exploration_curiosity_threshold, 0.7);
        assert_eq!(daemon.reflection_interval, 40);
        assert_eq!(daemon.max_log, 50);
    }

    #[test]
    fn test_activity_summary_ordering() {
        let mut daemon = AlwaysOnDaemon::new();
        daemon.log_activity(DaemonActivity::Idle, "first".into());
        daemon.log_activity(DaemonActivity::Consolidating, "second".into());
        let summary = daemon.activity_summary(2);
        assert_eq!(summary.len(), 2);
        assert!(summary[0].contains("Consolidating"));
        assert!(summary[1].contains("Idle"));
    }

    #[test]
    fn test_consolidation_triggers_on_low_quality() {
        let mut daemon = AlwaysOnDaemon::new();
        daemon.last_consolidation_tick = daemon.tick_count;
        let input = DaemonInput {
            recent_consolidation_quality: 0.1,
            cognitive_load: 0.2,
            ..default_input()
        };
        assert!(daemon.should_consolidate(&input));
    }

    #[test]
    fn test_exploration_blocked_by_high_load() {
        let mut daemon = AlwaysOnDaemon::new();
        let input = DaemonInput {
            curiosity_signal: 0.9,
            cognitive_load: 0.9,
            ..default_input()
        };
        assert!(!daemon.should_explore(&input));
    }
}
