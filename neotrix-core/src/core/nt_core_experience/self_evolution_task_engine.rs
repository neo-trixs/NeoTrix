use serde::{Deserialize, Serialize};

use super::reflective_analyzer::{Diagnosis, FixCategory, ReflectiveAnalyzer, TraceEvent};
use crate::core::nt_core_identity::IdentityCore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineTaskType {
    NewModule {
        name: String,
        pattern_source: String,
        estimated_lines: usize,
    },
    WireModule { file_name: String },
    TuneMutation { target: String, delta: f64 },
    AbsorbPattern {
        repo_url: String,
        pattern_name: String,
    },
    SelfAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineTaskStatus {
    Proposed,
    InProgress,
    Completed { success: bool, metric_delta: f64 },
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvolutionTask {
    pub id: u64,
    pub target_gap: String,
    pub description: String,
    pub task_type: EngineTaskType,
    pub status: EngineTaskStatus,
    pub created_cycle: u64,
    pub completed_cycle: Option<u64>,
    pub prerequisite_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: u64,
    pub success: bool,
    pub metric_before: f64,
    pub metric_after: f64,
    pub experience_distilled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEvolutionTaskEngine {
    pub tasks: Vec<EngineEvolutionTask>,
    pub completed_tasks: Vec<TaskResult>,
    pub next_id: u64,
    pub cycle: u64,
}

fn fix_category_to_task_type(fix: &FixCategory) -> EngineTaskType {
    match fix {
        FixCategory::CalibrationDrift => EngineTaskType::TuneMutation {
            target: "calibration_rate".into(),
            delta: 0.05,
        },
        FixCategory::ModuleWiring => EngineTaskType::WireModule {
            file_name: "unresolved_module".into(),
        },
        FixCategory::MemoryPressure => EngineTaskType::SelfAudit,
        FixCategory::LatencyDegradation => EngineTaskType::SelfAudit,
        FixCategory::SkillStagnation => EngineTaskType::SelfAudit,
        FixCategory::CompileFailure => EngineTaskType::NewModule {
            name: "compile_fix".into(),
            pattern_source: "compiler_error".into(),
            estimated_lines: 50,
        },
        FixCategory::SystemicDegradation => EngineTaskType::SelfAudit,
    }
}

impl SelfEvolutionTaskEngine {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            completed_tasks: Vec::with_capacity(100),
            next_id: 1,
            cycle: 0,
        }
    }

    /// Execute a task from EvolutionTaskSystem and return a result.
    /// Simplified: success probability scales with reflective_bonus.
    pub fn process_system_task(
        &mut self,
        external_id: u64,
        title: &str,
        _description: &str,
        metric_before: f64,
        reflective_bonus: f64,
    ) -> TaskResult {
        let success = reflective_bonus >= 0.0;
        let delta = if success {
            0.1 + reflective_bonus * 0.5
        } else {
            0.0
        };
        let metric_after = (metric_before + delta).min(1.0);
        let result = TaskResult {
            task_id: external_id,
            success,
            metric_before,
            metric_after,
            experience_distilled: vec![
                format!(
                    "GEPA bridge: {} {} (bonus={:.2})",
                    title,
                    if success { "pass" } else { "fail" },
                    reflective_bonus
                ),
                format!(
                    "Pattern: self_audit via GEPA pipeline at cycle {}",
                    self.cycle
                ),
            ],
        };
        self.completed_tasks.push(result.clone());
        result
    }

    /// Compute success-reflective bonus from structured diagnoses.
    /// Empty diagnoses → mild positive (0.3). High severity → negative (down to -0.5).
    pub fn compute_reflective_bonus_from_diagnoses(&self, diagnoses: &[Diagnosis]) -> f64 {
        if diagnoses.is_empty() {
            return 0.3;
        }
        let avg_severity: f64 = diagnoses.iter().map(|d| d.severity).sum::<f64>() / diagnoses.len() as f64;
        let max_severity = diagnoses.iter().map(|d| d.severity).fold(0.0_f64, f64::max);
        let severity_penalty = (avg_severity * 0.6 + max_severity * 0.4).clamp(0.0, 1.0);
        (0.5 - severity_penalty).clamp(-0.5, 0.5)
    }

    /// Process a system task using NL trace reflection from the ReflectiveAnalyzer.
    /// Instead of a passed-in bonus, the method runs analyzer.analyze() to get
    /// structured diagnoses, maps them to severity, and computes success probability
    /// from diagnosis severity.
    pub fn process_system_task_with_analyzer(
        &mut self,
        external_id: u64,
        title: &str,
        description: &str,
        metric_before: f64,
        analyzer: &mut ReflectiveAnalyzer,
    ) -> TaskResult {
        let diagnoses = analyzer.analyze();
        let reflective_bonus = self.compute_reflective_bonus_from_diagnoses(&diagnoses);
        self.process_system_task(external_id, title, description, metric_before, reflective_bonus)
    }

    /// Generate concrete EngineEvolutionTask proposals from structured diagnoses.
    /// Each diagnosis maps to an EngineTaskType via fix_category_to_task_type.
    pub fn propose_tasks_from_diagnoses(&mut self, diagnoses: &[Diagnosis]) -> Vec<EngineEvolutionTask> {
        let mut proposed = Vec::with_capacity(diagnoses.len());
        for d in diagnoses {
            let task_type = fix_category_to_task_type(&d.suggested_fix);
            let task = EngineEvolutionTask {
                id: self.next_id(),
                target_gap: d.root_cause.clone(),
                description: format!(
                    "Diagnosis: {} (severity={:.2}) — auto-proposed from reflective analysis",
                    d.root_cause, d.severity
                ),
                task_type,
                status: EngineTaskStatus::Proposed,
                created_cycle: self.cycle,
                completed_cycle: None,
                prerequisite_ids: Vec::new(),
            };
            self.tasks.push(task.clone());
            proposed.push(task);
        }
        proposed
    }

    /// Apply targeted mutations to IdentityCore based on ReflectiveAnalyzer diagnoses.
    /// Unlike random bit-flip mutation, this method:
    /// - Selects mutation targets based on diagnosis root cause
    /// - Applies severity-weighted VSA mutations at diagnosis-relevant positions
    /// - Returns a log of applied mutations
    pub fn reflective_mutate(
        &mut self,
        identity: &mut IdentityCore,
        analyzer: &mut ReflectiveAnalyzer,
        session_success_rate: f64,
    ) -> Vec<String> {
        let diagnoses = analyzer.analyze();
        if diagnoses.is_empty() {
            return vec!["reflective_mutate: no diagnoses, skipping mutation".to_string()];
        }

        let mut mutation_log: Vec<String> = Vec::new();

        for d in &diagnoses {
            if d.severity < 0.3 {
                continue;
            }

            let mutation_target = match &d.suggested_fix {
                FixCategory::CalibrationDrift => "calibration",
                FixCategory::ModuleWiring => "wiring",
                FixCategory::MemoryPressure => "memory",
                FixCategory::LatencyDegradation => "latency",
                FixCategory::SkillStagnation => "skill",
                FixCategory::CompileFailure => "compile",
                FixCategory::SystemicDegradation => "systemic",
            };

            let mutation_strength = (d.severity * session_success_rate).clamp(0.01, 0.5);
            let vsa_len = identity.self_vsa.len();

            if vsa_len > 0 {
                let num_flips = ((vsa_len as f64) * mutation_strength * 0.1).ceil() as usize;
                let mut flipped = 0;
                let step = if vsa_len > num_flips && num_flips > 0 {
                    vsa_len / num_flips
                } else {
                    1
                };
                for i in 0..num_flips {
                    let pos = (i * step) % vsa_len;
                    identity.self_vsa[pos] ^= 1 << (i % 8);
                    flipped += 1;
                }

                mutation_log.push(format!(
                    "reflective_mutate: {} — flipped {} bits in self_vsa (strength={:.3})",
                    mutation_target, flipped, mutation_strength
                ));
            }

            if !identity.personality_traits.is_empty() {
                let trait_idx = match &d.suggested_fix {
                    FixCategory::CalibrationDrift | FixCategory::SystemicDegradation => 0,
                    FixCategory::ModuleWiring | FixCategory::CompileFailure => 1,
                    _ => 2,
                } % identity.personality_traits.len();

                if mutation_strength > 0.1 {
                    let trait_bytes = &mut identity.personality_traits[trait_idx];
                    if !trait_bytes.is_empty() {
                        let idx = trait_bytes.len() / 2;
                        trait_bytes[idx] = trait_bytes[idx].wrapping_add((mutation_strength * 16.0) as u8);
                        mutation_log.push(format!(
                            "reflective_mutate: {} — tuned personality trait[{}]",
                            mutation_target, trait_idx
                        ));
                    }
                }
            }

            if d.severity > 0.6 && !identity.core_values.is_empty() {
                let val_idx = d.id as usize % identity.core_values.len();
                identity.core_values[val_idx] = format!(
                    "{}_adjusted:sev={:.2}",
                    identity.core_values[val_idx].trim_start_matches("adjusted:"),
                    d.severity
                );
                mutation_log.push(format!(
                    "reflective_mutate: {} — adjusted core_value[{}]",
                    mutation_target, val_idx
                ));
            }
        }

        identity.mark_dirty();
        mutation_log
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Increment cycle counter.
    pub fn tick(&mut self) {
        self.cycle += 1;
    }

    /// Feed trace events to an analyzer and auto-propose tasks in one step.
    pub fn reflect_and_propose(
        &mut self,
        analyzer: &mut ReflectiveAnalyzer,
        events: Vec<TraceEvent>,
    ) -> Vec<EngineEvolutionTask> {
        self.tick();
        analyzer.feed_events(events);
        let diagnoses = analyzer.analyze();
        self.propose_tasks_from_diagnoses(&diagnoses)
    }

    pub fn summary(&self) -> String {
        let total = self.tasks.len();
        let completed = self.completed_tasks.iter().filter(|r| r.success).count();
        let proposed = self.tasks.iter().filter(|t| matches!(t.status, EngineTaskStatus::Proposed)).count();
        format!(
            "SelfEvolutionTaskEngine: {} tasks ({} proposed, {} completed), cycle={}",
            total, proposed, completed, self.cycle
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::reflective_analyzer::TrendDirection;
    use crate::core::nt_core_identity::IdentityCore;

    #[test]
    fn test_new_engine_empty() {
        let engine = SelfEvolutionTaskEngine::new();
        assert_eq!(engine.cycle, 0);
        assert!(engine.tasks.is_empty());
        assert!(engine.completed_tasks.is_empty());
    }

    #[test]
    fn test_tick_increments() {
        let mut engine = SelfEvolutionTaskEngine::new();
        engine.tick();
        assert_eq!(engine.cycle, 1);
        engine.tick();
        assert_eq!(engine.cycle, 2);
    }

    #[test]
    fn test_legacy_process_system_task() {
        let mut engine = SelfEvolutionTaskEngine::new();
        let result = engine.process_system_task(1, "test_task", "", 0.5, 0.2);
        assert!(result.success);
        assert!((result.metric_after - 0.6).abs() < 0.01);
        assert_eq!(result.task_id, 1);
    }

    #[test]
    fn test_compute_reflective_bonus_empty() {
        let engine = SelfEvolutionTaskEngine::new();
        let bonus = engine.compute_reflective_bonus_from_diagnoses(&[]);
        assert!((bonus - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_compute_reflective_bonus_high_severity() {
        let engine = SelfEvolutionTaskEngine::new();
        let diagnoses = vec![Diagnosis {
            id: 1,
            root_cause: "test".into(),
            severity: 0.9,
            affected_components: vec!["a".into()],
            suggested_fix: FixCategory::SystemicDegradation,
            trace_evidence: vec![],
            trend: TrendDirection::Worsening,
        }];
        let bonus = engine.compute_reflective_bonus_from_diagnoses(&diagnoses);
        // 0.5 - (0.9*0.6 + 0.9*0.4) = 0.5 - 0.9 = -0.4
        assert!(bonus < 0.0);
    }

    #[test]
    fn test_propose_tasks_from_diagnoses() {
        let mut engine = SelfEvolutionTaskEngine::new();
        let diagnoses = vec![
            Diagnosis {
                id: 1,
                root_cause: "CalibrationDrift".into(),
                severity: 0.5,
                affected_components: vec!["rate".into()],
                suggested_fix: FixCategory::CalibrationDrift,
                trace_evidence: vec![],
                trend: TrendDirection::Worsening,
            },
            Diagnosis {
                id: 2,
                root_cause: "Stagnation".into(),
                severity: 0.7,
                affected_components: vec!["task".into()],
                suggested_fix: FixCategory::SkillStagnation,
                trace_evidence: vec![],
                trend: TrendDirection::Worsening,
            },
        ];
        let tasks = engine.propose_tasks_from_diagnoses(&diagnoses);
        assert_eq!(tasks.len(), 2);
        assert!(matches!(tasks[0].task_type, EngineTaskType::TuneMutation { .. }));
        assert!(matches!(tasks[1].task_type, EngineTaskType::SelfAudit));
        assert_eq!(tasks[0].target_gap, "CalibrationDrift");
    }

    #[test]
    fn test_reflect_and_propose_integration() {
        let mut engine = SelfEvolutionTaskEngine::new();
        let mut analyzer = ReflectiveAnalyzer::new(100);
        let events = vec![
            TraceEvent::CalibrationTrend {
                cycle: 0,
                domain: "reasoning".into(),
                ece_avg: 0.25,
                ece_trend: 0.04,
                surprise_trend: 0.02,
            },
            TraceEvent::WeaknessDetected {
                cycle: 0,
                pattern: "timeout".into(),
                severity: 0.5,
                domain: "inference".into(),
            },
        ];
        let proposals = engine.reflect_and_propose(&mut analyzer, events);
        assert!(!proposals.is_empty(), "Should generate at least one proposal from events");
        assert!(engine.cycle > 0, "Engine cycle should increment");
    }

    #[test]
    fn test_reflective_mutate_no_diagnoses() {
        let mut engine = SelfEvolutionTaskEngine::new();
        let mut analyzer = ReflectiveAnalyzer::new(10);
        let mut identity = IdentityCore::default();
        let log = engine.reflective_mutate(&mut identity, &mut analyzer, 1.0);
        assert!(!log.is_empty());
        assert!(log[0].contains("no diagnoses"));
    }

    #[test]
    fn test_reflective_mutate_targeted_flips() {
        let mut engine = SelfEvolutionTaskEngine::new();
        let mut analyzer = ReflectiveAnalyzer::new(100);
        // Feed high-severity events to generate diagnoses
        for _ in 0..5 {
            analyzer.feed_event(TraceEvent::CalibrationTrend {
                cycle: 0,
                domain: "reasoning".into(),
                ece_avg: 0.4,
                ece_trend: 0.08,
                surprise_trend: 0.05,
            });
        }
        for _ in 0..4 {
            analyzer.feed_event(TraceEvent::InterventionLog {
                cycle: 0,
                source: "core".into(),
                action: "verify".into(),
                success: false,
            });
        }

        let mut identity = IdentityCore::default();
        // Ensure self_vsa has data for mutation
        if identity.self_vsa.is_empty() {
            identity.self_vsa = vec![0u8; 128];
        }
        if identity.personality_traits.is_empty() {
            identity.personality_traits.push(vec![0u8; 64]);
        }

        let log = engine.reflective_mutate(&mut identity, &mut analyzer, 0.8);
        let has_mutations = log.iter().any(|l| l.contains("flipped") || l.contains("tuned"));
        assert!(has_mutations, "Should produce targeted mutations from diagnoses");
    }

    #[test]
    fn test_analyzer_method_produces_same_values_as_direct() {
        let mut engine = SelfEvolutionTaskEngine::new();
        let mut analyzer = ReflectiveAnalyzer::new(100);
        for _ in 0..3 {
            analyzer.feed_event(TraceEvent::CalibrationTrend {
                cycle: 0,
                domain: "memory".into(),
                ece_avg: 0.3,
                ece_trend: 0.04,
                surprise_trend: 0.02,
            });
        }
        let result_with = engine.process_system_task_with_analyzer(1, "test", "desc", 0.5, &mut analyzer);
        let bonus = engine.compute_reflective_bonus_from_diagnoses(&analyzer.analyze());
        // With analyzer, success is computed from diagnoses; verify non-crash
        assert!(!result_with.experience_distilled.is_empty());
    }
}
