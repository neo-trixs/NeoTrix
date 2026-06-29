use super::types::*;
use crate::core::nt_core_consciousness::volition::ActionCandidate;
use crate::core::nt_core_self::AttentionDomain;
use crate::neotrix::nt_mind::curiosity_drive::CuriosityDrive;

impl ConsciousnessIntegration {
    pub fn handle_input_pipeline_batch(&mut self, items: &[(&str, &str)]) {
        for (_content_type, _content) in items {
            // stub: no real InputSemanticType dependency
            self.text_feed_count += 1;
        }
        if self.cycle % 20 == 0 {
            log::debug!(
                "[consciousness] input pipeline: batch of {} items",
                items.len()
            );
        }
    }

    /// Encode all buffered text into VSA vectors using the n-gram semantic encoder.
    pub fn encode_inputs(&mut self) -> Vec<Vec<u8>> {
        let texts: Vec<String> = self.text_buffer.drain(..).collect();
        let mut results = Vec::with_capacity(texts.len());
        for text in &texts {
            let vsa = self.ngram_encoder.encode_text(text);
            results.push(vsa);
        }
        results
    }

    /// Feed text into the VSA encoding pipeline.
    pub fn feed_text(&mut self, text: &str, _source: &str) -> Vec<u8> {
        let vector = self.ngram_encoder.encode_text(text);
        self.text_feed_count += 1;
        self.push_vsa_buffer(vector.clone());
        self.push_text_buffer(text.to_string());
        vector
    }

    /// Get the most recent VSA-encoded vector, or a random one if buffer is empty.
    pub fn get_vsa_input(&mut self) -> Vec<u8> {
        self.vsa_buffer
            .back()
            .cloned()
            .unwrap_or_else(|| self.ngram_encoder.encode_text("default"))
    }

    /// Get a batch of VSA vectors from the buffer.
    pub fn get_vsa_batch(&self, n: usize) -> Vec<Vec<u8>> {
        let len = self.vsa_buffer.len();
        let start = if len > n { len - n } else { 0 };
        self.vsa_buffer
            .iter()
            .skip(start)
            .take(n)
            .cloned()
            .collect()
    }

    /// Handle feeding text from external caller.
    pub fn handle_feed_text(&mut self, text: &str, source: &str) -> String {
        let vector = self.feed_text(text, source);
        let hex: String = vector
            .iter()
            .take(8)
            .map(|b| format!("{:02x}", b))
            .collect();
        format!(
            "encoded {} bytes -> VSA ..{}, buffer={}",
            text.len(),
            hex,
            self.vsa_buffer.len()
        )
    }

    /// Derive a variant vector from a base VSA by rotation + deterministic mask.
    #[cfg(test)]
    fn _derive_vsa_variant(base: &[u8], variant_seed: u64) -> Vec<u8> {
        if base.is_empty() {
            return vec![(variant_seed & 0xFF) as u8; 64];
        }
        let dim = base.len();
        let mut result = base.to_vec();
        let offset = (usize::try_from(variant_seed).unwrap_or(0)) % dim;
        result.rotate_left(offset);
        for (i, b) in result.iter_mut().enumerate() {
            *b ^= ((variant_seed.wrapping_add(i as u64)) & 0xFF) as u8;
        }
        result
    }

    /// Bridge: drain curiosity drive queries → feed into exploration orchestrator.
    pub fn curiosity_orchestrator_bridge(&mut self, _curiosity: &mut CuriosityDrive) -> usize {
        if self.pending_curiosity_gain > 0.0 {
            if self.cycle % 10 == 0 {
                log::debug!(
                    "[consciousness] dgmh→curiosity: gain={:.3}",
                    self.pending_curiosity_gain
                );
            }
            self.pending_curiosity_gain = 0.0;
        }
        self.orchestrator.seed_from_gaps(&[]);
        if self.cycle % 5 == 0 {
            log::debug!("[consciousness] curiosity→orchestrator: 0 queries seeded");
        }
        0
    }

    /// Bridge: epistemic gaps → curriculum generator.
    pub fn epistemic_gap_bridge(&mut self, min_coverage: f64) -> usize {
        let gaps = self.epistemic.identify_gaps(min_coverage);
        let count = gaps.len();
        if count > 0 {
            let gap_data: Vec<(String, AttentionDomain, f64)> = gaps
                .iter()
                .map(|c| {
                    (
                        c.label.clone(),
                        AttentionDomain::Reasoning,
                        1.0 - c.confidence,
                    )
                })
                .collect();
            self.curriculum.generate_from_gaps(&gap_data);
            if self.cycle % 10 == 0 {
                log::debug!(
                    "[consciousness] epistemic→curriculum: {} gaps seeded",
                    count
                );
            }
        }
        count
    }

    /// Bridge: CurriculumGenerator → background thinking focus.
    pub fn curriculum_thinking_bridge(&mut self) -> Option<String> {
        let challenges = self.curriculum.next_challenge(1);
        if challenges.is_empty() {
            return None;
        }
        let task = &challenges[0];
        if self.cycle % 5 == 0 {
            log::debug!(
                "[consciousness] curriculum→think: '{}' at tier {:?}",
                task.description,
                task.difficulty
            );
        }
        Some(task.description.clone())
    }

    /// Bridge: ValueSystem → VolitionEngine.
    pub fn value_volition_bridge(&mut self) -> String {
        let unsatisfied = self.value_system.unsatisfied_values(0.5);
        if unsatisfied.is_empty() {
            return "no_value_drive".to_string();
        }
        let _top_value = unsatisfied[0];
        let action_desc = "satisfy_value".to_string();
        let action_vsa = self.ngram_encoder.encode_text(&action_desc);
        let candidate = ActionCandidate::new(action_vsa, &action_desc);
        self.volition.propose(candidate);
        if let Some(candidate) = self.volition.select_best() {
            if self.cycle % 5 == 0 {
                log::debug!(
                    "[consciousness] value→volition: '{}' (ev={})",
                    candidate.description,
                    candidate.expected_value
                );
            }
            candidate.description
        } else {
            "no_candidate".to_string()
        }
    }
}
