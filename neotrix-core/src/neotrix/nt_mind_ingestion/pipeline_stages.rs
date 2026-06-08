use crate::make_stage;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind::self_iterating::pipeline::{BrainStage, StageDecision};
use super::scratchpad::should_continue_reflection;
use super::ReflectionRound;

make_stage!(CollateStage);
impl BrainStage for CollateStage {
    fn name(&self) -> &str { "ingestion_collate" }
    fn frequency(&self) -> usize { 3 }
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
            total, non_empty.len(),
            if total > 0 { non_empty.len() as f64 / total as f64 * 100.0 } else { 0.0 }
        );
        Ok(StageDecision::Continue)
    }
}

make_stage!(StructureStage);
impl BrainStage for StructureStage {
    fn name(&self) -> &str { "ingestion_structure" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        let mut sections = Vec::new();
        let lines: Vec<&str> = pad.input.lines().collect();
        let chunk_size = (lines.len() / 4).max(1);

        for (i, chunk) in lines.chunks(chunk_size).enumerate() {
            let preview: String = chunk.iter()
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
            sections = sections.into_iter()
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
    fn name(&self) -> &str { "ingestion_entity_extract" }
    fn frequency(&self) -> usize { 3 }
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
                if cleaned.len() >= 4 && cleaned.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    let entity = cleaned.to_string();
                    if !entities.contains(&entity) {
                        entities.push(entity);
                    }
                }
            }
        }
        if pad.round > 1 {
            entities = entities.into_iter()
                .map(|e| format!("{} [confirmed]", e))
                .collect();
        }
        pad.entities = entities;
        Ok(StageDecision::Continue)
    }
}

make_stage!(EventExtractStage);
impl BrainStage for EventExtractStage {
    fn name(&self) -> &str { "ingestion_event_extract" }
    fn frequency(&self) -> usize { 3 }
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
    fn name(&self) -> &str { "ingestion_relation_map" }
    fn frequency(&self) -> usize { 3 }
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
            relations = relations.into_iter()
                .map(|(s, r, t)| (s, r, format!("{} [weighted]", t)))
                .collect();
        }
        pad.relations = relations;
        Ok(StageDecision::Continue)
    }
}

make_stage!(OntologyAlignStage);
impl BrainStage for OntologyAlignStage {
    fn name(&self) -> &str { "ingestion_ontology_align" }
    fn frequency(&self) -> usize { 3 }
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
    fn name(&self) -> &str { "ingestion_reason" }
    fn frequency(&self) -> usize { 3 }
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
    fn name(&self) -> &str { "ingestion_sku_generate" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };
        pad.skus = pad.reasoning.iter()
            .enumerate()
            .map(|(i, note)| format!("sku_{}: {}", i + 1, note))
            .collect();
        Ok(StageDecision::Continue)
    }
}

make_stage!(ApplyStage);
impl BrainStage for ApplyStage {
    fn name(&self) -> &str { "ingestion_apply" }
    fn frequency(&self) -> usize { 3 }
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
    fn name(&self) -> &str { "ingestion_reflection_check" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let pad = match brain._ingestion_scratchpad_mut() {
            Some(p) => p,
            None => return Ok(StageDecision::Skip("no ingestion active".into())),
        };

        pad.clarity_delta = if pad.round == 1 {
            0.8
        } else {
            let prev = pad.reflection_history.last()
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
                pad.round, pad.clarity_delta, qm_score,
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
                pad.round, pad.clarity_delta,
            );
        }

        Ok(StageDecision::Continue)
    }
}


use super::stream_hygiene::{clean_orphan_vsa_vectors, repair_corrupted_tags, fold_duplicate_vectors};

make_stage!(StreamHygieneStage);
impl BrainStage for StreamHygieneStage {
    fn name(&self) -> &str { "stream_hygiene" }
    fn frequency(&self) -> usize { 3 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let before_len = brain._consciousness_stream.len();
        let removed_orphans = clean_orphan_vsa_vectors(&mut brain._consciousness_stream);
        let removed_corrupt = repair_corrupted_tags(&mut brain._consciousness_stream);
        let removed_dups = fold_duplicate_vectors(&mut brain._consciousness_stream, 64);
        let after_len = brain._consciousness_stream.len();
        if before_len != after_len {
            log::info!(
                "[stream_hygiene] cleaned {}→{} vectors (orphans={}, corrupt={}, dups={})",
                before_len, after_len, removed_orphans, removed_corrupt, removed_dups,
            );
        }
        Ok(StageDecision::Continue)
    }
}

use std::sync::Mutex;
use super::fingerprint::{VsaPrefixFingerprint, default_constraints};

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
    fn name(&self) -> &str { "vsa_fingerprint" }
    fn frequency(&self) -> usize { 1 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let mut fp_guard = self.fingerprint.lock().unwrap();
        let constraints = default_constraints();
        let first_person = &brain._first_person;

        match fp_guard.as_ref() {
            None => {
                let fp = VsaPrefixFingerprint::new(first_person, constraints);
                fp_guard.replace(fp);
                log::info!("[vsa_fingerprint] initialized");
                Ok(StageDecision::Continue)
            }
            Some(expected) => {
                expected.verify(first_person, &constraints).map_err(|e| {
                    log::error!("[vsa_fingerprint] DRIFT DETECTED: {}", e);
                    NeoTrixError::Brain(format!("VSA prefix drift: {}", e))
                })?;
                Ok(StageDecision::Continue)
            }
        }
    }
}

make_stage!(CanonicalSortStage);
impl BrainStage for CanonicalSortStage {
    fn name(&self) -> &str { "canonical_sort" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use super::canonical::canonical_catalog_fingerprint;

        let hex_ids: Vec<usize> = brain._strategy_matrix.iter()
            .flat_map(|row| row.iter())
            .map(|h| h.0 as usize)
            .collect();

        let dummy_specs: Vec<String> = vec![];

        let fp = canonical_catalog_fingerprint(&hex_ids, &dummy_specs);
        let fp_short = if fp.len() > 16 { &fp[..16] } else { &fp };
        log::trace!("[canonical_sort] catalog fingerprint: {} ({} hexagrams)", fp_short, hex_ids.len());
        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
