use super::ConsciousnessIntegration;
use crate::core::nt_core_spatial::reasoner::SpatialEvidenceFactory as EF;

impl ConsciousnessIntegration {
    /// Lazy-init spatial reasoner, enqueue text_buffer content as spatial queries,
    /// then process one pending query using real CI subsystems.
    pub fn handle_spatial_reasoner_tick(&mut self) -> String {
        let reasoner = self.spatial_reasoner.get_or_insert_with(|| {
            crate::core::nt_core_spatial::reasoner::SpatialReasoner::with_defaults()
        });

        // Enqueue spatial-related text from buffer
        while let Some(text) = self.text_buffer.pop_front() {
            let trimmed = text.trim();
            if trimmed.len() > 20
                && (trimmed.contains("where")
                    || trimmed.contains("location")
                    || trimmed.contains("place")
                    || trimmed.contains("scene")
                    || trimmed.contains("position")
                    || trimmed.contains("spatial"))
            {
                reasoner.enqueue_query(trimmed.to_string());
            }
        }

        // Extract data needed for closure (avoids &mut self capture)
        let l1_data = if !self.attractor_state.is_empty() {
            let decoded = self.vsa_decoder.decode(
                &self.attractor_state,
                "spatial_l1",
                self.cycle,
                self.specious_present.average_coherence(),
                self.neuromodulator.arousal_contribution(),
            );
            Some(decoded.title)
        } else {
            None
        };
        let l2_data = if !self.vsa_buffer.is_empty() {
            let buf: Vec<&[u8]> = self.vsa_buffer.iter().map(|v| v.as_slice()).collect();
            let merged = crate::core::nt_core_hcube::SpatialSceneEngine::bundle_scene(&buf);
            Some((
                buf.len(),
                merged.len(),
                self.physics_commonsense.energy.kinetic,
            ))
        } else {
            None
        };
        let l3_count = self.spatial_graph.as_ref().map(|g| g.len()).unwrap_or(0);

        // Process one pending query with data-driven closures (no &mut self capture)
        let _processed = reasoner.process_pending(
            &mut |_query: &str| {
                let title = l1_data.as_ref()?;
                Some(EF::level1("vsa_decoder_scene", title, 0.5))
            },
            &mut |_query: &str| {
                let (buf_len, _merged_len, energy) = l2_data.as_ref()?;
                Some(EF::level2(
                    "scene_bundle",
                    &format!("scene_objects={}_energy={:.3}", buf_len, energy),
                    0.45,
                ))
            },
            &mut |_query: &str| {
                if l3_count == 0 {
                    return None;
                }
                Some(EF::level3(
                    "spatial_graph",
                    &format!("known_nodes={}_query={}", l3_count, _query),
                    0.35,
                ))
            },
        );

        reasoner.report()
    }
}
