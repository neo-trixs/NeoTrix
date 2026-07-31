use std::path::PathBuf;

use super::SelfIteratingBrain;
use super::pipeline::{BrainStage, StageDecision};
use crate::core::nt_core_policy::E8Outcome;
use crate::neotrix::l8_autonomic_impl::nt_mind_skill_engine::SkillEngine;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_memory_kb::ProceduralMemoryRecord;

pub struct ProceduralMemoryStage;
impl Default for ProceduralMemoryStage { fn default() -> Self { Self } }
impl ProceduralMemoryStage { pub fn new() -> Self { Self } }
impl BrainStage for ProceduralMemoryStage {
    fn name(&self) -> &str { "procedural_memory" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let Some(ref kb) = brain._nt_memory_kb else {
            return Ok(StageDecision::Skip("no KB".into()));
        };

        let outcomes = &brain._transition_learner.outcomes;
        if outcomes.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let recent: Vec<_> = outcomes.iter().rev().take(20).collect();
        let mut sequences: Vec<Vec<u8>> = Vec::new();
        let mut current_seq: Vec<u8> = Vec::new();

        for outcome in &recent {
            if outcome.reward > 0.4 {
                current_seq.push(outcome.mode.0);
            } else {
                if current_seq.len() >= 2 {
                    sequences.push(current_seq.clone());
                }
                current_seq.clear();
            }
        }
        if current_seq.len() >= 2 {
            sequences.push(current_seq);
        }

        if sequences.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let existing_skills = kb.list_procedural_memories(50).unwrap_or_default();
        let mut created_ids: Vec<String> = Vec::new();

        for (i, seq) in sequences.iter().enumerate() {
            let is_duplicate = existing_skills.iter().any(|s| {
                let existing_seq: Vec<u8> = s.e8_sequence.clone();
                existing_seq == *seq || (existing_seq.len() >= 2 && seq.len() >= 2
                    && existing_seq.windows(2).any(|w| seq.windows(2).any(|s| w == s)))
            });
            if is_duplicate {
                continue;
            }

            let trigger = *seq.first().unwrap_or(&0);
            let matching: Vec<&E8Outcome> = recent.iter().copied().filter(|o| seq.contains(&o.mode.0)).collect();
            let avg_reward = if matching.is_empty() {
                0.0
            } else {
                matching.iter().map(|o| o.reward).sum::<f64>() / matching.len() as f64
            };

            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let skill_id = format!("proc_skill_{}_{}", brain.iteration, i);
            let record = ProceduralMemoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                skill_id: skill_id.clone(),
                name: format!("E8 Skill (seq-{})", i),
                description: format!("Learned E8 pattern: {} states, avg_reward={:.3}", seq.len(), avg_reward),
                e8_sequence: seq.clone(),
                trigger_pattern: vec![trigger],
                success_rate: avg_reward,
                execution_count: 1,
                avg_reward,
                created_at: now.clone(),
                updated_at: now,
                tags: vec!["procedural".to_string(), "auto_discovered".to_string()],
            };

            match kb.store_procedural_memory(&record) {
                Ok(_) => {
                    // Bridge: also install as a usable YAML-frontmatter skill file
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    let skills_dir = PathBuf::from(&home).join(".neotrix").join("skills");
                    let mut skill_engine = SkillEngine::new(skills_dir);
                    let _ = skill_engine.install_from_procedural(&record);

                    created_ids.push(skill_id);
                    log::info!("[procedural-memory] created skill {} ({} states, reward={:.3})",
                        record.name, seq.len(), avg_reward);
                }
                Err(e) => {
                    log::warn!("[procedural-memory] failed to store: {}", e);
                }
            }
        }

        if !created_ids.is_empty() {
            let summary = format!("[procedural-memory] created {} new skills", created_ids.len());
            if let Some(ref mut router) = brain.attention_router {
                router.wm().broadcast(&summary);
            }
        }

        Ok(StageDecision::Continue)
    }
}
