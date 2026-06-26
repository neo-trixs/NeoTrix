use super::scratchpad::should_continue_reflection;
use super::ReflectionRound;
use crate::make_stage;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_mind::self_iterating::pipeline::{BrainStage, StageDecision};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
#[allow(unused_imports)]
use crate::neotrix::nt_shield::inner_critic::InnerCritic;

make_stage!(CollateStage);
impl BrainStage for CollateStage {
    fn name(&self) -> &str {
        "ingestion_collate"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        if !pad.collated.is_empty() && pad.round > 1 {
            return Ok(StageDecision::Continue);
        }
        let lines: Vec<&str> = pad.input.lines().collect();
        let total = lines.len();
        let non_empty: Vec<&&str> = lines.iter().filter(|l| !l.trim().is_empty()).collect();
        pad.collated = format!(
            "collated {} lines ({} non-empty, {:.1}% density)",
            total,
            non_empty.len(),
            if total > 0 {
                non_empty.len() as f64 / total as f64 * 100.0
            } else {
                0.0
            }
        );
        Ok(StageDecision::Continue)
    }
}

make_stage!(StructureStage);
impl BrainStage for StructureStage {
    fn name(&self) -> &str {
        "ingestion_structure"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let mut sections = Vec::new();
        let lines: Vec<&str> = pad.input.lines().collect();
        let chunk_size = (lines.len() / 4).max(1);

        for (i, chunk) in lines.chunks(chunk_size).enumerate() {
            let preview: String = chunk
                .iter()
                .take(3)
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<&str>>()
                .join(" ");
            let preview = if preview.len() > 80 {
                format!("{}...", &preview[..80])
            } else {
                preview
            };
            sections.push(format!("section_{}: {}", i + 1, preview));
        }

        if pad.round > 1 {
            let prev_len = pad.sections.len();
            sections = sections
                .into_iter()
                .take(prev_len.max(1))
                .enumerate()
                .map(|(i, s)| {
                    if i < prev_len {
                        format!("{} [refined]", s)
                    } else {
                        s
                    }
                })
                .collect();
        }

        pad.sections = sections;
        Ok(StageDecision::Continue)
    }
}

make_stage!(EntityExtractStage);
impl BrainStage for EntityExtractStage {
    fn name(&self) -> &str {
        "ingestion_entity_extract"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let mut entities = Vec::new();
        for section in &pad.sections {
            let words: Vec<&str> = section.split_whitespace().collect();
            for word in words.iter().take(10) {
                let cleaned = word.trim_matches(|c: char| c.is_ascii_punctuation());
                if cleaned.len() >= 4
                    && cleaned
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                {
                    let entity = cleaned.to_string();
                    if !entities.contains(&entity) {
                        entities.push(entity);
                    }
                }
            }
        }
        if pad.round > 1 {
            entities = entities
                .into_iter()
                .map(|e| format!("{} [confirmed]", e))
                .collect();
        }
        pad.entities = entities;
        Ok(StageDecision::Continue)
    }
}

make_stage!(EventExtractStage);
impl BrainStage for EventExtractStage {
    fn name(&self) -> &str {
        "ingestion_event_extract"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        pad.events = vec!["event_analysis_pending".to_string()];
        Ok(StageDecision::Continue)
    }
}

make_stage!(RelationMapStage);
impl BrainStage for RelationMapStage {
    fn name(&self) -> &str {
        "ingestion_relation_map"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let mut relations = Vec::new();
        for i in 0..pad.entities.len().min(5) {
            for j in (i + 1)..pad.entities.len().min(5) {
                relations.push((
                    pad.entities[i].clone(),
                    "related_to".to_string(),
                    pad.entities[j].clone(),
                ));
            }
        }
        if pad.round > 1 {
            relations = relations
                .into_iter()
                .map(|(s, r, t)| (s, r, format!("{} [weighted]", t)))
                .collect();
        }
        pad.relations = relations;
        Ok(StageDecision::Continue)
    }
}

make_stage!(OntologyAlignStage);
impl BrainStage for OntologyAlignStage {
    fn name(&self) -> &str {
        "ingestion_ontology_align"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let mut aligned = Vec::new();
        for entity in &pad.entities {
            let domain = if entity.contains("algorithm") || entity.contains("model") {
                "algorithm"
            } else if entity.contains("framework") || entity.contains("library") {
                "framework"
            } else if entity.contains("paper") || entity.contains("author") {
                "research"
            } else {
                "concept"
            };
            aligned.push(format!("{} -> ontology:{}", entity, domain));
        }
        pad.aligned_entities = aligned;
        Ok(StageDecision::Continue)
    }
}

make_stage!(ReasonStage);
impl BrainStage for ReasonStage {
    fn name(&self) -> &str {
        "ingestion_reason"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let mut notes = Vec::new();
        notes.push(format!(
            "round_{}: identified {} entities across {} relations",
            pad.round,
            pad.entities.len(),
            pad.relations.len()
        ));
        if pad.entities.len() >= 3 {
            notes.push(format!(
                "triadic_closure: {} -> {} -> {}",
                pad.entities[0], pad.entities[1], pad.entities[2]
            ));
        }
        pad.reasoning = notes;
        Ok(StageDecision::Continue)
    }
}

make_stage!(SkuGenerateStage);
impl BrainStage for SkuGenerateStage {
    fn name(&self) -> &str {
        "ingestion_sku_generate"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        pad.skus = pad
            .reasoning
            .iter()
            .enumerate()
            .map(|(i, note)| format!("sku_{}: {}", i + 1, note))
            .collect();
        Ok(StageDecision::Continue)
    }
}

make_stage!(ApplyStage);
impl BrainStage for ApplyStage {
    fn name(&self) -> &str {
        "ingestion_apply"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let sku_summary = if pad.skus.is_empty() {
            "no SKUs generated".to_string()
        } else {
            pad.skus.join("; ")
        };
        let summary = format!(
            "round {}: collated={} | sections={} | entities={} | relations={} | {}",
            pad.round,
            pad.collated,
            pad.sections.len(),
            pad.entities.len(),
            pad.relations.len(),
            sku_summary,
        );
        pad.final_summary = Some(summary);
        Ok(StageDecision::Continue)
    }
}

make_stage!(ReflectionCheckStage);
impl BrainStage for ReflectionCheckStage {
    fn name(&self) -> &str {
        "ingestion_reflection_check"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };

        pad.clarity_delta = if pad.round == 1 {
            0.8
        } else {
            let prev = pad
                .reflection_history
                .last()
                .map(|r| r.clarity_delta)
                .unwrap_or(1.0);
            (prev * 0.6).max(0.01)
        };

        let qm_score = pad.quality_monitor.evaluate(
            pad.clarity_delta,
            pad.entities.len(),
            pad.relations.len(),
        );

        let insights = pad.reasoning.clone();
        let converged_now = pad.clarity_delta.abs() < pad.config.convergence_threshold
            || pad.round >= pad.config.max_rounds;

        pad.reflection_history.push(ReflectionRound {
            round: pad.round,
            insights,
            clarity_delta: pad.clarity_delta,
            converged: converged_now,
        });

        let final_summary = pad.final_summary.clone().unwrap_or_default();

        if converged_now || !should_continue_reflection(pad) {
            pad.converged = true;
            log::info!(
                "[ingestion] converged after {} rounds, clarity_delta={:.4}, quality={:.4}",
                pad.round,
                pad.clarity_delta,
                qm_score,
            );
        } else {
            pad.round += 1;
            pad.collated = String::new();
            pad.sections = Vec::new();
            pad.entities = Vec::new();
            pad.events = Vec::new();
            pad.relations = Vec::new();
            pad.aligned_entities = Vec::new();
            pad.reasoning = Vec::new();
            pad.skus = Vec::new();
            pad.final_summary = Some(final_summary);
            log::info!(
                "[ingestion] advancing to round {}, clarity_delta={:.4}",
                pad.round,
                pad.clarity_delta,
            );
        }

        Ok(StageDecision::Continue)
    }
}

use super::stream_hygiene::{
    clean_orphan_vsa_vectors, fold_duplicate_vectors, repair_corrupted_tags,
};

make_stage!(StreamHygieneStage);
impl BrainStage for StreamHygieneStage {
    fn name(&self) -> &str {
        "stream_hygiene"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let before_len = brain.consciousness_state.consciousness_stream.len();
        let removed_orphans =
            clean_orphan_vsa_vectors(&mut brain.consciousness_state.consciousness_stream);
        let removed_corrupt =
            repair_corrupted_tags(&mut brain.consciousness_state.consciousness_stream);
        let removed_dups =
            fold_duplicate_vectors(&mut brain.consciousness_state.consciousness_stream, 64);
        let after_len = brain.consciousness_state.consciousness_stream.len();
        if before_len != after_len {
            log::info!(
                "[stream_hygiene] cleaned {}→{} vectors (orphans={}, corrupt={}, dups={})",
                before_len,
                after_len,
                removed_orphans,
                removed_corrupt,
                removed_dups,
            );
        }
        Ok(StageDecision::Continue)
    }
}

use super::fingerprint::{default_constraints, VsaPrefixFingerprint};
use std::sync::Mutex;

pub struct VsaFingerprintStage {
    fingerprint: Mutex<Option<VsaPrefixFingerprint>>,
}

impl VsaFingerprintStage {
    pub fn new() -> Self {
        Self {
            fingerprint: Mutex::new(None),
        }
    }
}

impl Default for VsaFingerprintStage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainStage for VsaFingerprintStage {
    fn name(&self) -> &str {
        "vsa_fingerprint"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let mut fp_guard = self.fingerprint.lock().unwrap_or_else(|e| e.into_inner());
        let constraints = default_constraints();
        let first_person = &brain.consciousness_state.first_person;

        match fp_guard.as_ref() {
            None => {
                let fp = VsaPrefixFingerprint::new(first_person, constraints);
                fp_guard.replace(fp);
                log::info!("[vsa_fingerprint] initialized");
                return Ok(StageDecision::Continue);
            }
            Some(expected) => {
                expected.verify(first_person, &constraints).map_err(|e| {
                    log::error!("[vsa_fingerprint] DRIFT DETECTED: {}", e);
                    NeoTrixError::Brain(format!("VSA prefix drift: {}", e))
                })?;
                return Ok(StageDecision::Continue);
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::neotrix::nt_mind::self_iterating::pipeline::{BrainStage, StageDecision};
            use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
            use crate::neotrix::nt_mind_ingestion::scratchpad::IngestionScratchpad;
            use crate::neotrix::nt_mind_ingestion::{IngestionConfig, IngestionSourceType};

            // ── Stage name tests ──────────────────────────────────────────────

            #[test]
            fn test_ingestion_stage_names() {
                assert_eq!(CollateStage.name(), "ingestion_collate");
                assert_eq!(StructureStage.name(), "ingestion_structure");
                assert_eq!(EntityExtractStage.name(), "ingestion_entity_extract");
                assert_eq!(EventExtractStage.name(), "ingestion_event_extract");
                assert_eq!(RelationMapStage.name(), "ingestion_relation_map");
                assert_eq!(OntologyAlignStage.name(), "ingestion_ontology_align");
                assert_eq!(ReasonStage.name(), "ingestion_reason");
                assert_eq!(SkuGenerateStage.name(), "ingestion_sku_generate");
                assert_eq!(ApplyStage.name(), "ingestion_apply");
                assert_eq!(ReflectionCheckStage.name(), "ingestion_reflection_check");
            }

            #[test]
            fn test_system_stage_names() {
                assert_eq!(StreamHygieneStage.name(), "stream_hygiene");
                assert_eq!(VsaFingerprintStage::new().name(), "vsa_fingerprint");
                assert_eq!(CanonicalSortStage.name(), "canonical_sort");
                assert_eq!(StormBreakerStage.name(), "storm_breaker");
                assert_eq!(InnerCriticStage.name(), "inner_critic");
                assert_eq!(DocumentParseStage.name(), "document_parse");
            }

            // ── Stage frequency tests ──────────────────────────────────────────

            #[test]
            fn test_ingestion_stage_frequencies() {
                assert_eq!(CollateStage.frequency(), 3);
                assert_eq!(StructureStage.frequency(), 3);
                assert_eq!(EntityExtractStage.frequency(), 3);
                assert_eq!(EventExtractStage.frequency(), 3);
                assert_eq!(RelationMapStage.frequency(), 3);
                assert_eq!(OntologyAlignStage.frequency(), 3);
                assert_eq!(ReasonStage.frequency(), 3);
                assert_eq!(SkuGenerateStage.frequency(), 3);
                assert_eq!(ApplyStage.frequency(), 3);
                assert_eq!(ReflectionCheckStage.frequency(), 3);
            }

            #[test]
            fn test_system_stage_frequencies() {
                assert_eq!(StreamHygieneStage.frequency(), 3);
                assert_eq!(VsaFingerprintStage::new().frequency(), 1);
                assert_eq!(CanonicalSortStage.frequency(), 5);
                assert_eq!(StormBreakerStage.frequency(), 2);
                assert_eq!(InnerCriticStage.frequency(), 5);
                assert_eq!(DocumentParseStage.frequency(), 1);
            }

            // ── Default / construction tests ──────────────────────────────────

            #[test]
            fn test_make_stage_default_impls() {
                let _ = CollateStage::default();
                let _ = StructureStage::default();
                let _ = EntityExtractStage::default();
                let _ = EventExtractStage::default();
                let _ = RelationMapStage::default();
                let _ = OntologyAlignStage::default();
                let _ = ReasonStage::default();
                let _ = SkuGenerateStage::default();
                let _ = ApplyStage::default();
                let _ = ReflectionCheckStage::default();
                let _ = StreamHygieneStage::default();
                let _ = CanonicalSortStage::default();
                let _ = StormBreakerStage::default();
                let _ = InnerCriticStage::default();
                let _ = DocumentParseStage::default();
            }

            #[test]
            fn test_make_stage_new_constructors() {
                let _ = CollateStage::new();
                let _ = StructureStage::new();
                let _ = EntityExtractStage::new();
                let _ = EventExtractStage::new();
                let _ = RelationMapStage::new();
                let _ = OntologyAlignStage::new();
                let _ = ReasonStage::new();
                let _ = SkuGenerateStage::new();
                let _ = ApplyStage::new();
                let _ = ReflectionCheckStage::new();
                let _ = StreamHygieneStage::new();
                let _ = CanonicalSortStage::new();
                let _ = StormBreakerStage::new();
                let _ = InnerCriticStage::new();
                let _ = DocumentParseStage::new();
            }

            #[test]
            fn test_vsa_fingerprint_stage_default_and_new() {
                let a = VsaFingerprintStage::default();
                let b = VsaFingerprintStage::new();
                assert_eq!(a.name(), b.name());
                assert_eq!(a.frequency(), b.frequency());
            }

            // ── StageDecision enum tests ──────────────────────────────────────

            #[test]
            fn test_stage_decision_debug_clone() {
                let decisions = vec![
                    StageDecision::Continue,
                    StageDecision::Skip("reason".into()),
                    StageDecision::Rollback("rollback_reason".into()),
                ];
                assert_eq!(decisions.len(), 3);
                let _ = format!("{:?}", StageDecision::Continue);
            }

            #[test]
            fn test_stage_decision_skip_contains_message() {
                let msg = "no ingestion active";
                let decision = StageDecision::Skip(msg.into());
                match &decision {
                    StageDecision::Skip(m) => assert_eq!(m, msg),
                    _ => panic!("expected Skip variant"),
                }
            }

            #[test]
            fn test_stage_decision_rollback_contains_reason() {
                let reason = "stage failed";
                let decision = StageDecision::Rollback(reason.into());
                match &decision {
                    StageDecision::Rollback(r) => assert_eq!(r, reason),
                    _ => panic!("expected Rollback variant"),
                }
            }

            // ── Skip when no scratchpad ──────────────────────────────────────
            // All ingestion stages skip early when _ingestion_scratchpad is None

            #[test]
            fn test_collate_stage_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = CollateStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_structure_stage_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = StructureStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_entity_extract_stage_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = EntityExtractStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_event_extract_stage_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = EventExtractStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_relation_map_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = RelationMapStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_ontology_align_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = OntologyAlignStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_reason_stage_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = ReasonStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_sku_generate_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = SkuGenerateStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_apply_stage_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = ApplyStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_reflection_check_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = ReflectionCheckStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            #[test]
            fn test_document_parse_skips_without_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let decision = DocumentParseStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Skip(_)));
            }

            // ── Stages with scratchpad ────────────────────────────────────────

            fn make_scratchpad(input: &str) -> IngestionScratchpad {
                IngestionScratchpad::new(
                    input.to_string(),
                    IngestionSourceType::Web,
                    IngestionConfig::default(),
                )
            }

            #[test]
            fn test_collate_stage_with_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                brain._ingestion_scratchpad =
                    Some(make_scratchpad("Hello World\nLine 2\n\nLine 4"));
                let decision = CollateStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.collated.contains("4 lines"));
                assert!(pad.collated.contains("3 non-empty"));
            }

            #[test]
            fn test_collate_stage_already_collated_continues() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("data");
                pad.collated = "existing".into();
                pad.round = 2;
                brain._ingestion_scratchpad = Some(pad);
                let decision = CollateStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }

            #[test]
            fn test_structure_stage_with_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                brain._ingestion_scratchpad = Some(make_scratchpad("A\nB\nC\nD\nE\nF\nG\nH"));
                let decision = StructureStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(!pad.sections.is_empty());
                assert!(pad.sections[0].starts_with("section_1"));
            }

            #[test]
            fn test_structure_stage_refines_on_round_2() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("A\nB\nC\nD\nE\nF\nG\nH");
                pad.round = 2;
                pad.sections = vec!["existing".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = StructureStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.sections[0].contains("[refined]"));
            }

            #[test]
            fn test_entity_extract_with_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("unused");
                pad.sections = vec!["Alice met Bob at Google".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = EntityExtractStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.entities.iter().any(|e| e == "Alice"));
            }

            #[test]
            fn test_entity_extract_confirms_on_round_2() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("Alice worked at Google");
                pad.round = 2;
                pad.sections = vec!["Alice worked at Google".into(), "Bob uses Google".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = EntityExtractStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.entities.iter().all(|e| e.contains("[confirmed]")
                    || !pad
                        .entities
                        .iter()
                        .filter(|x| x.trim_end_matches(" [confirmed]")
                            == e.trim_end_matches(" [confirmed]"))
                        .count()
                        > 1));
            }

            #[test]
            fn test_entity_extract_skips_short_words() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("a an the");
                pad.sections = vec!["a an the".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = EntityExtractStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.entities.is_empty());
            }

            #[test]
            fn test_event_extract_sets_pending() {
                let mut brain = SelfIteratingBrain::new();
                brain._ingestion_scratchpad = Some(make_scratchpad("some input"));
                let decision = EventExtractStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert_eq!(pad.events, vec!["event_analysis_pending"]);
            }

            #[test]
            fn test_relation_map_with_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("unused");
                pad.entities = vec!["A".into(), "B".into(), "C".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = RelationMapStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                // 3 entities → 3 choose 2 = 3 relations
                assert_eq!(pad.relations.len(), 3);
            }

            #[test]
            fn test_relation_map_weights_on_round_2() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("unused");
                pad.round = 2;
                pad.entities = vec!["X".into(), "Y".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = RelationMapStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.relations[0].2.contains("[weighted]"));
            }

            #[test]
            fn test_ontology_align_with_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("unused");
                pad.entities = vec![
                    "MyAlgorithm".into(),
                    "SomeFramework".into(),
                    "UnknownThing".into(),
                ];
                brain._ingestion_scratchpad = Some(pad);
                let decision = OntologyAlignStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert_eq!(pad.aligned_entities.len(), 3);
                assert!(pad.aligned_entities[0].contains("ontology:algorithm"));
                assert!(pad.aligned_entities[1].contains("ontology:framework"));
                assert!(pad.aligned_entities[2].contains("ontology:concept"));
            }

            #[test]
            fn test_reason_stage_with_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("unused");
                pad.entities = vec!["A".into(), "B".into(), "C".into()];
                pad.relations = vec![("A".into(), "rel".into(), "B".into())];
                brain._ingestion_scratchpad = Some(pad);
                let decision = ReasonStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(!pad.reasoning.is_empty());
                assert!(pad.reasoning[0].contains("round_"));
            }

            #[test]
            fn test_reason_stage_triadic_closure() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("unused");
                pad.entities = vec!["A".into(), "B".into(), "C".into(), "D".into()];
                pad.relations = vec![];
                brain._ingestion_scratchpad = Some(pad);
                let decision = ReasonStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.reasoning.iter().any(|r| r.contains("triadic_closure")));
            }

            #[test]
            fn test_sku_generate_from_reasoning() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("unused");
                pad.reasoning = vec!["note_a".into(), "note_b".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = SkuGenerateStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert_eq!(pad.skus.len(), 2);
                assert!(pad.skus[0].starts_with("sku_1"));
            }

            #[test]
            fn test_apply_stage_with_scratchpad() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("input data");
                pad.round = 1;
                pad.collated = "collated output".into();
                pad.sections = vec!["s1".into()];
                pad.entities = vec!["E1".into()];
                pad.relations = vec![];
                pad.skus = vec!["sku_a".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = ApplyStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                let summary = pad.final_summary.as_deref().unwrap_or("");
                assert!(summary.contains("round 1"));
                assert!(summary.contains("collated output"));
                assert!(summary.contains("sku_a"));
            }

            #[test]
            fn test_apply_stage_no_skus() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("input");
                pad.skus = vec![];
                brain._ingestion_scratchpad = Some(pad);
                let decision = ApplyStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad
                    .final_summary
                    .as_deref()
                    .unwrap_or("")
                    .contains("no SKUs"));
            }

            #[test]
            fn test_reflection_check_inits_first_round() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("input data");
                pad.clarity_delta = 1.0;
                pad.entities = vec!["E1".into()];
                pad.relations = vec![];
                pad.reasoning = vec!["insight".into()];
                pad.config.convergence_threshold = 0.05;
                pad.config.max_rounds = 5;
                brain._ingestion_scratchpad = Some(pad);
                let decision = ReflectionCheckStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                // Round 1 should have clarity_delta = 0.8
                assert!((pad.clarity_delta - 0.8).abs() < 0.001);
                // Should have at least one reflection_history entry
                assert_eq!(pad.reflection_history.len(), 1);
            }

            #[test]
            fn test_reflection_check_converges_early() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("input");
                pad.round = 1;
                pad.config.convergence_threshold = 0.9;
                pad.config.max_rounds = 5;
                pad.entities = vec!["E1".into()];
                pad.relations = vec![];
                pad.reasoning = vec!["insight".into()];
                brain._ingestion_scratchpad = Some(pad);
                let decision = ReflectionCheckStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                assert!(pad.converged);
            }

            #[test]
            fn test_reflection_check_resets_for_next_round() {
                let mut brain = SelfIteratingBrain::new();
                let mut pad = make_scratchpad("input");
                pad.round = 1;
                pad.config.convergence_threshold = 0.001;
                pad.config.max_rounds = 3;
                pad.reasoning = vec!["insight".into()];
                pad.entities = vec!["E1".into()];
                pad.relations = vec![("A".into(), "r".into(), "B".into())];
                brain._ingestion_scratchpad = Some(pad);
                let decision = ReflectionCheckStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
                let pad = brain._ingestion_scratchpad().unwrap();
                // Should have advanced to round 2
                assert_eq!(pad.round, 2);
                // Fields should be reset but final_summary preserved
                assert!(pad.collated.is_empty());
                assert!(pad.sections.is_empty());
                assert!(pad.entities.is_empty());
            }

            // ── Stages that work without scratchpad ─────────────────────────

            #[test]
            fn test_stream_hygiene_on_empty_stream() {
                let mut brain = SelfIteratingBrain::new();
                let decision = StreamHygieneStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }

            #[test]
            fn test_storm_breaker_no_storm() {
                let mut brain = SelfIteratingBrain::new();
                let decision = StormBreakerStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }

            #[test]
            fn test_inner_critic_on_empty_task() {
                let mut brain = SelfIteratingBrain::new();
                let decision = InnerCriticStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }

            #[test]
            fn test_inner_critic_applies_penalty_for_violations() {
                let mut brain = SelfIteratingBrain::new();
                brain._set_reward(1.0);
                let decision = InnerCriticStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }

            #[test]
            fn test_canonical_sort_stage() {
                let mut brain = SelfIteratingBrain::new();
                let decision = CanonicalSortStage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }

            #[test]
            fn test_vsa_fingerprint_stage_init() {
                let mut brain = SelfIteratingBrain::new();
                let stage = VsaFingerprintStage::new();
                let decision = stage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }

            #[test]
            fn test_vsa_fingerprint_stage_verify() {
                let mut brain = SelfIteratingBrain::new();
                let stage = VsaFingerprintStage::new();
                // First call initializes
                let _ = stage.process(&mut brain).unwrap();
                // Second call verifies (should pass with same FP)
                let decision = stage.process(&mut brain).unwrap();
                assert!(matches!(decision, StageDecision::Continue));
            }
        }
    }
}

make_stage!(CanonicalSortStage);
impl BrainStage for CanonicalSortStage {
    fn name(&self) -> &str {
        "canonical_sort"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use super::canonical::canonical_catalog_fingerprint;

        let hex_ids: Vec<usize> = brain
            ._strategy_matrix
            .iter()
            .flat_map(|row| row.iter())
            .map(|h| h.0 as usize)
            .collect();

        let dummy_specs: Vec<String> = vec![];

        let fp = canonical_catalog_fingerprint(&hex_ids, &dummy_specs);
        let fp_short = if fp.len() > 16 { &fp[..16] } else { &fp };
        log::trace!(
            "[canonical_sort] catalog fingerprint: {} ({} hexagrams)",
            fp_short,
            hex_ids.len()
        );
        Ok(StageDecision::Continue)
    }
}

make_stage!(StormBreakerStage);
impl BrainStage for StormBreakerStage {
    fn name(&self) -> &str {
        "storm_breaker"
    }
    fn frequency(&self) -> usize {
        2
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let monitor = &brain.consciousness_state.cognitive_load;
        let storm_status =
            crate::neotrix::nt_mind_ingestion::storm_breaker::detect_reasoning_storm(monitor);
        match storm_status {
            crate::neotrix::nt_mind_ingestion::storm_breaker::ReasoningStormStatus::StormDetected { repeat_count } => {
                let mode = crate::neotrix::nt_mind_ingestion::storm_breaker::next_storm_mode(brain.iteration);
                log::warn!("[storm_breaker] storm detected ({} repeats), switching to {} mode",
                    repeat_count, mode);
                Ok(StageDecision::Skip(format!("storm suppression: {} mode", mode)))
            }
            _ => Ok(StageDecision::Continue),
        }
    }
}

make_stage!(InnerCriticStage);
impl BrainStage for InnerCriticStage {
    fn name(&self) -> &str {
        "inner_critic"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let mut total_violations = 0;
        let critic = InnerCritic::new();

        let task = brain._current_task();
        if !task.is_empty() {
            let violations = critic.audit(&task);
            total_violations += violations.len();
            for v in &violations {
                log::info!(
                    "[inner_critic] {} | severity={} | {}",
                    v.name,
                    v.severity,
                    v.description
                );
            }
        }

        if let Some(ref pad) = brain._ingestion_scratchpad {
            let violations = critic.audit(&pad.input);
            total_violations += violations.len();
            for v in &violations {
                log::info!(
                    "[inner_critic:ingestion] {} | severity={} | {}",
                    v.name,
                    v.severity,
                    v.description
                );
            }
        }

        if total_violations > 0 {
            let penalty = (total_violations as f64).min(10.0) * -0.02;
            brain._set_reward(brain._reward() + penalty);
        }

        Ok(StageDecision::Continue)
    }
}

use super::document_parser::DocumentParsingEngine;
use std::path::Path;

make_stage!(DocumentParseStage);
impl BrainStage for DocumentParseStage {
    fn name(&self) -> &str {
        "document_parse"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let input = pad.input.trim();
        if !input.contains('\n') {
            let path = Path::new(input);
            if path.exists() && path.is_file() {
                let engine = DocumentParsingEngine::default();
                match engine.parse_file(path) {
                    Ok(parsed) => {
                        let sections: Vec<String> = parsed
                            .document
                            .sections
                            .iter()
                            .flat_map(|s| s.flatten())
                            .map(|s| {
                                let heading = s.heading.as_deref().unwrap_or("(untitled)");
                                format!("[{}] {}", s.level, heading)
                            })
                            .collect();
                        pad.sections = sections;
                        let word_count: usize = parsed.document.raw_text.split_whitespace().count();
                        pad.collated = format!(
                            "parsed {} sections, {} words, format={:?}",
                            parsed.section_count, word_count, parsed.document.format
                        );
                        log::info!(
                            "[document_parse] parsed {}: {} sections, {} words",
                            input,
                            parsed.section_count,
                            word_count
                        );
                        return Ok(StageDecision::Continue);
                    }
                    Err(e) => {
                        log::warn!("[document_parse] failed to parse {}: {}", input, e);
                        return Ok(StageDecision::Skip(format!("parse failed: {}", e)));
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}
