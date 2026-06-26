use super::super::core::RewardSource;
use super::super::cortex_memory::CmsConfig;
use super::super::distillation::{apply_principles, avoid_anti_patterns, ExperienceDistiller};
use super::pipeline_core::*;
use super::SelfIteratingBrain;
use crate::core::nt_core_bank::mem::ReasoningMemory;
use crate::make_stage;

make_stage!(MemoryStorageStage);
impl BrainStage for MemoryStorageStage {
    fn name(&self) -> &str {
        "memory_storage"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.auto_memory_iteration
            && brain
                .iteration
                .is_multiple_of(brain.memory_iteration_interval)
        {
            brain.reasoning_bank.iterate_memories(0.85, 0.1);
            let all_mems: Vec<ReasoningMemory> =
                brain.reasoning_bank.memories().iter().cloned().collect();
            let principles = ExperienceDistiller::distill(&all_mems);
            if !principles.is_empty() {
                apply_principles(&mut brain.brain.capability, &principles, 0.6);
            }
            let anti_patterns = ExperienceDistiller::contrastive_reflect(&all_mems);
            if !anti_patterns.is_empty() {
                avoid_anti_patterns(&mut brain.brain.capability, &anti_patterns);
            }
            if let Some(ref mut gm) = brain.group_manager {
                gm.evolve_group();
            }
        }

        let cms_config = CmsConfig::default();
        let cms_result = brain.cortex.consolidate_cms(brain.iteration, &cms_config);
        if cms_result.nt_world_sense_to_topic + cms_result.topic_to_event + cms_result.event_to_fact
            > 0
        {
            log::trace!(
                "CMS: S→T {} T→E {} E→F {}",
                cms_result.nt_world_sense_to_topic,
                cms_result.topic_to_event,
                cms_result.event_to_fact
            );
        }

        Ok(StageDecision::Continue)
    }
}

make_stage!(ReasoningBankStorageStage);
impl BrainStage for ReasoningBankStorageStage {
    fn name(&self) -> &str {
        "bank_storage"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let task_type = brain.task_scratch.current_task_type;
        let reward = brain._reward();
        let source = brain._reward_source();
        let edits = brain._take_micro_edits();

        let memory = if source == RewardSource::External {
            ReasoningMemory::with_external_reward(&task, task_type, &edits, reward)
        } else {
            ReasoningMemory::new(&task, task_type, &edits, reward)
        };

        let embedding = brain._take_task_embedding();
        if let Some(emb) = embedding {
            brain.reasoning_bank.store_with_embedding(memory, emb);
        } else {
            brain.reasoning_bank.store(memory);
        }
        brain._set_micro_edits(edits);
        Ok(StageDecision::Continue)
    }
}

make_stage!(TrajectoryCollectStage);
impl BrainStage for TrajectoryCollectStage {
    fn name(&self) -> &str {
        "trajectory_collect"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        brain
            ._trajectory_collector
            .begin(brain.task_scratch.current_task.clone());
        for step in &brain.seal_rl.stage_results {
            brain._trajectory_collector.record_step(
                crate::core::nt_core_gwt::module_def::SpecialistType::Planner,
                brain._e8_policy.best_mode(),
                step.stage_name.clone(),
                brain.task_scratch.current_task.clone(),
                format!("efc={:.3}", step.efc),
                None,
                true,
                None,
            );
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(CoachAndUpdateStage);
impl BrainStage for CoachAndUpdateStage {
    fn name(&self) -> &str {
        "coach_and_update"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let trajectories: Vec<crate::core::nt_core_prm::AgentTrajectory> =
            brain._trajectory_collector.collected.drain(..).collect();
        if trajectories.is_empty() {
            return Ok(StageDecision::Continue);
        }
        if let Some(ref mut coach) = brain._coach {
            for traj in &trajectories {
                let scores = coach.score_episode(traj);
                for score in &scores {
                    if let Some(step) = traj.steps.get(score.step_idx) {
                        brain._e8_policy.set_previous(step.e8_mode);
                        brain._e8_policy.update(score.score);
                    }
                }
                let outcome_reward = traj.outcome_reward.unwrap_or(0.5);
                brain._transition_learner.record(
                    &traj.task,
                    traj.steps
                        .first()
                        .map(|s| s.e8_mode)
                        .unwrap_or(brain._e8_policy.best_mode()),
                    outcome_reward,
                    brain.iteration,
                );
                let avg_score =
                    scores.iter().map(|s| s.score).sum::<f64>() / scores.len().max(1) as f64;
                brain.task_scratch.reward = brain.task_scratch.reward * 0.9 + avg_score * 0.1;
            }
            brain._e8_policy.decay_epsilon();
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(MetaImprovementStage);
impl BrainStage for MetaImprovementStage {
    fn name(&self) -> &str {
        "meta_improvement"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_mind_ingestion::meta_improvement::MetaDiagnostics;
        if brain._meta_agent.is_none() {
            let mut agent = crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAgent::new();
            agent.meta_layer_can_rewrite_self = true;
            brain._meta_agent = Some(agent);
        }
        if let Some(ref mut agent) = brain._meta_agent {
            let diag = MetaDiagnostics::new(brain.iteration as u64);
            let (action, self_edit) = agent.observe_and_act(&diag);
            match action {
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::CreateStage {
                    name,
                    description: _,
                    frequency,
                } => {
                    if !agent.created_stages.contains(&name.to_string())
                        && brain.meta_additions.len() < agent.max_stages
                    {
                        let stage =
                            crate::neotrix::nt_mind_ingestion::meta_improvement::DynamicStage::new(
                                name, "", frequency,
                            );
                        brain.meta_additions.push(Box::new(stage));
                        agent.created_stages.push(name.to_string());
                        log::info!("[dgm-h] created stage: {}", name);
                    }
                }
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::RemoveStage {
                    name,
                } => {
                    brain.meta_additions.retain(|s| s.name() != name);
                    agent.created_stages.retain(|n| n != name);
                    log::info!("[dgm-h] removed stage: {}", name);
                }
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::ModifyConfig {
                    param: _,
                    value: _,
                } => {
                    log::info!("[dgm-h] config modification (stub)");
                }
                crate::neotrix::nt_mind_ingestion::meta_improvement::MetaAction::NoOp => {}
            }
            if let Some(edit) = self_edit {
                log::info!("[dgm-h] meta self-edit: {:?}", edit);
            }
        }
        Ok(StageDecision::Continue)
    }
}
